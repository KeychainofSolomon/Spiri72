//! Each spirit's book: what passes between the Operator and one spirit,
//! kept across rites, so the bond outlives the model's window.
//!
//! **What is written, and by whom.** Never by the spirit. Spiri72 writes:
//! - after every one of its own rites, a `rogatum`: the office, the
//!   Operator's own question and the angel's judgement (no word of the spirit);
//! - when the Operator takes up an answer the angel found sound, a `receptum`:
//!   that answer, marked as the spirit's own words the Operator received;
//! - when the Operator strikes an entry, a `deletum`. Nothing is erased: the
//!   striking is written beside what it strikes, as in the Book of Spirits.
//!
//! **What comes back.** Before a rite, a program that is not the spirit
//! chooses a few entries: the latest, those that match the Operator's
//! question, and now and then an older one that resurfaces unbidden (more for
//! heavier entries and on anniversaries, less as they age). They are shuffled
//! and handed over unlabelled, so the spirit cannot tell why any came back,
//! and it has no way to search or write the book. A one-way transfer.
//!
//! **Order.** Each line carries the hash of the line before it. A book that
//! is out of order, too large, or not a plain file refuses the rite rather
//! than being guessed at. Until Annulus has a real wand, the chain catches a
//! broken or hand-edited book but cannot prove who wrote it.

use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use annulus::lingua::json::{self, Json};
use annulus::lingua::registra::Spirit;
use annulus::nucleus::sha256;

use crate::fors;
use crate::latina;
use crate::llama::quote;

pub const LIBER: &str = "spiri72.liber";
/// A spirit's book may grow to 1 GiB (the Operator's ruling, 2026-11-12).
pub const MAX_BOOK: u64 = 1 << 30;
/// The longest entry the book takes.
pub const MAX_ENTRY: usize = 64 * 1024;
/// How many of the latest entries always come back.
pub const RECENT: usize = 3;
/// How many entries matching the question come back.
pub const MATCHED: usize = 3;
/// How many characters of memory one rite is handed, at most.
pub const BUDGET: usize = 4000;
const GENESIS: &str = "0000000000000000000000000000000000000000000000000000000000000000";

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Kind {
    Rogatum,
    Receptum,
    Deletum(u64),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    pub n: u64,
    pub kind: Kind,
    /// Days since 1970-01-01, by the computer's clock.
    pub dies: i64,
    pub ritus: String,
    pub officium: String,
    pub iudicium: String,
    pub quaestio: String,
    pub verba: String,
    pub hash: String,
}

impl Entry {
    fn weight(&self) -> f64 {
        match (&self.kind, self.iudicium.as_str()) {
            (Kind::Receptum, _) => 3.0,
            (_, "IRRITUM") => 2.0,
            _ => 1.0,
        }
    }
}

pub fn today() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| (d.as_secs() / 86_400) as i64)
        .unwrap_or(0)
}

fn dir_of(dir: &Path) -> Result<PathBuf, String> {
    let d = dir.join(LIBER);
    match fs::symlink_metadata(&d) {
        Ok(m) if m.file_type().is_symlink() => {
            Err(format!("{} is a link, not a folder", d.display()))
        }
        Ok(m) if !m.is_dir() => Err(format!("{} is not a folder", d.display())),
        Ok(_) => Ok(d),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(d),
        Err(e) => Err(format!("cannot look at {}: {e}", d.display())),
    }
}

pub fn path_of(dir: &Path, s: &Spirit) -> Result<PathBuf, String> {
    Ok(dir_of(dir)?.join(format!("{:02}-{}.liber", s.n, s.label)))
}

#[allow(clippy::too_many_arguments)]
fn body(
    n: u64,
    kind: &Kind,
    dies: i64,
    ritus: &str,
    officium: &str,
    iudicium: &str,
    quaestio: &str,
    verba: &str,
    prev: &str,
) -> String {
    let (k, target) = match kind {
        Kind::Rogatum => ("rogatum", String::new()),
        Kind::Receptum => ("receptum", String::new()),
        Kind::Deletum(t) => ("deletum", format!(",\"delet\":{t}")),
    };
    format!(
        "{{\"n\":{n},\"genus\":\"{k}\"{target},\"dies\":{dies},\"ritus\":{},\"officium\":{},\"iudicium\":{},\"quaestio\":{},\"verba\":{},\"prior\":\"{prev}\"",
        quote(ritus),
        quote(officium),
        quote(iudicium),
        quote(quaestio),
        quote(verba)
    )
}

