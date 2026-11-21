//! The concordance (`concordantia`): which words of a book mean the same, so
//! that the search for passages meets a question in the book's own words. No
//! model is asked; it is plain lookup.
//!
//! It has three sources:
//!
//! - **Word families** (`radix`): endings that make one word of another are cut
//!   away, so `digestion` and `digestive` both read `digest`, and `conserved`
//!   and `conservation` both read `conserv`.
//! - **What a book defines itself**: an abbreviation set after its words
//!   (`Single Responsibility Principle (SRP)`), and a plant's two Latin names set
//!   before its common name (`Hypericum perforatum (St. John's wort)`). These
//!   hold only where the book uses them more than once.
//! - **The Operator's own file**, `concordantia.txt` in the books' folder, one
//!   group to a line: `liver = hepatic`, `insomnia = sleeplessness`.
//!
//! Every word, in the passages and in the question alike, is set to the form
//! its group shares, so both meet on the same form. A group's first word or
//! phrase is that form; an abbreviation is the form of its phrase.

use std::collections::BTreeMap;

use crate::scientia;

/// The name of the Operator's concordance, in the books' folder.
pub const FILE: &str = "concordantia.txt";

/// The concordance written into a new books folder: equivalents of plain
/// language (English and American spellings, the Latin adjectives of the
/// body and the sky, the words of programming), never answers to questions.
pub const SEMEN: &str = "\
# The concordance: words that mean the same, one group to a line.
# A question and a book meet on the first word of the group.
# Write words or whole phrases, parted by `=` or by commas.
# Lines starting with # are notes. Spiri72 reads this at every start.

# spellings
behaviour = behavior
colour = color
centre = center
programme = program
analyse = analyze
haemorrhoid = hemorrhoid
oesophagus = esophagus
anaemia = anemia
tumour = tumor
oestrogen = estrogen
foetus = fetus
foetal = fetal
paediatric = pediatric
haemoglobin = hemoglobin

# the body
liver = hepatic
kidney = renal, nephric
heart = cardiac
lung = pulmonary
skin = cutaneous, dermal
stomach = gastric
bowel = intestine, intestinal
insomnia = sleeplessness
blood sugar = blood glucose

# the sky
sun = solar
moon = lunar
star = stellar

# programming
procedure = function
lambda = anonymous procedure, anonymous function
";

/// Parts of a plant: never taken for a plant's common name.
const PARTES: &[&str] = &[
    "aerial", "bark", "berry", "berries", "bud", "buds", "bulb", "flower", "flowers", "fruit",
    "gum", "herb", "leaf", "leaves", "oil", "plant", "resin", "rhizome", "root", "roots", "seed",
    "seeds", "stem", "whole", "wood",
];

/// How the second of a plant's Latin names ends (`perforatum`, `officinalis`,
/// `sativa`, `marianum`, `canadensis`).
const LATINA: &[&str] = &[
    "a", "ae", "ata", "ensis", "es", "i", "ica", "ii", "is", "ix", "ns", "oides", "on", "um", "us",
];

/// Short words skipped between an abbreviation's initials (`Law of Demeter`).
const PARVA: &[&str] = &["a", "an", "and", "for", "in", "of", "on", "the", "to"];

/// A word cut to the root its family shares, after `scientia`'s plain stem:
/// endings that make a noun of a verb or an adjective of a noun fall away
/// (`-ation`, `-ion`, `-ive`, `-ical`, `-al`, `-ity`, `-ness`, `-ment`, `-ic`)
/// and a last silent `e`, never leaving fewer than four letters.
pub fn radix(w: &str) -> String {
    const ENDINGS: [&str; 13] = [
        "ational", "ations", "ation", "itions", "ition", "ments", "ment", "ness", "ities", "ity",
        "ical", "ions", "ive",
    ];
    const SHORT: [&str; 4] = ["ion", "al", "ic", "e"];
    if !w.is_ascii() || w.chars().any(|c| c.is_ascii_digit()) {
        return w.to_string();
    }
    let mut w = w.to_string();
    for _ in 0..2 {
        let before = w.clone();
        for e in ENDINGS.iter().chain(SHORT.iter()) {
            if let Some(r) = w.strip_suffix(e) {
                if r.len() >= 4 {
                    w = r.to_string();
                    break;
                }
            }
        }
        if w == before {
            break;
        }
    }
    w
}

/// A text's words of matter, stemmed and cut to their roots, in order. The
/// words in `keep` are matter even where `scientia` would set them aside.
pub fn radices(text: &str, keep: &[&str]) -> Vec<String> {
    scientia::matter_in_order(text, keep)
        .iter()
        // a word that is empty of matter once stemmed (`makes`) stays empty
        .filter(|w| keep.contains(&w.as_str()) || !scientia::vacuum(w))
        .map(|w| radix(w))
        .collect()
}

