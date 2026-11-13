use std::fs;
use std::io::{BufRead, Write};
use std::path::Path;

use annulus::lingua::exsequi::Voluntas;
use annulus::lingua::json::Json;
use annulus::lingua::registra::{goetia, Spirit};
use annulus::mandata::ritus_with;
use annulus::officia::{signature, Class};

use crate::ingenium::record_verdict;
use crate::llama::quote;
use crate::memoria;
use crate::persona::entry;

pub const RITES: &str = "spiri72.ritus";
pub const HOUSE: &str = "annulus.domus";

/// The house a real wand keeps. A real wand refuses a house the test wand set up, so the
/// two are kept apart in the Operator's folder.
pub const HOUSE_VERA: &str = "annulus.domus.virga";

/// The drive of the real wand named in the session's flags (`--virga E:\`), if any.
pub fn virga_of(flags: &[String]) -> Option<&str> {
    flags
        .chunks(2)
        .find(|c| c[0] == "--virga")
        .and_then(|c| c.get(1))
        .map(String::as_str)
}
const MAX_QUESTION: usize = 2000;

/// The sky for one sitting of a spirit. A held spirit (called at `held`, an
/// open hour, and not yet given licence to depart) is declared with
/// `--tentus`, so its hours are those of its calling. A spirit not yet held
/// is called at a fixed hour, which becomes the hour of its calling.
pub fn held_sky(spirit: &Spirit, sky: &[String], held: Option<i64>) -> (Vec<String>, i64) {
    let mut flags: Vec<String> = sky.to_vec();
    let t0 = match held {
        Some(t0) => {
            flags.push("--tentus".into());
            flags.push(format!("{}@{}", spirit.n, annulus::horae::iso(t0)));
            t0
        }
        None => {
            let t0 = crate::aula::caelum_of(sky)
                .hora
                .unwrap_or_else(annulus::horae::now);
            if !sky.iter().any(|f| f == "--hora") {
                flags.push("--hora".into());
                flags.push(annulus::horae::iso(t0));
            }
            t0
        }
    };
    (flags, t0)
}

/// A spirit stays held only while its answers stand: any other judgement
/// banishes it (`RELEGA`), and a refused or broken rite ends the hold.
pub fn still_held(verdict: Option<&str>) -> bool {
    verdict == Some("RATUM")
}

pub struct Restraints {
    pub before: &'static str,
    pub after: &'static str,
}

pub fn restraints(n: u32) -> Restraints {
    let (before, after) = match n {
        9 => ("", "    OFFER v;\n    CONSTRINGE v PER NOMEN \"Adonai\";\n"),
        13 => ("    GERE VIRGAM;\n", ""),
        32 => ("", "    APPELLA v;\n"),
        63 => ("    GERE PENTAGRAMMUM;\n", ""),
        68 => (
            "",
            "    OFFER v;\n    CONSTRINGE v PER NOMEN \"Tetragrammaton\";\n",
        ),
        _ => ("", ""),
    };
    Restraints { before, after }
}

pub fn literal(s: &str) -> String {
    let mut out = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\t' => out.push_str("\\t"),
            '\n' => out.push_str("\\n"),
            c if c.is_control() => {}
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

pub fn find(answer: &str) -> Option<&'static Spirit> {
    let register = goetia();
    let answer = answer.trim();
    if let Ok(n) = answer.parse::<u32>() {
        return register.spirits.iter().find(|s| s.n == n);
    }
    let label = answer.to_lowercase().replace(['-', ' '], "_");
    let n = register.labels.get(&label)?;
    register.spirits.iter().find(|s| s.n == *n)
}

pub fn unbindable(s: &Spirit) -> Option<String> {
    if s.ranks.len() > 1 {
        return Some(format!(
            "{} holds more than one rank, and the ink for such a seal is not yet ruled",
            s.name
        ));
    }
    match s.ranks.first().map(String::as_str) {
        Some("president") => Some(format!(
            "{} is a President, who comes only after his king is called, and the kings are not built yet",
            s.name
        )),
        _ if telling_offices(s).is_empty() => Some(format!(
            "{} has no office of telling that Spiri72 can speak through yet",
            s.name
        )),
        _ => None,
    }
}

pub fn telling_offices(s: &Spirit) -> Vec<String> {
    s.offices
        .iter()
        .filter(|o| signature(o).map(|g| g.class) == Some(Class::Telling))
        .cloned()
        .collect()
}

