use std::io::{Read, Write};
use std::net::TcpListener;
use std::thread;

use spiri72::llama::{parse_url, quote, read_reply, Rogatio};

#[test]
fn only_this_computer_is_reached() {
    for url in [
        "http://127.0.0.1:8080",
        "http://localhost:8080/",
        "http://[::1]:9000",
    ] {
        assert!(parse_url(url).is_ok(), "{url}");
    }
    for url in [
        "https://127.0.0.1:8080",
        "http://example.com:8080",
        "http://10.0.0.2:8080",
        "http://0.0.0.0:8080",
        "http://127.0.0.1",
        "http://127.0.0.1:0",
        "http://127.0.0.1:8080/v1",
        "http://user@127.0.0.1:8080",
        "http://127.0.0.1.evil.com:8080",
        "127.0.0.1:8080",
    ] {
        assert!(parse_url(url).is_err(), "{url}");
    }
}

#[test]
fn text_is_quoted_for_json() {
    assert_eq!(quote("a\"b\\c\nd\u{1}"), "\"a\\\"b\\\\c\\nd\\u0001\"");
    let body = Rogatio {
        system: "s",
        user: "u\"",
        grammar: "root ::= \"x\"",
        max_tokens: 5,
        temperature: 0.5,
    }
    .body();
    assert!(annulus::lingua::json::parse(&body).is_ok(), "{body}");
}

const OK: &str = r#"{"choices":[{"message":{"role":"assistant","content":"{\"exitus\":\"obsequium\",\"responsum\":\"ita\"}"}}]}"#;

#[test]
fn replies_are_read_plain_or_chunked() {
    let plain = format!(
        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{OK}",
        OK.len()
    );
    assert!(read_reply(plain.as_bytes()).unwrap().contains("obsequium"));
    let (a, b) = OK.split_at(10);
    let chunked = format!(
        "HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n{:x}\r\n{a}\r\n{:x}\r\n{b}\r\n0\r\n\r\n",
        a.len(),
        b.len()
    );
    assert!(read_reply(chunked.as_bytes())
        .unwrap()
        .contains("obsequium"));
    let failed = "HTTP/1.1 500 Internal Server Error\r\n\r\n{\"error\":\"grammar\"}";
    assert!(read_reply(failed.as_bytes()).unwrap_err().contains("500"));
    assert!(read_reply(b"HTTP/1.1 200 OK\r\n\r\n{}").is_err());
    assert!(read_reply(b"garbage").is_err());
}

#[test]
fn a_request_reaches_a_local_server() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let server = thread::spawn(move || {
        let (mut s, _) = listener.accept().unwrap();
        let mut buf = vec![0u8; 65536];
        let mut got = Vec::new();
        while !String::from_utf8_lossy(&got).contains("}]") || !got.ends_with(b"}") {
            let n = s.read(&mut buf).unwrap();
            if n == 0 {
                break;
            }
            got.extend_from_slice(&buf[..n]);
        }
        let reply = format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{OK}",
            OK.len()
        );
        s.write_all(reply.as_bytes()).unwrap();
        String::from_utf8(got).unwrap()
    });
    let exemplar = parse_url(&format!("http://127.0.0.1:{port}")).unwrap();
    let reply = exemplar
        .ask(&Rogatio {
            system: "sys",
            user: "usr",
            grammar: "root ::= \"x\"",
            max_tokens: 7,
            temperature: 0.1,
        })
        .unwrap();
    assert!(reply.contains("obsequium"));
    let request = server.join().unwrap();
    assert!(request.starts_with("POST /v1/chat/completions HTTP/1.1\r\n"));
    assert!(request.contains("\"grammar\":\"root ::= \\\"x\\\"\""));
    assert!(request.contains("\"enable_thinking\":false"));
}

#[test]
fn an_absent_model_is_an_error() {
    let port = TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port();
    let exemplar = parse_url(&format!("http://127.0.0.1:{port}")).unwrap();
    let e = exemplar
        .ask(&Rogatio {
            system: "",
            user: "",
            grammar: "",
            max_tokens: 1,
            temperature: 0.0,
        })
        .unwrap_err();
    assert!(e.contains("cannot reach"), "{e}");
}
