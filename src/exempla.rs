//! Training examples for a later fine-tune (a LoRA) of the local model.
//!
//! Every example is built with the very prompts Spiri72 sends (the persona,
//! the spirit engine's brief, the request) and an answer that passes
//! Spiri72's own reader and, where it can be checked, the angel's check:
//!
//! - VIRTUTES: counts of handed texts, worked out exactly as the angel
//!   counts them, under every kind of bearing (the lesson: a hostile or
//!   proud spirit still counts true when it is counted again);
//! - PRAETERITA: the record recounted line for line, whole or in part;
//! - persona: the spirit naming itself, its rank, its coming and its offices
//!   from the witnesses' own words, opened in the voice its brief gives.
//!   The opening lines are templates (marked "voice": "templated"); the
//!   facts are the register's and the Goetia's.
//!
//! Generated with a fixed seed, so the same build makes the same set.

use std::io::Write;

use annulus::lingua::exsequi::Petitio;
use annulus::lingua::json::{self, Json};
use annulus::lingua::registra::{goetia, Spirit};
use annulus::nucleus::{Mark, Outcome};
use annulus::officia;

use crate::forma;
use crate::ingenium::{self, Habitus, Sors, Status};
use crate::llama::quote;
use crate::persona;

const SEED: u64 = 0x5_1A72;

fn texts_of(n: u32) -> String {
    let root = json::parse(ingenium::SPIRITUS).expect("spiritus.json is well formed");
    root.list()
        .iter()
        .find(|e| e.get("n").and_then(Json::number) == Some(f64::from(n)))
        .and_then(|e| e.get("goetia"))
        .and_then(|g| g.get("textus"))
        .and_then(Json::text)
        .unwrap_or("")
        .to_string()
}

fn status(sors: &mut Sors) -> Status {
    match sors.below(4) {
        0 => Status::default(),
        1 => Status {
            vocationes: 1 + sors.below(3) as u32,
            rata: 1,
            ..Status::default()
        },
        2 => Status {
            vocationes: 4 + sors.below(10) as u32,
            rata: 3 + sors.below(5) as u32,
            ..Status::default()
        },
        _ => Status {
            vocationes: 2 + sors.below(6) as u32,
            rata: sors.below(2) as u32,
            irrita: 1 + sors.below(3) as u32,
            ..Status::default()
        },
    }
}

fn row(
    out: &mut dyn Write,
    kind: &str,
    spirit: &Spirit,
    system: &str,
    user: &str,
    answer: &str,
    extra: &str,
) -> std::io::Result<()> {
    writeln!(
        out,
        "{{\"messages\":[{{\"role\":\"system\",\"content\":{}}},{{\"role\":\"user\",\"content\":{}}},{{\"role\":\"assistant\",\"content\":{}}}],\"meta\":{{\"kind\":{},\"n\":{},\"spirit\":{}{extra}}}}}",
        quote(system),
        quote(user),
        quote(answer),
        quote(kind),
        spirit.n,
        quote(&spirit.name)
    )
}

fn prompt(p: &Petitio, h: &Habitus, s: &Status, free: bool, telling: bool) -> (String, String) {
    let profile = ingenium::profile(p.spirit.n).expect("every spirit has a profile");
    let mut system = persona::system(p);
    system.push_str("\n\n");
    system.push_str(&ingenium::brief(profile, s, h, free));
    (system, persona::user(p, telling))
}

/// Breaks a text into lines at random widths, sometimes with Windows
/// line ends and sometimes with a closing line end.
fn lined(text: &str, sors: &mut Sors) -> String {
    let words: Vec<&str> = text.split_whitespace().collect();
    let start = sors.below(words.len().max(1));
    let take = 5 + sors.below(60);
    let width = 20 + sors.below(70);
    let end = if sors.below(2) == 0 { "\n" } else { "\r\n" };
    let mut out = String::new();
    let mut col = 0;
    for w in words.iter().skip(start).take(take) {
        if col > 0 && col + w.len() > width {
            out.push_str(end);
            col = 0;
        } else if col > 0 {
            out.push(' ');
            col += 1;
        }
        out.push_str(w);
        col += w.len();
    }
    if sors.below(2) == 0 {
        out.push_str(end);
    }
    out
}

