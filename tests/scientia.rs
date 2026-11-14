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

#[test]
fn a_short_key_is_not_met_by_a_common_word() {
    // `ares` once lost its `s` and met every question holding the word `are`
    let l = parse(
        "astronomia",
        "# claves: herb\n\n== mars | ares ==\nMars, the red planet.\n",
    );
    assert_eq!(consult(&[l], 10, "what are the virtues of this?"), None);
}

#[test]
fn a_phrase_clave_must_stand_whole_within_one_office() {
    let l = parse("x", "# claves: things to come\n\n== a ==\nA.\n");
    assert_eq!(l.claves, ["things to come"]);
    assert!(open_to(&l, &["declares things to come".to_string()]));
    // the single word is not the phrase
    assert!(!open_to(
        &l,
        &["compels the souls of the dead to come before the exorcist".to_string()]
    ));
    // nor may the phrase be pieced together across two offices
    assert!(!open_to(
        &l,
        &[
            "knows all things".to_string(),
            "to come quickly".to_string()
        ]
    ));
}

/// The starter libraries, as shipped.
fn starters() -> Vec<scientia::Liber> {
    ["astronomia", "flora", "tarot"]
        .iter()
        .map(|n| {
            let path = format!("{}/scientia/{n}.txt", env!("CARGO_MANIFEST_DIR"));
            parse(n, &std::fs::read_to_string(path).expect("library file"))
        })
        .collect()
}

#[test]
fn a_described_entry_is_found_without_its_name() {
    let libri = starters();
    // Buer reads the herbal; the question never says "barley"
    let hit = consult(&libri, 10, "Which grain do brewers malt for ale?").expect("a finding");
    assert_eq!(hit.terminus, "barley");
    assert!(hit.textus.starts_with("== barley"));
}

#[test]
fn a_closed_library_is_never_reached_by_description() {
    let libri = starters();
    // Furfur (34) has no office of herbs or stars: however the thing is
    // described, nothing of the herbal or the heavens is handed to him
    for q in [
        "Which grain do brewers malt for ale?",
        "What is the slow drift of the equinoxes called?",
        "Which star's rising brings the hottest days of summer?",
    ] {
        let hit = consult(&libri, 34, q);
        let from_shut = hit
            .as_ref()
            .is_some_and(|c| c.liber == "the herbal" || c.liber == libri[0].nomen);
        assert!(!from_shut, "{q:?} reached a closed library: {hit:?}");
    }
}

#[test]
fn a_bare_number_names_nothing_without_its_matter() {
    let libri = starters();
    // Vassago (3) reads the cards; a day of the month is not a card
    assert_eq!(consult(&libri, 3, "I was born on the 13 of March."), None);
    let hit = consult(&libri, 3, "What is card 13?").expect("the card");
    assert_eq!(hit.terminus, "13");
}

#[test]
fn a_name_two_libraries_hold_goes_where_the_question_points() {
    let libri = starters();
    // Marax (21) reads both the heavens and the cards
    let sky = consult(&libri, 21, "What is the sun?").expect("the sun");
    assert!(sky.textus.starts_with("== sun |"));
    let card = consult(&libri, 21, "What does the Sun card mean?").expect("the card");
    assert!(card.textus.starts_with("== the-sun |"));
    // Asmoday (32) reads the heavens but not the cards: the card is not his
    assert_eq!(consult(&libri, 32, "What does the Sun card mean?"), None);
}

#[test]
fn a_homonym_in_an_office_opens_no_library() {
    let libri = starters();
    let tarot = libri.iter().find(|l| l.liber == "tarot").expect("tarot");
    // Vassago (3) declares things to come: the cards are his
    assert!(scientia::opens(tarot, 3));
    // Murmur (54) compels the dead "to come" before the exorcist, and Sitri
    // (12) discloses "the secrets of women": neither is an office of divining
    assert!(!scientia::opens(tarot, 54));
    assert!(!scientia::opens(tarot, 12));
}
