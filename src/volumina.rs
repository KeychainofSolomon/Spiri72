//! Whole books (`volumina`): the Operator's own books, from which a spirit may
//! be handed the book's own passages.
//!
//! The books lie under `spiri72.volumina/` in the Operator's folder, beside
//! `spiri72.scientia`, as PDF or plain text. They are the Operator's and never
//! part of this program. Beside each book lies a short header file of the same
//! name ending in `.claves`:
//!
//! ```text
//! # nomen: Medical Herbalism (Hoffmann, 2003)
//! # claves: herb, plant, medicinal
//! # aetas: modern
//! ```
//!
//! A book with no such file, or with no claves in it, opens to no spirit and is
//! not read at all: what is not granted is denied. A book opens to a spirit only
//! where one of its claves stands whole in one of the spirit's offices, exactly
//! as a library does.
//!
//! A PDF is read once, page by page, with the reader in `liber.rs` (its own
//! text only: no eyes, no model). What was read is kept under
//! `spiri72.volumina/.lecta/`, so a book is read again only when its file
//! changes. A page that is only a picture is counted and reported as unread.
//! Each page is then cut into passages of a few paragraphs, each knowing its
//! page.
//!
//! Finding is the arithmetic of `scientia`: the question's words of matter,
//! weighed by their rarity among the passages of the books open to this spirit.
//! A passage is handed only if it shares enough rare matter and a fair part of
//! the question; if too many passages answer equally, the match is doubtful
//! and nothing is handed. At most three passages are handed, within a set
//! number of characters, each a true copy marked with its book and page. A
//! book can hold any text, so every passage is handed fenced and sealed as
//! untrusted text, and shown as it stands to the Operator.

use std::fs;
use std::path::Path;
use std::time::UNIX_EPOCH;

use crate::concordantia::{self, Concordantia};
use crate::scientia::{self, Entry, Liber, Rogatum};

/// The folder, under the Operator's folder, that holds the books.
pub const DIR: &str = "spiri72.volumina";

/// The folder, within `DIR`, where what was read of each PDF is kept.
pub const LECTA: &str = ".lecta";

/// The note written once into a new books folder, telling how it is used.
pub const LEGE: &str = "LEGE.md";

/// How many characters of passages are handed in one rite, unless the Operator
/// sets another number (`book-characters = N` in `spiri72.ini`).
pub const SIGNA: usize = 3000;

/// The bounds of that setting.
pub const SIGNA_MINIMA: usize = 1500;
pub const SIGNA_MAXIMA: usize = 9000;

/// The most passages handed in one rite.
pub const PASSUS_MAXIMI: usize = 3;

/// How many places of a book (or of the open books) may answer nearly as well
/// as the best before the match is held doubtful: a question that answers
/// equally well all over the books is answered by none of them. Passages that
/// stand near each other in one book, each within `VICINIA` pages of the next,
/// are one place, a section speaking of its matter, and no sign of doubt.
const TURBA: usize = 7;

/// How many pages apart two passages may stand and still be one place (a
/// section may run over several pages); in a book without pages, how many
/// passages apart.
const VICINIA: u32 = 4;
const VICINIA_PASSUUM: u32 = 12;

/// The least part of the question's matter, by weight, that a passage must
/// hold (a word no open book holds weighs a whole word against it).
const PARS_LIBRI: f64 = 0.55;

/// The least weight of rare matter a passage must share, however short the
/// question: two words of middling rarity, or one very rare word alone.
const PONDUS_LIBRI: f64 = 0.6;

/// How many of a passage's first lines may be its heading.
const CAPITA_PRIMA: usize = 2;

/// The longest line, in words, taken for a heading.
const CAPUT_MAXIMUM: usize = 8;

/// The most words of matter a heading may hold to name the question's subject
/// by its one rarest word (`Psoriasis`, `Herbs for Psoriasis`).
const CAPUT_NOMINIS: usize = 3;

/// How rare the question's rarest word must be to stand as its subject.
const RARISSIMI: f64 = 0.35;

/// How far a passage must weigh above the next before it stands alone: below
/// this part of the best, a passage is no rival.
const SECUNDI_LIBRI: f64 = 0.8;

/// Words that `scientia` sets aside as asking, which in a long book are
/// matter: a question of how to name a thing is about naming.
const NOMINANDI: &[&str] = &["name", "named", "names", "call", "called", "use", "used"];

