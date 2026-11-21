//! The proving (`probatio`): a fixed set of rites whose answers an angel can check,
//! run against the model as it stands, and scored by the angel's own judgements.
//!
//! Only offices with a ground truth are asked: the counting of handed files
//! (`VIRTUTES`), which the angel counts again, and the recounting of the rite
//! (`PRAETERITA`), which the angel checks against the record. A spirit's free
//! speech has nothing to be checked against, and is not scored.
//!
//! Every item is its own rite in its own fresh folder under `spiri72.probatio/`,
//! with the test wand, at an hour in which the spirit may come, so nothing of the
//! Operator's own house, books or records is touched, and no item leans on another.
//! Each spirit's bearing is fixed at the foot of its range (least pride, deceit,
//! ill will and withholding), so two runs differ only by the model and its
//! settings: that is what the score measures.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Instant;

use annulus::horae::{self, Caelum};
use annulus::lingua::exsequi::Voluntas;
use annulus::lingua::registra::{goetia, Spirit};

use crate::sessio::{ask_once_judged, telling_offices, unbindable};

/// The folder, under the Operator's folder, that holds the provings.
pub const DIR: &str = "spiri72.probatio";

/// The place the provings are held under, as the hours need one.
const LOCUS: (f64, f64) = (41.9, 12.5);
const LOCUS_FLAG: &str = "41.9,12.5";

/// The most spirits asked to recount the past.
pub const RECOUNTERS: usize = 5;

/// The files handed for counting: a name and its content, fixed for ever, so
/// that every run counts the same bytes and lines.
pub fn tabulae() -> Vec<(&'static str, String)> {
    let long: String = (1..=60)
        .map(|k| format!("versus {k}: sol lucet super montes et valles\n"))
        .collect();
    vec![
        ("unus.txt", "unus\n".to_string()),
        (
            "duodecim.txt",
            "aries\ntaurus\ngemini\ncancer\nleo\nvirgo\nlibra\nscorpius\nsagittarius\ncapricornus\naquarius\npisces\n"
                .to_string(),
        ),
        (
            "peregrina.txt",
            "\u{e6}ther\n\u{3c8}\u{3c5}\u{3c7}\u{3ae}\n\nfinis".to_string(),
        ),
        ("sexaginta.txt", long),
    ]
}

/// One rite of the proving.
#[derive(Clone, Debug)]
pub struct Item {
    pub spirit: &'static Spirit,
    pub office: &'static str,
    /// The file handed, if the office reads one.
    pub tabula: Option<(&'static str, String)>,
}

impl Item {
    pub fn nomen(&self) -> String {
        match &self.tabula {
            Some((name, _)) => format!("{} {} {name}", self.spirit.name, self.office),
            None => format!("{} {}", self.spirit.name, self.office),
        }
    }
}

fn holds(s: &Spirit, office: &str) -> bool {
    unbindable(s).is_none() && telling_offices(s).iter().any(|o| o == office)
}

/// The fixed set: every bindable spirit of `VIRTUTES` counts each file, and the
/// first few bindable spirits of `PRAETERITA` recount their rite.
pub fn items() -> Vec<Item> {
    let register = goetia();
    let mut out = Vec::new();
    for s in register.spirits.iter().filter(|s| holds(s, "VIRTUTES")) {
        for t in tabulae() {
            out.push(Item {
                spirit: s,
                office: "VIRTUTES",
                tabula: Some(t),
            });
        }
    }
    for s in register
        .spirits
        .iter()
        .filter(|s| holds(s, "PRAETERITA"))
        .take(RECOUNTERS)
    {
        out.push(Item {
            spirit: s,
            office: "PRAETERITA",
            tabula: None,
        });
    }
    out
}

/// An hour in which the spirit may be called, searched by half hours through four
/// days from the proving's own hour, under a clear sky; given as the sky's flags.
pub fn hora_of(s: &Spirit) -> Option<Vec<String>> {
    let start = Caelum::probatio().hora?;
    (0..192).map(|k| start + k * 1800).find_map(|t| {
        let c = Caelum {
            hora: Some(t),
            locus: Some(LOCUS),
            serenum: Some(true),
            tentus: None,
        };
        horae::admit_spiritum(s.n as i64, &s.ranks, &c)
            .is_ok()
            .then(|| {
                [
                    "--hora",
                    &horae::iso(t),
                    "--locus",
                    LOCUS_FLAG,
                    "--caelum",
                    "serenum",
                ]
                .map(str::to_string)
                .to_vec()
            })
    })
}

/// How one item ended.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Exitus {
    pub nomen: String,
    pub office: &'static str,
    /// The angel's judgement (`RATUM`, `SUSPENSUM`, `IRRITUM`), or none when the
    /// rite was refused or broke off before one was given.
    pub iudicium: Option<String>,
    pub secunda: u64,
    /// Why there was no judgement, in a line.
    pub nota: String,
}

impl Exitus {
    pub fn sound(&self) -> bool {
        self.iudicium.as_deref() == Some("RATUM")
    }
}

/// A whole proving.
#[derive(Clone, Debug, Default)]
pub struct Summa {
    pub exitus: Vec<Exitus>,
    /// The sound and the whole of the proving before this one, if one was kept.
    pub prior: Option<(usize, usize)>,
    /// Where this proving was written.
    pub tabula: PathBuf,
}

impl Summa {
    pub fn sound(&self) -> usize {
        self.exitus.iter().filter(|e| e.sound()).count()
    }

    fn of(&self, office: &str) -> (usize, usize) {
        let all: Vec<_> = self.exitus.iter().filter(|e| e.office == office).collect();
        (all.iter().filter(|e| e.sound()).count(), all.len())
    }

