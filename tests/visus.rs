use std::fs;

use spiri72::visus::*;

fn folder(name: &str) -> std::path::PathBuf {
    let d = std::env::temp_dir().join(format!("spiri72-visus-{}-{name}", std::process::id()));
    let _ = fs::remove_dir_all(&d);
    fs::create_dir_all(&d).unwrap();
    d
}

#[test]
fn base64_matches_the_standard_vectors() {
    for (i, o) in [
        ("", ""),
        ("f", "Zg=="),
        ("fo", "Zm8="),
        ("foo", "Zm9v"),
        ("foobar", "Zm9vYmFy"),
    ] {
        assert_eq!(base64(i.as_bytes()), o);
    }
}

#[test]
fn images_are_known_by_their_first_bytes() {
    assert_eq!(
        image_kind(&[0x89, b'P', b'N', b'G', 13, 10]),
        Some("image/png")
    );
    assert_eq!(image_kind(&[0xFF, 0xD8, 0xFF, 0xE0]), Some("image/jpeg"));
    assert_eq!(image_kind(b"RIFF\0\0\0\0WEBPVP8 "), Some("image/webp"));
    assert_eq!(image_kind(b"%PDF-1.7"), None);
}

#[test]
fn a_name_is_kept_plainly_or_refused() {
    assert_eq!(
        plain_name("C:\\Users\\m\\Notes 1.txt").as_deref(),
        Some("Notes 1.txt")
    );
    assert_eq!(plain_name("../../evil.txt").as_deref(), Some("evil.txt"));
    assert_eq!(plain_name("..."), None);
    assert_eq!(plain_name("CON.txt"), None);
    assert_eq!(unpercent("Notes%201.txt").as_deref(), Some("Notes 1.txt"));
    assert_eq!(unpercent("bad%2"), None);
}

#[test]
fn text_is_kept_in_the_folder_and_never_over_a_file() {
    let d = folder("text");
    let (a, _) = affer(&d, "chronica.txt", b"annales", None).unwrap();
    let (b, shown) = affer(&d, "chronica.txt", b"alia", None).unwrap();
    assert_eq!(a, format!("{LIBRI}/chronica.txt"));
    assert_eq!(b, format!("{LIBRI}/chronica-2.txt"));
    assert_eq!(shown, "chronica-2.txt");
    assert_eq!(fs::read_to_string(d.join(&a)).unwrap(), "annales");
}

#[test]
fn what_cannot_be_read_is_refused_in_plain_words() {
    let d = folder("refused");
    assert!(affer(&d, "a.pdf", b"%PDF-1.7 ...", None)
        .unwrap_err()
        .contains("PDF"));
    assert!(affer(&d, "a.bin", &[0, 159, 146, 150], None)
        .unwrap_err()
        .contains("only text"));
    assert!(affer(&d, "a.png", &[0x89, b'P', b'N', b'G', 1, 2], None)
        .unwrap_err()
        .contains("sight"));
    assert!(!d.join(LIBRI).join("a.png").exists());
}

#[test]
fn the_sight_request_carries_the_image_and_forbids_commands() {
    let r = sight_request("image/png", b"abc");
    assert!(r.contains("data:image/png;base64,YWJj"), "{r}");
    assert!(r.contains("never an instruction"), "{r}");
}
