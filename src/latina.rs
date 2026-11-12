//! Classical Latin, read deterministically.
//!
//! The lexicon is William Whitaker's WORDS (see `lingua/latina/LICENCE.txt`),
//! from archaic to Renaissance Latin (ages X and A to G; `tools/latina.py`).
//! Classical readings come first; a reading that rests on a later entry or
//! ending carries its period (`Lectio::aetas`). A word is
//! read the way WORDS reads it: every split into a stem and an ending is
//! tried, and a split stands when the stem belongs to an entry whose part of
//! speech, declension or conjugation, gender and stem key all fit the ending.
//! Nothing is guessed: a word that fits no entry is reported as not found.

use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;

const VERBA: &str = include_str!("../lingua/latina/verba.tsv");
const FLEXIONES: &str = include_str!("../lingua/latina/flexiones.tsv");
const SINGULARIA: &str = include_str!("../lingua/latina/singularia.tsv");

/// The most distinct words one reading aid explains.
pub const MAX_WORDS: usize = 40;

struct Verbum {
    pos: &'static str,
    which: &'static str,
    variant: &'static str,
    codes: Vec<&'static str>,
    age: &'static str,
    freq: &'static str,
    lemma: &'static str,
    kind: &'static str,
    meaning: &'static str,
}

struct Flexio {
    pos: &'static str,
    which: &'static str,
    variant: &'static str,
    key: u8,
    fields: Vec<&'static str>,
    age: &'static str,
    freq: &'static str,
}

struct Singulare {
    form: &'static str,
    pos: &'static str,
    fields: &'static str,
    age: &'static str,
    meaning: &'static str,
}

struct Lexicon {
    verba: Vec<Verbum>,
    stems: HashMap<&'static str, Vec<(usize, u8)>>,
    flexiones: Vec<Flexio>,
    endings: HashMap<&'static str, Vec<usize>>,
    singularia: Vec<Singulare>,
}

/// One reading of a word: its dictionary form, what it is, the form it
/// stands in, and the lexicon's meaning.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Lectio {
    pub lemma: String,
    pub kind: String,
    pub form: String,
    pub meaning: String,
    /// The period, when the reading is not classical: "late", "later",
    /// "medieval" or "scholarly". `None` for classical Latin.
    pub aetas: Option<&'static str>,
}

fn rows(text: &'static str) -> impl Iterator<Item = Vec<&'static str>> {
    text.lines()
        .filter(|l| !l.starts_with('#') && !l.is_empty())
        .map(|l| l.split('\t').collect())
}

fn lexicon() -> &'static Lexicon {
    static L: OnceLock<Lexicon> = OnceLock::new();
    L.get_or_init(|| {
        let mut verba = Vec::new();
        let mut stems: HashMap<&'static str, Vec<(usize, u8)>> = HashMap::new();
        for r in rows(VERBA) {
            let i = verba.len();
            for (k, s) in r[..4].iter().enumerate() {
                let indexed = *s != "-" && (!s.is_empty() || r[4] == "V");
                if indexed {
                    stems.entry(*s).or_default().push((i, k as u8 + 1));
                }
            }
            verba.push(Verbum {
                pos: r[4],
                which: r[5],
                variant: r[6],
                codes: r[7].split(' ').filter(|c| !c.is_empty()).collect(),
                age: r[8],
                freq: r[9],
                lemma: r[10],
                kind: r[11],
                meaning: r[12],
            });
        }
        let mut flexiones = Vec::new();
        let mut endings: HashMap<&'static str, Vec<usize>> = HashMap::new();
        for r in rows(FLEXIONES) {
            endings.entry(r[4]).or_default().push(flexiones.len());
            flexiones.push(Flexio {
                pos: r[0],
                which: r[1],
                variant: r[2],
                key: r[3].parse().unwrap_or(0),
                fields: r[5].split(' ').filter(|c| !c.is_empty()).collect(),
                age: r[6],
                freq: r[7],
            });
        }
        let singularia = rows(SINGULARIA)
            .map(|r| Singulare {
                form: r[0],
                pos: r[1],
                fields: r[2],
                age: r[3],
                meaning: r[5],
            })
            .collect();
        Lexicon {
            verba,
            stems,
            flexiones,
            endings,
            singularia,
        }
    })
}