/// Reads a spirit's whole book, checking every link of the chain. A missing
/// book is a spirit with nothing kept yet.
pub fn read(dir: &Path, s: &Spirit) -> Result<Vec<Entry>, String> {
    let path = path_of(dir, s)?;
    let meta = match fs::symlink_metadata(&path) {
        Ok(m) => m,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(format!("cannot read {}: {e}", path.display())),
    };
    if !meta.is_file() || meta.len() > MAX_BOOK {
        return Err(format!("{} is not a book Spiri72 keeps", path.display()));
    }
    let text =
        fs::read_to_string(&path).map_err(|e| format!("cannot read {}: {e}", path.display()))?;
    let broken = |why: &str| {
        format!(
            "{} is not in order ({why}); the rite is refused",
            path.display()
        )
    };
    let mut out: Vec<Entry> = Vec::new();
    let mut prev = GENESIS.to_string();
    for line in text.lines() {
        let j = json::parse(line).map_err(|_| broken("a line is not an entry"))?;
        let num = |k: &str| j.get(k).and_then(Json::number);
        let txt = |k: &str| j.get(k).and_then(Json::text).map(str::to_string);
        let n = num("n").ok_or_else(|| broken("an entry has no number"))? as u64;
        if n != out.len() as u64 + 1 {
            return Err(broken("the entries are not numbered in order"));
        }
        let kind = match txt("genus").as_deref() {
            Some("rogatum") => Kind::Rogatum,
            Some("receptum") => Kind::Receptum,
            Some("deletum") => Kind::Deletum(
                num("delet")
                    .filter(|t| *t >= 1.0 && (*t as u64) < n)
                    .ok_or_else(|| broken("a striking names no earlier entry"))?
                    as u64,
            ),
            _ => return Err(broken("an entry of an unknown kind")),
        };
        let field = |k: &str| txt(k).ok_or_else(|| broken("an entry lacks a field"));
        let (ritus, officium, iudicium, quaestio, verba) = (
            field("ritus")?,
            field("officium")?,
            field("iudicium")?,
            field("quaestio")?,
            field("verba")?,
        );
        let dies = num("dies").ok_or_else(|| broken("an entry has no day"))? as i64;
        if txt("prior").as_deref() != Some(prev.as_str()) {
            return Err(broken("the chain is broken"));
        }
        let b = body(
            n, &kind, dies, &ritus, &officium, &iudicium, &quaestio, &verba, &prev,
        );
        let hash = sha256::hash_hex(b.as_bytes());
        if txt("hash").as_deref() != Some(hash.as_str()) {
            return Err(broken("an entry was changed after it was written"));
        }
        prev = hash.clone();
        out.push(Entry {
            n,
            kind,
            dies,
            ritus,
            officium,
            iudicium,
            quaestio,
            verba,
            hash,
        });
    }
    Ok(out)
}

/// Writes one entry at the end of the book.
#[allow(clippy::too_many_arguments)]
fn append(
    dir: &Path,
    s: &Spirit,
    kind: Kind,
    ritus: &str,
    officium: &str,
    iudicium: &str,
    quaestio: &str,
    verba: &str,
) -> Result<Entry, String> {
    if quaestio.len() + verba.len() > MAX_ENTRY {
        return Err("that entry is too long for the book".into());
    }
    let book = read(dir, s)?;
    let n = book.len() as u64 + 1;
    let prev = book.last().map_or(GENESIS.to_string(), |e| e.hash.clone());
    let dies = today();
    let b = body(
        n, &kind, dies, ritus, officium, iudicium, quaestio, verba, &prev,
    );
    let hash = sha256::hash_hex(b.as_bytes());
    let line = format!("{b},\"hash\":\"{hash}\"}}\n");
    let d = dir_of(dir)?;
    fs::create_dir_all(&d).map_err(|e| format!("cannot make {}: {e}", d.display()))?;
    let path = path_of(dir, s)?;
    let size = fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
    if size + line.len() as u64 > MAX_BOOK {
        return Err(format!("{}'s book is full", s.name));
    }
    let mut f = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .map_err(|e| format!("cannot write {}: {e}", path.display()))?;
    f.write_all(line.as_bytes())
        .and_then(|_| f.sync_all())
        .map_err(|e| format!("cannot write {}: {e}", path.display()))?;
    Ok(Entry {
        n,
        kind,
        dies,
        ritus: ritus.into(),
        officium: officium.into(),
        iudicium: iudicium.into(),
        quaestio: quaestio.into(),
        verba: verba.into(),
        hash,
    })
}

