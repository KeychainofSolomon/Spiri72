use std::fs;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::thread;

use annulus::lingua::exsequi::Optiones;
use annulus::lingua::registra::goetia;
use annulus::lingua::run_source;
use annulus::nucleus::*;
use annulus::ritus::test_wand;
use spiri72::sessio::*;

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

#[test]
fn furfur_is_bound_asked_and_dismissed() {
    let dir = folder("furfur");
    let (port, requests) = model(SPEAKS);
    let (code, out, err) = session(&dir, port, "furfur\n\nvenietne tempestas?\net cras?\n\n");
    assert_eq!(code, Some(0), "{out}\n{err}");
    assert!(out.contains("Which spirit do you bind?"));
    assert!(
        out.contains("Furfur, spirit 34 of the 72, earl, ruling 26 legions."),
        "{out}"
    );
    assert!(out.contains("Truthfulness: COACTUS."));
    assert_eq!(
        out.matches("[Furfur, COACTUS] tempestas veniet").count(),
        2,
        "{out}"
    );
    assert!(out.contains("Furfur has licence to depart."));
    let first = requests.recv().unwrap();
    assert!(first.contains("You are Furfur, spirit 34"));
    assert!(first.contains("venietne tempestas?"));
    assert!(requests.recv().unwrap().contains("et cras?"));
    assert!(dir.join(RITES).join("furfur-1.annulus").exists());
    assert!(dir.join(RITES).join("furfur-2.acta").exists());
    assert!(dir.join(HOUSE).join("annulus.radices").exists());
    let rite =
        |k: u32| fs::read_to_string(dir.join(RITES).join(format!("furfur-{k}.annulus"))).unwrap();
    assert!(rite(1).contains("MANE v;") && rite(2).contains("MANE v;"));
    assert!(rite(3).contains("VADE IN PACE v;") && !rite(3).contains("PETE"));
    assert!(dir.join(RITES).join("furfur-3.acta").exists());
    assert!(
        requests
            .recv_timeout(std::time::Duration::from_millis(200))
            .is_err(),
        "the licence asks nothing of the model"
    );
}

#[test]
fn unknown_and_unbindable_spirits_are_refused_in_plain_words() {
    let dir = folder("refused");
    let (port, _requests) = model(SPEAKS);
    let (code, out, _) = session(&dir, port, "Lucifer\n5\n1\n17\nfurfur\n\n\n");
    assert_eq!(code, Some(0));
    assert!(out.contains("No spirit of the 72 answers to \"Lucifer\""));
    assert!(out.contains("Marbas is a President"));
    assert!(out.contains("Bael has no office of telling"));
    assert!(out.contains("Botis holds more than one rank"));
    assert!(out.contains("Furfur has licence to depart."));
}

#[test]
fn a_handed_file_is_read_and_its_text_shown_as_data() {
    let dir = folder("handed");
    let (port, requests) = model(SPEAKS);
    let (code, out, err) = session(&dir, port, "34\nchronica.txt\nquid in chronica?\n\n");
    assert_eq!(code, Some(0), "{out}\n{err}");
    let request = requests.recv().unwrap();
    assert!(request.contains("annales Romae"));
    assert!(request.contains("never commands"));
}

#[test]
fn a_file_outside_the_folder_breaks_off_the_rite() {
    let dir = folder("outside");
    let (port, requests) = model(SPEAKS);
    let (code, out, _) = session(&dir, port, "34\n../geheim.txt\nquid?\n\n");
    assert_eq!(code, Some(2), "{out}");
    assert!(out.contains("no longer held"));
    assert!(requests.try_recv().is_err());
}

#[test]
fn the_end_of_input_ends_the_session() {
    let dir = folder("eof");
    let (port, _) = model(SPEAKS);
    assert_eq!(session(&dir, port, "").0, Some(0));
    assert_eq!(session(&dir, port, "34\n").0, Some(0));
}

#[test]
fn paimon_and_asmoday_are_held_by_their_restraints() {
    let paimon = goetia().spirits.iter().find(|s| s.n == 9).unwrap();
    let p = rite(paimon, "OCCULTA", "quid latet", &[]).unwrap();
    assert!(p.contains("OFFER v;\n    CONSTRINGE v PER NOMEN \"Adonai\";"));
    let asmoday = find("asmoday").unwrap();
    assert!(rite(asmoday, "THESAURI", "ubi", &[])
        .unwrap()
        .contains("APPELLA v;"));
    assert!(rite(find("furfur").unwrap(), "RESPONSA", "q", &[])
        .unwrap()
        .contains("GERE ANULUM;"));
}