/// Words of asking that carry no matter in a book, beside those of `scientia`:
/// among them the orders given to the spirit (`write`, `teach`, `explain`) and
/// the frame of a comparison (`the difference between`).
const ROGANDI: &[&str] = &[
    "best",
    "better",
    "between",
    "compare",
    "compose",
    "describe",
    "difference",
    "differences",
    "explain",
    "just",
    "list",
    "look",
    "many",
    "mean",
    "meaning",
    "means",
    "much",
    "need",
    "please",
    "really",
    "right",
    "teach",
    "think",
    "versus",
    "want",
    "way",
    "ways",
    "write",
    "wrong",
];

/// A book larger than this is not read.
const MOLES_MAXIMA: u64 = 512 * 1024 * 1024;

/// The first line of a kept reading; a file that opens otherwise is read anew.
const SIGNUM_LECTI: &str = "spiri72 lecta 1";

const LEGE_TEXT: &str = "\
# spiri72.volumina

Whole books for the spirits. Put a book here as a PDF or a plain text file.

Beside each book put a small text file with the same name, ending in
`.claves`. For `herbal.pdf` that is `herbal.claves`:

    # nomen: Medical Herbalism (Hoffmann, 2003)
    # claves: herb, plant, medicinal
    # aetas: modern

- `nomen` is the name shown with every passage.
- `claves` are the words or whole phrases that open the book: a spirit may
  read it only if one of them stands in one of that spirit's offices.
- `aetas` is optional. Write `modern` for a book of the present age; its
  passages are then marked so, and never passed off as ancient.

A book without a `.claves` file opens to no spirit and is not read.

`concordantia.txt` holds words that mean the same, one group to a line
(`liver = hepatic`). A question and a book meet on the group's first word.
Spiri72 also learns from each book the abbreviations and plant names it
sets out itself, such as `Single Responsibility Principle (SRP)`.

A PDF is read once; what was read is kept in `.lecta` and used until the
book's file changes. Delete `.lecta` to have every book read anew. Pages
that are only pictures (scans) are not read; Spiri72 says how many.
";

/// One book: its header, and its passages as the entries of a `Liber`, so the
/// arithmetic of `scientia` weighs them unchanged. A passage has no keys: it
/// is never named, only described.
#[derive(Clone, Debug)]
pub struct Volumen {
    /// `liber.nomen` is the book's shown name, `liber.claves` its gate, and
    /// each of `liber.entries` one passage (`raw` is the passage as it stands).
    pub liber: Liber,
    /// Where each passage stands, in step with `liber.entries`.
    pub loci: Vec<Locus>,
    /// Each passage's words of matter, stemmed, in the order they stand: for
    /// counting how often a word recurs and which words stand side by side.
    pub ordines: Vec<Vec<String>>,
    /// The heading lines of each passage (short lines that do not end a
    /// sentence), as words of matter: a passage headed by the question's own
    /// matter is the place the book treats it.
    pub capita: Vec<Vec<Vec<String>>>,
    /// How this book's words are read: the Operator's concordance and what the
    /// book defines itself. A question is read the same way.
    pub concordantia: Concordantia,
    /// How many passages were set aside as tables (contents, index).
    pub tabulae: usize,
    /// The age the Operator marked the book with (`modern`), if any.
    pub aetas: Option<String>,
    /// How many pages the book has, and how many were only pictures.
    pub paginae: usize,
    pub pictae: usize,
}

/// Where a passage stands in its book.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Locus {
    /// The page, counted from one; none for a text file without page breaks.
    pub pagina: Option<u32>,
    /// The passage's number in the whole book, counted from one.
    pub ordo: u32,
}

/// One passage handed to a spirit in a rite, kept for the Operator to see.
#[derive(Clone, Debug, PartialEq)]
pub struct Lectio {
    /// The book's shown name.
    pub liber: String,
    /// The book's marked age (`modern`), if any.
    pub aetas: Option<String>,
    /// Where it stands, in words (`page 31`, or `part 12` where no pages are known).
    pub locus: String,
    /// The passage as it stands in the book, a true copy.
    pub textus: String,
}

/// What a book's `.claves` file says.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Titulus {
    pub nomen: Option<String>,
    pub claves: Vec<String>,
    pub aetas: Option<String>,
}

