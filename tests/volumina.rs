//! Whole books (`volumina`): reading, cutting into passages, gating, keeping
//! what was read, and the measure of passage search.
//!
//! The measure book below is written for this test (no book's text is held in
//! this repository). Like `tests/quaesitio.rs`, the measure has a floor that
//! may only rise and ceilings that may only fall.

use std::fs;
use std::path::PathBuf;

use spiri72::volumina::{self, bind, consult, consult_as, divide, purga, tabula, titulus};

/// Questions about matter the book treats: every one must find its own page.
const PLAIN_FLOOR: usize = 14;
/// Questions put in other words: at least this many right, at most this many wrong.
const OBLIQUE_FLOOR: usize = 6;
const OBLIQUE_WRONG_CEILING: usize = 1;
/// Questions the book has nothing to say on: at most this many find anything.
const IDLE_CEILING: usize = 1;

/// The measure book, one page per item. Page 1 is its contents, page 16 its
/// index: tables that must never be handed.
const PAGES: [&str; 16] = [
    "Contents\n\
     The Anger of Crowds ........................ 2\n\
     Jealousy and Its Watchfulness .............. 3\n\
     Sleep and the Tired Mind ................... 4\n\
     Grief After a Loss ......................... 5\n\
     How Habits Form ............................ 6\n\
     Fear in the Body ........................... 7\n\
     Trust Between Strangers .................... 8\n\
     The Quarrel of Brothers .................... 9\n\
     Memory and Its Errors ...................... 10\n\
     Shame and Pride ............................ 11\n\
     Hunger and Mood ............................ 12\n\
     Courtship and Desire ....................... 13\n\
     Rivalry at Work ............................ 14\n\
     Forgiveness ................................ 15",
    "The Anger of Crowds\n\
     A crowd grows angry faster than any one person in it. Each member sees the \
     others shouting and takes their rage as proof that rage is right. Rumour \
     travels through a crowd like fire through dry grass, and a mob will do what \
     none of its members would do alone. Leaders who wish to calm a crowd speak \
     slowly, name its grievance aloud, and give it something to do other than riot.",
    "Jealousy and Its Watchfulness\n\
     Jealousy is the fear of losing a beloved to a rival. The jealous lover \
     watches, reads every glance, and finds betrayal in small things: a late \
     return, a letter unread, a name spoken warmly. Jealousy feeds on suspicion \
     and starves on plain speech; the cure most often given is honest talk \
     between the lovers before the suspicion hardens into certainty.",
    "Sleep and the Tired Mind\n\
     A mind deprived of sleep judges poorly. After two nights without rest a \
     person misreads faces, takes offence easily and remembers little of what \
     was learned. Sleep sorts the day's memories and clears the mind of its \
     waste. Those who keep regular hours of rest are steadier in temper than \
     those who sleep at random.",
    "Grief After a Loss\n\
     Grief after the death of someone loved does not run in neat stages. It comes \
     in waves: a smell, a song or an empty chair brings it back. The bereaved do \
     best when they are allowed to mourn openly and are not hurried. Grief \
     softens with time but rarely leaves entirely; most learn to carry it.",
    "How Habits Form\n\
     A habit forms when an action is repeated in the same place, at the same \
     hour, for the same reward. In time the cue alone sets the action going, \
     with no choice made. To break a habit one changes the cue or the reward; \
     willpower alone seldom holds, for the habit runs below deliberate thought.",
    "Fear in the Body\n\
     Fear begins in the body before the mind names it. The heart quickens, the \
     breath grows shallow, the hands sweat and the stomach tightens. This is the \
     body readying itself to fight or flee. A frightened person can be calmed \
     by slow breathing, which tells the body the danger has passed.",
    "Trust Between Strangers\n\
     Strangers decide within moments whether to trust one another. They read the \
     face, the voice and the hands. Trust once given grows when promises are kept \
     and collapses at the first discovered lie. Merchants in old markets built \
     trust slowly, by small honest dealings, before they risked great ones.",
    "The Quarrel of Brothers\n\
     Brothers quarrel most bitterly over inheritance and over their parents' \
     favour. Siblings who compete for the same praise as children often carry the \
     rivalry into old age. A quarrel between brothers is mended, when it is \
     mended, by a third kinsman whom both respect.",
    "Memory and Its Errors\n\
     Memory is not a record but a retelling. Each time a person recalls an event \
     the memory is rebuilt, and details drift. Witnesses to the same accident \
     remember different colours, different words, different numbers of people. \
     A confident memory is not therefore a true one.",
    "Shame and Pride\n\
     Shame turns the eye inward: the ashamed person wishes to hide and to vanish. \
     Pride lifts the head and seeks to be seen. Both are feelings about one's \
     standing before others. A proud person wounded in public often answers with \
     anger, for anger is easier to bear than shame.",
    "Hunger and Mood\n\
     A hungry person is quicker to anger and slower to forgive. Judges, it is \
     said, grow harsher in the hours before their meal. Low blood sugar makes the \
     mind restless and the temper short. A meal often settles an argument that \
     reasoning could not.",
    "Courtship and Desire\n\
     Courtship among people follows customs that differ from land to land, yet \
     some things recur: gifts, attention, and the slow revealing of oneself. \
     Desire is sharpened by obstacle and dulled by ease. A suitor who is always \
     available is valued less than one who must be waited for.",
    "Rivalry at Work\n\
     Rivalry among workers who seek the same promotion can drive them to better \
     work or to sabotage. Where rewards are shared, colleagues help one another; \
     where only one may win, they hide their knowledge. A wise master rewards the \
     workshop as a whole as well as the single worker.",
    "Forgiveness\n\
     To forgive is to give up the claim to revenge. Forgiveness does not erase \
     the wrong or require friendship afterwards. Those who forgive often sleep \
     better and carry less anger, whatever the offender does. Forgiveness asked \
     for and granted mends a bond more surely than forgiveness given in silence.",
    "Index\n\
     anger, 2, 11, 12, 15\n\
     brothers, 9\n\
     courtship, 13\n\
     crowd, 2\n\
     fear, 3, 7\n\
     forgiveness, 15\n\
     grief, 5\n\
     habit, 6\n\
     jealousy, 3\n\
     memory, 4, 10\n\
     rivalry, 9, 14\n\
     sleep, 4, 15",
];