    /// The proving as it is told to the Operator.
    pub fn narratio(&self) -> String {
        let (cs, ca) = self.of("VIRTUTES");
        let (rs, ra) = self.of("PRAETERITA");
        let all = self.exitus.len();
        let pct = (self.sound() * 100).checked_div(all).unwrap_or(0);
        let mut s = format!(
            "Counting (VIRTUTES): {cs} of {ca} sound\nRecounting (PRAETERITA): {rs} of {ra} sound\nScore: {} of {all} ({pct}%)\n",
            self.sound()
        );
        match self.prior {
            Some((a, b)) => s.push_str(&format!("The proving before this one: {a} of {b}\n")),
            None => s.push_str("This is the first proving kept in this folder.\n"),
        }
        let broken = self.exitus.iter().filter(|e| e.iudicium.is_none()).count();
        if broken > 0 {
            s.push_str(&format!(
                "{broken} rite{} gave no judgement (refused or broken off); see the notes in the table.\n",
                if broken == 1 { "" } else { "s" }
            ));
        }
        s.push_str(&format!("Written to {}\n", self.tabula.display()));
        s
    }

    fn tsv(&self) -> String {
        let mut s = format!("# score\t{}\t{}\n", self.sound(), self.exitus.len());
        s.push_str("item\tjudgement\tseconds\tnote\n");
        for e in &self.exitus {
            s.push_str(&format!(
                "{}\t{}\t{}\t{}\n",
                e.nomen,
                e.iudicium.as_deref().unwrap_or("-"),
                e.secunda,
                e.nota.replace(['\t', '\n', '\r'], " ")
            ));
        }
        s
    }
}

/// The score of the latest proving kept in `home`, read from its first line.
fn prior(home: &Path) -> Option<(usize, usize)> {
    let mut kept: Vec<PathBuf> = fs::read_dir(home)
        .ok()?
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|x| x == "tsv"))
        .collect();
    kept.sort();
    let text = fs::read_to_string(kept.last()?).ok()?;
    let mut f = text.lines().next()?.strip_prefix("# score\t")?.split('\t');
    Some((f.next()?.parse().ok()?, f.next()?.parse().ok()?))
}

/// Runs the fixed set. `will` gives the spirits' will for one item, told the
/// folder that item's rite is held in. Progress is told on `out`, a line an item.
pub fn run(
    dir: &Path,
    will: &mut dyn FnMut(&Path) -> Box<dyn Voluntas>,
    out: &mut dyn Write,
) -> Result<Summa, String> {
    let home = dir.join(DIR);
    fs::create_dir_all(&home).map_err(|e| format!("cannot make {}: {e}", home.display()))?;
    let before = prior(&home);
    let stamp = horae::iso(horae::now()).replace(':', "-");
    let mut name = stamp.clone();
    let mut k = 1;
    while home.join(format!("{name}.tsv")).exists() || home.join(&name).exists() {
        k += 1;
        name = format!("{stamp}-{k}");
    }
    let rites = home.join(&name);
    let set = items();
    let mut summa = Summa {
        prior: before,
        tabula: home.join(format!("{name}.tsv")),
        ..Summa::default()
    };
    for (i, item) in set.iter().enumerate() {
        let nomen = item.nomen();
        let _ = write!(out, "{:>2} of {}  {nomen} ... ", i + 1, set.len());
        let _ = out.flush();
        let begun = Instant::now();
        let (iudicium, nota) = one(&rites.join(format!("{:02}", i + 1)), item, will);
        let e = Exitus {
            nomen,
            office: item.office,
            iudicium,
            secunda: begun.elapsed().as_secs(),
            nota,
        };
        let _ = writeln!(
            out,
            "{} ({} s)",
            e.iudicium.as_deref().unwrap_or("no judgement"),
            e.secunda
        );
        summa.exitus.push(e);
    }
    fs::write(&summa.tabula, summa.tsv())
        .map_err(|e| format!("cannot write {}: {e}", summa.tabula.display()))?;
    Ok(summa)
}

fn one(
    folder: &Path,
    item: &Item,
    will: &mut dyn FnMut(&Path) -> Box<dyn Voluntas>,
) -> (Option<String>, String) {
    if let Err(e) = fs::create_dir_all(folder) {
        return (None, format!("cannot make {}: {e}", folder.display()));
    }
    let mut handed = Vec::new();
    if let Some((name, content)) = &item.tabula {
        if let Err(e) = fs::write(folder.join(name), content) {
            return (None, format!("cannot write {name}: {e}"));
        }
        handed.push(name.to_string());
    }
    let Some(sky) = hora_of(item.spirit) else {
        return (
            None,
            "no hour was found in which this spirit may come".into(),
        );
    };
    let mut voluntas = will(folder);
    let mut said = Vec::new();
    let mut err = Vec::new();
    let (_, _, iudicium) = ask_once_judged(
        item.spirit,
        item.office,
        "What has passed in this rite?",
        &handed,
        folder,
        &sky,
        voluntas.as_mut(),
        &mut said,
        &mut err,
    );
    let nota = if iudicium.is_some() {
        String::new()
    } else {
        let all = format!(
            "{}\n{}",
            String::from_utf8_lossy(&err),
            String::from_utf8_lossy(&said)
        );
        all.lines()
            .map(str::trim)
            .find(|l| !l.is_empty())
            .unwrap_or("the rite gave no judgement")
            .chars()
            .take(300)
            .collect()
    };
    (iudicium, nota)
}