pub fn rite(s: &Spirit, office: &str, question: &str, handed: &[String]) -> Result<String, String> {
    let ink = s.ink_class().ok_or_else(|| {
        format!(
            "{} holds more than one rank, and the ink for such a seal is not yet ruled",
            s.name
        )
    })?;
    let asks = signature(office).is_none_or(|g| g.quaestio);
    if !asks && handed.is_empty() {
        return Err(format!(
            "{} needs a file handed to it to read",
            office.to_lowercase()
        ));
    }
    let r = restraints(s.n);
    let grants: String = handed
        .iter()
        .map(|p| format!("    liber {} legere;\n", literal(p)))
        .collect();
    let args: String = asks
        .then(|| literal(question))
        .into_iter()
        .chain(handed.iter().map(|p| literal(p)))
        .collect::<Vec<_>>()
        .join(", ");
    Ok(format!(
        "FACULTAS {{\n{grants}    atramentum {ink} 2;\n}}\nCIRCULUS RITU goetia {{\n    INTENTIO {intent};\n    GERE ANULUM;\n{before}    SIT s = INSCRIBE SIGILLUM {label} ATRAMENTO CONSECRA ATRAMENTUM {ink};\n    SIT v = INVOCA SPIRITUM {n} AD OFFICIUM {office};\n{after}    SIT p = PETE v({args});\n    SIT iudicium = EXAMINA p;\n    MANIFESTA iudicium;\n    SI iudicium == RATUM {{\n        MANIFESTA RESPONSUM p;\n        MANE v;\n    }} ALITER {{\n        RELEGA v;\n    }}\n}}\n",
        ink = literal(ink),
        intent = literal(question),
        before = r.before,
        after = r.after,
        label = s.label,
        n = s.n,
        office = office.to_lowercase(),
    ))
}

fn texts(e: &Json, key: &str) -> Vec<String> {
    e.get(key)
        .map(Json::list)
        .unwrap_or(&[])
        .iter()
        .filter_map(|x| x.get("text").and_then(Json::text).map(str::to_string))
        .collect()
}

/// The licence to depart for a held spirit: it is called once more, in the
/// hours of its calling, and sent away in peace, so its record ends with
/// `dimissus`.
pub fn licence(s: &Spirit, office: &str) -> Result<String, String> {
    let ink = s
        .ink_class()
        .ok_or_else(|| format!("{} holds more than one rank", s.name))?;
    let r = restraints(s.n);
    Ok(format!(
        "FACULTAS {{\n    atramentum {ink} 2;\n}}\nCIRCULUS RITU goetia {{\n    INTENTIO \"licentia discedendi\";\n    GERE ANULUM;\n{before}    SIT s = INSCRIBE SIGILLUM {label} ATRAMENTO CONSECRA ATRAMENTUM {ink};\n    SIT v = INVOCA SPIRITUM {n} AD OFFICIUM {office};\n{after}    VADE IN PACE v;\n}}\n",
        ink = literal(ink),
        before = r.before,
        after = r.after,
        label = s.label,
        n = s.n,
        office = office.to_lowercase(),
    ))
}

/// Gives a held spirit licence to depart, as its own short rite.
#[allow(clippy::too_many_arguments)]
pub fn give_licence(
    spirit: &Spirit,
    office: &str,
    held: i64,
    dir: &Path,
    sky: &[String],
    will: &mut dyn Voluntas,
    out: &mut dyn Write,
    err: &mut dyn Write,
) -> u8 {
    let program = match licence(spirit, office) {
        Ok(p) => p,
        Err(e) => {
            let _ = writeln!(err, "{e}");
            return 2;
        }
    };
    let (sky, _) = held_sky(spirit, sky, Some(held));
    match perform(spirit, &program, dir, &sky, will, out, err) {
        Ok((code, _, _)) => code,
        Err(code) => code,
    }
}

pub fn describe(s: &Spirit) -> String {
    let e = entry(s.n);
    let list = |k: &str| {
        let v = e.map(|e| texts(e, k)).unwrap_or_default();
        if v.is_empty() {
            "none recorded".to_string()
        } else {
            v.join("; ")
        }
    };
    let offices = e
        .and_then(|e| e.get("offices"))
        .map(Json::list)
        .unwrap_or(&[])
        .iter()
        .filter_map(|o| {
            Some(format!(
                "{} ({})",
                o.get("text").and_then(Json::text)?,
                o.get("kind").and_then(Json::text)?
            ))
        })
        .collect::<Vec<_>>()
        .join("; ");
    format!(
        "{name}, spirit {n} of the 72, {ranks}, ruling {legions} legions.\n  Offices: {offices}\n  Harm: {harm}\n  Restrained by: {counter}\n  Truthfulness: {veracity}. {condition}",
        name = s.name,
        n = s.n,
        ranks = s.ranks.join(" and "),
        legions = s.legions,
        harm = list("malice"),
        counter = list("counterbalances"),
        veracity = s.veracity,
        condition = s.veracity_condition,
    )
}