const CLAVES: &str =
    "# nomen: The Measure Book\n# claves: thoughts, love, discord, wrath\n# aetas: modern\n";

/// Furfur (34) causes love between man and woman; the gate opens to him.
const FURFUR: u32 = 34;
/// Buer (10) teaches the virtues of herbs; the gate does not open to him.
const BUER: u32 = 10;

fn book() -> volumina::Volumen {
    let pages: Vec<Option<String>> = PAGES.iter().map(|p| Some(p.to_string())).collect();
    bind("mensura", &titulus(CLAVES), &pages, true, volumina::SIGNA)
}

fn folder(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("spiri72-volumina-{}-{name}", std::process::id()));
    let _ = fs::remove_dir_all(&d);
    fs::create_dir_all(&d).unwrap();
    d
}

/// The page each case must be found on.
const PLAIN: &[(&str, u32)] = &[
    ("Why does a crowd grow angry, and how is a mob calmed?", 2),
    (
        "What is jealousy, and what cures the suspicion of a jealous lover?",
        3,
    ),
    ("What does a lack of sleep do to the mind?", 4),
    ("How does grief come after the death of someone loved?", 5),
    ("How does a habit form, and how is a habit broken?", 6),
    ("What does fear do in the body?", 7),
    ("How do strangers come to trust one another?", 8),
    ("Why do brothers quarrel, and how is the quarrel mended?", 9),
    ("Are memories true, or do witnesses remember wrongly?", 10),
    ("What is the difference between shame and pride?", 11),
    ("Does hunger make a person quicker to anger?", 12),
    ("What is courtship, and what sharpens desire?", 13),
    (
        "What does rivalry among workers seeking promotion lead to?",
        14,
    ),
    (
        "What does it mean to forgive, and does forgiveness need friendship?",
        15,
    ),
];

const OBLIQUE: &[(&str, u32)] = &[
    (
        "My lover reads every glance I give and suspects betrayal",
        3,
    ),
    (
        "After two nights without rest I take offence at everything",
        4,
    ),
    ("My heart quickens and my hands sweat when danger comes", 7),
    (
        "Two witnesses saw the same accident and tell it differently",
        10,
    ),
    ("Judges grow harsher before their meal, is that so?", 12),
    (
        "The colleagues in my workshop hide their knowledge from each other",
        14,
    ),
    ("A suitor who is always available is valued less, why?", 13),
    ("Rumour travels through the mob like fire", 2),
];

