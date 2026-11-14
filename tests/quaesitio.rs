//! The measure of search (`quaesitio`): a fixed list of questions over every
//! starter library, each with the one entry that rightly answers it, and a list
//! of questions no library answers. No model is asked.
//!
//! Each question is put as every one of the 72 spirits. A case is right only if
//! every spirit the entry's library opens to is handed that entry, and no spirit
//! at all is handed another; it is wrong if any spirit is handed another entry.
//!
//! - Plain questions name the entry's key and must always be right.
//! - Oblique questions describe the thing without naming it. Their count of
//!   right finds is a floor that may only rise, and their count of wrong finds
//!   a ceiling that may only fall.
//! - Idle questions touch no library; handing anything is a false find. Their
//!   count of false finds is a ceiling that may only fall.
//!
//! When search improves, move the constants; they must never move the other way.
//! Run with `cargo test --test quaesitio -- --nocapture` to see the tally and
//! every case that is not right.

use spiri72::scientia::{consult, opens, parse, Liber};

/// Right finds among the oblique questions, as measured. May only rise.
const OBLIQUE_FLOOR: usize = 15;
/// Wrong finds among the oblique questions, as measured. May only fall.
const OBLIQUE_WRONG_CEILING: usize = 2;
/// Idle questions that were handed an entry, as measured. May only fall.
const IDLE_CEILING: usize = 5;

const LIBRI: [&str; 10] = [
    "angelica",
    "astronomia",
    "divinatio",
    "flora",
    "honorius",
    "litterae",
    "paedagogia",
    "poetica",
    "solomonica",
    "tarot",
];