fn ask(input: &mut dyn BufRead, out: &mut dyn Write, prompt: &str) -> Option<String> {
    let _ = write!(out, "{prompt}");
    let _ = out.flush();
    let mut line = String::new();
    match input.read_line(&mut line) {
        Ok(0) | Err(_) => None,
        Ok(_) => Some(line.trim().to_string()),
    }
}

fn choose_spirit(
    input: &mut dyn BufRead,
    out: &mut dyn Write,
) -> Option<(&'static Spirit, String)> {
    loop {
        let answer = ask(
            input,
            out,
            "\nWhich spirit do you bind? Give a number from 1 to 72 or a name: ",
        )?;
        if answer.is_empty() {
            continue;
        }
        let Some(s) = find(&answer) else {
            let _ = writeln!(out, "No spirit of the 72 answers to {answer:?}.");
            continue;
        };
        let _ = writeln!(out, "\n{}", describe(s));
        if let Some(why) = unbindable(s) {
            let _ = writeln!(out, "{why}. Choose another.");
            continue;
        }
        let offices = telling_offices(s);
        let office = match offices.as_slice() {
            [] => continue,
            [one] => one.clone(),
            many => loop {
                let pick = ask(
                    input,
                    out,
                    &format!(
                        "Through which office do you charge {}? ({}): ",
                        s.name,
                        many.join(", ")
                    ),
                )?;
                if let Some(o) = many.iter().find(|o| o.eq_ignore_ascii_case(&pick)) {
                    break o.clone();
                }
            },
        };
        return Some((s, office));
    }
}

#[allow(clippy::too_many_arguments)]
pub fn ask_once(
    spirit: &Spirit,
    office: &str,
    question: &str,
    handed: &[String],
    dir: &Path,
    sky: &[String],
    will: &mut dyn Voluntas,
    out: &mut dyn Write,
    err: &mut dyn Write,
) -> u8 {
    ask_once_named(spirit, office, question, handed, dir, sky, will, out, err).0
}

/// As `ask_once`, also giving the rite's name (for example `furfur-3`) once
/// the angel has judged it, so its answer can be taken into the book.
#[allow(clippy::too_many_arguments)]
pub fn ask_once_named(
    spirit: &Spirit,
    office: &str,
    question: &str,
    handed: &[String],
    dir: &Path,
    sky: &[String],
    will: &mut dyn Voluntas,
    out: &mut dyn Write,
    err: &mut dyn Write,
) -> (u8, Option<String>) {
    let (code, name, _) =
        ask_once_judged(spirit, office, question, handed, dir, sky, will, out, err);
    (code, name)
}

/// Writes a rite into the Operator's folder and runs it through Annulus,
/// giving its exit code, its path, and what it said.
fn perform(
    spirit: &Spirit,
    program: &str,
    dir: &Path,
    sky: &[String],
    will: &mut dyn Voluntas,
    out: &mut dyn Write,
    err: &mut dyn Write,
) -> Result<(u8, std::path::PathBuf, Vec<u8>), u8> {
    let rites = dir.join(RITES);
    if let Err(e) = fs::create_dir_all(&rites) {
        let _ = writeln!(err, "error: cannot make {}: {e}", rites.display());
        return Err(1);
    }
    let mut written = None;
    for k in 1u32..=100_000 {
        let p = rites.join(format!("{}-{k}.annulus", spirit.label));
        match fs::OpenOptions::new().write(true).create_new(true).open(&p) {
            Ok(mut f) => {
                if let Err(e) = f.write_all(program.as_bytes()).and_then(|_| f.sync_all()) {
                    let _ = writeln!(err, "error: cannot write {}: {e}", p.display());
                    return Err(1);
                }
                written = Some(p);
                break;
            }
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(e) => {
                let _ = writeln!(err, "error: cannot write {}: {e}", p.display());
                return Err(1);
            }
        }
    }
    let Some(path) = written else {
        let _ = writeln!(
            err,
            "error: no free name for the rite in {}",
            rites.display()
        );
        return Err(1);
    };
    // With a real wand its `--virga` flag rides in `sky`; otherwise the test wand.
    let (wand, house): (&[&str], &str) = if virga_of(sky).is_some() {
        (&[], HOUSE_VERA)
    } else {
        (&["--virga-probationis"], HOUSE)
    };
    let args: Vec<String> = [path.to_string_lossy().as_ref()]
        .into_iter()
        .chain(wand.iter().copied())
        .chain([
            "--mundus",
            &dir.to_string_lossy(),
            "--domus",
            &dir.join(house).to_string_lossy(),
        ])
        .map(|s| s.to_string())
        .chain(sky.iter().cloned())
        .collect();
    let mut said = Tee {
        out,
        kept: Vec::new(),
    };
    let code = match ritus_with(&args, &mut said, err, Some(will)) {
        Ok(code) => code,
        Err(usage) => {
            let _ = writeln!(err, "error: {}", usage.0);
            return Err(2);
        }
    };
    Ok((code, path, said.kept))
}

