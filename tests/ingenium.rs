use std::fs;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::thread;

const BINARY: &str = env!("CARGO_BIN_EXE_spiri72");

fn folder(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("spiri72-s-{}-{name}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("chronica.txt"), "annales Romae").unwrap();
    dir
}

fn model(content: &'static str) -> (u16, mpsc::Receiver<String>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(mut s) = stream else { return };
            let mut got = Vec::new();
            let mut buf = vec![0u8; 65536];
            loop {
                let n = s.read(&mut buf).unwrap_or(0);
                got.extend_from_slice(&buf[..n]);
                let text = String::from_utf8_lossy(&got).to_string();
                if let Some((head, body)) = text.split_once("\r\n\r\n") {
                    let len: usize = head
                        .lines()
                        .find_map(|l| l.strip_prefix("Content-Length: "))
                        .and_then(|v| v.parse().ok())
                        .unwrap_or(0);
                    if body.len() >= len {
                        break;
                    }
                }
                if n == 0 {
                    break;
                }
            }
            let body = format!(
                "{{\"choices\":[{{\"message\":{{\"content\":{}}}}}]}}",
                spiri72::llama::quote(content)
            );
            let reply = format!(
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{body}",
                body.len()
            );
            let _ = s.write_all(reply.as_bytes());
            let _ = tx.send(String::from_utf8_lossy(&got).to_string());
        }
    });
    (port, rx)
}