/// One line of the Operator's own header, made safe to show: a single line
/// without control characters, of bounded length.
fn line(s: &str) -> String {
    purga(s)
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
        .take(120)
        .collect()
}

/// Reads a `.claves` file: lines of `# key: value`. Anything else is ignored.
pub fn titulus(text: &str) -> Titulus {
    let mut t = Titulus::default();
    for l in text.lines() {
        let Some((k, v)) = l.trim().trim_start_matches('#').split_once(':') else {
            continue;
        };
        let v = line(v);
        if v.is_empty() {
            continue;
        }
        match k.trim() {
            "nomen" => t.nomen = Some(v),
            "aetas" => t.aetas = Some(v),
            "claves" => {
                t.claves = v
                    .split(',')
                    .map(|c| scientia::words(c).join(" "))
                    .filter(|c| !c.is_empty())
                    .collect();
            }
            _ => {}
        }
    }
    t
}

/// A book's text made plain: line ends alike, tabs as spaces, joined letters
/// spelled out, and no control
/// or invisible steering characters at all (a book can hold any text, and its
/// passages are shown to the Operator). Only the line break is kept.
pub fn purga(text: &str) -> String {
    text.replace("\r\n", "\n")
        // letters printed joined (`ﬁ`, `ﬀ`) are spelled out, so `deﬁnition`
        // meets `definition`
        .replace('\u{fb00}', "ff")
        .replace('\u{fb01}', "fi")
        .replace('\u{fb02}', "fl")
        .replace('\u{fb03}', "ffi")
        .replace('\u{fb04}', "ffl")
        .replace(['\u{fb05}', '\u{fb06}'], "st")
        .chars()
        .filter_map(|c| match c {
            '\n' => Some('\n'),
            '\t' | '\r' | '\u{a0}' => Some(' '),
            c if c.is_control() => None,
            '\u{200b}'..='\u{200f}'
            | '\u{2028}'..='\u{202e}'
            | '\u{2060}'..='\u{206f}'
            | '\u{feff}' => None,
            c => Some(c),
        })
        .collect()
}

fn length(s: &str) -> usize {
    s.chars().count()
}

/// A line longer than `max` cut into pieces no longer than it, at a space
/// where one is found.
fn pieces(line: &str, max: usize) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    for word in line.split(' ') {
        let mut word = word.to_string();
        while length(&word) > max {
            if !cur.is_empty() {
                out.push(std::mem::take(&mut cur));
            }
            let head: String = word.chars().take(max).collect();
            word = word.chars().skip(max).collect();
            out.push(head);
        }
        if !cur.is_empty() && length(&cur) + 1 + length(&word) > max {
            out.push(std::mem::take(&mut cur));
        }
        if !cur.is_empty() {
            cur.push(' ');
        }
        cur.push_str(&word);
    }
    if !cur.trim().is_empty() {
        out.push(cur);
    }
    out
}

/// Cuts one page into passages of at most `max` characters. A passage closes
/// at the end of a sentence once it is well filled, at a blank line once it
/// is fairly filled, or before a heading once it holds a quarter, so that a
/// heading opens the passage that treats its matter; a short remainder joins
/// the passage before it if it fits.
pub fn divide(text: &str, max: usize) -> Vec<String> {
    let max = max.max(200);
    let (full, fair) = (max * 6 / 10, max * 4 / 10);
    let mut out: Vec<String> = Vec::new();
    let mut cur = String::new();
    let close = |cur: &mut String, out: &mut Vec<String>| {
        let t = cur.trim();
        if !t.is_empty() {
            out.push(t.to_string());
        }
        cur.clear();
    };
    for l in text.lines() {
        let l = l.trim_end();
        if l.trim().is_empty() {
            if length(&cur) >= fair {
                close(&mut cur, &mut out);
            } else if !cur.is_empty() && !cur.ends_with("\n\n") {
                cur.push('\n');
            }
            continue;
        }
        if caput(l) && length(&cur) >= max / 4 {
            close(&mut cur, &mut out);
        }
        for piece in pieces(l, max) {
            if !cur.is_empty() && length(&cur) + 1 + length(&piece) > max {
                close(&mut cur, &mut out);
            }
            if !cur.is_empty() && !cur.ends_with('\n') {
                cur.push('\n');
            }
            cur.push_str(&piece);
            let ended = piece
                .trim_end_matches(['"', '\'', ')', '\u{201d}', '\u{2019}'])
                .ends_with(['.', '?', '!']);
            if ended && length(&cur) >= full {
                close(&mut cur, &mut out);
            }
        }
    }
    close(&mut cur, &mut out);
    if out.len() >= 2 {
        let n = out.len();
        if length(&out[n - 1]) < max / 5 && length(&out[n - 2]) + 1 + length(&out[n - 1]) <= max {
            let tail = out.pop().unwrap_or_default();
            if let Some(prev) = out.last_mut() {
                prev.push('\n');
                prev.push_str(&tail);
            }
        }
    }
    out
}