/// As `ask_once_named`, also giving the angel's judgement, when there was one.
#[allow(clippy::too_many_arguments)]
pub fn ask_once_judged(
    spirit: &Spirit,
    office: &str,
    question: &str,
    handed: &[String],
    dir: &Path,
    sky: &[String],
    will: &mut dyn Voluntas,
    out: &mut dyn Write,
    err: &mut dyn Write,
) -> (u8, Option<String>, Option<String>) {
    let program = match rite(spirit, office, question, handed) {
        Ok(p) => p,
        Err(e) => {
            let _ = writeln!(err, "{e}");
            return (2, None, None);
        }
    };
    let (code, path, kept) = match perform(spirit, &program, dir, sky, will, out, err) {
        Ok(r) => r,
        Err(code) => return (code, None, None),
    };
    let rites = dir.join(RITES);
    // The rite's first line is the angel's judgement (MANIFESTA iudicium).
    let first = String::from_utf8_lossy(&kept)
        .lines()
        .map(str::trim)
        .find(|l| !l.is_empty())
        .map(str::to_string);
    match first.as_deref() {
        Some(v @ ("RATUM" | "SUSPENSUM" | "IRRITUM")) => {
            if let Err(e) = record_verdict(dir, spirit, v) {
                let _ = writeln!(err, "error: the judgement was not recorded: {e}");
                return (code.max(1), None, Some(v.to_string()));
            }
            let name = path
                .file_stem()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();
            let words = crate::aula::read_output(spirit, &String::from_utf8_lossy(&kept))
                .verba
                .unwrap_or_default();
            let kept = format!(
                "{{\"n\":{},\"officium\":{},\"quaestio\":{},\"iudicium\":{},\"verba\":{}}}\n",
                spirit.n,
                quote(office),
                quote(question),
                quote(v),
                quote(&words)
            );
            if let Err(e) = fs::write(rites.join(format!("{name}.verba")), kept) {
                let _ = writeln!(err, "error: the answer was not kept: {e}");
                return (code.max(1), None, Some(v.to_string()));
            }
            if let Err(e) = memoria::note_rite(dir, spirit, &name, office, question, v) {
                let _ = writeln!(err, "error: the rite was not written in the book: {e}");
                return (code.max(1), None, Some(v.to_string()));
            }
            (code, Some(name), Some(v.to_string()))
        }
        _ => (code, None, None),
    }
}

struct Tee<'a> {
    out: &'a mut dyn Write,
    kept: Vec<u8>,
}

impl Write for Tee<'_> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let n = self.out.write(buf)?;
        if self.kept.len() < 4096 {
            self.kept.extend_from_slice(&buf[..n]);
        }
        Ok(n)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.out.flush()
    }
}

/// What the Operator is told of the wand in use.
pub fn wand_words(sky: &[String]) -> String {
    match virga_of(sky) {
        Some(d) => {
            format!("The real wand at {d}: each question asks its words in the wand's window.")
        }
        None => {
            "This session uses the test wand: nothing done here is valid for a real installation."
                .into()
        }
    }
}