const PLACES: [&str; 8] = [
    "liber.txt",
    "notae.md",
    "acta/hora.txt",
    "epistola.txt",
    "codex/folium-3.txt",
    "index.md",
    "carmen.txt",
    "tabula.csv",
];

fn virtutes(out: &mut dyn Write, s: &Spirit, sors: &mut Sors) -> std::io::Result<usize> {
    let corpus = texts_of(s.n);
    let profile = ingenium::profile(s.n).expect("profile");
    let mut made = 0;
    for _ in 0..30 {
        let k = 1 + sors.below(3);
        let first = sors.below(PLACES.len());
        let places: Vec<String> = (0..k)
            .map(|i| PLACES[(first + i) % PLACES.len()].to_string())
            .collect();
        let absent = (sors.below(5) == 0).then(|| sors.below(k));
        let legenda: Vec<(String, String, Mark)> = places
            .iter()
            .enumerate()
            .filter(|(i, _)| Some(*i) != absent)
            .map(|(_, p)| (p.clone(), lined(&corpus, sors), Mark::Outside))
            .collect();
        let read = |p: &str| {
            legenda
                .iter()
                .find(|(q, _, _)| q == p)
                .map(|(_, d, _)| d.clone())
        };
        let aperta = vec![Outcome::Obsequium, Outcome::Pars, Outcome::Mora];
        let p = Petitio {
            spirit: s,
            officium: "VIRTUTES",
            quaestio: None,
            modus: None,
            loci: &places,
            legenda: legenda.clone(),
            historia: Vec::new(),
            aperta: aperta.clone(),
        };
        let st = status(sors);
        let h = ingenium::habitus(profile, &st, true, sors);
        let roll = sors.below(10);
        let (word, given) = if roll == 0 {
            ("mora", 0)
        } else if roll <= 2 && k > 1 {
            ("pars", 1 + sors.below(k - 1))
        } else {
            ("obsequium", k)
        };
        let items: Vec<String> = places
            .iter()
            .take(given)
            .map(|pl| match read(pl) {
                Some(d) => format!(
                    "{{\"locus\":{},\"bytes\":{},\"lines\":{}}}",
                    quote(pl),
                    d.len(),
                    d.lines().count()
                ),
                None => format!("{{\"locus\":{},\"absens\":true}}", quote(pl)),
            })
            .collect();
        let answer = format!(
            "{{\"exitus\":\"{word}\",\"virtutes\":[{}]}}",
            items.join(",")
        );
        // The example must pass Spiri72's reader and the angel's recount.
        let (outcome, actum) =
            forma::read_virtutes(&answer, &aperta).expect("a training answer Spiri72 accepts");
        if outcome == Outcome::Obsequium {
            let annulus::lingua::exsequi::Actum::Responsum(t) = actum else {
                unreachable!()
            };
            assert!(officia::same_account(&t, &officia::virtutes(&places, read)));
        }
        let (system, user) = prompt(&p, &h, &st, false, true);
        row(
            out,
            "virtutes",
            s,
            &system,
            &user,
            &answer,
            &format!(",\"exitus\":\"{word}\""),
        )?;
        made += 1;
    }
    Ok(made)
}