/// The groups of words that mean the same, each set to its shared form.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Concordantia {
    /// One word to the form it is read as.
    verba: BTreeMap<String, String>,
    /// A phrase (roots in order) to the form it is read as, longest first.
    phrases: Vec<(Vec<String>, String)>,
}

impl Concordantia {
    /// Reads the Operator's file: each line a group of words or phrases parted
    /// by `=` or commas, its first the shared form.
    pub fn parse(text: &str, keep: &[&str]) -> Concordantia {
        let mut c = Concordantia::default();
        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let items: Vec<Vec<String>> = line
                .split(['=', ','])
                .map(|i| radices(i, keep))
                .filter(|i| !i.is_empty())
                .collect();
            let Some(first) = items.first() else { continue };
            let form = first.join("");
            for item in &items {
                c.add(item, &form);
            }
        }
        c.order();
        c
    }

    fn add(&mut self, item: &[String], form: &str) {
        match item {
            [] => {}
            [one] => {
                if one != form {
                    self.verba
                        .entry(one.clone())
                        .or_insert_with(|| form.to_string());
                }
            }
            many => {
                if !self.phrases.iter().any(|(p, _)| p == many) {
                    self.phrases.push((many.to_vec(), form.to_string()));
                }
            }
        }
    }

    fn order(&mut self) {
        self.phrases
            .sort_by(|a, b| b.0.len().cmp(&a.0.len()).then(a.0.cmp(&b.0)));
    }

    /// This concordance with another's groups added; where both set a word,
    /// this one's stands.
    pub fn with(&self, other: &Concordantia) -> Concordantia {
        let mut c = self.clone();
        for (w, f) in &other.verba {
            c.verba.entry(w.clone()).or_insert_with(|| f.clone());
        }
        for (p, f) in &other.phrases {
            c.add(p, f);
        }
        c.order();
        c
    }

    /// Whether it holds no group at all.
    pub fn is_empty(&self) -> bool {
        self.verba.is_empty() && self.phrases.is_empty()
    }

    /// How many words and phrases it sets.
    pub fn len(&self) -> usize {
        self.verba.len() + self.phrases.len()
    }

    /// The concordance as lines, one word or phrase to a line, a tab, then the
    /// form it is read as (words are roots: letters and digits, no tabs).
    pub fn lines(&self) -> String {
        let mut out = String::new();
        for (p, f) in &self.phrases {
            out.push_str(&format!("{}\t{f}\n", p.join(" ")));
        }
        for (w, f) in &self.verba {
            out.push_str(&format!("{w}\t{f}\n"));
        }
        out
    }

    /// Reads back what `lines` wrote; a line that is not word, tab, form is
    /// skipped.
    pub fn from_lines(text: &str) -> Concordantia {
        let mut c = Concordantia::default();
        for line in text.lines() {
            let Some((p, f)) = line.split_once('\t') else {
                continue;
            };
            let item: Vec<String> = p
                .split(' ')
                .filter(|w| !w.is_empty())
                .map(String::from)
                .collect();
            if !f.is_empty() && !f.contains(char::is_whitespace) {
                c.add(&item, f);
            }
        }
        c.order();
        c
    }

    /// Every word and phrase it sets, with the form each is read as.
    pub fn groups(&self) -> Vec<(String, String)> {
        let mut v: Vec<(String, String)> = self
            .phrases
            .iter()
            .map(|(p, f)| (p.join(" "), f.clone()))
            .chain(self.verba.iter().map(|(w, f)| (w.clone(), f.clone())))
            .collect();
        v.sort();
        v
    }

    /// Roots in order, each phrase of a group read as its shared form, then
    /// each word.
    pub fn apply(&self, seq: Vec<String>) -> Vec<String> {
        let seq = if self.phrases.is_empty() {
            seq
        } else {
            let mut out = Vec::with_capacity(seq.len());
            let mut i = 0;
            'next: while i < seq.len() {
                for (p, form) in &self.phrases {
                    if seq[i..].starts_with(p) {
                        out.push(form.clone());
                        i += p.len();
                        continue 'next;
                    }
                }
                out.push(seq[i].clone());
                i += 1;
            }
            out
        };
        seq.into_iter()
            .map(|w| self.verba.get(&w).cloned().unwrap_or(w))
            .collect()
    }
}

/// A word as written, without the marks around it.
fn bare(w: &str) -> &str {
    w.trim_matches(|c: char| !c.is_alphanumeric())
}

