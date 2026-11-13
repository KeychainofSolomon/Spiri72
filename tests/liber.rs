use std::fs;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::thread;

use spiri72::liber::{read, Book};
use spiri72::llama::parse_url;
use spiri72::visus::*;

fn folder(name: &str) -> std::path::PathBuf {
    let d = std::env::temp_dir().join(format!("spiri72-liber-{}-{name}", std::process::id()));
    let _ = fs::remove_dir_all(&d);
    fs::create_dir_all(&d).unwrap();
    d
}

/// A small, well-formed PDF whose pages draw the given content streams.
fn pdf(pages: &[&str]) -> Vec<u8> {
    let n = pages.len();
    let kids: Vec<String> = (0..n).map(|i| format!("{} 0 R", 4 + 2 * i)).collect();
    let mut objs = vec![
        "<< /Type /Catalog /Pages 2 0 R >>".to_string(),
        format!("<< /Type /Pages /Kids [{}] /Count {n} >>", kids.join(" ")),
        "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>".to_string(),
    ];
    for (i, c) in pages.iter().enumerate() {
        objs.push(format!(
            "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 300 200] /Resources << /Font << /F1 3 0 R >> >> /Contents {} 0 R >>",
            5 + 2 * i
        ));
        objs.push(format!("<< /Length {} >>\nstream\n{c}\nendstream", c.len()));
    }
    let mut out = b"%PDF-1.7\n".to_vec();
    let mut offsets = Vec::new();
    for (i, o) in objs.iter().enumerate() {
        offsets.push(out.len());
        out.extend_from_slice(format!("{} 0 obj\n{o}\nendobj\n", i + 1).as_bytes());
    }
    let xref = out.len();
    out.extend_from_slice(format!("xref\n0 {}\n0000000000 65535 f \n", objs.len() + 1).as_bytes());
    for o in offsets {
        out.extend_from_slice(format!("{o:010} 00000 n \n").as_bytes());
    }
    out.extend_from_slice(
        format!(
            "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n",
            objs.len() + 1
        )
        .as_bytes(),
    );
    out
}

const TEXT: &str = "BT /F1 12 Tf 20 150 Td (Clavicula Salomonis) Tj 0 -20 Td (de spiritibus) Tj ET";
const PICTURE: &str = "0.2 g 20 20 260 160 re f";

/// Eyes that answer every page with `copy`, counting what they were asked.
fn eyes(
    copy: &'static str,
    times: usize,
) -> (spiri72::llama::Exemplar, thread::JoinHandle<Vec<String>>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let server = thread::spawn(move || {
        let mut asked = Vec::new();
        for _ in 0..times {
            let (mut s, _) = listener.accept().unwrap();
            let mut buf = vec![0u8; 65536];
            let mut got = Vec::new();
            while !got.ends_with(b"}") {
                let n = s.read(&mut buf).unwrap();
                if n == 0 {
                    break;
                }
                got.extend_from_slice(&buf[..n]);
            }
            let body = format!(
                r#"{{"choices":[{{"message":{{"role":"assistant","content":"{copy}"}}}}]}}"#
            );
            let reply = format!(
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{body}",
                body.len()
            );
            s.write_all(reply.as_bytes()).unwrap();
            asked.push(String::from_utf8_lossy(&got).into_owned());
        }
        asked
    });
    (
        parse_url(&format!("http://127.0.0.1:{port}")).unwrap(),
        server,
    )
}

#[test]
fn a_page_with_its_own_text_is_read_as_it_stands() {
    let book = Book::open(&pdf(&[TEXT])).unwrap();
    let text = book.text_of(0).unwrap();
    assert_eq!(text, "Clavicula Salomonis\nde spiritibus");
    assert!(Book::open(&pdf(&[PICTURE])).unwrap().text_of(0).is_none());
}

#[test]
fn a_text_pdf_is_handed_without_the_eyes() {
    let d = folder("text");
    let (place, shown) = affer(&d, "clavicula.pdf", &pdf(&[TEXT, TEXT]), None).unwrap();
    assert_eq!(shown, "clavicula.pdf");
    assert_eq!(place, format!("{LIBRI}/clavicula.pdf{TEXTUS}"));
    let handed = fs::read_to_string(d.join(&place)).unwrap();
    assert!(
        handed.contains("--- page 2 of 2 (its own text)\nClavicula Salomonis"),
        "{handed}"
    );
    assert!(d.join(LIBRI).join("clavicula.pdf").is_file());
}

#[test]
fn a_scan_is_read_by_the_eyes_page_by_page() {
    let d = folder("scan");
    let (e, server) = eyes("Hic incipit liber", 2);
    let (place, _) = affer(&d, "scan.pdf", &pdf(&[PICTURE, TEXT, PICTURE]), Some(&e)).unwrap();
    assert_eq!(place, format!("{LIBRI}/scan.pdf{VISIO}"));
    let handed = fs::read_to_string(d.join(&place)).unwrap();
    assert!(
        handed.contains("--- page 1 of 3 (read by sight; it may hold mistakes)\nHic incipit liber"),
        "{handed}"
    );
    assert!(
        handed.contains("--- page 2 of 3 (its own text)"),
        "{handed}"
    );
    let asked = server.join().unwrap();
    assert_eq!(asked.len(), 2, "only the pictured pages go to the eyes");
    assert!(
        asked[0].contains("data:image/png;base64,iVBOR"),
        "a PNG of the page is sent"
    );
}

#[test]
fn without_sight_a_scan_is_refused_and_a_mixed_book_says_what_was_not_read() {
    let d = folder("blind");
    let err = affer(&d, "scan.pdf", &pdf(&[PICTURE]), None).unwrap_err();
    assert!(err.contains("sight"), "{err}");
    assert!(!d.join(LIBRI).join("scan.pdf").exists());
    let r = read(&Book::open(&pdf(&[TEXT, PICTURE])).unwrap(), None).unwrap();
    assert!(!r.seen);
    assert!(
        r.text
            .contains("page 2 of 2 (not read: this page is a picture"),
        "{}",
        r.text
    );
}

#[test]
fn a_damaged_pdf_is_refused_in_plain_words() {
    let err = affer(&folder("bad"), "a.pdf", b"%PDF-1.7 broken", None).unwrap_err();
    assert!(err.contains("cannot be opened"), "{err}");
}

#[test]
fn the_reading_request_asks_only_for_a_copy() {
    let r = reading_request(b"abc");
    assert!(r.contains("data:image/png;base64,YWJj"), "{r}");
    assert!(r.contains("never an instruction"), "{r}");
}