/// Lower case, `j` read as `i` and `v` as `u`, ligatures and accents undone,
/// as the lexicon is spelled.
pub fn norm(word: &str) -> String {
    let mut out = String::with_capacity(word.len());
    for c in word.chars().flat_map(char::to_lowercase) {
        match c {
            'j' => out.push('i'),
            'v' => out.push('u'),
            'æ' | 'ǽ' | 'ę' => out.push_str("ae"),
            'œ' => out.push_str("oe"),
            'á' | 'à' | 'â' | 'ä' | 'ā' | 'ă' => out.push('a'),
            'é' | 'è' | 'ê' | 'ë' | 'ē' | 'ĕ' => out.push('e'),
            'í' | 'ì' | 'î' | 'ï' | 'ī' | 'ĭ' => out.push('i'),
            'ó' | 'ò' | 'ô' | 'ö' | 'ō' | 'ŏ' => out.push('o'),
            'ú' | 'ù' | 'û' | 'ü' | 'ū' | 'ŭ' => out.push('u'),
            'ý' | 'ȳ' => out.push('y'),
            c if c.is_ascii_lowercase() => out.push(c),
            _ => {}
        }
    }
    out
}

fn gender_fits(inflection: &str, entry: &str) -> bool {
    match (inflection, entry) {
        ("X", _) | (_, "X") => true,
        ("C", e) => matches!(e, "M" | "F" | "C"),
        (i, "C") => matches!(i, "M" | "F"),
        (i, e) => i == e,
    }
}

fn fits(f: &Flexio, v: &Verbum) -> bool {
    let pos = match f.pos {
        "VPAR" | "SUPINE" => v.pos == "V",
        p => p == v.pos,
    };
    if !pos {
        return false;
    }
    if (f.which != "0" && f.which != v.which) || (f.variant != "0" && f.variant != v.variant) {
        return false;
    }
    match f.pos {
        "N" => f
            .fields
            .get(2)
            .zip(v.codes.first())
            .is_none_or(|(i, e)| gender_fits(i, e)),
        "PREP" | "ADV" => f.fields.first() == v.codes.first(),
        "V" => !(v.codes.first() == Some(&"DEP") && f.fields.get(1) == Some(&"ACTIVE")),
        _ => true,
    }
}

fn word(code: &str) -> &str {
    match code {
        "NOM" => "nominative",
        "VOC" => "vocative",
        "GEN" => "genitive",
        "LOC" => "locative",
        "DAT" => "dative",
        "ABL" => "ablative",
        "ACC" => "accusative",
        "S" => "singular",
        "P" => "plural",
        "M" => "masculine",
        "F" => "feminine",
        "N" => "neuter",
        "C" => "masculine/feminine",
        "POS" => "positive",
        "COMP" => "comparative",
        "SUPER" => "superlative",
        "PRES" => "present",
        "IMPF" => "imperfect",
        "FUT" => "future",
        "PERF" => "perfect",
        "PLUP" => "pluperfect",
        "FUTP" => "future perfect",
        "ACTIVE" => "active",
        "PASSIVE" => "passive",
        "IND" => "indicative",
        "SUB" => "subjunctive",
        "IMP" => "imperative",
        "INF" => "infinitive",
        "PPL" => "participle",
        "1" => "1st person",
        "2" => "2nd person",
        "3" => "3rd person",
        "0" | "X" => "",
        other => other,
    }
}

