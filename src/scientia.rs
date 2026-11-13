//! Knowledge libraries (`scientia`): standing texts a spirit may consult, each
//! open only to spirits of the matching inclination.
//!
//! A library is a plain file under `spiri72.scientia/` in the Operator's folder,
//! so the Operator may read and grow it by hand. It opens with a short header and
//! then holds keyed entries:
//!
//! ```text
//! # liber: flora
//! # nomen: the herbal
//! # claves: herb, plant, tree, root
//!
//! == barley | hordeum | bere ==
//! Barley, a grass sown for grain...
//! ```
//!
//! The model is never fed a whole library. When a question names something a
//! library holds, the engine narrows to that one entry by its key and hands the
//! spirit only that entry, a true copy sealed like any other handed text. The
//! spirit answers in its own voice; the verbatim entry is shown to the Operator
//! so the reading can be checked against the source.
//!
//! A library opens to a spirit only where the spirit's own offices, as the
//! witnesses record them, touch the library's matter (its `claves`). Furfur, who
//! raises storms, cannot read the herbal; Buer, who teaches the virtues of herbs,
//! can.

use std::fs;
use std::path::Path;

/// The folder, under the Operator's folder, that holds the libraries.
pub const DIR: &str = "spiri72.scientia";

/// The starter libraries, written out on first run if none are present. The
/// Operator may edit them and add more.
const SEEDS: [(&str, &str); 2] = [
    ("flora.txt", include_str!("../scientia/flora.txt")),
    ("astronomia.txt", include_str!("../scientia/astronomia.txt")),
];

/// One keyed entry of a library.
#[derive(Clone, Debug)]
pub struct Entry {
    /// The names this entry answers to, lowercased (`barley`, `hordeum`, `bere`).
    pub claves: Vec<String>,
    /// The whole entry as written, heading and all, handed back verbatim.
    pub raw: String,
}

/// A whole library: its name, the matter it covers, and its entries.
#[derive(Clone, Debug)]
pub struct Liber {
    /// The slug, from the file name (`flora`).
    pub liber: String,
    /// The name shown to the Operator (`the herbal`).
    pub nomen: String,
    /// The words of the matter this library covers; a spirit may read it only if
    /// one of its offices touches one of these.
    pub claves: Vec<String>,
    pub entries: Vec<Entry>,
}

/// What a spirit drew from a library in one rite, kept for the Operator to see.
#[derive(Clone, Debug, PartialEq)]
pub struct Consultatio {
    /// The library's shown name (`the herbal`).
    pub liber: String,
    /// The term the question narrowed to (`barley`).
    pub terminus: String,
    /// The entry as written, a true copy.
    pub textus: String,
}

/// Lowercase alphanumeric words of a string; everything else is a break.
fn words(s: &str) -> Vec<String> {
    s.to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| !w.is_empty())
        .map(str::to_string)
        .collect()
}

/// A crude singular: drop a trailing `s` from a word of more than three letters,
/// so `herbs` and `stars` meet `herb` and `star`.
fn singular(w: &str) -> &str {
    match w.strip_suffix('s') {
        Some(stem) if stem.len() > 2 => stem,
        _ => w,
    }
}

fn same(a: &str, b: &str) -> bool {
    a == b || singular(a) == singular(b)
}