/// The words of matter of a text, stemmed and cut to their roots, in the
/// order they stand, each read as the concordance reads it. Words of naming
/// (`NOMINANDI`) are matter here.
fn ordo(text: &str, c: &Concordantia) -> Vec<String> {
    c.apply(concordantia::radices(text, NOMINANDI))
}

/// The heading lines of a passage, among its first `CAPITA_PRIMA` lines (a
/// section's heading stands at its top; a short line in its middle is a row of
/// a table or a list): lines of at most `CAPUT_MAXIMUM` words that begin with
/// a letter, do not end in a full stop, a comma, a colon or a page number, and
/// are not code (no brackets or operators), each as its words of matter. A
/// last full stop after a botanist's initial (`Hypericum perforatum L.`) is
/// no end of a sentence.
fn capita(text: &str, c: &Concordantia) -> Vec<Vec<String>> {
    text.lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .take(CAPITA_PRIMA)
        .filter(|l| caput(l))
        .map(|l| ordo(l, c))
        .filter(|w| !w.is_empty())
        .collect()
}

/// Whether one line reads as a heading (see `capita`).
fn caput(l: &str) -> bool {
    let l = l.trim();
    let n = l.split_whitespace().count();
    let initial = l
        .rsplit(' ')
        .next()
        .is_some_and(|w| w.len() <= 3 && w.ends_with('.') && w.starts_with(char::is_uppercase));
    (1..=CAPUT_MAXIMUM).contains(&n)
        && (initial || !l.ends_with(['.', ',', ';', ':']))
        && !l.contains(['(', ')', '{', '}', '=', ';', '<', '>'])
        && l.chars().next().is_some_and(char::is_alphabetic)
        // a running head carries the page's number: `The Law of Demeter 97`
        && !l.ends_with(|c: char| c.is_ascii_digit())
}

/// Whether a passage is a table (contents or index) rather than writing: most
/// of its lines run out in dot leaders or end in a page number (`clean, 154`,
/// `69–70`). Such a passage holds every heading word at once and would answer
/// every question, so it is never handed.
pub fn tabula(text: &str) -> bool {
    let lines: Vec<&str> = text
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .collect();
    if lines.len() < 3 {
        return false;
    }
    let listed = lines
        .iter()
        .filter(|l| {
            let leader = l.contains("....") || l.contains(". . .");
            let tail =
                l.trim_end_matches(|c: char| c.is_ascii_digit() || "-\u{2013},  ".contains(c));
            let numbered = tail.len() < l.len() && l.ends_with(|c: char| c.is_ascii_digit());
            leader || numbered
        })
        .count();
    listed * 10 >= lines.len() * 4
}

/// The setting kept within its bounds.
pub fn signa(n: Option<usize>) -> usize {
    n.unwrap_or(SIGNA).clamp(SIGNA_MINIMA, SIGNA_MAXIMA)
}

/// Binds a book from its header and its pages (none for a page that is only a
/// picture). `paged` says whether the pages are true pages of the book; where
/// not, a passage is marked by its number in the book alone. The book is read
/// with what it defines itself and nothing else (see `bind_with`).
pub fn bind(
    slug: &str,
    titulus: &Titulus,
    pages: &[Option<String>],
    paged: bool,
    signa: usize,
) -> Volumen {
    bind_with(slug, titulus, pages, paged, signa, &Concordantia::default())
}