const IDLE: &[&str] = &[
    "Will it rain tomorrow?",
    "What are the virtues of rue and barley?",
    "Who was king of France in 1500?",
    "How do I build a stone tower?",
    "Teach me the liberal sciences and geometry",
    "Where is the treasure hidden?",
    "What time is it?",
    "Tell me of the planet Mars and its motion",
    "How do I fix the engine of my car?",
    "Write me a poem about the sea",
];

fn pages_found(question: &str) -> Vec<u32> {
    consult(&[book()], FURFUR, question, volumina::SIGNA)
        .iter()
        .map(|l| l.locus.trim_start_matches("page ").parse().unwrap())
        .collect()
}

#[test]
fn plain_questions_find_their_page() {
    let right = PLAIN
        .iter()
        .filter(|(q, page)| {
            let found = pages_found(q);
            if !found.contains(page) {
                eprintln!("missed: {q:?} found {found:?}, wanted page {page}");
            }
            found.contains(page)
        })
        .count();
    assert!(
        right >= PLAIN_FLOOR,
        "{right} of {} plain questions",
        PLAIN.len()
    );
}

#[test]
fn oblique_questions_never_grow_worse() {
    let (mut right, mut wrong) = (0, 0);
    for (q, page) in OBLIQUE {
        let found = pages_found(q);
        if found.contains(page) {
            right += 1;
        } else if !found.is_empty() {
            wrong += 1;
            eprintln!("wrong: {q:?} found {found:?}, wanted page {page}");
        }
    }
    eprintln!("oblique: {right} right, {wrong} wrong of {}", OBLIQUE.len());
    assert!(right >= OBLIQUE_FLOOR, "only {right} right");
    assert!(wrong <= OBLIQUE_WRONG_CEILING, "{wrong} wrong");
}

#[test]
fn idle_questions_never_find_more() {
    let found: Vec<&&str> = IDLE.iter().filter(|q| !pages_found(q).is_empty()).collect();
    assert!(
        found.len() <= IDLE_CEILING,
        "idle questions found something: {found:?}"
    );
}

#[test]
fn the_contents_and_the_index_are_never_handed() {
    assert!(tabula(PAGES[0]));
    assert!(tabula(PAGES[15]));
    assert!(
        PAGES[1..15].iter().all(|p| !tabula(p)),
        "a page of writing taken for a table"
    );
    let v = book();
    assert_eq!(v.tabulae, 2);
    // a question naming many headings at once would have found the contents
    let q = "anger jealousy sleep grief habit fear trust quarrel memory shame hunger";
    for l in consult(&[v], FURFUR, q, volumina::SIGNA) {
        assert!(!l.textus.contains("......"), "the contents were handed");
        assert!(!l.textus.starts_with("Index"), "the index was handed");
    }
}

#[test]
fn a_closed_book_is_never_searched() {
    let v = book();
    assert!(volumina::opens(&v, FURFUR));
    assert!(!volumina::opens(&v, BUER));
    assert!(consult(&[v], BUER, PLAIN[1].0, volumina::SIGNA).is_empty());
}

#[test]
fn passages_are_true_copies_marked_by_book_page_and_age() {
    let got = consult(&[book()], FURFUR, PLAIN[5].0, volumina::SIGNA);
    assert!(!got.is_empty());
    for l in &got {
        assert_eq!(l.liber, "The Measure Book");
        assert_eq!(l.aetas.as_deref(), Some("modern"));
        let page: usize = l.locus.trim_start_matches("page ").parse().unwrap();
        // the copy stands, word for word, on the page it is marked with
        let flat = |s: &str| s.split_whitespace().collect::<Vec<_>>().join(" ");
        assert!(
            flat(PAGES[page - 1]).contains(&flat(&l.textus)),
            "{}",
            l.textus
        );
    }
}