fn session(dir: &PathBuf, port: u16, typed: &str) -> (Option<i32>, String, String) {
    let mut child = Command::new(BINARY)
        .current_dir(dir)
        .args([
            "--terminal",
            "--hora",
            "2024-04-13T14:30:00Z",
            "--locus",
            "41.9,12.5",
            "--exemplar",
            &format!("http://127.0.0.1:{port}"),
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(typed.as_bytes())
        .unwrap();
    let o = child.wait_with_output().unwrap();
    (
        o.status.code(),
        String::from_utf8_lossy(&o.stdout).to_string(),
        String::from_utf8_lossy(&o.stderr).to_string(),
    )
}

const SPEAKS: &str = r#"{"exitus":"obsequium","responsum":"tempestas veniet"}"#;

use spiri72::ingenium::{self, Sors, Status};

#[test]
fn every_spirit_has_a_whole_profile_matching_the_register() {
    let reg = &annulus::lingua::registra::goetia().spirits;
    assert_eq!(ingenium::profiles().len(), 72);
    for s in reg {
        let p = ingenium::profile(s.n).unwrap();
        assert_eq!(p.name, s.name);
        assert_eq!(p.veracity, s.veracity);
        for (lo, hi) in [p.superbia, p.fallacia, p.hostilitas, p.reticentia] {
            assert!(0.0 <= lo && lo <= hi && hi <= 1.0, "{}", s.name);
        }
    }
}

#[test]
fn a_bearing_stays_within_its_ranges_and_varies() {
    let p = ingenium::profile(13).unwrap();
    let mut sors = Sors::new(7);
    let mut seen = Vec::new();
    for _ in 0..200 {
        let h = ingenium::habitus(p, &Status::default(), true, &mut sors);
        assert!(p.superbia.0 <= h.superbia && h.superbia <= p.superbia.1);
        assert!(p.hostilitas.0 <= h.hostilitas && h.hostilitas <= p.hostilitas.1);
        seen.push(h.hostilitas);
    }
    seen.sort_by(f32::total_cmp);
    assert!(
        seen[199] - seen[0] > 0.1,
        "people are not fully deterministic"
    );
}

#[test]
fn spirits_who_lie_unless_compelled_are_compelled_in_spiri72s_rites() {
    for n in [32, 34, 44, 64, 68] {
        let p = ingenium::profile(n).unwrap();
        assert!(ingenium::coactus(p, true), "{n}");
        let h = ingenium::habitus(p, &Status::default(), true, &mut Sors::new(3));
        assert!(h.fallacia <= 0.15, "{n}");
        let b = ingenium::brief(p, &Status::default(), &h, true);
        assert!(b.contains("you answer truly"), "{b}");
    }
    // Outside Spiri72's own rites only the triangle is assumed.
    assert!(!ingenium::coactus(ingenium::profile(32).unwrap(), false));
    assert!(ingenium::coactus(ingenium::profile(34).unwrap(), false));
    // The liars are never compelled by the engine.
    assert!(!ingenium::coactus(ingenium::profile(6).unwrap(), true));
}

#[test]
fn the_brief_is_words_never_numbers() {
    let mut sors = Sors::new(11);
    for p in ingenium::profiles() {
        let st = Status {
            vocationes: 5,
            rata: 4,
            suspensa: 1,
            irrita: 1,
        };
        let h = ingenium::habitus(p, &st, true, &mut sors);
        let b = ingenium::brief(p, &st, &h, true);
        let own: Vec<char> = p
            .forma
            .iter()
            .chain(&p.ira_adventus)
            .chain(&p.cultus)
            .chain(&p.testimonium)
            .chain(&p.reditus)
            .flat_map(|f| f.chars())
            .collect();
        for c in b.chars().filter(char::is_ascii_digit) {
            assert!(own.contains(&c), "{}: {b}", p.name);
        }
        assert!(!b.contains("0."), "{}: {b}", p.name);
    }
}

#[test]
fn being_caught_lowers_deceit_within_its_own_range() {
    let p = ingenium::profile(6).unwrap();
    let caught = Status {
        vocationes: 3,
        irrita: 1,
        ..Status::default()
    };
    for seed in 1..50 {
        let a = ingenium::habitus(p, &Status::default(), true, &mut Sors::new(seed));
        let b = ingenium::habitus(p, &caught, true, &mut Sors::new(seed));
        assert!(b.fallacia <= a.fallacia && b.fallacia >= p.fallacia.0);
    }
}

#[test]
fn the_angels_verdicts_are_kept_and_shape_the_next_brief() {
    let dir = folder("status");
    let (port, requests) = model(SPEAKS);
    let (code, out, err) = session(&dir, port, "furfur\n\nvenietne tempestas?\net cras?\n\n");
    assert_eq!(code, Some(0), "{out}\n{err}");
    let first = requests.recv().unwrap();
    assert!(first.contains("Your bearing in this rite"), "{first}");
    assert!(
        !first.contains("called you"),
        "the first rite knows no history"
    );
    let second = requests.recv().unwrap();
    assert!(
        second.contains("This Operator has called you once before."),
        "{second}"
    );
    let st = ingenium::load(&dir, 34);
    assert_eq!(st.vocationes, 2);
    assert_eq!(st.rata, 2, "{out}");
}

#[test]
fn the_engine_can_be_left_off() {
    let dir = folder("sine");
    let (port, requests) = model(SPEAKS);
    let mut child = Command::new(BINARY)
        .current_dir(&dir)
        .args([
            "--terminal",
            "--sine-ingenio",
            "--hora",
            "2024-04-13T14:30:00Z",
            "--locus",
            "41.9,12.5",
            "--exemplar",
            &format!("http://127.0.0.1:{port}"),
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(b"furfur\n\nvenietne tempestas?\n\n")
        .unwrap();
    let o = child.wait_with_output().unwrap();
    assert_eq!(o.status.code(), Some(0));
    assert!(!requests.recv().unwrap().contains("Your bearing"));
}

#[test]
fn the_training_set_is_made_and_every_answer_is_one_spiri72_accepts() {
    let mut buf = Vec::new();
    let (v, pr, pe) = spiri72::exempla::exempla(&mut buf).unwrap();
    assert_eq!(pe, 72 * 4);
    assert!(v > 0 && pr > 0);
    let text = String::from_utf8(buf).unwrap();
    assert_eq!(text.lines().count(), v + pr + pe);
    for l in text.lines() {
        annulus::lingua::json::parse(l).unwrap();
    }
    let mut again = Vec::new();
    spiri72::exempla::exempla(&mut again).unwrap();
    assert_eq!(
        text.as_bytes(),
        again.as_slice(),
        "the same build makes the same set"
    );
}

#[test]
fn offices_are_put_in_the_first_person_or_quoted() {
    use spiri72::exempla::first_person as f;
    assert_eq!(f("brings back the thief"), "I bring back the thief");
    assert_eq!(f("teaches all sciences"), "I teach all sciences");
    assert_eq!(f("carries the exorcist"), "I carry the exorcist");
    assert_eq!(f("is a liar"), "I am a liar");
    assert_eq!(f("a liar"), "of me they write: a liar");
}

#[test]
fn every_spirit_has_a_nature_and_keeps_the_solomonic_order() {
    let st = Status::default();
    for p in ingenium::profiles() {
        assert!(!p.naturae.is_empty(), "{}", p.name);
        let h = ingenium::habitus(p, &st, true, &mut Sors::new(5));
        let b = ingenium::brief(p, &st, &h, true);
        assert!(b.contains("Above all stands God"), "{b}");
        assert!(b.contains("not a man"), "{b}");
        if p.rank.starts_with("king") {
            assert!(b.contains("You are yourself a king"), "{b}");
        }
    }
    let beleth = ingenium::profile(13).unwrap();
    let h = ingenium::habitus(beleth, &st, true, &mut Sors::new(5));
    let b = ingenium::brief(beleth, &st, &h, true);
    assert!(b.contains("with Belial, Asmoday and Gaap."), "{b}");
    let bael = ingenium::profile(1).unwrap();
    let h = ingenium::habitus(bael, &st, true, &mut Sors::new(5));
    assert!(ingenium::brief(bael, &st, &h, true).contains("You are yourself a king"));
    let agares = ingenium::profile(2).unwrap();
    let h = ingenium::habitus(agares, &st, true, &mut Sors::new(5));
    assert!(ingenium::brief(agares, &st, &h, true).contains("under Amaymon, King of the East"));
}

#[test]
fn a_former_choir_is_spoken_only_where_the_goetia_gives_it() {
    let st = Status::default();
    let with: Vec<u32> = ingenium::profiles()
        .iter()
        .filter(|p| !p.chori.is_empty())
        .map(|p| p.n)
        .collect();
    assert_eq!(with, vec![2, 8, 9, 13, 33, 35, 40, 47, 49, 53, 54]);
    let phenex = ingenium::profile(37).unwrap();
    let h = ingenium::habitus(phenex, &st, true, &mut Sors::new(2));
    assert!(ingenium::brief(phenex, &st, &h, true).contains("Seventh Throne"));
    let belial = ingenium::profile(68).unwrap();
    let h = ingenium::habitus(belial, &st, true, &mut Sors::new(2));
    let b = ingenium::brief(belial, &st, &h, true);
    assert!(b.contains("fell first"), "{b}");
    assert!(!b.contains("guarded single souls"), "{b}");
}