/// As `bind`, reading the book with the Operator's concordance `common` as
/// well as with what the book defines itself.
pub fn bind_with(
    slug: &str,
    titulus: &Titulus,
    pages: &[Option<String>],
    paged: bool,
    signa: usize,
    common: &Concordantia,
) -> Volumen {
    let pages: Vec<Option<String>> = pages.iter().map(|p| p.as_deref().map(purga)).collect();
    let pages = pages.as_slice();
    let lexicon = common.with(&concordantia::of_book(pages, NOMINANDI));
    let max = self::signa(Some(signa)) / PASSUS_MAXIMI;
    let mut entries = Vec::new();
    let mut loci = Vec::new();
    let mut ordines = Vec::new();
    let mut heads = Vec::new();
    let mut tabulae = 0;
    for (i, page) in pages.iter().enumerate() {
        let Some(text) = page else { continue };
        for raw in divide(text, max) {
            if tabula(&raw) {
                tabulae += 1;
                continue;
            }
            let seq = ordo(&raw, &lexicon);
            let mut verba = seq.clone();
            verba.sort();
            verba.dedup();
            ordines.push(seq);
            heads.push(capita(&raw, &lexicon));
            loci.push(Locus {
                pagina: paged.then_some(i as u32 + 1),
                ordo: loci.len() as u32 + 1,
            });
            entries.push(Entry {
                claves: Vec::new(),
                verba,
                raw,
            });
        }
    }
    Volumen {
        liber: Liber {
            liber: slug.to_string(),
            nomen: titulus.nomen.clone().unwrap_or_else(|| line(slug)),
            claves: titulus.claves.clone(),
            entries,
        },
        loci,
        ordines,
        capita: heads,
        tabulae,
        concordantia: lexicon,
        aetas: titulus.aetas.clone(),
        paginae: pages.len(),
        pictae: pages.iter().filter(|p| p.is_none()).count(),
    }
}

/// What was read of a PDF, as kept on disk: a first line naming the book file's
/// size and time, then each page under a form-feed line with its number
/// (`picta` after the number for a page that was only a picture). A page's text
/// holds no form feed, for `purga` has taken every control character out.
fn write_lecta(path: &Path, size: u64, time: u64, pages: &[Option<String>]) {
    let mut s = format!("{SIGNUM_LECTI} {size} {time} {}\n", pages.len());
    for (i, p) in pages.iter().enumerate() {
        match p {
            Some(t) => s.push_str(&format!("\u{c}{}\n{}\n", i + 1, t.trim_end())),
            None => s.push_str(&format!("\u{c}{} picta\n", i + 1)),
        }
    }
    let tmp = path.with_extension("tmp");
    if fs::write(&tmp, s).is_ok() && fs::rename(&tmp, path).is_err() {
        let _ = fs::remove_file(&tmp);
    }
}

/// Reads a kept reading back, if it is of this very file (same size and time)
/// and whole. Anything amiss gives none, and the book is read anew.
fn read_lecta(path: &Path, size: u64, time: u64) -> Option<Vec<Option<String>>> {
    let text = fs::read_to_string(path).ok()?;
    let (head, rest) = text.split_once('\n')?;
    let fields: Vec<&str> = head
        .strip_prefix(SIGNUM_LECTI)?
        .split_whitespace()
        .collect();
    let [s, t, n] = fields.as_slice() else {
        return None;
    };
    if s.parse::<u64>().ok()? != size || t.parse::<u64>().ok()? != time {
        return None;
    }
    let n: usize = n.parse().ok()?;
    let mut pages: Vec<Option<String>> = Vec::new();
    for block in rest.split('\u{c}').skip(1) {
        let (mark, body) = block.split_once('\n').unwrap_or((block, ""));
        let mut mark = mark.split_whitespace();
        if mark.next()?.parse::<usize>().ok()? != pages.len() + 1 {
            return None;
        }
        pages.push(match mark.next() {
            Some("picta") => None,
            Some(_) => return None,
            None => Some(purga(body.trim_end())),
        });
    }
    (pages.len() == n).then_some(pages)
}

/// The pages of a PDF: from what was kept, or read now and kept.
fn pdf_pages(
    path: &Path,
    kept: &Path,
    name: &str,
    say: &mut dyn FnMut(String),
) -> Result<Vec<Option<String>>, String> {
    let meta = fs::metadata(path).map_err(|e| format!("cannot be read: {e}"))?;
    let time = meta
        .modified()
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map_or(0, |d| d.as_secs());
    if let Some(pages) = read_lecta(kept, meta.len(), time) {
        return Ok(pages);
    }
    say(format!(
        "Reading {name} for the first time. This is done once and may take a few minutes."
    ));
    let bytes = fs::read(path).map_err(|e| format!("cannot be read: {e}"))?;
    let book = crate::liber::Book::open(&bytes)?;
    let pages: Vec<Option<String>> = (0..book.page_count())
        .map(|i| book.text_of(i).map(|t| purga(&t)))
        .collect();
    if let Some(dir) = kept.parent() {
        let _ = fs::create_dir_all(dir);
    }
    write_lecta(kept, meta.len(), time, &pages);
    Ok(pages)
}