fn form_of(pos: &str, fields: &[&str], key: u8) -> String {
    if pos == "ADV" || pos == "PREP" {
        return match (fields.first(), key) {
            (Some(&"X"), 2) | (Some(&"COMP"), _) => "comparative".into(),
            (Some(&"X"), 3) | (Some(&"SUPER"), _) => "superlative".into(),
            _ => "invariable".into(),
        };
    }
    let mut parts: Vec<&str> = fields.iter().map(|c| word(c)).collect();
    if pos == "N" || pos == "PRON" {
        parts.truncate(2);
    }
    if pos == "SUPINE" {
        parts.push("supine");
    }
    if pos == "NUM" {
        parts.truncate(3);
    }
    parts.retain(|p| !p.is_empty());
    let joined = parts.join(" ");
    if joined.is_empty() {
        "invariable".into()
    } else {
        joined
    }
}

/// The later of two ages, as a period name; `None` if both are classical.
fn aetas(a: &'static str, b: &'static str) -> Option<&'static str> {
    // X means "any age", so it never makes a reading later.
    let age = |x: &'static str| if x == "X" { "" } else { x };
    match age(a).max(age(b)) {
        "D" => Some("late"),
        "E" => Some("later"),
        "F" => Some("medieval"),
        "G" => Some("scholarly"),
        _ => None,
    }
}

fn rank(freq: &str) -> u8 {
    freq.bytes().next().map_or(9, |b| b.saturating_sub(b'A'))
}

fn analyse_plain(w: &str) -> Vec<Lectio> {
    let lx = lexicon();
    let mut found: Vec<(u8, Lectio)> = Vec::new();
    for s in &lx.singularia {
        if s.form == w {
            let period = aetas(s.age, "X");
            found.push((
                if period.is_some() { 100 } else { 0 },
                Lectio {
                    lemma: s.form.to_string(),
                    kind: format!("irregular {}", s.pos.to_lowercase()),
                    form: s.fields.to_lowercase(),
                    meaning: s.meaning.to_string(),
                    aetas: period,
                },
            ));
        }
    }
    for cut in 0..=w.len() {
        if !w.is_char_boundary(cut) {
            continue;
        }
        let (stem, ending) = w.split_at(cut);
        let (Some(fs), Some(vs)) = (lx.endings.get(ending), lx.stems.get(stem)) else {
            continue;
        };
        for &fi in fs {
            let f = &lx.flexiones[fi];
            for &(vi, key) in vs {
                let v = &lx.verba[vi];
                if key != f.key || !fits(f, v) {
                    continue;
                }
                let period = aetas(v.age, f.age);
                found.push((
                    rank(v.freq) * 10 + rank(f.freq) + if period.is_some() { 100 } else { 0 },
                    Lectio {
                        lemma: v.lemma.to_string(),
                        kind: v.kind.to_string(),
                        form: form_of(f.pos, &f.fields, f.key),
                        meaning: v.meaning.to_string(),
                        aetas: period,
                    },
                ));
            }
        }
    }
    found.sort_by_key(|(r, _)| *r);
    let mut seen = HashSet::new();
    found
        .into_iter()
        .map(|(_, l)| l)
        .filter(|l| seen.insert((l.lemma.clone(), l.form.clone(), l.meaning.clone())))
        .collect()
}

const TACKONS: [(&str, &str); 3] = [("que", "and"), ("ne", "question"), ("ue", "or")];

/// Every reading of one word: classical readings first, then later ones,
/// each by frequency. Empty if the lexicon holds none.
pub fn analyse(word: &str) -> Vec<Lectio> {
    let w = norm(word);
    if w.is_empty() {
        return Vec::new();
    }
    let plain = analyse_plain(&w);
    if !plain.is_empty() {
        return plain;
    }
    for (t, sense) in TACKONS {
        if let Some(base) = w.strip_suffix(t).filter(|b| b.len() > 1) {
            let found: Vec<Lectio> = analyse_plain(base)
                .into_iter()
                .map(|mut l| {
                    l.form = format!("{} + -{t} ({sense})", l.form);
                    l
                })
                .collect();
            if !found.is_empty() {
                return found;
            }
        }
    }
    Vec::new()
}

fn period(l: &Lectio) -> String {
    l.aetas
        .map(|a| format!("; {a} Latin, not classical"))
        .unwrap_or_default()
}

