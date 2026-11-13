//! The tables: what is fixed of each of the 72, held by Spiri72 and never
//! asked of the model. Name, rank, legions and offices come from the Annulus
//! Goetia register and nowhere else. The prompt states them, the custos
//! checks the spirit's words against them, and the terminal and the browser
//! show them, so the model never has to know them.

use annulus::lingua::registra::{goetia, Register, Spirit};
use std::sync::OnceLock;

/// The whole register (Annulus reads it once).
pub fn register() -> &'static Register {
    goetia()
}

/// The words that name each rank, English and Latin. "count" is the old
/// English for earl.
pub const RANKS: &[(&str, &[&str])] = &[
    ("king", &["king", "rex"]),
    ("duke", &["duke", "dux"]),
    ("prince", &["prince", "princeps"]),
    ("marquis", &["marquis", "marquess", "marchio"]),
    ("earl", &["earl", "count", "comes"]),
    ("president", &["president", "praeses"]),
    ("knight", &["knight", "eques"]),
];

/// The rank a word names, if any.
pub fn rank_of_word(w: &str) -> Option<&'static str> {
    RANKS
        .iter()
        .find(|(_, words)| words.contains(&w))
        .map(|(r, _)| *r)
}

/// Whether the spirit holds this rank (a spirit may hold two).
pub fn holds_rank(s: &Spirit, rank: &str) -> bool {
    s.ranks.iter().any(|r| r == rank)
}

/// The fixed line for a spirit, as the prompt gives it.
pub fn titulus(s: &Spirit) -> String {
    let offices = if s.offices.is_empty() {
        "none".to_string()
    } else {
        s.offices.join(", ")
    };
    format!(
        "your name is {}; you are spirit {} of the 72; your rank is {}; you rule {} legions; your offices under the Constitution are {}",
        s.name,
        s.n,
        s.ranks.join(" and "),
        s.legions,
        offices
    )
}

/// Every other spirit's name in lower case, with its number, for the
/// custos to tell when a sentence speaks of someone else.
pub fn names() -> &'static [(u32, String)] {
    static N: OnceLock<Vec<(u32, String)>> = OnceLock::new();
    N.get_or_init(|| {
        register()
            .spirits
            .iter()
            .map(|s| (s.n, s.name.to_lowercase()))
            .collect()
    })
}

/// What each office does in Annulus, in plain words. The machinery is
/// Annulus's own (`annulus::officia`); this only says it.
pub const OFFICES: &[(&str, &str)] = &[
    (
        "PRAESENTIA",
        "tell what is present: you read the texts handed to you and answer",
    ),
    (
        "FUTURA",
        "tell what is to come: you read the texts handed to you and answer",
    ),
    (
        "OCCULTA",
        "tell what is hidden: you read the texts handed to you and answer",
    ),
    (
        "THESAURI",
        "tell of treasures: you read the texts handed to you and answer",
    ),
    (
        "AMISSA",
        "tell of what is lost: you read the texts handed to you and answer",
    ),
    (
        "RESPONSA",
        "answer questions: you read the texts handed to you and answer",
    ),
    (
        "DOCTRINA",
        "teach your arts: you answer, and give a certificate of the computations asked, which the angel computes again",
    ),
    (
        "PRAETERITA",
        "tell what is past in this run; the angel checks every claim against the record",
    ),
    (
        "VIRTUTES",
        "measure the places handed to you, their bytes and lines; the angel counts them again",
    ),
    ("AEDIFICATIO", "build: make new places with their contents"),
    ("EVERSIO", "overthrow: delete the places handed to you"),
    ("VECTIO", "carry: move or copy one place into one new place"),
    (
        "FURTUM",
        "take away: remove one place and set its contents in one new place",
    ),
    (
        "CONCORDIA",
        "reconcile: join sources into one target that you write",
    ),
    ("DISCORDIA", "divide: split one source into new places"),
];

/// The plain words for an office, if it is one Annulus knows.
pub fn office_words(office: &str) -> Option<&'static str> {
    OFFICES.iter().find(|(o, _)| *o == office).map(|(_, w)| *w)
}

/// The verbs a work may use on a place, given what the place allows.
pub fn verbs_for(perms: &[annulus::nucleus::Perm]) -> Vec<&'static str> {
    crate::forma::VERBS
        .iter()
        .filter(|(_, v)| perms.contains(&v.needs()))
        .map(|(w, _)| *w)
        .collect()
}

/// What may be done to a place, in plain words.
pub fn perm_words(perms: &[annulus::nucleus::Perm]) -> String {
    use annulus::nucleus::Perm::*;
    perms
        .iter()
        .map(|p| match p {
            Legere => "read",
            Scribere => "write",
            Creare => "create",
            Delere => "delete",
        })
        .collect::<Vec<_>>()
        .join(" and ")
}

/// The spirit's mechanical powers: every office it holds, then exactly what
/// this rite allows it, place by place. Nothing outside this exists for it.
pub fn facultates(s: &Spirit, office: &str, modus: Option<&str>, loci: &[String]) -> String {
    let (working, recorded): (Vec<&String>, Vec<&String>) = s
        .offices
        .iter()
        .partition(|o| annulus::officia::signature(o).is_some());
    let mut out =
        String::from("What you can do, by the offices you hold that Annulus can carry out:\n");
    if working.is_empty() {
        out.push_str("- none\n");
    }
    for o in working {
        let words = office_words(o).unwrap_or("an office Annulus carries out");
        out.push_str(&format!("- {o}: {words}\n"));
    }
    if !recorded.is_empty() {
        let names: Vec<&str> = recorded.iter().map(|o| o.as_str()).collect();
        out.push_str(&format!(
            "Recorded of you, but with no working in Annulus, so not to be called on here: {}.\n",
            names.join(", ")
        ));
    }
    if !s.artes.is_empty() {
        let checked: Vec<&str> = annulus::doctrina::KINDS
            .iter()
            .map(|(_, art)| *art)
            .filter(|art| s.artes.iter().any(|a| a == art))
            .collect();
        out.push_str(&format!("Your arts: {}.", s.artes.join(", ")));
        if checked.is_empty() {
            out.push_str(" The angel can check none of them yet, so your lessons are held.\n");
        } else {
            out.push_str(&format!(
                " Of these the angel can check {} by computing again.\n",
                checked.join(", ")
            ));
        }
    }
    out.push_str(&format!(
        "\nIn this rite you are charged through {office} alone."
    ));
    if let Some(m) = modus {
        out.push_str(&format!(" The manner is {m}."));
    }
    let telling = annulus::officia::signature(office).map(|g| g.class)
        == Some(annulus::officia::Class::Telling);
    match annulus::officia::places(office, modus, loci.len()) {
        Some(perms) if !loci.is_empty() => {
            out.push_str(" These are the only places you may touch:\n");
            for (place, p) in loci.iter().zip(&perms) {
                out.push_str(&format!("- {place}: you may {}", perm_words(p)));
                if !telling {
                    let verbs = verbs_for(p);
                    if !verbs.is_empty() {
                        out.push_str(&format!("; verbs: {}", verbs.join(", ")));
                    }
                }
                out.push('\n');
            }
        }
        _ => out.push_str(" No place is handed to you.\n"),
    }
    out.push_str(
        "You have no other office, place, verb or power; anything else you propose is refused.",
    );
    out
}
