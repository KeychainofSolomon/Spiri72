use std::fs;
use std::io::{BufRead, Write};
use std::path::{Path, PathBuf};

use annulus::lingua::exsequi::Voluntas;
use annulus::lingua::json::Json;
use annulus::lingua::registra::{goetia, Spirit};
use annulus::mandata::ritus_with;
use annulus::officia::{signature, Class};

use crate::persona::entry;

pub const RITES: &str = "spiri72.ritus";
pub const HOUSE: &str = "annulus.domus";
const MAX_QUESTION: usize = 2000;

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
        Some("duke") => Some(format!(
            "{} is a Duke, and Dukes are refused until the hours of the sky are declared",
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
    let r = restraints(s.n);
    let grants: String = handed
        .iter()
        .map(|p| format!("    liber {} legere;\n", literal(p)))
        .collect();
    let args: String = std::iter::once(literal(question))
        .chain(handed.iter().map(|p| literal(p)))
        .collect::<Vec<_>>()
        .join(", ");
    Ok(format!(
        "FACULTAS {{\n{grants}    atramentum {ink} 2;\n}}\nCIRCULUS RITU goetia {{\n    INTENTIO {intent};\n    GERE ANULUM;\n{before}    SIT s = INSCRIBE SIGILLUM {label} ATRAMENTO CONSECRA ATRAMENTUM {ink};\n    SIT v = INVOCA SPIRITUM {n} AD OFFICIUM {office};\n{after}    SIT p = PETE v({args});\n    SIT iudicium = EXAMINA p;\n    MANIFESTA iudicium;\n    SI iudicium == RATUM {{\n        MANIFESTA RESPONSUM p;\n        VADE IN PACE v;\n    }} ALITER {{\n        RELEGA v;\n    }}\n}}\n",
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
    will: &mut dyn Voluntas,
    out: &mut dyn Write,
    err: &mut dyn Write,
) -> u8 {
    let rites = dir.join(RITES);
    if let Err(e) = fs::create_dir_all(&rites) {
        let _ = writeln!(err, "error: cannot make {}: {e}", rites.display());
        return 1;
    }
    let program = match rite(spirit, office, question, handed) {
        Ok(p) => p,
        Err(e) => {
            let _ = writeln!(err, "{e}");
            return 2;
        }
    };
    let path: PathBuf = (1u32..)
        .map(|k| rites.join(format!("{}-{k}.annulus", spirit.label)))
        .find(|p| !p.exists())
        .expect("a free name");
    if let Err(e) = fs::write(&path, program) {
        let _ = writeln!(err, "error: cannot write {}: {e}", path.display());
        return 1;
    }
    let args: Vec<String> = [
        path.to_string_lossy().as_ref(),
        "--virga-probationis",
        "--mundus",
        &dir.to_string_lossy(),
        "--domus",
        &dir.join(HOUSE).to_string_lossy(),
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();
    match ritus_with(&args, out, err, Some(will)) {
        Ok(code) => code,
        Err(usage) => {
            let _ = writeln!(err, "error: {}", usage.0);
            2
        }
    }
}

pub fn sessio(
    input: &mut dyn BufRead,
    out: &mut dyn Write,
    err: &mut dyn Write,
    will: &mut dyn Voluntas,
    dir: &Path,
) -> u8 {
    let _ = writeln!(
        out,
        "Spiri72, with {}.\nThis session uses the test wand: nothing done here is valid for a real installation.",
        will.nomen()
    );
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
        "\n{} is bound through {office}. Each question is its own rite: the spirit is called, examined and dismissed.",
        spirit.name
    );
    let mut worst = 0u8;
    while let Some(question) = ask(
        input,
        out,
        "\nYour question (press Enter alone to give licence to depart): ",
    ) {
        if question.is_empty() {
            break;
        }
        if question.chars().count() > MAX_QUESTION {
            let _ = writeln!(
                out,
                "That question is longer than {MAX_QUESTION} characters."
            );
            continue;
        }
        let _ = writeln!(out);
        let code = ask_once(spirit, &office, &question, &handed, dir, will, out, err);
        worst = worst.max(code);
        if code == 2 {
            let _ = writeln!(
                out,
                "The rite was refused or broken off; {} is no longer held.",
                spirit.name
            );
            break;
        }
    }
    let _ = writeln!(out, "\n{} has licence to depart.", spirit.name);
    worst
}