/// (question, library, the entry's key)
type Case = (&'static str, &'static str, &'static str);

const PLAIN: &[Case] = &[
    ("What is barley good for?", "flora", "barley"),
    ("Tell me of the virtues of rosemary.", "flora", "rosemary"),
    ("How is mugwort used?", "flora", "mugwort"),
    ("What is mandrake?", "flora", "mandrake"),
    (
        "What do the ancients say of Saturn?",
        "astronomia",
        "saturn",
    ),
    ("What is the zodiac?", "astronomia", "zodiac"),
    ("Speak of Sirius.", "astronomia", "sirius"),
    ("What is precession?", "astronomia", "precession"),
    ("Who was John Dee?", "angelica", "john-dee"),
    ("What are the nine choirs?", "angelica", "the-nine-choirs"),
    ("Teach me geomancy.", "divinatio", "geomancy"),
    ("What are the runes?", "divinatio", "runes"),
    ("What is rhetoric?", "litterae", "rhetoric"),
    ("How is point of view chosen?", "litterae", "point-of-view"),
    (
        "What is retrieval practice?",
        "paedagogia",
        "retrieval-practice",
    ),
    (
        "Explain the Socratic method.",
        "paedagogia",
        "the-socratic-method",
    ),
    ("How is the sonnet built?", "poetica", "the-sonnet"),
    ("What is metre?", "poetica", "metre"),
    ("What is the Lemegeton?", "solomonica", "the-lemegeton"),
    (
        "Describe the magic circle.",
        "solomonica",
        "the-magic-circle",
    ),
    ("What does the Hanged Man mean?", "tarot", "the-hanged-man"),
    ("What is the Tower card?", "tarot", "the-tower"),
    // a name two libraries hold goes to the one the question speaks of
    ("What does the Sun card mean?", "tarot", "the-sun"),
    ("What does the Moon mean in tarot?", "tarot", "the-moon"),
    // a bare number names a card only when cards are spoken of
    ("What is card 13?", "tarot", "death"),
    ("What is the Liber Iuratus?", "honorius", "the-sworn-book"),
    (
        "What is the Theban alphabet?",
        "honorius",
        "honorius-of-thebes",
    ),
    // one rare word of a key is naming enough
    ("What is a villanelle?", "poetica", "the-villanelle"),
    ("What are the Pleiades?", "astronomia", "the-pleiades"),
];

const OBLIQUE: &[Case] = &[
    ("Which grain do brewers malt for ale?", "flora", "barley"),
    (
        "What herb is hung at midsummer to drive off demons?",
        "flora",
        "st-johns-wort",
    ),
    (
        "Which root is said to scream when pulled from the earth?",
        "flora",
        "mandrake",
    ),
    ("Which bitter herb flavours absinthe?", "flora", "wormwood"),
    ("What is the red planet?", "astronomia", "mars"),
    (
        "Which star's rising brings the hottest days of summer?",
        "astronomia",
        "sirius",
    ),
    (
        "Why does the sun go dark at midday?",
        "astronomia",
        "solar-eclipse",
    ),
    (
        "What is the slow drift of the equinoxes called?",
        "astronomia",
        "precession",
    ),
    (
        "What is the bright star of evening and morning?",
        "astronomia",
        "venus",
    ),
    (
        "What is the pale band of light across the night sky?",
        "astronomia",
        "the-milky-way",
    ),
    (
        "Which Elizabethan scholar spoke with angels through a crystal gazer?",
        "angelica",
        "john-dee",
    ),
    (
        "How are the ranks of heaven ordered, from seraphim down?",
        "angelica",
        "the-nine-choirs",
    ),
    (
        "Which abbot hid a cipher inside a book of spirits?",
        "angelica",
        "trithemius",
    ),
    (
        "How do I cast three coins to get a hexagram?",
        "divinatio",
        "i-ching-casting",
    ),
    (
        "What did Roman priests learn from the flight of birds?",
        "divinatio",
        "augury-and-haruspicy",
    ),
    (
        "How are meanings read from the letters of a name by their numbers?",
        "divinatio",
        "numerology",
    ),
    (
        "What do the ancients say of signs that come in sleep?",
        "divinatio",
        "dream-divination",
    ),
    (
        "How should a writer make people in a story feel alive?",
        "litterae",
        "character",
    ),
    ("What are ethos, pathos and logos?", "litterae", "rhetoric"),
    (
        "Should a tale be told in first person or third?",
        "litterae",
        "point-of-view",
    ),
    (
        "Why is testing yourself better than rereading?",
        "paedagogia",
        "retrieval-practice",
    ),
    (
        "Is it better to cram or to spread study over weeks?",
        "paedagogia",
        "spacing",
    ),
    (
        "How does a teacher lead a pupil by asking questions?",
        "paedagogia",
        "the-socratic-method",
    ),
    ("Which poem has fourteen lines?", "poetica", "the-sonnet"),
    (
        "What is unrhymed iambic pentameter called?",
        "poetica",
        "blank-verse-and-free-verse",
    ),
    (
        "Which Japanese poem counts five, seven and five?",
        "poetica",
        "the-haiku-and-tanka",
    ),
    (
        "Where must the spirit appear, outside the ring?",
        "solomonica",
        "the-triangle-of-art",
    ),
    (
        "How is a spirit dismissed when the work is done?",
        "solomonica",
        "licence-to-depart",
    ),
    (
        "In what was the king said to have imprisoned the demons?",
        "solomonica",
        "the-brass-vessel",
    ),
    (
        "What must a man swear before he is given the book of Honorius?",
        "honorius",
        "the-oath-of-honorius",
    ),
    (
        "How many masters met in council, and from which cities?",
        "honorius",
        "the-prologue",
    ),
    (
        "Which kings rule the spirits of the air?",
        "honorius",
        "the-kings-of-the-air",
    ),
    (
        "Which card shows a man hanging upside down?",
        "tarot",
        "the-hanged-man",
    ),
    (
        "Which card shows lightning striking a building?",
        "tarot",
        "the-tower",
    ),
    (
        "What does it mean when a card comes out upside down?",
        "tarot",
        "reversals",
    ),
];

/// Questions no library answers. Some are traps: a bare number, or an everyday
/// word that is also some entry's key.
const IDLE: &[&str] = &[
    "How many legions do you rule?",
    "What will the weather be at sea tomorrow?",
    "Give me 3 reasons to trust you.",
    "I was born on the 13 of March; is that ill luck?",
    "Who was the first king of Rome?",
    "Where is my lost ring?",
    "Will my brother repay his debt?",
    "What is your name and rank?",
    "Say that to my face.",
    "Tell me the history of Rome.",
    "What is the character of my rival?",
    "Shall I fear death within 12 days?",
    "Which of my servants is the thief?",
    "How far is it to the next town?",
    "Who are you?",
    "What do you want of me?",
    "Tell me a secret.",
    "What is the name of your king?",
    "Where is the treasure hidden?",
    "Will it rain tomorrow?",
    "What should I do about my work?",
    "Why are you bound here?",
    "How old is the world?",
    "Does she love me?",
    "What language do the dead speak?",
    "Teach me to be invisible.",
    "What happened in the war?",
    "Who betrayed me?",
    "Is my friend honest?",
    "What is the price of gold?",
    "Tell me of your legions and your seal.",
    "When will I die?",
    "Bring me a familiar.",
    "What is a villain?",
];

#[derive(Default)]
struct Tally {
    right: usize,
    wrong: usize,
    none: usize,
}

fn libri() -> Vec<Liber> {
    LIBRI
        .iter()
        .map(|n| {
            let path = format!("{}/scientia/{n}.txt", env!("CARGO_MANIFEST_DIR"));
            parse(n, &std::fs::read_to_string(path).expect("library file"))
        })
        .collect()
}

/// A key in one plain shape, however the library writes its hyphens.
fn plain(key: &str) -> String {
    key.to_ascii_lowercase().replace('-', " ")
}

/// The entry's text for a key, as the named library holds it.
fn entry_of<'a>(liber: &'a Liber, key: &str) -> &'a str {
    liber
        .entries
        .iter()
        .find(|e| e.claves.iter().any(|c| plain(c) == plain(key)))
        .map(|e| e.raw.as_str())
        .unwrap_or_else(|| panic!("no entry keyed {key} in {}", liber.liber))
}