#[test]
fn no_more_than_three_passages_within_the_set_characters() {
    let mut pages: Vec<Option<String>> = Vec::new();
    for k in 0..12 {
        pages.push(Some(format!(
            "Of wrath, the {k}th telling. Wrath kindles in the breast and burns \
             through reason; wrath spoken aloud grows, wrath held in sours. {}",
            "The angry man forgets his friends. ".repeat(10)
        )));
    }
    let v = bind("ira", &titulus(CLAVES), &pages, true, 1500);
    for signa in [1500, 3000, 9000] {
        let got = consult(
            std::slice::from_ref(&v),
            FURFUR,
            "what is wrath and the angry man",
            signa,
        );
        assert!(got.len() <= volumina::PASSUS_MAXIMI);
        let n: usize = got.iter().map(|l| l.textus.chars().count()).sum();
        assert!(n <= volumina::signa(Some(signa)), "{n} characters handed");
    }
    // the setting is kept within its bounds
    assert_eq!(volumina::signa(Some(10)), volumina::SIGNA_MINIMA);
    assert_eq!(volumina::signa(Some(1 << 30)), volumina::SIGNA_MAXIMA);
    assert_eq!(volumina::signa(None), volumina::SIGNA);
}

#[test]
fn a_page_is_cut_into_passages_of_bounded_length() {
    let long = format!(
        "{}\n\n{}",
        "A sentence of the book. ".repeat(80),
        "x".repeat(2500)
    );
    for max in [200, 600, 1000] {
        let parts = divide(&long, max);
        assert!(parts.len() > 1);
        for p in &parts {
            assert!(p.chars().count() <= max, "{} > {max}", p.chars().count());
        }
        // nothing of the page is lost
        let kept: usize = parts
            .iter()
            .map(|p| p.split_whitespace().collect::<String>().len())
            .sum();
        assert_eq!(kept, long.split_whitespace().collect::<String>().len());
    }
}

#[test]
fn hidden_and_control_characters_are_taken_out() {
    let dirty = "wrath\u{202e}gnol\u{200b} \u{1b}[31mred\u{7}\u{feff}\r\nnext\tline\u{c}";
    let clean = purga(dirty);
    assert_eq!(clean, "wrathgnol [31mred\nnext line");
    assert!(clean.chars().all(|c| c == '\n' || !c.is_control()));
}

#[test]
fn the_header_is_read_and_made_safe() {
    let t = titulus("# nomen: Behave\u{202e} (Sapolsky)\n# claves: Thoughts, love ,, discord\n# aetas: modern\nnoise\n");
    assert_eq!(t.nomen.as_deref(), Some("Behave (Sapolsky)"));
    assert_eq!(t.claves, ["thoughts", "love", "discord"]);
    assert_eq!(t.aetas.as_deref(), Some("modern"));
    assert_eq!(titulus("nothing here"), Default::default());
}

/// A small, well-formed PDF whose pages draw the given content streams.
fn pdf(pages: &[&str]) -> Vec<u8> {
    let n = pages.len();
    let kids: Vec<String> = (0..n).map(|i| format!("{} 0 R", 4 + 2 * i)).collect();
    let mut objs = vec![
        "<< /Type /Catalog /Pages 2 0 R >>".to_string(),
        format!("<< /Type /Pages /Kids [{}] /Count {n} >>", kids.join(" ")),
        "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>".to_string(),
    ];
    for (i, c) in pages.iter().enumerate() {
        objs.push(format!(
            "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 600 400] /Resources << /Font << /F1 3 0 R >> >> /Contents {} 0 R >>",
            5 + 2 * i
        ));
        objs.push(format!("<< /Length {} >>\nstream\n{c}\nendstream", c.len()));
    }
    let mut out = b"%PDF-1.7\n".to_vec();
    let mut offsets = Vec::new();
    for (i, o) in objs.iter().enumerate() {
        offsets.push(out.len());
        out.extend_from_slice(format!("{} 0 obj\n{o}\nendobj\n", i + 1).as_bytes());
    }
    let xref = out.len();
    out.extend_from_slice(format!("xref\n0 {}\n0000000000 65535 f \n", objs.len() + 1).as_bytes());
    for o in offsets {
        out.extend_from_slice(format!("{o:010} 00000 n \n").as_bytes());
    }
    out.extend_from_slice(
        format!(
            "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n",
            objs.len() + 1
        )
        .as_bytes(),
    );
    out
}