/// The pages of a text file: parted at form feeds if it has any (then they are
/// its pages), else the whole file as one run without page numbers.
fn text_pages(path: &Path) -> Result<(Vec<Option<String>>, bool), String> {
    let bytes = fs::read(path).map_err(|e| format!("cannot be read: {e}"))?;
    let text = String::from_utf8_lossy(&bytes);
    if text.contains('\u{c}') {
        let pages = text
            .split('\u{c}')
            .map(|p| (!p.trim().is_empty()).then(|| purga(p)))
            .collect();
        Ok((pages, true))
    } else {
        Ok((vec![Some(purga(&text))], false))
    }
}

/// Loads every book under `<dir>/spiri72.volumina` that has a `.claves` file
/// with claves in it, making the folder (with a note on its use) if it is not
/// there. `say` is told, in plain words, what was found, read and left unread.
/// A book that cannot be read is told of and skipped, never fatal.
pub fn load(dir: &Path, signa: usize, say: &mut dyn FnMut(String)) -> Vec<Volumen> {
    let home = dir.join(DIR);
    if !home.exists() {
        if fs::create_dir_all(&home).is_err() {
            return Vec::new();
        }
        let _ = fs::write(home.join(LEGE), LEGE_TEXT);
        let _ = fs::write(home.join(concordantia::FILE), concordantia::SEMEN);
        say(format!(
            "A folder for whole books is made at {} (see {LEGE} inside it).",
            home.display()
        ));
    }
    let common = fs::read_to_string(home.join(concordantia::FILE))
        .map(|t| Concordantia::parse(&purga(&t), NOMINANDI))
        .unwrap_or_default();
    let Ok(read) = fs::read_dir(&home) else {
        return Vec::new();
    };
    let kind = |p: &Path| {
        p.extension()
            .and_then(|x| x.to_str())
            .map(str::to_lowercase)
            .filter(|x| x == "pdf" || x == "txt")
    };
    let mut paths: Vec<_> = read
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.is_file() && kind(p).is_some())
        .collect();
    paths.sort();
    let mut volumina = Vec::new();
    for path in paths {
        let (Some(name), Some(stem)) = (
            path.file_name().and_then(|s| s.to_str()),
            path.file_stem().and_then(|s| s.to_str()),
        ) else {
            continue;
        };
        let shown = line(name);
        let header = fs::read_to_string(home.join(format!("{stem}.claves")))
            .map(|t| titulus(&t))
            .unwrap_or_default();
        if header.claves.is_empty() {
            say(format!(
                "{shown}: no claves are set for it (a file {}.claves beside it), so it opens to no spirit and is not read.",
                line(stem)
            ));
            continue;
        }
        if fs::metadata(&path).map_or(0, |m| m.len()) > MOLES_MAXIMA {
            say(format!("{shown}: too large to read; it is left alone."));
            continue;
        }
        let pages = if kind(&path).as_deref() == Some("pdf") {
            let kept = home.join(LECTA).join(format!("{name}.txt"));
            pdf_pages(&path, &kept, &shown, say).map(|p| (p, true))
        } else {
            text_pages(&path)
        };
        let (pages, paged) = match pages {
            Ok(p) => p,
            Err(e) => {
                say(format!("{shown}: {e}"));
                continue;
            }
        };
        let v = bind_with(stem, &header, &pages, paged, signa, &common);
        let open = (1..=72).filter(|n| opens(&v, *n)).count();
        let mut told = format!(
            "{} ({shown}): {} passages from {} pages",
            v.liber.nomen,
            v.liber.entries.len(),
            v.paginae - v.pictae
        );
        if v.pictae > 0 {
            told.push_str(&format!(
                "; {} pages are only pictures and were not read",
                v.pictae
            ));
        }
        told.push_str(&match open {
            0 => "; its claves stand in no spirit's offices, so it opens to none".to_string(),
            1 => "; it opens to 1 spirit".to_string(),
            n => format!("; it opens to {n} spirits"),
        });
        say(told + ".");
        volumina.push(v);
    }
    volumina
}