/// Reads a library file. Lines of `# key: value` before the first entry are the
/// header; `== a | b ==` opens an entry whose body runs to the next heading.
pub fn parse(liber: &str, text: &str) -> Liber {
    let mut nomen = liber.to_string();
    let mut claves = Vec::new();
    let mut entries: Vec<Entry> = Vec::new();
    let mut cur: Option<(Vec<String>, Vec<String>)> = None;
    let close = |cur: Option<(Vec<String>, Vec<String>)>, entries: &mut Vec<Entry>| {
        if let Some((keys, mut lines)) = cur {
            while lines.last().is_some_and(|l| l.trim().is_empty()) {
                lines.pop();
            }
            entries.push(Entry {
                claves: keys,
                raw: lines.join("\n"),
            });
        }
    };
    for line in text.lines() {
        let trimmed = line.trim();
        if cur.is_none() && trimmed.starts_with('#') {
            if let Some((k, v)) = trimmed.trim_start_matches('#').split_once(':') {
                match k.trim() {
                    "nomen" => nomen = v.trim().to_string(),
                    "claves" => {
                        claves = v.split(',').flat_map(words).collect();
                    }
                    _ => {}
                }
            }
            continue;
        }
        if let Some(head) = trimmed
            .strip_prefix("==")
            .and_then(|r| r.strip_suffix("=="))
        {
            close(cur.take(), &mut entries);
            // each alias between the bars is one key, which may be several words
            // (`herb of grace`); it is matched as a whole, never word by word.
            let keys = head
                .split('|')
                .map(|a| words(a).join(" "))
                .filter(|k| !k.is_empty())
                .collect();
            cur = Some((keys, vec![line.to_string()]));
        } else if let Some((_, lines)) = cur.as_mut() {
            lines.push(line.to_string());
        }
    }
    close(cur, &mut entries);
    Liber {
        liber: liber.to_string(),
        nomen,
        claves,
        entries,
    }
}

/// Loads every library under `<dir>/spiri72.scientia`, writing the starters first
/// if the folder is absent. A file that cannot be read is skipped, never fatal.
pub fn load(dir: &Path) -> Vec<Liber> {
    let home = dir.join(DIR);
    if !home.exists() && fs::create_dir_all(&home).is_ok() {
        for (name, body) in SEEDS {
            let _ = fs::write(home.join(name), body);
        }
    }
    let mut libri = Vec::new();
    let Ok(read) = fs::read_dir(&home) else {
        return libri;
    };
    let mut paths: Vec<_> = read
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|x| x == "txt"))
        .collect();
    paths.sort();
    for path in paths {
        let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else {
            continue;
        };
        if let Ok(text) = fs::read_to_string(&path) {
            libri.push(parse(stem, &text));
        }
    }
    libri
}

/// Whether a spirit whose offices read like `office_texts` may open a library:
/// one of the library's `claves` must stand in one of the spirit's offices.
pub fn open_to(liber: &Liber, office_texts: &[String]) -> bool {
    let office_words: Vec<String> = office_texts.iter().flat_map(|t| words(t)).collect();
    liber
        .claves
        .iter()
        .any(|c| office_words.iter().any(|w| same(w, c)))
}

/// The offices of a spirit, as the register records them, for gating.
fn office_texts(n: u32) -> Vec<String> {
    crate::persona::entry(n)
        .and_then(|e| e.get("offices"))
        .map(annulus::lingua::json::Json::list)
        .unwrap_or(&[])
        .iter()
        .filter_map(|o| {
            o.get("text")
                .and_then(annulus::lingua::json::Json::text)
                .map(str::to_string)
        })
        .collect()
}

/// Finds the one entry a question narrows to within a library: an entry whose key
/// stands as a whole word (or its singular) in the question. The key that matches
/// is returned as the term.
fn narrow<'a>(liber: &'a Liber, question: &[String]) -> Option<(&'a str, &'a Entry)> {
    for entry in &liber.entries {
        for key in &entry.claves {
            let kw = words(key);
            if kw.is_empty() {
                continue;
            }
            let hit = question
                .windows(kw.len())
                .any(|win| win.iter().zip(&kw).all(|(a, b)| same(a, b)));
            if hit {
                return Some((key.as_str(), entry));
            }
        }
    }
    None
}

/// Given the loaded libraries, a spirit and its question, returns the one entry
/// the spirit may consult, if any: a library open to the spirit by its offices,
/// holding an entry the question names. The first such match wins.
pub fn consult(libri: &[Liber], spirit: u32, question: &str) -> Option<Consultatio> {
    if libri.is_empty() {
        return None;
    }
    let offices = office_texts(spirit);
    let qwords = words(question);
    for liber in libri {
        if !open_to(liber, &offices) {
            continue;
        }
        if let Some((term, entry)) = narrow(liber, &qwords) {
            return Some(Consultatio {
                liber: liber.nomen.clone(),
                terminus: term.to_string(),
                textus: entry.raw.clone(),
            });
        }
    }
    None
}
