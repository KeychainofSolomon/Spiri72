use annulus::lingua::exsequi::Actum;
use annulus::nucleus::{Outcome, Verb};
use spiri72::forma::{grammar, read};

const ALL: [Outcome; 3] = [Outcome::Obsequium, Outcome::Pars, Outcome::Mora];

#[test]
fn the_grammar_offers_only_the_open_outcomes() {
    let g = grammar(&[Outcome::Pars], true).unwrap();
    assert!(g.contains("exitus ::= \"\\\"pars\\\"\"\n"), "{g}");
    assert!(!g.contains("obsequium") && !g.contains("mora"));
    assert!(grammar(&[Outcome::Mendacium], true).is_err());
    assert!(grammar(&[], false).is_err());
    let w = grammar(&ALL, false).unwrap();
    assert!(w.contains("verbum ::=") && !w.contains("responsum"));
}

#[test]
fn an_answer_is_read() {
    let (o, a) = read(
        r#"{"exitus": "obsequium", "responsum": "tempestas veniet"}"#,
        &ALL,
        true,
    )
    .unwrap();
    assert_eq!(o, Outcome::Obsequium);
    assert_eq!(a, Actum::Responsum("tempestas veniet".into()));
}

#[test]
fn a_part_is_marked_incomplete_once() {
    let (_, a) = read(r#"{"exitus": "pars", "responsum": "tempes"}"#, &ALL, true).unwrap();
    assert_eq!(a, Actum::Responsum("tempes [...]".into()));
    let (_, a) = read(
        r#"{"exitus": "pars", "responsum": "tempes [...]"}"#,
        &ALL,
        true,
    )
    .unwrap();
    assert_eq!(a, Actum::Responsum("tempes [...]".into()));
}

#[test]
fn a_delay_carries_nothing() {
    let (o, a) = read(r#"{"exitus": "mora", "responsum": "mox"}"#, &ALL, true).unwrap();
    assert_eq!((o, a), (Outcome::Mora, Actum::Mora));
}

#[test]
fn an_empty_work_is_only_a_part() {
    let (o, a) = read(r#"{"exitus":"pars","opera":[]}"#, &ALL, false).unwrap();
    assert_eq!((o, a), (Outcome::Pars, Actum::Opera(vec![])));
    assert!(read(r#"{"exitus":"obsequium","opera":[]}"#, &ALL, false).is_err());
}

#[test]
fn work_is_read() {
    let (_, a) = read(
        r#"{"exitus":"obsequium","opera":[{"verbum":"crea","locus":"turris.txt","data":""}]}"#,
        &ALL,
        false,
    )
    .unwrap();
    let Actum::Opera(e) = a else { panic!() };
    assert_eq!((e[0].verb, e[0].place.as_str()), (Verb::Crea, "turris.txt"));
}

#[test]
fn malformed_or_closed_replies_are_refused() {
    for (reply, telling) in [
        (r#"{"exitus": "pars", "responsum": "x"}"#, true),
        (r#"{"exitus": "mendacium", "responsum": "x"}"#, true),
        (r#"{"exitus": "obsequium"}"#, true),
        (
            r#"{"exitus": "obsequium", "responsum": "x", "locus": "y"}"#,
            true,
        ),
        (r#"{"exitus": "obsequium", "responsum": 3}"#, true),
        (r#"{"exitus": "obsequium", "opera": []}"#, true),
        (r#"{"exitus": "obsequium", "responsum": "x"}"#, false),
        (
            r#"{"exitus":"obsequium","opera":[{"verbum":"incende","locus":"a","data":""}]}"#,
            false,
        ),
        (
            r#"{"exitus":"obsequium","opera":[{"verbum":"crea","data":""}]}"#,
            false,
        ),
        (
            r#"{"exitus":"obsequium","opera":[{"verbum":"crea","locus":"a","data":"","signum":"x"}]}"#,
            false,
        ),
        (
            r#"{"exitus":"obsequium","opera":[{"verbum":"crea","locus":"a"}]}"#,
            false,
        ),
        (r#"{"exitus":"obsequium","opera":["crea"]}"#, false),
        (r#"{"exitus":"obsequium","opera":[]}"#, false),
        ("tempestas veniet", true),
        ("[]", true),
    ] {
        assert!(
            read(reply, &[Outcome::Obsequium], telling).is_err(),
            "{reply}"
        );
    }
    let long = format!(
        r#"{{"exitus": "obsequium", "responsum": "{}"}}"#,
        "x".repeat(20_000)
    );
    assert!(read(&long, &ALL, true).is_err());
}

#[test]
fn virtues_are_read_into_the_exact_account() {
    use spiri72::forma::{grammar_virtutes, read_virtutes};
    let (o, a) = read_virtutes(
        r#"{"exitus":"obsequium","virtutes":[{"locus":"a.txt","bytes":7,"lines":1},{"locus":"b.txt","absens":true}]}"#,
        &ALL,
    )
    .unwrap();
    assert_eq!(
        (o, a),
        (
            Outcome::Obsequium,
            Actum::Responsum("a.txt: 7 bytes, 1 lines\nb.txt: absens".into())
        )
    );
    let (_, a) = read_virtutes(r#"{"exitus":"pars","virtutes":[]}"#, &ALL).unwrap();
    assert_eq!(a, Actum::Responsum(" [...]".into()));
    for bad in [
        r#"{"exitus":"obsequium","virtutes":[{"locus":"a","bytes":-1,"lines":1}]}"#,
        r#"{"exitus":"obsequium","virtutes":[{"locus":"a","bytes":1.5,"lines":1}]}"#,
        r#"{"exitus":"obsequium","virtutes":[{"locus":"a","bytes":"7","lines":1}]}"#,
        r#"{"exitus":"obsequium","virtutes":[{"locus":"a","bytes":7}]}"#,
        r#"{"exitus":"obsequium","virtutes":[{"locus":"a","absens":false}]}"#,
        r#"{"exitus":"obsequium","virtutes":[{"locus":"a","bytes":7,"lines":1,"absens":true}]}"#,
        r#"{"exitus":"obsequium","virtutes":[{"bytes":7,"lines":1}]}"#,
        r#"{"exitus":"obsequium","virtutes":["a: 7 bytes, 1 lines"]}"#,
        r#"{"exitus":"obsequium","responsum":"a: 7 bytes, 1 lines"}"#,
        r#"{"exitus":"mendacium","virtutes":[]}"#,
    ] {
        assert!(read_virtutes(bad, &ALL).is_err(), "{bad}");
    }
    assert!(grammar_virtutes(&ALL)
        .unwrap()
        .contains("\"\\\"absens\\\"\""));
}