/// Whether a book opens to spirit `n`, by the offices the register records.
pub fn opens(v: &Volumen, n: u32) -> bool {
    scientia::opens(&v.liber, n)
}

/// A question's own words of matter in the order asked, stemmed, without
/// the words of asking.
fn asked(question: &str, c: &Concordantia) -> Vec<String> {
    let kept: Vec<String> = scientia::words(question)
        .into_iter()
        .filter(|w| !ROGANDI.contains(&w.as_str()))
        .collect();
    ordo(&kept.join(" "), c)
}

/// How a passage answers a question: the rarity of the question's words it
/// holds, each counted more when it recurs (by the logarithm, so a word said
/// ten times weighs about twice a word said once); and, as much again, for
/// each pair of the question's words that stand side by side in the passage
/// as they stand in the question (`boy scout`, `single responsibility`).
/// A heading line of the passage that holds two of the question's words or
/// more, or a short heading holding the question's rarest word where that
/// word recurs in the passage, adds their rarity again: the book names that
/// matter as its subject there (`The Single Responsibility Principle`,
/// `Don't Return Null`, `Tannins`, `Psoriasis`). Returns the
/// score, how many distinct words it holds, their plain weight (each once),
/// which is what the question's part is measured by, and whether a heading
/// held them.
fn score(
    materia: &[String],
    raritas: &[f64],
    pairs: &[(usize, usize)],
    seq: &[String],
    heads: &[Vec<String>],
) -> (f64, usize, f64, bool) {
    let mut counts = vec![0usize; materia.len()];
    let mut at: Vec<Option<usize>> = Vec::with_capacity(seq.len());
    for w in seq {
        let k = materia.binary_search(w).ok();
        if let Some(k) = k {
            counts[k] += 1;
        }
        at.push(k);
    }
    let (mut score, mut held, mut plain) = (0.0, 0, 0.0);
    for (k, &n) in counts.iter().enumerate() {
        if n > 0 && raritas[k] > 0.0 {
            score += raritas[k] * (1.0 + (n as f64).ln() * 0.5);
            plain += raritas[k];
            held += 1;
        }
    }
    for &(a, b) in pairs {
        if at.windows(2).any(|w| w[0] == Some(a) && w[1] == Some(b)) {
            score += (raritas[a] + raritas[b]) / 2.0;
        }
    }
    // the question's rarest word, if it is said more than once in the passage
    let rarest = (0..materia.len())
        .filter(|&k| raritas[k] > 0.0)
        .max_by(|&a, &b| raritas[a].total_cmp(&raritas[b]));
    let subject = rarest.filter(|&k| raritas[k] >= RARISSIMI && counts[k] >= 2);
    let headed = heads
        .iter()
        .map(|h| {
            let mut ks: Vec<usize> = h
                .iter()
                .filter_map(|w| materia.binary_search(w).ok())
                .filter(|&k| raritas[k] > 0.0)
                .collect();
            ks.sort();
            ks.dedup();
            // a short heading naming the question's subject (`Psoriasis`)
            let names = subject.is_some_and(|k| ks.contains(&k)) && h.len() <= CAPUT_NOMINIS;
            if ks.len() >= 2 || names {
                ks.iter().map(|&k| raritas[k]).sum()
            } else {
                0.0
            }
        })
        .fold(0.0_f64, f64::max);
    (score + headed, held, plain, headed > 0.0)
}