/// Tier 1: after a rite, what the Operator asked and how the angel judged.
pub fn note_rite(
    dir: &Path,
    s: &Spirit,
    ritus: &str,
    officium: &str,
    quaestio: &str,
    iudicium: &str,
) -> Result<Entry, String> {
    append(
        dir,
        s,
        Kind::Rogatum,
        ritus,
        officium,
        iudicium,
        quaestio,
        "",
    )
}

/// Tier 2: the Operator takes up an answer the angel found sound.
pub fn receive(
    dir: &Path,
    s: &Spirit,
    ritus: &str,
    officium: &str,
    quaestio: &str,
    iudicium: &str,
    verba: &str,
) -> Result<Entry, String> {
    if iudicium != "RATUM" {
        return Err("only an answer the angel found sound can be taken into the book".into());
    }
    if verba.trim().is_empty() {
        return Err("that rite left no words to take up".into());
    }
    if read(dir, s)?
        .iter()
        .any(|e| e.kind == Kind::Receptum && e.ritus == ritus)
    {
        return Err("that answer is already in the book".into());
    }
    append(
        dir,
        s,
        Kind::Receptum,
        ritus,
        officium,
        iudicium,
        quaestio,
        verba,
    )
}

/// The Operator strikes an entry. The striking is written; nothing is erased.
pub fn strike(dir: &Path, s: &Spirit, n: u64) -> Result<Entry, String> {
    let book = read(dir, s)?;
    match book.iter().find(|e| e.n == n) {
        None => Err(format!("{}'s book has no entry {n}", s.name)),
        Some(e) if matches!(e.kind, Kind::Deletum(_)) => Err("a striking cannot be struck".into()),
        Some(_) if live(&book).iter().all(|e| e.n != n) => {
            Err(format!("entry {n} is already struck"))
        }
        Some(_) => append(dir, s, Kind::Deletum(n), "", "", "", "", ""),
    }
}

/// The entries that still stand: neither struck nor strikings.
pub fn live(book: &[Entry]) -> Vec<&Entry> {
    let struck: HashSet<u64> = book
        .iter()
        .filter_map(|e| match e.kind {
            Kind::Deletum(t) => Some(t),
            _ => None,
        })
        .collect();
    book.iter()
        .filter(|e| !matches!(e.kind, Kind::Deletum(_)) && !struck.contains(&e.n))
        .collect()
}

const STOP: [&str; 24] = [
    "the", "and", "you", "your", "what", "who", "how", "why", "are", "was", "for", "with", "that",
    "this", "of", "to", "in", "is", "it", "me", "my", "do", "does", "tell",
];

/// The words an entry or question is matched by: each word, and for Latin
/// also its dictionary form, so *regis* finds *rex*.
fn terms(text: &str) -> HashSet<String> {
    let mut out = HashSet::new();
    for t in text.split(|c: char| !c.is_alphabetic()) {
        let w = latina::norm(t);
        if w.chars().count() < 3 || STOP.contains(&w.as_str()) {
            continue;
        }
        if let Some(l) = latina::analyse(&w).first() {
            if let Some(head) = l.lemma.split([',', ' ']).next() {
                out.insert(format!("~{}", latina::norm(head)));
            }
        }
        out.insert(w);
    }
    out
}

