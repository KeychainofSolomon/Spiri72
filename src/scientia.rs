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
//! library holds, or plainly describes it without naming it, the engine narrows
//! to that one entry and hands the spirit only that entry, a true copy sealed
//! like any other handed text. The search is plain arithmetic on words; no
//! model is asked, and a doubtful finding hands nothing. The
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
const SEEDS: [(&str, &str); 9] = [
    ("flora.txt", include_str!("../scientia/flora.txt")),
    ("astronomia.txt", include_str!("../scientia/astronomia.txt")),
    ("tarot.txt", include_str!("../scientia/tarot.txt")),
    ("divinatio.txt", include_str!("../scientia/divinatio.txt")),
    ("solomonica.txt", include_str!("../scientia/solomonica.txt")),
    ("angelica.txt", include_str!("../scientia/angelica.txt")),
    ("paedagogia.txt", include_str!("../scientia/paedagogia.txt")),
    ("litterae.txt", include_str!("../scientia/litterae.txt")),
    ("poetica.txt", include_str!("../scientia/poetica.txt")),
];

/// One keyed entry of a library.
#[derive(Clone, Debug)]
pub struct Entry {
    /// The names this entry answers to, lowercased (`barley`, `hordeum`, `bere`).
    pub claves: Vec<String>,
    /// The whole entry as written, heading and all, handed back verbatim.
    pub raw: String,
    /// The words of matter in the entry (keys and body), stemmed, sorted and
    /// distinct: what a question is weighed against when it names no key.
    pub verba: Vec<String>,
}