/// As `consult`, for a spirit whose offices read like `offices`.
pub fn consult_as(
    volumina: &[Volumen],
    offices: &[String],
    question: &str,
    signa: usize,
) -> Vec<Lectio> {
    let open: Vec<&Volumen> = volumina
        .iter()
        .filter(|v| scientia::open_to(&v.liber, offices))
        .collect();
    if open.is_empty() {
        return Vec::new();
    }
    // the question is read as the open books are read
    let lexicon = open
        .iter()
        .fold(Concordantia::default(), |c, v| c.with(&v.concordantia));
    let order = asked(question, &lexicon);
    let libri: Vec<&Liber> = open.iter().map(|v| &v.liber).collect();
    let mut materia = order.clone();
    materia.sort();
    materia.dedup();
    let q = Rogatum::from_matter(&libri, question, materia);
    let (materia, raritas) = q.materia();
    // the pairs of the question's words that stand side by side in it
    let mut pairs: Vec<(usize, usize)> = order
        .windows(2)
        .filter_map(|w| {
            let a = materia.binary_search(&w[0]).ok()?;
            let b = materia.binary_search(&w[1]).ok()?;
            (a != b && raritas[a] > 0.0 && raritas[b] > 0.0).then_some((a, b))
        })
        .collect();
    pairs.sort();
    pairs.dedup();
    // The weight of all the question's matter: each word's rarity, and for a
    // word no open passage holds, the weight of the rarest word they could
    // hold (one held by a single passage). A word every passage holds is
    // known, and weighs nothing either way.
    let total = libri.iter().map(|l| l.entries.len()).sum::<usize>() as f64;
    let ignotum = ((total + 1.0) / 2.0).ln() / (total + 1.0).ln().max(f64::MIN_POSITIVE);
    let whole: f64 = materia
        .iter()
        .zip(raritas)
        .map(|(w, &r)| {
            let known = r > 0.0
                || libri
                    .iter()
                    .flat_map(|l| &l.entries)
                    .any(|e| e.verba.binary_search(w).is_ok());
            if known {
                r
            } else {
                ignotum
            }
        })
        .sum();
    // every passage that holds a fair part of the question: (score, book, passage)
    let mut found: Vec<(f64, usize, usize)> = Vec::new();
    for (b, v) in open.iter().enumerate() {
        for (e, seq) in v.ordines.iter().enumerate() {
            let (s, held, plain, headed) = score(materia, raritas, &pairs, seq, &v.capita[e]);
            // two of the question's words, or a heading naming its subject
            let enough = (held >= 2 || headed)
                && plain >= PONDUS_LIBRI.min(whole)
                && plain >= whole * PARS_LIBRI;
            if enough {
                found.push((s, b, e));
            }
        }
    }
    found.sort_by(|x, y| y.0.total_cmp(&x.0).then((x.1, x.2).cmp(&(y.1, y.2))));
    let Some(&(best, _, _)) = found.first() else {
        return Vec::new();
    };
    let near = found
        .iter()
        .take_while(|f| f.0 >= best * SECUNDI_LIBRI)
        .count();
    // the places those passages stand in: neighbours in one book are one
    let mut stands: Vec<(usize, u32, u32)> = found[..near]
        .iter()
        .map(|&(_, b, e)| {
            let l = open[b].loci[e];
            match l.pagina {
                Some(p) => (b, p, VICINIA),
                None => (b, l.ordo, VICINIA_PASSUUM),
            }
        })
        .collect();
    stands.sort();
    let places = stands
        .windows(2)
        .filter(|w| w[0].0 != w[1].0 || w[1].1 - w[0].1 > w[1].2)
        .count()
        + 1;
    if places > TURBA {
        return Vec::new();
    }
    let budget = self::signa(Some(signa));
    let mut spent = 0;
    let mut handed: Vec<(usize, usize)> = Vec::new();
    for &(_, b, e) in found.iter().take(near.min(PASSUS_MAXIMI)) {
        let n = length(&open[b].liber.entries[e].raw);
        if spent + n > budget {
            break;
        }
        spent += n;
        handed.push((b, e));
    }
    // as they stand in the books: the earliest is most often where the book
    // sets the matter out (measured on the trial books)
    handed.sort();
    handed
        .into_iter()
        .map(|(b, e)| {
            let v = open[b];
            let l = v.loci[e];
            Lectio {
                liber: v.liber.nomen.clone(),
                aetas: v.aetas.clone(),
                locus: match l.pagina {
                    Some(p) => format!("page {p}"),
                    None => format!("part {}", l.ordo),
                },
                textus: v.liber.entries[e].raw.clone(),
            }
        })
        .collect()
}

/// Given the loaded books, a spirit and its question, returns the passages the
/// spirit is handed, if any, from books open to it by its offices: at most
/// `PASSUS_MAXIMI`, within `signa` characters in all. No model is asked; a
/// closed book is never searched, nor weighed.
pub fn consult(volumina: &[Volumen], spirit: u32, question: &str, signa: usize) -> Vec<Lectio> {
    if volumina.is_empty() {
        return Vec::new();
    }
    consult_as(volumina, &scientia::office_texts(spirit), question, signa)
}