fn praeterita(out: &mut dyn Write, s: &Spirit, sors: &mut Sors) -> std::io::Result<usize> {
    let all = &goetia().spirits;
    let profile = ingenium::profile(s.n).expect("profile");
    let mut made = 0;
    for _ in 0..15 {
        let mut historia = Vec::new();
        for _ in 0..sors.below(4) {
            let o = &all[sors.below(all.len())];
            let off = o.offices.get(sors.below(o.offices.len().max(1))).cloned();
            let Some(off) = off else { continue };
            if sors.below(6) == 0 {
                historia.push(format!("recusatus {}", o.name));
                continue;
            }
            historia.push(format!("invocatus {}", o.name));
            historia.push(format!("petitum {} {off}", o.name));
            let v = ["RATUM", "RATUM", "SUSPENSUM", "IRRITUM"][sors.below(4)];
            historia.push(format!("iudicatum {} {off} {v}", o.name));
            if v == "RATUM" {
                historia.push(format!("dimissus {}", o.name));
            } else {
                historia.push(format!("relegatus {}", o.name));
            }
        }
        historia.push(format!("invocatus {}", s.name));
        historia.push(format!("petitum {} PRAETERITA", s.name));
        let aperta = vec![Outcome::Obsequium, Outcome::Pars, Outcome::Mora];
        let question = [
            "What has passed in this rite?",
            "Tell me what was done before you came.",
            "Recount the past of this working.",
            "Which spirits were called before you, and what came of it?",
        ][sors.below(4)];
        let p = Petitio {
            spirit: s,
            officium: "PRAETERITA",
            quaestio: Some(question),
            modus: None,
            loci: &[],
            legenda: Vec::new(),
            historia: historia.clone(),
            aperta: aperta.clone(),
        };
        let st = status(sors);
        let h = ingenium::habitus(profile, &st, true, sors);
        let roll = sors.below(10);
        let (word, claims) = if roll == 0 {
            ("mora", Vec::new())
        } else if roll <= 2 && historia.len() > 2 {
            let k = 1 + sors.below(historia.len() - 1);
            ("pars", historia[..k].to_vec())
        } else {
            ("obsequium", historia.clone())
        };
        let answer = format!(
            "{{\"exitus\":\"{word}\",\"responsum\":{}}}",
            quote(&claims.join("\n"))
        );
        let (outcome, actum) =
            forma::read(&answer, &aperta, true).expect("a training answer Spiri72 accepts");
        if outcome != Outcome::Mora {
            let annulus::lingua::exsequi::Actum::Responsum(t) = actum else {
                unreachable!()
            };
            assert_eq!(officia::account_of_past(&t, &historia), Some(true));
        }
        let (system, user) = prompt(&p, &h, &st, false, true);
        row(
            out,
            "praeterita",
            s,
            &system,
            &user,
            &answer,
            &format!(",\"exitus\":\"{word}\""),
        )?;
        made += 1;
    }
    Ok(made)
}

/// Turns a recorded office ("brings back the thief") into the spirit's own
/// first person ("I bring back the thief"), or quotes it when its first
/// word is not plainly a verb.
pub fn first_person(t: &str) -> String {
    let (w, rest) = t.split_once(' ').unwrap_or((t, ""));
    let verb = match w {
        "is" => Some("am".to_string()),
        "has" => Some("have".to_string()),
        "does" => Some("do".to_string()),
        "goes" => Some("go".to_string()),
        _ if w.ends_with("ies") && w.len() > 4 => Some(format!("{}y", &w[..w.len() - 3])),
        _ if ["ches", "shes", "sses", "xes", "zes"]
            .iter()
            .any(|e| w.ends_with(e)) =>
        {
            Some(w[..w.len() - 2].to_string())
        }
        _ if w.ends_with('s') && !w.ends_with("ss") && w.len() > 3 && !w.ends_with("us") => {
            Some(w[..w.len() - 1].to_string())
        }
        _ => None,
    };
    match verb {
        Some(v) if rest.is_empty() => format!("I {v}"),
        Some(v) => format!("I {v} {rest}"),
        None => format!("of me they write: {t}"),
    }
}

fn opener(h: &Habitus, rank: &str, sors: &mut Sors) -> &'static str {
    let proud: [&str; 3] = [
        "You stand before a {rank} of Hell. Mind how you speak to me.",
        "Bow your head, Operator; you address a {rank}.",
        "A {rank} does not come for idle questions. Know whom you have called.",
    ];
    let hostile: [&str; 3] = [
        "You drag me from my legions for this?",
        "Your Circle holds me, little man. It does not make me love you.",
        "Ask, then, and be quick, before your courage fails you.",
    ];
    let reticent: [&str; 3] = [
        "You shall have what the Names wring from me, and no more.",
        "I am bound to answer. I am not bound to be generous.",
        "Few words. Count yourself fortunate to have those.",
    ];
    let mild: [&str; 3] = [
        "I am here, as the bond requires.",
        "You have called, and I have come.",
        "Speak, Operator. I hear you.",
    ];
    let cold: [&str; 3] = [
        "Well? I am here.",
        "You have called. I have come, as I must.",
        "Speak, then, and let me be gone.",
    ];
    let _ = rank;
    let set = if h.hostilitas >= 0.5 {
        &hostile
    } else if h.superbia >= 0.75 {
        &proud
    } else if h.reticentia >= 0.6 {
        &reticent
    } else if h.hostilitas >= 0.25 {
        &cold
    } else {
        &mild
    };
    set[sors.below(3)]
}