const PAGE_WRATH: &str = "BT /F1 12 Tf 20 350 Td (Of wrath and the angry heart) Tj 0 -20 Td (Wrath burns through reason and sours the angry heart.) Tj ET";
const PAGE_PICTURE: &str = "0.2 g 20 20 260 160 re f";
const PAGE_LOVE: &str = "BT /F1 12 Tf 20 350 Td (Of love between man and woman) Tj 0 -20 Td (Love is kindled by attention and kept by kindness.) Tj ET";

fn told(dir: &std::path::Path) -> (Vec<volumina::Volumen>, Vec<String>) {
    let mut said = Vec::new();
    let v = volumina::load(dir, volumina::SIGNA, &mut |s| said.push(s));
    (v, said)
}

#[test]
fn a_pdf_is_read_once_kept_and_read_again_when_it_changes() {
    let d = folder("pdf");
    // the first start makes the folder, with a note on its use
    let (v, said) = told(&d);
    assert!(v.is_empty());
    let home = d.join(volumina::DIR);
    assert!(home.join(volumina::LEGE).exists());
    // the concordance is no book
    assert!(
        !said.iter().any(|s| s.contains(spiri72::concordantia::FILE)),
        "{said:?}"
    );
    assert!(
        said.iter().any(|s| s.contains("folder for whole books")),
        "{said:?}"
    );

    fs::write(
        home.join("liber.pdf"),
        pdf(&[PAGE_WRATH, PAGE_PICTURE, PAGE_LOVE]),
    )
    .unwrap();
    fs::write(home.join("liber.claves"), CLAVES).unwrap();
    let (v, said) = told(&d);
    assert_eq!(v.len(), 1, "{said:?}");
    assert_eq!((v[0].paginae, v[0].pictae), (3, 1));
    assert!(
        said.iter().any(|s| s.contains("for the first time")),
        "{said:?}"
    );
    assert!(
        said.iter().any(|s| s.contains("1 pages are only pictures")),
        "{said:?}"
    );
    let kept = home.join(volumina::LECTA).join("liber.pdf.txt");
    assert!(kept.exists());
    let got = consult(
        &v,
        FURFUR,
        "what is wrath and the angry heart",
        volumina::SIGNA,
    );
    assert_eq!(
        got.first().map(|l| l.locus.as_str()),
        Some("page 1"),
        "{got:?}"
    );
    let got = consult(
        &v,
        FURFUR,
        "how is love between man and woman kindled",
        volumina::SIGNA,
    );
    assert_eq!(
        got.first().map(|l| l.locus.as_str()),
        Some("page 3"),
        "{got:?}"
    );

    // the second start reads what was kept, not the book
    let (v2, said) = told(&d);
    assert!(
        !said.iter().any(|s| s.contains("for the first time")),
        "{said:?}"
    );
    assert_eq!(v2[0].passus(), v[0].passus());

    // a changed book is read anew
    fs::write(home.join("liber.pdf"), pdf(&[PAGE_LOVE, PAGE_WRATH])).unwrap();
    let (v3, said) = told(&d);
    assert!(
        said.iter().any(|s| s.contains("for the first time")),
        "{said:?}"
    );
    assert_eq!((v3[0].paginae, v3[0].pictae), (2, 0));

    // a spoiled reading is never trusted: the book is read anew (the kept
    // index is taken away, or it would answer without the reading)
    fs::remove_file(home.join(volumina::LECTA).join("liber.pdf.index")).unwrap();
    fs::write(&kept, "spiri72 lecta 1 1 1 1\n\u{c}9\nforged").unwrap();
    let (v4, said) = told(&d);
    assert!(
        said.iter().any(|s| s.contains("for the first time")),
        "{said:?}"
    );
    assert_eq!(v4[0].paginae, 2);
}

#[test]
fn a_book_without_claves_is_neither_read_nor_opened() {
    let d = folder("nameless");
    let home = d.join(volumina::DIR);
    fs::create_dir_all(&home).unwrap();
    fs::write(home.join("nameless.pdf"), pdf(&[PAGE_WRATH])).unwrap();
    fs::write(
        home.join("empty.txt"),
        "Of wrath and the angry heart. Wrath burns.",
    )
    .unwrap();
    fs::write(home.join("empty.claves"), "# nomen: Empty\n# claves:\n").unwrap();
    let (v, said) = told(&d);
    assert!(v.is_empty());
    assert_eq!(
        said.iter()
            .filter(|s| s.contains("opens to no spirit"))
            .count(),
        2,
        "{said:?}"
    );
    assert!(
        !home.join(volumina::LECTA).exists(),
        "a book with no claves was read"
    );
}