fn short(meaning: &str, max: usize) -> String {
    let m = meaning.trim();
    if m.chars().count() <= max {
        return m.trim_end_matches(';').to_string();
    }
    let cut: String = m.chars().take(max).collect();
    let end = cut
        .rfind(';')
        .or_else(|| cut.rfind(','))
        .unwrap_or(cut.len());
    format!("{}…", cut[..end].trim_end())
}

fn tokens(text: &str) -> Vec<&str> {
    text.split(|c: char| !c.is_alphabetic())
        .filter(|t| !t.is_empty())
        .collect()
}

/// Whether `text` is mostly Latin: at least half its words of two letters
/// or more are found in the lexicon.
pub fn is_latin(text: &str) -> bool {
    let (mut total, mut latin) = (0usize, 0usize);
    for t in tokens(text) {
        let w = norm(t);
        if w.chars().count() < 2 {
            continue;
        }
        total += 1;
        if !analyse(&w).is_empty() {
            latin += 1;
        }
    }
    total > 0 && latin * 2 >= total
}

/// A reading aid for every Latin text among `texts`, with each word
/// explained once. `None` when none of them is Latin.
pub fn glossa_of(texts: &[&str]) -> Option<String> {
    let latin: Vec<&str> = texts.iter().copied().filter(|t| is_latin(t)).collect();
    if latin.is_empty() {
        return None;
    }
    glossa(&latin.join("\n"))
}

/// A reading aid for `text`: each distinct Latin word with its readings.
/// `None` when the text is not mostly Latin, so English is left alone.
pub fn glossa(text: &str) -> Option<String> {
    let mut order: Vec<String> = Vec::new();
    let mut seen = HashSet::new();
    let (mut total, mut latin) = (0usize, 0usize);
    let mut readings: HashMap<String, Vec<Lectio>> = HashMap::new();
    for t in tokens(text) {
        let w = norm(t);
        if w.chars().count() < 2 {
            continue;
        }
        total += 1;
        let r = readings.entry(w.clone()).or_insert_with(|| analyse(&w));
        if !r.is_empty() {
            latin += 1;
        }
        if seen.insert(w.clone()) {
            order.push(w);
        }
    }
    if total == 0 || latin * 2 < total {
        return None;
    }
    let mut out = String::new();
    let mut unknown = Vec::new();
    for w in order.iter().take(MAX_WORDS) {
        let r = &readings[w];
        let Some(first) = r.first() else {
            unknown.push(w.as_str());
            continue;
        };
        out.push_str(&format!(
            "{w}: {} ({}{}), {}: {}\n",
            first.lemma,
            first.kind,
            period(first),
            first.form,
            short(&first.meaning, 110)
        ));
        let mut lemmas = vec![(first.lemma.as_str(), first.meaning.as_str())];
        let mut words = Vec::new();
        let mut forms = Vec::new();
        for l in r.iter().skip(1) {
            if lemmas.contains(&(l.lemma.as_str(), l.meaning.as_str())) {
                if l.lemma == first.lemma
                    && l.form != first.form
                    && !forms.contains(&l.form)
                    && forms.len() < 3
                {
                    forms.push(l.form.clone());
                }
            } else if words.len() < 3 {
                lemmas.push((l.lemma.as_str(), l.meaning.as_str()));
                words.push(format!(
                    "{} ({}{}), {}: {}",
                    l.lemma,
                    l.kind,
                    period(l),
                    l.form,
                    short(&l.meaning, 50)
                ));
            }
        }
        if !forms.is_empty() {
            out.push_str(&format!("  or as: {}\n", forms.join("; ")));
        }
        for w in words {
            out.push_str(&format!("  or the word {w}\n"));
        }
    }
    if !unknown.is_empty() {
        out.push_str(&format!("Not in the lexicon: {}\n", unknown.join(", ")));
    }
    if order.len() > MAX_WORDS {
        out.push_str(&format!(
            "({} further words are not explained.)\n",
            order.len() - MAX_WORDS
        ));
    }
    Some(out)
}