pub fn sessio(
    input: &mut dyn BufRead,
    out: &mut dyn Write,
    err: &mut dyn Write,
    will: &mut dyn Voluntas,
    dir: &Path,
    sky: &[String],
) -> u8 {
    let _ = writeln!(out, "Spiri72, with {}.\n{}", will.nomen(), wand_words(sky));
    let Some((spirit, office)) = choose_spirit(input, out) else {
        return 0;
    };
    let handed: Vec<String> = match ask(
        input,
        out,
        &format!(
            "Hand {} a file from this folder to read? Give its name, or press Enter for none: ",
            spirit.name
        ),
    ) {
        None => return 0,
        Some(f) if f.is_empty() => Vec::new(),
        Some(f) => vec![f.replace('\\', "/")],
    };
    let _ = writeln!(
        out,
        "\n{} is bound through {office}, and held until you give licence to depart. Each question is examined by the angel; while held, its hours are those of its calling.",
        spirit.name
    );
    let mut worst = 0u8;
    let mut held: Option<i64> = None;
    let mut last: Option<String> = None;
    while let Some(question) = ask(
        input,
        out,
        "\nYour question (press Enter alone to give licence to depart): ",
    ) {
        if question.is_empty() {
            break;
        }
        if question == "+" {
            match last.take() {
                Some(r) => match receive(dir, &r) {
                    Ok(_) => {
                        let _ = writeln!(out, "It is written in {}'s book.", spirit.name);
                    }
                    Err(e) => {
                        let _ = writeln!(out, "It was not written: {e}.");
                    }
                },
                None => {
                    let _ = writeln!(out, "There is no sound answer to take up.");
                }
            }
            continue;
        }
        if question.chars().count() > MAX_QUESTION {
            let _ = writeln!(
                out,
                "That question is longer than {MAX_QUESTION} characters."
            );
            continue;
        }
        let _ = writeln!(out);
        let (sitting, t0) = held_sky(spirit, sky, held);
        let (code, ritus, verdict) = ask_once_judged(
            spirit, &office, &question, &handed, dir, &sitting, will, out, err,
        );
        worst = worst.max(code);
        let was_held = held.is_some();
        held = still_held(verdict.as_deref()).then_some(t0);
        if was_held && held.is_none() && code != 2 {
            let _ = writeln!(
                out,
                "{} is banished and no longer held; another question calls it anew, in its own hours.",
                spirit.name
            );
        }
        last = ritus.filter(|r| kept_sound(dir, r));
        if last.is_some() {
            let _ = writeln!(
                out,
                "(Type + to take this answer into {}'s book.)",
                spirit.name
            );
        }
        if code == 2 {
            let _ = writeln!(
                out,
                "The rite was refused or broken off; {} is no longer held.",
                spirit.name
            );
            break;
        }
    }
    if let Some(t0) = held {
        let mut quiet = Vec::new();
        let code = give_licence(spirit, &office, t0, dir, sky, will, &mut quiet, err);
        worst = worst.max(code);
    }
    let _ = writeln!(out, "\n{} has licence to depart.", spirit.name);
    worst
}

/// Takes the answer of a rite the angel found sound into its spirit's book.
/// The rite is named as `ask_once_named` gave it; the words are read from
/// what Spiri72 kept, never from the caller.
pub fn receive(dir: &Path, ritus: &str) -> Result<memoria::Entry, String> {
    let (label, k) = ritus
        .rsplit_once('-')
        .filter(|(l, k)| {
            !l.is_empty()
                && l.chars().all(|c| c.is_ascii_lowercase() || c == '_')
                && !k.is_empty()
                && k.chars().all(|c| c.is_ascii_digit())
        })
        .ok_or("that is not the name of a rite")?;
    let _ = k;
    let path = dir.join(RITES).join(format!("{ritus}.verba"));
    let meta =
        fs::symlink_metadata(&path).map_err(|_| "no answer of that rite was kept".to_string())?;
    if !meta.is_file() || meta.len() > memoria::MAX_ENTRY as u64 * 2 {
        return Err("that rite's answer is not one Spiri72 keeps".into());
    }
    let text = fs::read_to_string(&path).map_err(|e| format!("cannot read the rite: {e}"))?;
    let j = annulus::lingua::json::parse(text.trim())
        .map_err(|_| "that rite's answer is not in order")?;
    let txt = |k: &str| {
        j.get(k)
            .and_then(annulus::lingua::json::Json::text)
            .unwrap_or("")
            .to_string()
    };
    let n = j
        .get("n")
        .and_then(annulus::lingua::json::Json::number)
        .ok_or("that rite names no spirit")?;
    let spirit = goetia()
        .spirits
        .iter()
        .find(|s| s.n as f64 == n && s.label == label)
        .ok_or("that rite's spirit is not the one its name gives")?;
    memoria::receive(
        dir,
        spirit,
        ritus,
        &txt("officium"),
        &txt("quaestio"),
        &txt("iudicium"),
        &txt("verba"),
    )
}

/// Whether the kept answer of a rite was found sound and holds words.
pub fn kept_sound(dir: &Path, ritus: &str) -> bool {
    fs::read_to_string(dir.join(RITES).join(format!("{ritus}.verba")))
        .ok()
        .and_then(|t| annulus::lingua::json::parse(t.trim()).ok())
        .is_some_and(|j| {
            j.get("iudicium")
                .and_then(annulus::lingua::json::Json::text)
                == Some("RATUM")
                && j.get("verba")
                    .and_then(annulus::lingua::json::Json::text)
                    .is_some_and(|v| !v.trim().is_empty())
        })
}
