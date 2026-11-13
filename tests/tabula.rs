use annulus::lingua::registra::{goetia, Spirit};
use annulus::nucleus::Outcome;
use spiri72::forma::{grammar, grammar_working};
use spiri72::tabula::{facultates, office_words, titulus};

fn holder(office: &str) -> &'static Spirit {
    goetia()
        .spirits
        .iter()
        .find(|s| s.offices.iter().any(|o| o == office))
        .unwrap_or_else(|| panic!("no spirit holds {office}"))
}

#[test]
fn every_office_annulus_carries_out_is_described() {
    for s in &goetia().spirits {
        for o in &s.offices {
            let works = annulus::officia::signature(o).is_some();
            assert_eq!(office_words(o).is_some(), works, "{} holds {o}", s.name);
        }
    }
}

#[test]
fn the_fixed_line_comes_from_the_register() {
    let bael = goetia().by_number(1).unwrap();
    let t = titulus(bael);
    assert!(t.contains("your name is Bael"), "{t}");
    assert!(t.contains("spirit 1 of the 72"), "{t}");
    assert!(t.contains("your rank is king"), "{t}");
    assert!(t.contains("you rule 66 legions"), "{t}");
    for o in &bael.offices {
        assert!(t.contains(o.as_str()), "{t}");
    }
}

#[test]
fn the_powers_name_each_place_and_what_it_allows() {
    let s = holder("EVERSIO");
    let loci = vec!["old.txt".to_string(), "older.txt".to_string()];
    let f = facultates(s, "EVERSIO", None, &loci);
    assert!(f.contains("charged through EVERSIO alone"), "{f}");
    assert!(
        f.contains("- old.txt: you may delete; verbs: dele, aufer"),
        "{f}"
    );
    assert!(f.contains("- older.txt: you may delete"), "{f}");
    assert!(f.contains("no other office, place, verb or power"), "{f}");
    let telling = facultates(goetia().by_number(1).unwrap(), "PRAESENTIA", None, &[]);
    assert!(telling.contains("No place is handed to you"), "{telling}");
    // Bael's invisibility is recorded, but Annulus has no working for it.
    assert!(
        telling.contains("Recorded of you, but with no working in Annulus")
            && telling.contains("INVISIBILITAS"),
        "{telling}"
    );
}

#[test]
fn a_working_grammar_allows_only_handed_places_and_their_verbs() {
    let open = [Outcome::Obsequium, Outcome::Pars, Outcome::Mora];
    let loci = vec!["a \"quoted\" place.txt".to_string()];
    let g = grammar_working(&open, "EVERSIO", None, &loci).unwrap();
    assert!(g.contains("\\\"dele\\\""), "{g}");
    assert!(!g.contains("\\\"crea\\\""), "{g}");
    assert!(!g.contains("verbum ::="), "{g}");
    // The place is held as one exact JSON string.
    assert!(g.contains(r#""\"a \\\"quoted\\\" place.txt\"""#), "{g}");
    let moved = grammar_working(
        &open,
        "VECTIO",
        Some("transfer"),
        &["from.txt".to_string(), "to.txt".to_string()],
    )
    .unwrap();
    assert!(
        moved.contains("from.txt") && moved.contains("to.txt"),
        "{moved}"
    );
    // An office Annulus gives no table for keeps the full grammar.
    assert_eq!(
        grammar_working(&open, "EVERSIO", None, &[]).unwrap(),
        grammar(&open, false).unwrap()
    );
}