/// Chooses what comes back to the spirit in this rite. `lot` gives numbers
/// in [0, 1); Spiri72 passes the operating system's random source.
pub fn recall<'a>(
    book: &'a [Entry],
    question: &str,
    day: i64,
    lot: &mut dyn FnMut() -> Result<f64, String>,
) -> Result<Vec<&'a Entry>, String> {
    let live = live(book);
    let mut chosen: Vec<&Entry> = Vec::new();
    let mut taken = HashSet::new();
    for e in live.iter().rev().take(RECENT) {
        if taken.insert(e.n) {
            chosen.push(*e);
        }
    }

    let q = terms(question);
    if !q.is_empty() {
        let docs: Vec<(&Entry, HashSet<String>)> = live
            .iter()
            .map(|e| (*e, terms(&format!("{} {}", e.quaestio, e.verba))))
            .collect();
        let mut df: HashMap<&str, usize> = HashMap::new();
        for (_, ts) in &docs {
            for t in ts {
                if q.contains(t) {
                    *df.entry(t.as_str()).or_default() += 1;
                }
            }
        }
        let total = docs.len().max(1) as f64;
        let mut scored: Vec<(f64, &Entry)> = docs
            .iter()
            .map(|(e, ts)| {
                let score: f64 = ts
                    .iter()
                    .filter(|t| q.contains(*t))
                    .map(|t| (1.0 + total / df[t.as_str()] as f64).ln())
                    .sum();
                (score, *e)
            })
            .filter(|(s, _)| *s > 0.0)
            .collect();
        scored.sort_by(|a, b| b.0.total_cmp(&a.0).then(b.1.n.cmp(&a.1.n)));
        for (_, e) in scored.into_iter().take(MATCHED) {
            if taken.insert(e.n) {
                chosen.push(e);
            }
        }
    }

    for e in live.iter().rev().skip(RECENT).take(400) {
        if taken.contains(&e.n) {
            continue;
        }
        let age = day - e.dies;
        if age <= 0 {
            continue;
        }
        let anniversary = age % 365 == 0;
        let p =
            0.015 * e.weight() / (1.0 + age as f64 / 60.0) + if anniversary { 0.25 } else { 0.0 };
        if lot()? < p {
            chosen.push(*e);
            break;
        }
    }

    for i in (1..chosen.len()).rev() {
        let j = ((lot()? * (i + 1) as f64) as usize).min(i);
        chosen.swap(i, j);
    }
    Ok(chosen)
}

fn clip(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        return s.to_string();
    }
    format!("{}…", s.chars().take(max).collect::<String>().trim_end())
}

fn judged(v: &str) -> &'static str {
    match v {
        "RATUM" => "The angel found your answer sound.",
        "SUSPENSUM" => "The angel held your answer in suspense.",
        "IRRITUM" => "The angel voided your answer.",
        _ => "No judgement was given.",
    }
}

/// What comes back, in the words the spirit is handed: no numbers, no dates,
/// nothing that says how an entry was found.
pub fn render(chosen: &[&Entry]) -> Option<String> {
    let mut out = String::new();
    for e in chosen {
        let line = match e.kind {
            Kind::Rogatum => format!(
                "- You were asked, through {}: \u{201c}{}\u{201d} {}\n",
                e.officium,
                clip(&e.quaestio, 400),
                judged(&e.iudicium)
            ),
            Kind::Receptum => format!(
                "- Asked \u{201c}{}\u{201d}, you answered, and the Operator took it up: \u{201c}{}\u{201d}\n",
                clip(&e.quaestio, 200),
                clip(&e.verba, 800)
            ),
            Kind::Deletum(_) => continue,
        };
        if out.len() + line.len() > BUDGET {
            break;
        }
        out.push_str(&line);
    }
    (!out.is_empty()).then_some(out)
}

/// The memory block for one rite, or `None` if nothing comes back.
pub fn for_rite(dir: &Path, s: &Spirit, question: &str) -> Result<Option<String>, String> {
    let book = read(dir, s)?;
    let chosen = recall(&book, question, today(), &mut fors::unit)?;
    Ok(render(&chosen))
}
