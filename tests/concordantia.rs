//! The concordance: word families, what a book defines itself, the
//! Operator's own groups, and how a question meets a book through them.

use spiri72::concordantia::{of_book, radices, radix, Concordantia, SEMEN};
use spiri72::volumina::{bind, bind_with, consult_as, purga, titulus};

const KEEP: &[&str] = &["name", "named", "names", "call", "called", "use", "used"];

#[test]
fn a_word_family_meets_on_one_root() {
    for (a, b) in [
        ("digestion", "digestive"),
        ("conservation", "conserve"),
        ("toxicity", "toxic"),
        ("functional", "function"),
    ] {
        assert_eq!(radix(a), radix(b), "{a} and {b}");
    }
    // never cut below four letters, nor a number
    assert_eq!(radix("nation"), "nation");
    assert_eq!(radix("1500"), "1500");
    // a word that stems to a word empty of matter is empty too
    assert!(radices("what makes a good class", KEEP)
        .iter()
        .all(|w| w != "make"));
}

#[test]
fn the_operators_groups_are_read_and_applied() {
    let c = Concordantia::parse(
        "# a note\nliver = hepatic\nlambda = anonymous procedure\n\n",
        KEEP,
    );
    assert_eq!(c.len(), 2);
    let read = |t: &str| c.apply(radices(t, KEEP));
    assert_eq!(read("hepatic remedies"), read("liver remedies"));
    assert_eq!(read("an anonymous procedure"), read("lambda"));
    // the shipped concordance parses whole
    assert!(Concordantia::parse(SEMEN, KEEP).len() > 30);
}

#[test]
fn a_book_teaches_its_own_abbreviations_and_plant_names() {
    let pages = vec![Some(
        "The Single Responsibility Principle (SRP) says a class has one reason to change. \
         We return to the SRP often. Hypericum perforatum (St. John's wort) is a nervine. \
         Hypericum perforatum (St. John's wort) eases neuralgia. Silybum marianum (milk thistle) \
         appears once only. The doctor saw us (US) and IT staff (IT) and it was fine, as it is."
            .to_string(),
    )];
    let c = of_book(&pages, KEEP);
    let groups = c.groups();
    let has = |from: &str, to: &str| groups.iter().any(|(p, f)| p == from && f == to);
    assert!(has("singl responsibil principl", "srp"), "{groups:?}");
    assert!(has("st john s wort", "hypericumperforatum"), "{groups:?}");
    // set out once only: not taught
    assert!(
        !groups.iter().any(|(_, f)| f == "silybummarianum"),
        "{groups:?}"
    );
    // an abbreviation that is a plain word of the book is no abbreviation
    assert!(
        !groups.iter().any(|(_, f)| f == "it" || f == "us"),
        "{groups:?}"
    );
}

#[test]
fn a_question_meets_a_book_through_the_concordance() {
    let pages: Vec<Option<String>> = vec![
        Some("Of the cooling herbs. Many herbs cool the body and calm a fever in summer.".into()),
        Some(
            "Hypericum perforatum L.\nSt. John's Wort\nHypericum perforatum (St. John's wort) \
             eases neuralgia and calms the nerves; Hypericum perforatum is a nervine."
                .into(),
        ),
        Some("Hypericum perforatum (St. John's wort) is gathered in midsummer.".into()),
        Some("Remedies for the hepatic system protect the hepatic cells from toxins.".into()),
        Some("Of roots and barks. Roots are dug in autumn and dried in the shade.".into()),
    ];
    let t = titulus("# claves: herb\n");
    let offices = ["herbs".to_string()];
    let ask = |v: &spiri72::volumina::Volumen, q: &str| -> Vec<String> {
        consult_as(std::slice::from_ref(v), &offices, q, 3000)
            .into_iter()
            .map(|l| l.locus)
            .collect()
    };
    // the book's own names: a question in the common name finds the Latin
    let v = bind("herbal", &t, &pages, true, 3000);
    assert!(ask(&v, "what does hypericum perforatum do for the nerves").contains(&"page 2".into()));
    // the Operator's groups: liver meets hepatic
    let plain = ask(&v, "how is the liver protected");
    assert!(!plain.contains(&"page 4".into()), "{plain:?}");
    let common = Concordantia::parse("liver = hepatic", KEEP);
    let v = bind_with("herbal", &t, &pages, true, 3000, &common);
    let found = ask(&v, "how is the liver protected");
    assert!(found.contains(&"page 4".into()), "{found:?}");
}

#[test]
fn joined_letters_are_spelled_out() {
    assert_eq!(
        purga("de\u{fb01}nition, \u{fb02}ow, di\u{fb00}er, o\u{fb03}ce"),
        "definition, flow, differ, office"
    );
    // the long s of old printing
    assert_eq!(
        purga("\u{17f}oldiers of the Legion"),
        "soldiers of the Legion"
    );
}
