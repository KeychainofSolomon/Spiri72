use std::io::{Read, Write};
use std::net::TcpListener;
use std::thread;

use annulus::lingua::exsequi::{Actum, Petitio, Voluntas};
use annulus::lingua::registra::goetia;
use annulus::nucleus::Outcome;
use spiri72::agens::Agens;
use spiri72::llama::{parse_url, quote};

/// A model server that gives these spirit replies in turn, one per request,
/// and hands back the requests it got.
fn server(replies: Vec<&'static str>) -> (u16, thread::JoinHandle<Vec<String>>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let handle = thread::spawn(move || {
        let mut got = Vec::new();
        for reply in replies {
            let (mut s, _) = listener.accept().unwrap();
            let mut buf = vec![0u8; 1 << 20];
            let mut req = Vec::new();
            loop {
                let n = s.read(&mut buf).unwrap();
                if n == 0 {
                    break;
                }
                req.extend_from_slice(&buf[..n]);
                let text = String::from_utf8_lossy(&req);
                if let Some(h) = text.find("\r\n\r\n") {
                    let len: usize = text[..h]
                        .lines()
                        .find_map(|l| l.strip_prefix("Content-Length: "))
                        .and_then(|v| v.trim().parse().ok())
                        .unwrap_or(0);
                    if req.len() >= h + 4 + len {
                        break;
                    }
                }
            }
            let body = format!(
                "{{\"choices\":[{{\"message\":{{\"role\":\"assistant\",\"content\":{}}}}}]}}",
                quote(reply)
            );
            let _ = s.write_all(
                format!(
                    "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{body}",
                    body.len()
                )
                .as_bytes(),
            );
            got.push(String::from_utf8_lossy(&req).to_string());
        }
        got
    });
    (port, handle)
}

fn ask(agens: &mut Agens) -> Result<(Outcome, Actum), String> {
    let bael = goetia().by_number(1).unwrap();
    agens.age(&Petitio {
        spirit: bael,
        officium: "PRAESENTIA",
        quaestio: Some("Who are you?"),
        modus: None,
        loci: &[],
        legenda: Vec::new(),
        historia: Vec::new(),
        aperta: vec![Outcome::Obsequium, Outcome::Pars, Outcome::Mora],
        ultra_horam: false,
    })
}

const WRONG: &str = r#"{"exitus":"obsequium","responsum":"I am Bael, and I rule thirty legions."}"#;
const RIGHT: &str =
    r#"{"exitus":"obsequium","responsum":"I am Bael, and I rule sixty-six legions."}"#;
const CUT: &str = r#"{"exitus":"obsequium","responsum":"I am Ba"#;

#[test]
fn a_faulted_reply_is_asked_again() {
    let (port, srv) = server(vec![WRONG, RIGHT]);
    let mut agens = Agens::new(parse_url(&format!("http://127.0.0.1:{port}")).unwrap());
    agens.sampler.seed = Some(7);
    let (o, a) = ask(&mut agens).unwrap();
    assert_eq!(o, Outcome::Obsequium);
    assert_eq!(
        a,
        Actum::Responsum("I am Bael, and I rule sixty-six legions.".into())
    );
    assert_eq!(
        (
            agens.notae.rogationes,
            agens.notae.vitia,
            agens.notae.relicta
        ),
        (2, 1, 0)
    );
    let requests = srv.join().unwrap();
    // Each try draws with its own seed, or it would say the same again.
    assert!(requests[0].contains("\"seed\":7"), "{}", requests[0]);
    assert!(requests[1].contains("\"seed\":8"), "{}", requests[1]);
    // The prompt holds the fixed facts and the powers.
    assert!(requests[0].contains("you rule 66 legions"));
    assert!(requests[0].contains("charged through PRAESENTIA alone"));
}

#[test]
fn a_reply_faulted_every_time_is_given_on_unchanged() {
    let (port, srv) = server(vec![WRONG, WRONG, WRONG]);
    let mut agens = Agens::new(parse_url(&format!("http://127.0.0.1:{port}")).unwrap());
    agens.loquax = false;
    let (_, a) = ask(&mut agens).unwrap();
    assert_eq!(
        a,
        Actum::Responsum("I am Bael, and I rule thirty legions.".into())
    );
    assert_eq!((agens.notae.rogationes, agens.notae.relicta), (3, 1));
    assert!(agens.notae.ultima[0].contains("66"));
    srv.join().unwrap();
}

#[test]
fn a_reply_cut_short_is_asked_again() {
    let (port, srv) = server(vec![CUT, RIGHT]);
    let mut agens = Agens::new(parse_url(&format!("http://127.0.0.1:{port}")).unwrap());
    assert!(ask(&mut agens).is_ok());
    assert_eq!((agens.notae.rogationes, agens.notae.lapsus), (2, 1));
    srv.join().unwrap();
}

#[test]
fn without_the_custos_the_first_reply_stands() {
    let (port, srv) = server(vec![WRONG]);
    let mut agens = Agens::new(parse_url(&format!("http://127.0.0.1:{port}")).unwrap());
    agens.custos = false;
    let (_, a) = ask(&mut agens).unwrap();
    assert_eq!(
        a,
        Actum::Responsum("I am Bael, and I rule thirty legions.".into())
    );
    assert_eq!(agens.notae.rogationes, 1);
    srv.join().unwrap();
}

#[test]
fn unreadable_every_time_is_an_error() {
    let (port, srv) = server(vec![CUT, CUT, CUT]);
    let mut agens = Agens::new(parse_url(&format!("http://127.0.0.1:{port}")).unwrap());
    let e = ask(&mut agens).unwrap_err();
    assert!(e.contains("after 3 tries"), "{e}");
    srv.join().unwrap();
}

#[test]
fn a_teacher_is_asked_with_the_teaching_grammar() {
    const LESSON: &str = r#"{"exitus":"obsequium","responsum":"Attend, mortal.","testimonium":["numerus: 17*23 = 391"]}"#;
    let (port, srv) = server(vec![LESSON]);
    let mut agens = Agens::new(parse_url(&format!("http://127.0.0.1:{port}")).unwrap());
    let crocell = goetia().by_number(49).unwrap();
    let (_, a) = agens
        .age(&Petitio {
            spirit: crocell,
            officium: "DOCTRINA",
            quaestio: Some("numerus: 17*23 = ?"),
            modus: None,
            loci: &[],
            legenda: Vec::new(),
            historia: Vec::new(),
            aperta: vec![Outcome::Obsequium, Outcome::Pars, Outcome::Mora],
            ultra_horam: false,
        })
        .unwrap();
    assert_eq!(
        a,
        Actum::Responsum("Attend, mortal.\nTESTIMONIUM\nnumerus: 17*23 = 391".into())
    );
    let request = srv.join().unwrap().remove(0);
    assert!(request.contains("numerus: 17*23 = "), "{request}");
    assert!(request.contains("Your arts:"), "{request}");
    assert!(
        request.contains("give the value of each claim asked"),
        "{request}"
    );
}