#[test]
fn a_damaged_book_is_told_of_and_skipped() {
    let d = folder("damaged");
    let home = d.join(volumina::DIR);
    fs::create_dir_all(&home).unwrap();
    fs::write(home.join("broken.pdf"), b"%PDF-1.7\nthis is not a book").unwrap();
    fs::write(home.join("broken.claves"), CLAVES).unwrap();
    fs::write(home.join("good.txt"), PAGES.join("\u{c}")).unwrap();
    fs::write(home.join("good.claves"), CLAVES).unwrap();
    let (v, said) = told(&d);
    assert_eq!(v.len(), 1, "{said:?}");
    assert!(
        said.iter().any(|s| s.starts_with("broken.pdf:")),
        "{said:?}"
    );
    // a text book parted by form feeds keeps its pages
    assert_eq!(v[0].paginae, PAGES.len());
    let got = consult(&v, FURFUR, PLAIN[3].0, volumina::SIGNA);
    assert!(got.iter().any(|l| l.locus == "page 5"), "{got:?}");
}

#[test]
fn a_text_book_without_pages_is_marked_by_part() {
    let pages = vec![Some(PAGES[1..15].join("\n\n"))];
    let v = bind("ohne", &titulus(CLAVES), &pages, false, volumina::SIGNA);
    let got = consult_as(&[v], &["wrath".into()], PLAIN[0].0, volumina::SIGNA);
    assert!(!got.is_empty());
    assert!(got.iter().all(|l| l.locus.starts_with("part ")), "{got:?}");
}

// ---- Parts of a book with gates of their own ----

#[test]
fn a_header_names_parts_of_the_book_by_their_pages() {
    let t = titulus(
        "# nomen: Pliny\n\
         # claves: animals\n\
         # pages 60-120 (Book II, the heavens): stars, heavens\n\
         # pages 380\u{2013}470: herb, plants\n\
         # page 9: metals\n\
         # pages 20-10: backwards\n\
         # pages x-y: nonsense\n\
         # pages 5-6:\n",
    );
    assert_eq!(t.claves, ["animals"]);
    let got: Vec<(u32, u32, Option<&str>, Vec<&str>)> = t
        .sectiones
        .iter()
        .map(|s| {
            (
                s.a,
                s.b,
                s.nomen.as_deref(),
                s.claves.iter().map(String::as_str).collect(),
            )
        })
        .collect();
    assert_eq!(
        got,
        [
            (
                60,
                120,
                Some("Book II, the heavens"),
                vec!["stars", "heavens"]
            ),
            (380, 470, None, vec!["herb", "plants"]),
            (9, 9, None, vec!["metals"]),
        ]
    );
}

/// The measure book with no gate of its own, its parts gated apart: the
/// passions (pages 2-3) on love, sleep and the body (pages 4-7) on herbs.
fn parted() -> volumina::Volumen {
    let pages: Vec<Option<String>> = PAGES.iter().map(|p| Some(p.to_string())).collect();
    let t = titulus(
        "# nomen: The Measure Book\n\
         # pages 2-3 (the passions): love\n\
         # pages 4-7 (the body): herb\n",
    );
    bind("parted", &t, &pages, true, volumina::SIGNA)
}

#[test]
fn a_spirit_reads_only_the_parts_its_offices_open() {
    let v = parted();
    // Furfur's offices hold love: the passions open, the body does not
    assert!(volumina::opens(&v, FURFUR));
    let pages = |spirit, q| -> Vec<String> {
        consult(std::slice::from_ref(&v), spirit, q, volumina::SIGNA)
            .into_iter()
            .map(|l| l.locus)
            .collect()
    };
    let got = pages(FURFUR, PLAIN[1].0);
    assert_eq!(got, ["page 3 (the passions)"], "jealousy is in his part");
    assert!(
        pages(FURFUR, PLAIN[5].0).is_empty(),
        "fear in the body is not his to read"
    );
    assert!(
        pages(FURFUR, PLAIN[8].0).is_empty(),
        "nor is a page in no part"
    );
    // Buer's offices hold herbs: the body opens to him, the passions do not
    let got = pages(BUER, PLAIN[5].0);
    assert_eq!(
        got,
        ["page 7 (the body)"],
        "fear in the body is in his part"
    );
    assert!(pages(BUER, PLAIN[1].0).is_empty());
    // a spirit of neither reads nothing
    assert!(!volumina::opens(&v, 2));
}

