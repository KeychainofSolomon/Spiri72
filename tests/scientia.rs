//! The knowledge libraries: parsing, gating by inclination, and narrowing a
//! question to one entry.

use spiri72::scientia::{self, consult, open_to, parse, Consultatio};

const HERBAL: &str = "\
# liber: flora
# nomen: the herbal
# claves: herb, plant, tree

== barley | hordeum | bere ==
Barley, a grass sown for grain. Cold and dry.

== rue | ruta | herb-of-grace ==
Rue, hot and dry in the third degree. Bitter.
";

#[test]
fn parse_reads_header_and_entries() {
    let l = parse("flora", HERBAL);
    assert_eq!(l.liber, "flora");
    assert_eq!(l.nomen, "the herbal");
    assert_eq!(l.claves, ["herb", "plant", "tree"]);
    assert_eq!(l.entries.len(), 2);
    assert_eq!(l.entries[0].claves, ["barley", "hordeum", "bere"]);
    // the heading stays in the verbatim copy
    assert!(l.entries[0].raw.starts_with("== barley"));
    assert!(l.entries[0].raw.contains("Cold and dry."));
    // a multi-word alias stays one key, matched as a whole
    assert_eq!(l.entries[1].claves, ["rue", "ruta", "herb of grace"]);
}

#[test]
fn open_only_to_matching_inclination() {
    let l = parse("flora", HERBAL);
    // a spirit who teaches the virtues of herbs may read it (singular meets plural)
    assert!(open_to(
        &l,
        &["He knoweth the virtues of herbs and plants".to_string()]
    ));
    // one who only raises storms may not
    assert!(!open_to(
        &l,
        &["He raiseth lightnings, thunders and great storms".to_string()]
    ));
}

#[test]
fn narrow_finds_the_named_entry() {
    let libri = vec![parse("flora", HERBAL)];
    // Buer (10) teaches the virtues of herbs, so the herbal opens to him
    let hit = consult(&libri, 10, "good Buer, what is barley good for?");
    assert_eq!(
        hit,
        Some(Consultatio {
            liber: "the herbal".to_string(),
            terminus: "barley".to_string(),
            textus: libri[0].entries[0].raw.clone(),
        })
    );
}

#[test]
fn plural_in_question_still_finds_entry() {
    let libri = vec![parse("flora", HERBAL)];
    // "barleys" singularises to meet the key "barley"
    assert!(consult(&libri, 10, "tell me of barleys").is_some());
}

#[test]
fn closed_library_yields_nothing() {
    let libri = vec![parse("flora", HERBAL)];
    // Furfur (34) raises storms; the herbal is shut to him even when he asks of barley
    assert_eq!(consult(&libri, 34, "what of barley, Furfur?"), None);
}

#[test]
fn no_named_entry_yields_nothing() {
    let libri = vec![parse("flora", HERBAL)];
    // Buer may read the herbal, but nothing here is named
    assert_eq!(consult(&libri, 10, "tell me of the weather at sea"), None);
}

#[test]
fn empty_libraries_yield_nothing() {
    assert_eq!(scientia::consult(&[], 10, "barley"), None);
}
