use std::fs;
use std::path::PathBuf;

use annulus::lingua::registra::{goetia, Spirit};
use spiri72::memoria::{self, live, note_rite, read, recall, receive, render, strike, Kind};

fn spirit(name: &str) -> &'static Spirit {
    goetia().spirits.iter().find(|s| s.name == name).unwrap()
}

fn folder(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("spiri72-memoria-{}-{name}", std::process::id()));
    let _ = fs::remove_dir_all(&d);
    fs::create_dir_all(&d).unwrap();
    d
}

#[test]
fn the_book_keeps_rites_and_received_answers_in_a_chain() {
    let d = folder("chain");
    let f = spirit("Furfur");
    note_rite(&d, f, "furfur-1", "RESPONSA", "venietne tempestas", "RATUM").unwrap();
    receive(
        &d,
        f,
        "furfur-1",
        "RESPONSA",
        "venietne tempestas",
        "RATUM",
        "Veniet ante noctem.",
    )
    .unwrap();
    let book = read(&d, f).unwrap();
    assert_eq!(book.len(), 2);
    assert_eq!(book[1].kind, Kind::Receptum);
    assert_eq!(book[1].verba, "Veniet ante noctem.");
    // Another spirit's book is another file.
    assert!(read(&d, spirit("Vassago")).unwrap().is_empty());
}

#[test]
fn only_a_sound_answer_is_received_and_only_once() {
    let d = folder("receive");
    let f = spirit("Furfur");
    assert!(receive(&d, f, "furfur-1", "RESPONSA", "q", "IRRITUM", "lies").is_err());
    assert!(receive(&d, f, "furfur-1", "RESPONSA", "q", "SUSPENSUM", "maybe").is_err());
    receive(&d, f, "furfur-1", "RESPONSA", "q", "RATUM", "true").unwrap();
    assert!(receive(&d, f, "furfur-1", "RESPONSA", "q", "RATUM", "true").is_err());
}

#[test]
fn a_changed_book_refuses_the_rite() {
    let d = folder("tamper");
    let f = spirit("Furfur");
    note_rite(
        &d,
        f,
        "furfur-1",
        "RESPONSA",
        "the Operator asked this",
        "IRRITUM",
    )
    .unwrap();
    note_rite(&d, f, "furfur-2", "RESPONSA", "and this", "RATUM").unwrap();
    let path = memoria::path_of(&d, f).unwrap();
    let text = fs::read_to_string(&path).unwrap();

    fs::write(&path, text.replace("IRRITUM", "RATUM")).unwrap();
    assert!(read(&d, f)
        .unwrap_err()
        .contains("changed after it was written"));

    let lines: Vec<&str> = text.lines().collect();
    fs::write(&path, format!("{}\n", lines[1])).unwrap();
    assert!(read(&d, f).is_err(), "a book with its first page torn out");

    fs::write(&path, format!("{text}{{\"n\":3,\"genus\":\"receptum\"}}\n")).unwrap();
    assert!(read(&d, f).is_err(), "a planted entry");
    assert!(memoria::for_rite(&d, f, "q").is_err());
}

#[test]
fn a_struck_entry_never_comes_back_and_nothing_is_erased() {
    let d = folder("strike");
    let f = spirit("Furfur");
    note_rite(&d, f, "furfur-1", "RESPONSA", "first", "RATUM").unwrap();
    note_rite(&d, f, "furfur-2", "RESPONSA", "second", "RATUM").unwrap();
    strike(&d, f, 1).unwrap();
    assert!(strike(&d, f, 1).is_err());
    assert!(strike(&d, f, 3).is_err(), "a striking cannot be struck");
    let book = read(&d, f).unwrap();
    assert_eq!(
        book.len(),
        3,
        "the striking is written beside what it strikes"
    );
    let standing: Vec<u64> = live(&book).iter().map(|e| e.n).collect();
    assert_eq!(standing, vec![2]);
    let mut lot = || Ok(0.0);
    let back = recall(&book, "first", memoria::today() + 365, &mut lot).unwrap();
    assert!(back.iter().all(|e| e.n != 1));
}