#[test]
fn the_whole_book_gate_opens_every_page() {
    let pages: Vec<Option<String>> = PAGES.iter().map(|p| Some(p.to_string())).collect();
    let t = titulus("# claves: love\n# pages 4-7: herb\n");
    let v = bind("whole", &t, &pages, true, volumina::SIGNA);
    assert_eq!(
        volumina::aperta(&v, &["causes love".into()]).len(),
        v.passus()
    );
    let body = volumina::aperta(&v, &["teaches the virtues of herbs".into()]);
    assert!(!body.is_empty() && body.len() < v.passus());
}

#[test]
fn a_closed_part_is_never_handed_whatever_is_asked() {
    // every question of the measure, to spirits of each part and of none: what
    // is handed always stands on that spirit's own pages
    let v = parted();
    let mut handed = 0;
    // spirit 2 (Agares) has neither part: an empty range of his own
    for (spirit, own) in [(FURFUR, 2..4), (BUER, 4..8), (2, 0..0)] {
        let questions = PLAIN
            .iter()
            .chain(OBLIQUE)
            .map(|(q, _)| *q)
            .chain(IDLE.iter().copied());
        for q in questions {
            for l in consult(std::slice::from_ref(&v), spirit, q, volumina::SIGNA) {
                let page: u32 = l.locus["page ".len()..]
                    .split(' ')
                    .next()
                    .unwrap()
                    .parse()
                    .unwrap();
                assert!(
                    own.contains(&page),
                    "spirit {spirit} was handed page {page} for {q:?}"
                );
                handed += 1;
            }
        }
    }
    assert!(handed > 0, "the parts were never read at all");
}

#[test]
fn a_book_gated_only_by_parts_is_read_and_told_of() {
    let d = folder("parts");
    let home = d.join(volumina::DIR);
    fs::create_dir_all(&home).unwrap();
    fs::write(home.join("pars.txt"), PAGES.join("\u{c}")).unwrap();
    fs::write(
        home.join("pars.claves"),
        "# nomen: Parts\n# pages 2-3: love\n",
    )
    .unwrap();
    let (v, said) = told(&d);
    assert_eq!(v.len(), 1, "{said:?}");
    assert!(
        said.iter()
            .any(|s| s.contains("only to the parts their offices touch")),
        "{said:?}"
    );
}

// ---- The kept index: reused, made anew, never trusted blindly ----

/// A folder holding the measure book as a text book with pages, and what each
/// plain question finds in it.
fn kept(name: &str) -> (PathBuf, PathBuf) {
    let d = folder(name);
    let home = d.join(volumina::DIR);
    fs::create_dir_all(&home).unwrap();
    fs::write(home.join("mensura.txt"), PAGES.join("\u{c}")).unwrap();
    fs::write(home.join("mensura.claves"), CLAVES).unwrap();
    (d, home)
}

fn answers(v: &[volumina::Volumen]) -> Vec<Vec<(String, String)>> {
    PLAIN
        .iter()
        .chain(OBLIQUE)
        .map(|(q, _)| {
            consult(v, FURFUR, q, volumina::SIGNA)
                .into_iter()
                .map(|l| (l.locus, l.textus))
                .collect()
        })
        .collect()
}

#[test]
fn a_kept_index_answers_as_the_book_itself_does() {
    let (d, home) = kept("kept-same");
    let (first, _) = told(&d);
    let lecta = home.join(volumina::LECTA);
    let (index, passus) = (
        lecta.join("mensura.txt.index"),
        lecta.join("mensura.txt.passus"),
    );
    assert!(index.exists() && passus.exists());
    let made = fs::read(&index).unwrap();
    // the second start reads the kept index, and answers alike
    let (second, _) = told(&d);
    assert_eq!(
        fs::read(&index).unwrap(),
        made,
        "the index was made anew for nothing"
    );
    assert!(matches!(second[0].textus, volumina::Textus::Liber(..)));
    assert_eq!(answers(&second), answers(&first));
    // and as the same book bound in memory
    let pages: Vec<Option<String>> = PAGES.iter().map(|p| Some(p.to_string())).collect();
    let common = spiri72::concordantia::Concordantia::parse(
        &fs::read_to_string(home.join(spiri72::concordantia::FILE)).unwrap_or_default(),
        &["name", "named", "names", "call", "called", "use", "used"],
    );
    let memory = volumina::bind_with(
        "mensura",
        &titulus(CLAVES),
        &pages,
        true,
        volumina::SIGNA,
        &common,
    );
    assert_eq!(answers(&second), answers(&[memory]));
}

