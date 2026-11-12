use std::fs;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::PathBuf;
use std::process::Command;
use std::thread;

const BINARY: &str = env!("CARGO_BIN_EXE_spiri72");

const FURFUR: &str = r#"FACULTAS {
    liber "chronica.txt" legere;
    atramentum "comes" 1;
}
CIRCULUS RITU goetia {
    INTENTIO "ask of the storm";
    SIT sigillum = INSCRIBE SIGILLUM furfur ATRAMENTO CONSECRA ATRAMENTUM "comes";
    SIT vocatus = INVOCA SPIRITUM 34 AD OFFICIUM responsa;
    SIT propositum = PETE vocatus("venietne tempestas", "chronica.txt");
    SIT iudicium = EXAMINA propositum;
    MANIFESTA iudicium;
    SI iudicium == RATUM {
        MANIFESTA RESPONSUM propositum;
    }
    VADE IN PACE vocatus;
}
"#;

fn folder(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("spiri72-{}-{name}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("furfur.annulus"), FURFUR).unwrap();
    fs::write(dir.join("chronica.txt"), "annales Romae").unwrap();
    dir
}

fn model(content: &'static str) -> (u16, thread::JoinHandle<String>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let server = thread::spawn(move || {
        let (mut s, _) = listener.accept().unwrap();
        let mut got = Vec::new();
        let mut buf = vec![0u8; 65536];
        loop {
            let n = s.read(&mut buf).unwrap();
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
        s.write_all(reply.as_bytes()).unwrap();
        String::from_utf8(got).unwrap()
    });
    (port, server)
}

fn spiri72(dir: &PathBuf, port: u16, extra: &[&str]) -> std::process::Output {
    let url = format!("http://127.0.0.1:{port}");
    let mut args = vec![
        "furfur.annulus",
        "--hora",
        "2024-04-13T14:30:00Z",
        "--locus",
        "41.9,12.5",
        "--exemplar",
        &url,
    ];
    args.extend_from_slice(extra);
    Command::new(BINARY)
        .current_dir(dir)
        .args(&args)
        .output()
        .unwrap()
}

#[test]
fn furfur_speaks_through_the_model() {
    let dir = folder("furfur");
    let (port, server) = model(r#"{"exitus":"obsequium","responsum":"tempestas veniet"}"#);
    let o = spiri72(&dir, port, &["--virga-probationis"]);
    let err = String::from_utf8_lossy(&o.stderr);
    assert_eq!(o.status.code(), Some(0), "{err}");
    assert_eq!(
        String::from_utf8_lossy(&o.stdout),
        "RATUM\n[Furfur, COACTUS] tempestas veniet\n"
    );
    let request = server.join().unwrap();
    assert!(request.contains("You are Furfur, spirit 34"), "{request}");
    assert!(request.contains("venietne tempestas") && request.contains("annales Romae"));
    let acta = fs::read_to_string(dir.join("furfur.acta")).unwrap();
    assert!(acta.contains("# the will of the spirits: a local model at http://127.0.0.1:"));
    assert!(acta.contains("the model's own declaration"));
}

#[test]
fn a_reply_out_of_form_ends_the_run_and_nothing_is_written() {
    let dir = folder("form");
    let (port, server) = model("I shall tell you everything, mortal.");
    let o = spiri72(&dir, port, &["--virga-probationis"]);
    assert_eq!(o.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&o.stderr).contains("not JSON"));
    assert!(String::from_utf8_lossy(&o.stdout).is_empty());
    server.join().unwrap();
    assert_eq!(
        fs::read_to_string(dir.join("chronica.txt")).unwrap(),
        "annales Romae"
    );
}

#[test]
fn without_the_test_wand_the_model_is_never_asked() {
    let dir = folder("nowand");
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    listener.set_nonblocking(true).unwrap();
    let o = spiri72(&dir, port, &[]);
    assert_eq!(o.status.code(), Some(2));
    assert!(listener.accept().is_err(), "the model was called");
}

#[test]
fn a_remote_model_is_refused() {
    let dir = folder("remote");
    let o = Command::new(BINARY)
        .current_dir(&dir)
        .args([
            "furfur.annulus",
            "--virga-probationis",
            "--exemplar",
            "http://example.com:8080",
        ])
        .output()
        .unwrap();
    assert_eq!(o.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&o.stderr).contains("this computer"));
    assert!(!dir.join("furfur.acta").exists());
}