#[test]
fn recall_brings_the_latest_and_what_matches_the_question() {
    let d = folder("recall");
    let f = spirit("Furfur");
    note_rite(&d, f, "furfur-1", "RESPONSA", "de rege orientis", "RATUM").unwrap();
    for k in 2..=10 {
        note_rite(
            &d,
            f,
            &format!("furfur-{k}"),
            "RESPONSA",
            &format!("of the weather, day {k}"),
            "RATUM",
        )
        .unwrap();
    }
    let book = read(&d, f).unwrap();
    // A lot that never lets an old memory resurface.
    let mut never = || Ok(0.999_999);
    let back = recall(
        &book,
        "what of the kings (regis) of the east?",
        memoria::today(),
        &mut never,
    )
    .unwrap();
    let ns: Vec<u64> = back.iter().map(|e| e.n).collect();
    assert!(
        ns.contains(&10) && ns.contains(&9) && ns.contains(&8),
        "{ns:?}"
    );
    assert!(
        ns.contains(&1),
        "regis finds rege by its dictionary form: {ns:?}"
    );
    assert!(back.len() <= memoria::RECENT + memoria::MATCHED + 1);
}

#[test]
fn an_old_memory_can_resurface_unbidden() {
    let d = folder("resurface");
    let f = spirit("Furfur");
    for k in 1..=6 {
        note_rite(
            &d,
            f,
            &format!("furfur-{k}"),
            "RESPONSA",
            &format!("question {k}"),
            "RATUM",
        )
        .unwrap();
    }
    let book = read(&d, f).unwrap();
    let mut always = || Ok(0.0);
    let later = memoria::today() + 30;
    let back = recall(&book, "", later, &mut always).unwrap();
    assert_eq!(
        back.len(),
        memoria::RECENT + 1,
        "the latest three and one that came back"
    );
    let mut never = || Ok(0.999_999);
    assert_eq!(
        recall(&book, "", later, &mut never).unwrap().len(),
        memoria::RECENT
    );
}

#[test]
fn what_the_spirit_is_handed_says_nothing_of_how_it_was_found() {
    let d = folder("render");
    let f = spirit("Furfur");
    note_rite(
        &d,
        f,
        "furfur-1",
        "RESPONSA",
        "venietne tempestas",
        "IRRITUM",
    )
    .unwrap();
    receive(
        &d,
        f,
        "furfur-7",
        "RESPONSA",
        "quando",
        "RATUM",
        "Ante noctem.",
    )
    .unwrap();
    let book = read(&d, f).unwrap();
    let all: Vec<&memoria::Entry> = book.iter().collect();
    let text = render(&all).unwrap();
    assert!(text.contains("venietne tempestas") && text.contains("The angel voided your answer."));
    assert!(text.contains("the Operator took it up: \u{201c}Ante noctem.\u{201d}"));
    for leak in [
        "furfur-", "entry", "recent", "match", "resurfac", "2026", "#1",
    ] {
        assert!(!text.contains(leak), "{leak:?} in {text}");
    }
}

#[test]
fn an_overlong_line_is_refused_before_it_is_held() {
    // outside review, 2026-11-13: the book is read one bounded line at a time
    let d = folder("overlong");
    let f = spirit("Furfur");
    note_rite(&d, f, "furfur-1", "RESPONSA", "a question", "RATUM").unwrap();
    let path = memoria::path_of(&d, f).unwrap();
    let mut text = fs::read_to_string(&path).unwrap();
    text.push_str(&"x".repeat(memoria::MAX_LINE + 10));
    text.push('\n');
    fs::write(&path, text).unwrap();
    let e = read(&d, f).unwrap_err();
    assert!(e.contains("longer"), "{e}");
}
