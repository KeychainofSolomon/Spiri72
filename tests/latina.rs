use spiri72::latina::{analyse, glossa, glossa_of, is_latin, norm};

fn has(word: &str, lemma: &str, form: &str) -> bool {
    analyse(word)
        .iter()
        .any(|l| l.lemma.starts_with(lemma) && l.form == form)
}

#[test]
fn spelling_is_read_as_the_lexicon_writes_it() {
    assert_eq!(norm("Juravi"), "iuraui");
    assert_eq!(norm("cœlum"), "coelum");
    assert_eq!(norm("Æternus"), "aeternus");
}

#[test]
fn nouns_verbs_and_adjectives_are_parsed() {
    assert!(has("regis", "rex, regis", "genitive singular"));
    assert!(has("noctis", "nox, noctis", "genitive singular"));
    assert!(has(
        "constringi",
        "constringo",
        "present passive infinitive"
    ));
    assert!(has(
        "invocaretur",
        "inuoco",
        "imperfect passive subjunctive 3rd person singular"
    ));
    assert!(has(
        "hortatur",
        "hortor",
        "present passive indicative 3rd person singular"
    ));
    assert!(analyse("magna")
        .iter()
        .any(|l| l.lemma.starts_with("magnus")));
}

#[test]
fn esse_is_known() {
    assert!(has(
        "est",
        "sum, esse",
        "present active indicative 3rd person singular"
    ));
    assert!(has(
        "fuit",
        "sum, esse",
        "perfect active indicative 3rd person singular"
    ));
}

#[test]
fn a_deponent_has_no_active_finite_forms() {
    assert!(!analyse("horto")
        .iter()
        .any(|l| l.lemma.starts_with("hortor")));
}

#[test]
fn enclitics_are_taken_off() {
    let r = analyse("regesque");
    assert!(r
        .iter()
        .any(|l| l.lemma.starts_with("rex") && l.form.ends_with("+ -que (and)")));
}

#[test]
fn late_latin_is_left_out() {
    // Church Latin: "diabolus" and "baptizare" are late (age D or after) in WORDS.
    assert!(analyse("diabolus").is_empty());
    assert!(analyse("baptizare").is_empty());
    assert!(analyse("xyzzy").is_empty());
}

#[test]
fn english_gets_no_reading_aid() {
    assert!(!is_latin("The storm will come over the harbour tonight"));
    assert!(glossa("The storm will come over the harbour tonight").is_none());
    assert!(glossa_of(&["What do the old books say?"]).is_none());
}

#[test]
fn latin_gets_one() {
    let text = "in aliqua hora diei non potest constringi, nisi rex cui paret invocaretur";
    assert!(is_latin(text));
    let g = glossa_of(&["Tell me of this.", text]).unwrap();
    assert!(g.contains("constringi: constringo"), "{g}");
    assert!(g.contains("rex: rex, regis (M)"), "{g}");
    assert_eq!(
        g.matches("\nin: ").count() + usize::from(g.starts_with("in: ")),
        1
    );
}