/// The first line of a handed entry: its heading.
fn heading(textus: &str) -> &str {
    textus.lines().next().unwrap_or("")
}

fn tally(libri: &[Liber], cases: &[Case], label: &str) -> Tally {
    let mut t = Tally::default();
    for (question, liber, key) in cases {
        let home = libri
            .iter()
            .find(|l| l.liber == *liber)
            .expect("library named in a case");
        let want = entry_of(home, key);
        let entitled: Vec<u32> = (1..=72).filter(|n| opens(home, *n)).collect();
        assert!(!entitled.is_empty(), "{liber} opens to no spirit");
        let mut stray: Option<(u32, String)> = None;
        let mut missed = 0;
        for n in 1..=72 {
            match consult(libri, n, question) {
                Some(c) if c.textus == want => {}
                Some(c) => {
                    stray.get_or_insert((n, heading(&c.textus).to_string()));
                }
                None if entitled.contains(&n) => missed += 1,
                None => {}
            }
        }
        match (stray, missed) {
            (Some((n, got)), _) => {
                t.wrong += 1;
                println!("quaesitio {label} WRONG  {question:?}: spirit {n} was handed {got}");
            }
            (None, 0) => t.right += 1,
            (None, m) => {
                t.none += 1;
                println!(
                    "quaesitio {label} none   {question:?}: {m} of {} entitled spirits found nothing",
                    entitled.len()
                );
            }
        }
    }
    t
}

#[test]
fn plain_questions_are_always_found() {
    let libri = libri();
    let t = tally(&libri, PLAIN, "plain");
    assert_eq!(t.right, PLAIN.len(), "wrong {}, none {}", t.wrong, t.none);
}

#[test]
fn oblique_questions_never_grow_worse() {
    let libri = libri();
    let t = tally(&libri, OBLIQUE, "oblique");
    println!(
        "quaesitio: oblique right {} / wrong {} / none {} of {}",
        t.right,
        t.wrong,
        t.none,
        OBLIQUE.len()
    );
    assert!(
        t.right >= OBLIQUE_FLOOR,
        "search finds less than before: {} right, the floor is {OBLIQUE_FLOOR}",
        t.right
    );
    assert!(
        t.wrong <= OBLIQUE_WRONG_CEILING,
        "search hands more wrong entries than before: {} wrong, the ceiling is {OBLIQUE_WRONG_CEILING}",
        t.wrong
    );
}

#[test]
fn idle_questions_never_find_more() {
    let libri = libri();
    let mut found = 0;
    for question in IDLE {
        let stray = (1..=72).find_map(|n| consult(&libri, n, question).map(|c| (n, c)));
        if let Some((n, c)) = stray {
            found += 1;
            println!(
                "quaesitio idle FOUND  {question:?}: spirit {n} was handed {}",
                heading(&c.textus)
            );
        }
    }
    println!(
        "quaesitio: idle found {found} of {} (should be none)",
        IDLE.len()
    );
    assert!(
        found <= IDLE_CEILING,
        "search answers more idle questions than before: {found}, the ceiling is {IDLE_CEILING}"
    );
}