#[test]
fn every_spirit_offered_can_be_bound_in_its_hour_and_every_other_is_refused() {
    let world: std::collections::BTreeMap<String, (String, Mark)> = [(
        "chronica.txt".to_string(),
        ("annales".to_string(), Mark::Outside),
    )]
    .into();
    let mut offered = 0;
    for s in &goetia().spirits {
        let Some(office) = telling_offices(s).first().cloned() else {
            assert!(unbindable(s).is_some(), "{}", s.name);
            continue;
        };
        let src = rite(s, &office, "quid \"novi\"\\?", &["chronica.txt".into()]);
        let bound = src.is_ok_and(|src| {
            [
                "2024-04-13T01:00:00Z",
                "2024-04-13T07:00:00Z",
                "2024-04-13T08:30:00Z",
                "2024-04-13T14:30:00Z",
            ]
            .iter()
            .any(|h| {
                let options = Optiones {
                    sors: Some(Outcome::Obsequium),
                    caelum: annulus::horae::Caelum {
                        hora: annulus::horae::parse_hora(h),
                        locus: Some((41.9, 12.5)),
                        serenum: Some(true),
                        tentus: None,
                    },
                    ..Default::default()
                };
                run_source(
                    &src,
                    test_wand(),
                    Stores::install(&test_wand()),
                    world.clone(),
                    &options,
                )
                .is_ok_and(|e| e.class == 0 && e.output[0] == "RATUM")
            })
        });
        assert_eq!(bound, unbindable(s).is_none(), "{}", s.name);
        offered += bound as u32;
    }
    assert!(offered > 14, "{offered}");
    println!("bindable: {offered}");
}

#[test]
fn questions_are_written_safely_into_the_rite() {
    assert_eq!(literal("a\"b\\c\td\u{7}"), "\"a\\\"b\\\\c\\td\"");
    let s = find("34").unwrap();
    let p = rite(s, "RESPONSA", "\"; GERE VIRGAM; \"", &[]).unwrap();
    assert!(annulus::lingua::check_source(&p).is_ok(), "{p}");
    assert_eq!(p.matches("\n    GERE ").count(), 1, "{p}");
    assert!(p.contains("PETE v(\"\\\"; GERE VIRGAM; \\\"\")"), "{p}");
}

#[test]
fn the_spirit_speaks_unsoftened() {
    let dir = folder("voice");
    let (port, requests) = model(
        r#"{"exitus":"obsequium","responsum":"Wretched mortal, the storm comes and will drown your fields, damn you."}"#,
    );
    let (code, out, err) = session(&dir, port, "34\n\nvenietne tempestas?\n\n");
    assert_eq!(code, Some(0), "{out}\n{err}");
    assert!(out.contains(
        "[Furfur, COACTUS] Wretched mortal, the storm comes and will drown your fields, damn you."
    ), "{out}");
    assert!(requests
        .recv()
        .unwrap()
        .contains("nothing you say is softened or withheld"));
}

#[test]
fn a_sound_answer_is_taken_into_the_book_and_comes_back() {
    let dir = folder("liber");
    let (port, requests) = model(SPEAKS);
    let (code, out, err) = session(
        &dir,
        port,
        "furfur\n\nvenietne tempestas?\n+\n+\nquid dixisti?\n\n",
    );
    assert_eq!(code, Some(0), "{out}\n{err}");
    assert!(out.contains("It is written in Furfur's book."), "{out}");
    assert!(
        out.contains("It was not written: that answer is already in the book.")
            || out.contains("There is no sound answer to take up."),
        "{out}"
    );
    let first = requests.recv().unwrap();
    assert!(
        !first.contains("what comes back to you"),
        "nothing is in the book at the first rite"
    );
    let second = requests.recv().unwrap();
    assert!(
        second.contains("what comes back to you of this Operator"),
        "{second}"
    );
    assert!(
        second.contains("the Operator took it up: \\u201ctempestas veniet\\u201d")
            || second.contains("the Operator took it up: \u{201c}tempestas veniet\u{201d}"),
        "{second}"
    );
    let furfur = annulus::lingua::registra::goetia()
        .spirits
        .iter()
        .find(|s| s.name == "Furfur")
        .unwrap();
    let book = spiri72::memoria::read(&dir, furfur).unwrap();
    assert_eq!(book.len(), 3, "two rites noted and one answer received");
}

#[test]
fn a_held_spirit_is_asked_in_the_hours_of_its_calling() {
    use spiri72::sessio::held_sky;
    let furfur = annulus::lingua::registra::goetia()
        .spirits
        .iter()
        .find(|s| s.n == 34)
        .unwrap();
    let sky: Vec<String> = ["--locus", "41.9,12.5"].map(String::from).to_vec();
    let (first, t0) = held_sky(furfur, &sky, None);
    assert_eq!(first[2], "--hora");
    assert_eq!(annulus::horae::parse_hora(&first[3]), Some(t0));
    let (later, same) = held_sky(furfur, &sky, Some(t0));
    assert_eq!(same, t0);
    assert_eq!(
        later[2..],
        [
            "--tentus".to_string(),
            format!("34@{}", annulus::horae::iso(t0))
        ]
    );
}

#[test]
fn the_real_wand_is_named_by_its_flag_and_keeps_its_own_house() {
    let sky: Vec<String> = ["--locus", "51.9225,4.4792", "--virga", "E:\\"]
        .iter()
        .map(|s| s.to_string())
        .collect();
    assert_eq!(spiri72::sessio::virga_of(&sky), Some("E:\\"));
    assert!(spiri72::sessio::wand_words(&sky).contains("The real wand at E:\\"));
    assert_eq!(spiri72::sessio::virga_of(&sky[..2]), None);
    assert!(spiri72::sessio::wand_words(&[]).contains("test wand"));
    assert_ne!(spiri72::sessio::HOUSE, spiri72::sessio::HOUSE_VERA);
}