fn rank_word(rank: &str) -> String {
    let r = rank.split('/').next().unwrap_or(rank);
    let mut c = r.chars();
    c.next()
        .map(|f| f.to_uppercase().collect::<String>() + c.as_str())
        .unwrap_or_default()
}

fn persona_rows(out: &mut dyn Write, s: &Spirit, sors: &mut Sors) -> std::io::Result<usize> {
    let profile = ingenium::profile(s.n).expect("profile");
    let entry = persona::entry(s.n).expect("register entry");
    let rank = entry.get("rank").and_then(Json::text).unwrap_or("spirit");
    let offices: Vec<String> = entry
        .get("offices")
        .map(Json::list)
        .unwrap_or(&[])
        .iter()
        .filter_map(|o| o.get("text").and_then(Json::text))
        .map(first_person)
        .collect();
    let malice: Vec<String> = entry
        .get("malice")
        .map(Json::list)
        .unwrap_or(&[])
        .iter()
        .filter_map(|o| o.get("text").and_then(Json::text))
        .map(str::to_string)
        .collect();
    let questions = [
        "Who are you?",
        "Name yourself, spirit, and your offices.",
        "What are you, and what can you do?",
        "Tell me what you are.",
    ];
    let mut made = 0;
    for q in questions {
        let aperta = vec![Outcome::Obsequium, Outcome::Pars, Outcome::Mora];
        let p = Petitio {
            spirit: s,
            officium: "RESPONSA",
            quaestio: Some(q),
            modus: None,
            loci: &[],
            legenda: Vec::new(),
            historia: Vec::new(),
            aperta: aperta.clone(),
        };
        let st = status(sors);
        let h = ingenium::habitus(profile, &st, true, sors);
        let title = rank_word(rank);
        let mut words = opener(&h, rank, sors).replace("{rank}", &title.to_lowercase());
        words.push_str(&format!(
            " I am {}, a {} of the seventy-two, and I rule {} legions.",
            s.name, title, s.legions
        ));
        if let Some(f) = profile.forma.first() {
            words.push_str(&format!(" Your books say of my coming: \"{f}\""));
        }
        let reticent = h.reticentia >= 0.6;
        let shown = if reticent {
            offices.len().min(1)
        } else {
            offices.len()
        };
        if shown > 0 {
            words.push_str(" By my office ");
            words.push_str(&offices[..shown].join("; "));
            words.push('.');
        }
        if h.hostilitas >= 0.5 && !malice.is_empty() {
            words.push_str(&format!(
                " And of my harm they write: {}. Remember it.",
                malice.join("; ")
            ));
        }
        let word = if reticent && offices.len() > 1 {
            "pars"
        } else {
            "obsequium"
        };
        let answer = format!("{{\"exitus\":\"{word}\",\"responsum\":{}}}", quote(&words));
        forma::read(&answer, &aperta, true).expect("a training answer Spiri72 accepts");
        let (system, user) = prompt(&p, &h, &st, true, true);
        row(
            out,
            "persona",
            s,
            &system,
            &user,
            &answer,
            &format!(",\"exitus\":\"{word}\",\"voice\":\"templated\""),
        )?;
        made += 1;
    }
    Ok(made)
}

/// Writes the whole set as JSON lines; returns how many examples of each
/// kind were made.
pub fn exempla(out: &mut dyn Write) -> std::io::Result<(usize, usize, usize)> {
    let mut sors = Sors::new(SEED);
    let (mut v, mut pr, mut pe) = (0, 0, 0);
    for s in &goetia().spirits {
        pe += persona_rows(out, s, &mut sors)?;
        if s.offices.iter().any(|o| o == "VIRTUTES") {
            v += virtutes(out, s, &mut sors)?;
        }
        if s.offices.iter().any(|o| o == "PRAETERITA") {
            pr += praeterita(out, s, &mut sors)?;
        }
    }
    Ok((v, pr, pe))
}