/// A whole library: its name, the matter it covers, and its entries.
#[derive(Clone, Debug)]
pub struct Liber {
    /// The slug, from the file name (`flora`).
    pub liber: String,
    /// The name shown to the Operator (`the herbal`).
    pub nomen: String,
    /// The matter this library covers, as words or whole phrases (`herb`,
    /// `things to come`); a spirit may read it only if one of its offices holds
    /// one of these, a phrase standing whole and in order within one office.
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

/// A crude singular: drop a trailing `s` from a word of more than four letters,
/// so `herbs` and `stars` meet `herb` and `star`. Short words and words in
/// `-ss`, `-us` and `-is` are left whole, so `ares` never meets `are`, nor
/// `mars` a `mar`, and `venus` and `sirius` stay names.
fn singular(w: &str) -> &str {
    if w.ends_with("ss") || w.ends_with("us") || w.ends_with("is") {
        return w;
    }
    match w.strip_suffix('s') {
        Some(stem) if stem.len() > 3 => stem,
        _ => w,
    }
}

fn same(a: &str, b: &str) -> bool {
    a == b || singular(a) == singular(b)
}

/// Whether `phrase` stands whole and in order within `text`, word for word.
fn stands_in(phrase: &[String], text: &[String]) -> bool {
    !phrase.is_empty()
        && text
            .windows(phrase.len())
            .any(|win| win.iter().zip(phrase).all(|(a, b)| same(a, b)))
}

/// Words that carry no matter: a question is never weighed by these.
const VACUA: &[&str] = &[
    "a", "about", "all", "also", "am", "an", "and", "any", "are", "as", "at", "be", "been", "but",
    "by", "call", "called", "can", "come", "could", "did", "do", "does", "done", "down", "for",
    "from", "get", "give", "go", "good", "had", "has", "have", "he", "her", "here", "him", "his",
    "how", "i", "if", "in", "into", "is", "it", "its", "kind", "know", "like", "made", "make",
    "may", "me", "might", "mine", "more", "most", "must", "my", "name", "no", "not", "of", "on",
    "one", "or", "our", "out", "over", "said", "say", "shall", "she", "should", "show", "so",
    "some", "speak", "such", "tell", "than", "that", "the", "thee", "their", "them", "then",
    "there", "these", "they", "thing", "this", "those", "thou", "thy", "to", "under", "up", "us",
    "use", "used", "very", "was", "we", "were", "what", "when", "where", "which", "who", "whom",
    "whose", "why", "will", "with", "would", "you", "your",
];

/// A word cut to a plain stem for weighing (never for naming a key): plurals
/// and the endings `-ing` and `-ed` fall away, so `flavours`, `flavoured` and
/// `flavouring` all meet `flavour`, and `equinoxes` meets `equinox`.
fn stem(w: &str) -> String {
    if !w.is_ascii() {
        return w.to_string();
    }
    let n = w.len();
    let w = if n > 4 && w.ends_with("ies") {
        format!("{}y", &w[..n - 3])
    } else if n > 4
        && ["xes", "ches", "shes", "sses"]
            .iter()
            .any(|e| w.ends_with(e))
    {
        w[..n - 2].to_string()
    } else if n > 3 && w.ends_with('s') && !w.ends_with("ss") {
        w[..n - 1].to_string()
    } else {
        w.to_string()
    };
    let n = w.len();
    if n > 5 && w.ends_with("ing") {
        w[..n - 3].to_string()
    } else if n > 4 && w.ends_with("ed") {
        w[..n - 2].to_string()
    } else {
        w
    }
}

/// The words of matter in a text, stemmed, sorted and distinct.
fn matter(text: &str) -> Vec<String> {
    let mut v: Vec<String> = words(text)
        .iter()
        .filter(|w| !VACUA.contains(&w.as_str()))
        .map(|w| stem(w))
        .collect();
    v.sort();
    v.dedup();
    v
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
            let raw = lines.join("\n");
            entries.push(Entry {
                claves: keys,
                verba: matter(&raw),
                raw,
            });
        }
    };
    for line in text.lines() {
        let trimmed = line.trim();
        if cur.is_none() && trimmed.starts_with('#') {
            if let Some((k, v)) = trimmed.trim_start_matches('#').split_once(':') {
                match k.trim() {
                    "nomen" => nomen = v.trim().to_string(),
                    // each clave between the commas is one word or one whole phrase
                    "claves" => {
                        claves = v
                            .split(',')
                            .map(|c| words(c).join(" "))
                            .filter(|c| !c.is_empty())
                            .collect();
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
/// one of the library's `claves` must stand whole in one of the spirit's offices.
/// A phrase never reaches across two offices.
pub fn open_to(liber: &Liber, office_texts: &[String]) -> bool {
    let offices: Vec<Vec<String>> = office_texts.iter().map(|t| words(t)).collect();
    liber.claves.iter().any(|c| {
        let phrase = words(c);
        offices.iter().any(|o| stands_in(&phrase, o))
    })
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

/// Whether a library opens to spirit `n`, by the offices the register records.
pub fn opens(liber: &Liber, n: u32) -> bool {
    open_to(liber, &office_texts(n))
}

/// The least weight of shared matter at which an unnamed entry is handed. A
/// word held by one entry alone weighs a little under one; a word held by
/// every entry weighs nothing.
const PONDUS_MINIMUM: f64 = 1.5;

/// The least part of the question's matter the entry must hold, by weight. A
/// word no open library holds counts against it (`IGNOTI_PONDUS`): a question
/// mostly about things these libraries do not know is not theirs to answer.
const PARS_MINIMA: f64 = 0.5;

/// What a word unknown to every open library weighs against a finding.
const IGNOTI_PONDUS: f64 = 1.0;

/// How rare a word of an entry's own key must be for that one word to find the
/// entry: `villanelle` finds `the villanelle`, but `world` not `the world`.
const NOMINIS_RARITAS: f64 = 0.75;

/// How far the best entry must stand above the next (the next may weigh at
/// most this part of the best), or nothing is handed: a near tie is no finding.
const SECUNDI_PARS: f64 = 0.8;

/// What a question is weighed with: its words as asked, its words of matter,
/// and how rare each of those is among the entries of the open libraries (a
/// closed library weighs nothing, even here).
struct Rogatum {
    verba: Vec<String>,
    materia: Vec<String>,
    raritas: Vec<f64>,
}

impl Rogatum {
    fn new(libri: &[&Liber], question: &str) -> Rogatum {
        let materia = matter(question);
        let total = libri.iter().map(|l| l.entries.len()).sum::<usize>() as f64;
        let raritas = materia
            .iter()
            .map(|w| {
                let held = libri
                    .iter()
                    .flat_map(|l| &l.entries)
                    .filter(|e| e.verba.binary_search(w).is_ok())
                    .count() as f64;
                if held == 0.0 {
                    0.0
                } else {
                    ((total + 1.0) / (held + 1.0)).ln() / (total + 1.0).ln()
                }
            })
            .collect();
        Rogatum {
            verba: words(question),
            materia,
            raritas,
        }
    }

    /// The weight of the matter an entry shares with the question (rare words
    /// weigh more), and how many of the question's words it shares.
    fn weigh(&self, entry: &Entry) -> (f64, usize) {
        self.materia
            .iter()
            .zip(&self.raritas)
            .filter(|(w, _)| entry.verba.binary_search(w).is_ok())
            .fold((0.0, 0), |(p, n), (_, r)| (p + r, n + 1))
    }

    /// The weight of all the question's matter, unknown words included.
    fn whole(&self) -> f64 {
        self.raritas
            .iter()
            .map(|r| if *r == 0.0 { IGNOTI_PONDUS } else { *r })
            .sum()
    }

    /// Whether the question holds a rare word of one of the entry's own keys.
    fn half_names(&self, entry: &Entry) -> bool {
        let keys: Vec<String> = entry
            .claves
            .iter()
            .flat_map(|k| words(k))
            .map(|w| stem(&w))
            .collect();
        self.materia
            .iter()
            .zip(&self.raritas)
            .any(|(w, r)| *r >= NOMINIS_RARITAS && keys.contains(w))
    }

    /// How many of the question's words, outside the key itself, speak of the
    /// library's own matter (its claves or its name): `card` beside `the sun`.
    fn context(&self, liber: &Liber, key: &[String]) -> usize {
        let mut own: Vec<String> = liber
            .claves
            .iter()
            .flat_map(|c| words(c))
            .filter(|w| !VACUA.contains(&w.as_str()))
            .collect();
        own.push(liber.liber.to_lowercase());
        let mut seen: Vec<&String> = self
            .verba
            .iter()
            .filter(|w| !key.iter().any(|k| same(w, k)))
            .filter(|w| own.iter().any(|o| same(w, o)))
            .collect();
        seen.sort();
        seen.dedup();
        seen.len()
    }
}

/// A key's words without a leading `the`.
fn bare(key: &[String]) -> &[String] {
    match key.split_first() {
        Some((first, rest)) if first == "the" && !rest.is_empty() => rest,
        _ => key,
    }
}

/// Finds the entry a question names, among the open libraries: an entry whose
/// key stands as a whole phrase in the question. Where several are named, the
/// longer key wins (`elder futhark` over `elder`), then the library the rest of
/// the question speaks of (`the sun` of the cards, when cards are spoken of),
/// then the entry sharing more of the question's matter, then the first held.
/// A key that is only a number (`13`) is no naming unless the question also
/// speaks of that library's matter.
fn named(open: &[&Liber], shut: &[&Liber], q: &Rogatum) -> Option<Consultatio> {
    let mut best: Option<((usize, usize, f64), Consultatio)> = None;
    for liber in open {
        for entry in &liber.entries {
            for key in &entry.claves {
                let kw = words(key);
                if !stands_in(&kw, &q.verba) {
                    continue;
                }
                let context = q.context(liber, &kw);
                let number = kw.iter().all(|w| w.chars().all(|c| c.is_ascii_digit()));
                if number && context == 0 {
                    continue;
                }
                // The same name in a closed library, when the question speaks of
                // that library's matter and not of this one's, is not this
                // spirit's to answer: the moon of the cards is not the moon.
                let bare = bare(&kw);
                let foreign = context == 0
                    && shut.iter().any(|l| {
                        q.context(l, &kw) > 0
                            && l.entries
                                .iter()
                                .flat_map(|e| &e.claves)
                                .any(|k| bare == self::bare(&words(k)))
                    });
                if foreign {
                    continue;
                }
                let length = bare.len();
                let rank = (length, context, q.weigh(entry).0);
                if best.as_ref().is_none_or(|(b, _)| rank > *b) {
                    best = Some((
                        rank,
                        Consultatio {
                            liber: liber.nomen.clone(),
                            terminus: key.clone(),
                            textus: entry.raw.clone(),
                        },
                    ));
                }
            }
        }
    }
    best.map(|(_, c)| c)
}

/// Finds the entry a question describes without naming, among the open
/// libraries: the one sharing the most matter with the question, weighed by
/// rarity. It is handed only if it shares at least two words and weighs enough
/// (or the question holds a rare word of its own key), holds a fair part of the
/// question's matter, and stands clear of the next; otherwise nothing is
/// handed, for a wrong entry is worse than none. The term is its first key.
fn described(open: &[&Liber], q: &Rogatum) -> Option<Consultatio> {
    let mut best: Option<(f64, &Liber, &Entry)> = None;
    let mut second = 0.0_f64;
    for liber in open {
        for entry in &liber.entries {
            let (weight, shared) = q.weigh(entry);
            let enough = shared >= 2 && weight >= PONDUS_MINIMUM;
            if shared == 0 || !(enough || q.half_names(entry)) {
                second = second.max(weight);
                continue;
            }
            match best {
                Some((b, _, _)) if weight <= b => second = second.max(weight),
                Some((b, _, _)) => {
                    second = b;
                    best = Some((weight, liber, entry));
                }
                None => best = Some((weight, liber, entry)),
            }
        }
    }
    let (weight, liber, entry) = best?;
    if weight < q.whole() * PARS_MINIMA || second > weight * SECUNDI_PARS {
        return None;
    }
    Some(Consultatio {
        liber: liber.nomen.clone(),
        terminus: entry.claves.first()?.clone(),
        textus: entry.raw.clone(),
    })
}

/// Given the loaded libraries, a spirit and its question, returns the one entry
/// the spirit may consult, if any, from a library open to the spirit by its
/// offices: the entry the question names, or failing that the one it plainly
/// describes. No model is asked; a closed library is never searched.
pub fn consult(libri: &[Liber], spirit: u32, question: &str) -> Option<Consultatio> {
    let offices = office_texts(spirit);
    let (open, shut): (Vec<&Liber>, Vec<&Liber>) = libri.iter().partition(|l| open_to(l, &offices));
    if open.is_empty() {
        return None;
    }
    let q = Rogatum::new(&open, question);
    named(&open, &shut, &q).or_else(|| described(&open, &q))
}