/// What a book defines itself: each abbreviation set in brackets after the
/// words it stands for, and each plant's two Latin names set before its common
/// name in brackets. Only what the book uses at least twice is kept, and never
/// an abbreviation that is also a plain word of the book.
pub fn of_book(pages: &[Option<String>], keep: &[&str]) -> Concordantia {
    let text: String = pages
        .iter()
        .flatten()
        .map(String::as_str)
        .collect::<Vec<_>>()
        .join("\n");
    let tokens: Vec<&str> = text.split_whitespace().collect();
    let mut lower: BTreeMap<String, usize> = BTreeMap::new();
    for t in &tokens {
        *lower.entry(bare(t).to_string()).or_default() += 1;
    }
    let seen = |w: &str| lower.get(w).copied().unwrap_or(0);
    // (phrase as roots, form) -> times found
    let mut found: BTreeMap<(Vec<String>, String), usize> = BTreeMap::new();
    for (i, t) in tokens.iter().enumerate() {
        let Some(inner) = t.strip_prefix('(') else {
            continue;
        };
        // an abbreviation: `(SRP)`, `(SRP),`
        let abbr = inner.trim_end_matches([')', ',', '.', ';', ':']);
        let closed = inner.contains(')');
        if closed
            && (2..=6).contains(&abbr.len())
            && abbr.chars().all(|c| c.is_ascii_uppercase())
            && seen(&abbr.to_lowercase()) < 3
            // set out once and used again
            && seen(abbr) + seen(&format!("({abbr})")) >= 2
        {
            let letters: Vec<char> = abbr.to_lowercase().chars().collect();
            let mut words: Vec<&str> = Vec::new();
            let mut j = i;
            let mut need = letters.len();
            while need > 0 && j > 0 && i - j < letters.len() * 2 + 2 {
                j -= 1;
                let w = bare(tokens[j]);
                if w.is_empty() || !w.chars().all(char::is_alphabetic) {
                    break;
                }
                let lw = w.to_lowercase();
                if PARVA.contains(&lw.as_str()) {
                    words.push(w);
                    continue;
                }
                if lw.starts_with(letters[need - 1]) {
                    words.push(w);
                    need -= 1;
                } else {
                    break;
                }
            }
            if need == 0 {
                words.reverse();
                let phrase = radices(&words.join(" "), keep);
                if phrase.len() >= 2 {
                    let form = radix(&abbr.to_lowercase());
                    *found.entry((phrase, form)).or_default() += 2;
                }
            }
            continue;
        }
        // a plant's names: `Hypericum perforatum (St. John's wort)`
        if i < 2 {
            continue;
        }
        let (genus, species) = (bare(tokens[i - 2]), bare(tokens[i - 1]));
        let latin = genus.len() >= 3
            && species.len() >= 3
            && LATINA.iter().any(|e| species.ends_with(e))
            // a genus is a name, not a plain word of the book
            && seen(&genus.to_lowercase()) < 2
            && genus.chars().next().is_some_and(|c| c.is_ascii_uppercase())
            && genus.chars().skip(1).all(|c| c.is_ascii_lowercase())
            && species.chars().all(|c| c.is_ascii_lowercase())
            && !tokens[i - 2].ends_with([',', '.', ';', ':'])
            && !tokens[i - 1].ends_with([',', '.', ';', ':']);
        if !latin {
            continue;
        }
        let mut name: Vec<&str> = Vec::new();
        for k in i..tokens.len().min(i + 4) {
            let w = if k == i { inner } else { tokens[k] };
            let (w, end) = match w.find(')') {
                Some(at) => (&w[..at], true),
                None => (w, false),
            };
            name.push(w);
            if end {
                break;
            }
            if k + 1 == tokens.len().min(i + 4) {
                name.clear();
            }
        }
        let name = name.join(" ");
        let plain = !name.is_empty()
            && name
                .chars()
                .all(|c| c.is_alphabetic() || " .'\u{2019}-".contains(c))
            && name.chars().any(char::is_lowercase);
        if !plain {
            continue;
        }
        let common = radices(&name, keep);
        let parts = common.iter().all(|w| PARTES.contains(&w.as_str()));
        if common.is_empty() || parts || common.len() > 4 {
            continue;
        }
        let form = format!("{}{}", genus.to_lowercase(), species);
        *found.entry((common, form.clone())).or_default() += 1;
        let both = radices(&format!("{genus} {species}"), keep);
        *found.entry((both, form)).or_default() += 1;
    }
    // each phrase to the form it was most often given, if given twice or more
    let mut best: BTreeMap<Vec<String>, (usize, String)> = BTreeMap::new();
    for ((phrase, form), n) in found {
        let e = best.entry(phrase).or_insert((0, String::new()));
        if n > e.0 {
            *e = (n, form);
        }
    }
    let mut c = Concordantia::default();
    for (phrase, (n, form)) in best {
        if n >= 2 {
            c.add(&phrase, &form);
        }
    }
    c.order();
    c
}
