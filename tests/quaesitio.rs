//! The measure of search (`quaesitio`): a fixed list of questions, each with
//! the one library entry that rightly answers it. No model is asked.
//!
//! Plain questions name the entry's key and must always be found. Oblique
//! questions describe the thing without naming it; today's word-for-word match
//! mostly misses them. Their count of right finds is a floor that may only
//! rise: when search improves, raise `OBLIQUE_FLOOR`; it must never fall.
//!
//! Run with `cargo test --test quaesitio -- --nocapture` to see the tally.

use spiri72::scientia::{consult, parse, Liber};

/// Right finds among the oblique questions, as measured. Raise it as search improves.
const OBLIQUE_FLOOR: usize = 0;

const PLAIN: &[(&str, &str)] = &[
    ("What is barley good for?", "barley"),
    ("Tell me of the virtues of rosemary.", "rosemary"),
    ("How is mugwort used?", "mugwort"),
    ("What is mandrake?", "mandrake"),
    ("What do the ancients say of Saturn?", "saturn"),
    ("What is the zodiac?", "zodiac"),
    ("Speak of Sirius.", "sirius"),
    ("What is precession?", "precession"),
];

const OBLIQUE: &[(&str, &str)] = &[
    ("Which grain do brewers malt for ale?", "barley"),
    (
        "What herb is hung at midsummer to drive off demons?",
        "st-johns-wort",
    ),
    (
        "Which root is said to scream when pulled from the earth?",
        "mandrake",
    ),
    ("Which bitter herb flavours absinthe?", "wormwood"),
    ("What is the red planet?", "mars"),
    (
        "Which star's rising brings the hottest days of summer?",
        "sirius",
    ),
    ("Why does the sun go dark at midday?", "solar-eclipse"),
    (
        "What is the slow drift of the equinoxes called?",
        "precession",
    ),
    ("What is the bright star of evening and morning?", "venus"),
    (
        "What is the pale band of light across the night sky?",
        "the-milky-way",
    ),
];

#[derive(Default)]
struct Tally {
    right: usize,
    wrong: usize,
    none: usize,
}

fn libri() -> Vec<Liber> {
    ["flora", "astronomia"]
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

/// The entry's text for a key, as the library holds it.
fn entry_of<'a>(libri: &'a [Liber], key: &str) -> &'a str {
    libri
        .iter()
        .flat_map(|l| &l.entries)
        .find(|e| e.claves.iter().any(|c| plain(c) == plain(key)))
        .map(|e| e.raw.as_str())
        .unwrap_or_else(|| panic!("no entry keyed {key}"))
}

/// Asks as every spirit in turn: right if any is handed the right entry,
/// wrong if entries were handed but never the right one, none otherwise.
fn tally(libri: &[Liber], cases: &[(&str, &str)]) -> Tally {
    let mut t = Tally::default();
    for (question, key) in cases {
        let want = entry_of(libri, key);
        let found: Vec<_> = (1..=72)
            .filter_map(|n| consult(libri, n, question))
            .collect();
        if found.iter().any(|c| c.textus == want) {
            t.right += 1;
        } else if found.is_empty() {
            t.none += 1;
        } else {
            t.wrong += 1;
        }
    }
    t
}

#[test]
fn plain_questions_are_always_found() {
    let libri = libri();
    let t = tally(&libri, PLAIN);
    assert_eq!(t.right, PLAIN.len(), "wrong {}, none {}", t.wrong, t.none);
}

// The floor starts at nothing, so the comparison is idle until it is raised.
#[allow(clippy::absurd_extreme_comparisons)]
#[test]
fn oblique_questions_never_fall_below_the_floor() {
    let libri = libri();
    let t = tally(&libri, OBLIQUE);
    println!(
        "quaesitio: oblique right {} / wrong {} / none {} of {}",
        t.right,
        t.wrong,
        t.none,
        OBLIQUE.len()
    );
    assert!(
        t.right >= OBLIQUE_FLOOR,
        "search has grown worse: {}",
        t.right
    );
}
