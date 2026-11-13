use std::io::{Read, Write};
use std::net::TcpListener;
use std::thread;

use spiri72::llama::{parse_url, quote, read_reply, read_reply_counted, Rogatio, Sampler};

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
        sampler: &Sampler::default(),
    }
    .body();
    assert!(annulus::lingua::json::parse(&body).is_ok(), "{body}");
}

#[test]
fn the_whole_sampler_is_sent() {
    let mut sampler = Sampler::default();
    let body = |s: &Sampler| {
        Rogatio {
            system: "s",
            user: "u",
            grammar: "g",
            max_tokens: 5,
            sampler: s,
        }
        .body()
    };
    let plain = body(&sampler);
    for part in [
        "\"temperature\":0.7",
        "\"top_p\":0.8",
        "\"top_k\":20",
        "\"min_p\":0",
        "\"presence_penalty\":1.5",
    ] {
        assert!(plain.contains(part), "{part} in {plain}");
    }
    assert!(!plain.contains("seed"), "{plain}");
    assert!(plain.contains("\"cache_prompt\":true"), "{plain}");
    sampler.set("--seed", "72").unwrap();
    sampler.set("--top-k", "40").unwrap();
    let seeded = body(&sampler);
    assert!(seeded.contains("\"seed\":72") && seeded.contains("\"top_k\":40"));
    assert!(annulus::lingua::json::parse(&seeded).is_ok(), "{seeded}");
}

#[test]
fn sampler_values_out_of_range_are_refused() {
    let mut s = Sampler::default();
    for (flag, v) in [
        ("--temperature", "3"),
        ("--temperature", "NaN"),
        ("--top-p", "1.5"),
        ("--top-k", "-1"),
        ("--min-p", "2"),
        ("--presence-penalty", "9"),
        ("--seed", "x"),
        ("--nonsense", "1"),
    ] {
        assert!(s.set(flag, v).is_err(), "{flag} {v}");
    }
    assert_eq!(s, Sampler::default());
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
            sampler: &Sampler::default(),
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
            sampler: &Sampler::default(),
        })
        .unwrap_err();
    assert!(e.contains("cannot reach"), "{e}");
}

#[test]
fn the_cost_of_a_reply_is_read() {
    let body = r#"{"choices":[{"message":{"content":"x"}}],"usage":{"prompt_tokens":900,"completion_tokens":40},"timings":{"cache_n":850}}"#;
    let raw = format!(
        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{body}",
        body.len()
    );
    let r = read_reply_counted(raw.as_bytes()).unwrap();
    assert_eq!(
        (r.content.as_str(), r.prompt, r.cached, r.completion),
        ("x", 900, 850, 40)
    );
    // A server that gives no counts still gives the reply.
    let raw = format!(
        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{OK}",
        OK.len()
    );
    let r = read_reply_counted(raw.as_bytes()).unwrap();
    assert_eq!((r.prompt, r.cached, r.completion), (0, 0, 0));
}