#[test]
fn a_change_to_the_book_or_the_concordance_makes_the_index_anew() {
    let (d, home) = kept("kept-change");
    let index = home.join(volumina::LECTA).join("mensura.txt.index");
    told(&d);
    let first = fs::read(&index).unwrap();
    // the Operator's concordance changes how words are read
    fs::write(home.join(spiri72::concordantia::FILE), "jealousy = envy\n").unwrap();
    told(&d);
    let second = fs::read(&index).unwrap();
    assert_ne!(second, first, "a changed concordance kept the old index");
    // the book changes
    std::thread::sleep(std::time::Duration::from_millis(1100));
    fs::write(home.join("mensura.txt"), PAGES[1..].join("\u{c}")).unwrap();
    let (v, _) = told(&d);
    assert_ne!(
        fs::read(&index).unwrap(),
        second,
        "a changed book kept the old index"
    );
    assert_eq!(v[0].paginae, PAGES.len() - 1);
}

#[test]
fn a_damaged_index_is_made_anew_and_answers_alike() {
    let (d, home) = kept("kept-damaged");
    let lecta = home.join(volumina::LECTA);
    let (index, passus) = (
        lecta.join("mensura.txt.index"),
        lecta.join("mensura.txt.passus"),
    );
    let (good, _) = told(&d);
    let want = answers(&good);
    let whole = fs::read(&index).unwrap();
    let damages: Vec<(&str, Vec<u8>)> = vec![
        ("cut short", whole[..whole.len() / 2].to_vec()),
        ("emptied", Vec::new()),
        ("a byte changed near the end", {
            let mut b = whole.clone();
            let n = b.len() - 3;
            b[n] ^= 0x5a;
            b
        }),
        ("a word number out of range", {
            let mut b = whole.clone();
            let n = b.len() - 8;
            b[n..n + 4].copy_from_slice(&u32::MAX.to_le_bytes()[..]);
            b
        }),
        ("another book's mark", {
            let mut b = whole.clone();
            b[16] ^= 1;
            b
        }),
    ];
    for (what, bytes) in damages {
        fs::write(&index, &bytes).unwrap();
        let (v, _) = told(&d);
        assert_eq!(answers(&v), want, "{what}");
    }
    // the text file cut short: the index no longer fits it, and both are made anew
    let body = fs::read(&passus).unwrap();
    fs::write(&passus, &body[..body.len() - 10]).unwrap();
    let (v, _) = told(&d);
    assert_eq!(answers(&v), want);
    assert_eq!(fs::read(&passus).unwrap(), body);
}

#[test]
fn text_changed_on_disk_while_running_is_still_made_plain() {
    let (d, home) = kept("kept-tampered");
    let passus = home.join(volumina::LECTA).join("mensura.txt.passus");
    let (v, _) = told(&d);
    // the text file is overwritten under the running Spiri72, byte for byte as
    // long, with steering and control characters
    let body = fs::read(&passus).unwrap();
    let evil: Vec<u8> = "\u{202e}\u{1b}[2J"
        .bytes()
        .cycle()
        .take(body.len())
        .collect();
    fs::write(&passus, &evil).unwrap();
    for (q, _) in PLAIN {
        for l in consult(&v, FURFUR, q, volumina::SIGNA) {
            assert!(
                l.textus.chars().all(|c| c == '\n' || !c.is_control()),
                "{q}"
            );
            assert!(!l.textus.contains('\u{202e}'), "{q}");
        }
    }
    // the text file taken away: nothing is handed, nothing breaks
    fs::remove_file(&passus).unwrap();
    for (q, _) in PLAIN {
        assert!(consult(&v, FURFUR, q, volumina::SIGNA).is_empty(), "{q}");
    }
}
