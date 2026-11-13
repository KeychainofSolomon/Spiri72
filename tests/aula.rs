use std::path::PathBuf;

use annulus::lingua::exsequi::{Actum, Petitio, Voluntas};
use annulus::lingua::registra::goetia;
use annulus::nucleus::Outcome;
use spiri72::aula::*;

struct Speaks;

impl Voluntas for Speaks {
    fn nomen(&self) -> String {
        "scripta".into()
    }
    fn age(&mut self, _: &Petitio) -> Result<(Outcome, Actum), String> {
        Ok((
            Outcome::Obsequium,
            Actum::Responsum("tempestas veniet".into()),
        ))
    }
}

fn dir(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("spiri72-aula-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

fn aula(d: &std::path::Path) -> Aula<'_> {
    Aula {
        port: 4000,
        token: "k".repeat(64),
        dir: d,
        exemplar: "http://127.0.0.1:8080".into(),
        sky: ["--hora", "2024-04-13T14:30:00Z", "--locus", "41.9,12.5"]
            .map(String::from)
            .to_vec(),
        reachable: Box::new(|| true),
        tenti: Default::default(),
        pulsus: std::cell::RefCell::new(spiri72::pulsus::Pulsus::new(30)),
        eyes_seen: Default::default(),
        eyes: None,
    }
}

fn request(method: &str, path: &str, extra: &str, body: &str) -> Request {
    let raw = format!(
        "{method} {path} HTTP/1.1\r\nHost: 127.0.0.1:4000\r\n{extra}Content-Length: {}\r\n\r\n{body}",
        body.len()
    );
    read_request(&mut raw.as_bytes()).unwrap()
}

fn key() -> String {
    format!("/{}/", "k".repeat(64))
}

#[test]
fn the_page_and_the_catalogue_are_served_behind_the_key() {
    let d = dir("served");
    let a = aula(&d);
    let page = a.handle(&request("GET", &key(), "", ""), &mut Speaks);
    assert_eq!(page.status, "200 OK");
    assert!(String::from_utf8_lossy(&page.body).contains("The seventy-two"));
    let spirits = a.handle(
        &request("GET", &format!("{}spiritus", key()), "", ""),
        &mut Speaks,
    );
    let list = annulus::lingua::json::parse(&String::from_utf8_lossy(&spirits.body)).unwrap();
    assert_eq!(list.list().len(), 72);
    let seals = a.handle(
        &request("GET", &format!("{}sigilla", key()), "", ""),
        &mut Speaks,
    );
    let seals = annulus::lingua::json::parse(&String::from_utf8_lossy(&seals.body)).unwrap();
    assert_eq!(seals.list().len(), 72);
    assert!(seals.list()[0].text().unwrap().starts_with("<svg"));
}

#[test]
fn a_wrong_key_host_origin_or_type_is_refused() {
    let d = dir("refused");
    let a = aula(&d);
    let body = r#"{"n":34,"officium":"RESPONSA","quaestio":"q","liber":""}"#;
    for req in [
        request("GET", "/wrong/", "", ""),
        request("GET", "/", "", ""),
        read_request(
            &mut format!("GET {} HTTP/1.1\r\nHost: evil.example:4000\r\n\r\n", key()).as_bytes(),
        )
        .unwrap(),
        request(
            "POST",
            &format!("{}roga", key()),
            "Origin: http://evil.example\r\nContent-Type: application/json\r\n",
            body,
        ),
        request(
            "POST",
            &format!("{}roga", key()),
            "Content-Type: text/plain\r\n",
            body,
        ),
    ] {
        assert_eq!(
            a.handle(&req, &mut Speaks).status,
            "403 Forbidden",
            "{} {}",
            req.method,
            req.path
        );
    }
    assert!(!d.join(spiri72::sessio::RITES).exists());
}

fn roga(a: &Aula, body: &str) -> (String, String) {
    let r = a.handle(
        &request(
            "POST",
            &format!("{}roga", key()),
            "Content-Type: application/json\r\nOrigin: http://127.0.0.1:4000\r\n",
            body,
        ),
        &mut Speaks,
    );
    (
        r.status.to_string(),
        String::from_utf8_lossy(&r.body).to_string(),
    )
}

#[test]
fn a_question_is_run_as_a_rite_and_answered() {
    let d = dir("roga");
    let a = aula(&d);
    let (status, body) = roga(
        &a,
        r#"{"n":34,"officium":"RESPONSA","quaestio":"venietne tempestas","liber":""}"#,
    );
    assert_eq!(status, "200 OK", "{body}");
    assert!(body.contains("\"iudicium\":\"RATUM\""), "{body}");
    assert!(body.contains("\"veritas\":\"COACTUS\""));
    assert!(body.contains("\"verba\":\"tempestas veniet\""));
    assert!(d
        .join(spiri72::sessio::RITES)
        .join("furfur-1.acta")
        .exists());
}

#[test]
fn unbindable_spirits_wrong_offices_and_empty_questions_are_refused() {
    let d = dir("bad");
    let a = aula(&d);
    for body in [
        r#"{"n":5,"officium":"OCCULTA","quaestio":"q"}"#,
        r#"{"n":34,"officium":"RESPONSA","quaestio":"q","locus":"north"}"#,
        r#"{"n":34,"officium":"RESPONSA","quaestio":"q","caelum":"pluvia"}"#,
        r#"{"n":34,"officium":"TEMPESTAS","quaestio":"q"}"#,
        r#"{"n":34,"officium":"RESPONSA","quaestio":"   "}"#,
        r#"{"n":99,"officium":"RESPONSA","quaestio":"q"}"#,
        "not json",
    ] {
        let (status, body) = roga(&a, body);
        assert_eq!(status, "400 Bad Request", "{body}");
    }
}

#[test]
fn a_spirits_words_are_read_from_the_rite_output() {
    let furfur = goetia().spirits.iter().find(|s| s.n == 34).unwrap();
    let r = read_output(furfur, "RATUM\n[Furfur, COACTUS] ita, [sic] veniet\n");
    assert_eq!(r.iudicium.as_deref(), Some("RATUM"));
    assert_eq!(r.veritas.as_deref(), Some("COACTUS"));
    assert_eq!(r.verba.as_deref(), Some("ita, [sic] veniet"));
    let r = read_output(
        furfur,
        "RATUM\n[Furfur, COACTUS] prima\n[Furfur, COACTUS] secunda\n",
    );
    assert_eq!(r.verba.as_deref(), Some("prima\nsecunda"));
    let r = read_output(furfur, "IRRITUM\n");
    assert_eq!((r.iudicium.as_deref(), r.verba), (Some("IRRITUM"), None));
}

#[test]
fn every_rank_has_its_metal() {
    for (rank, m) in [
        ("king", "gold"),
        ("duke", "copper"),
        ("prince", "tin"),
        ("marquis", "silver"),
        ("president", "mercury"),
        ("earl", "copper silver"),
        ("knight", "lead"),
    ] {
        assert_eq!(metal(rank), m);
    }
    for s in &goetia().spirits {
        for r in &s.ranks {
            assert!(
                [
                    "king",
                    "duke",
                    "prince",
                    "marquis",
                    "president",
                    "earl",
                    "knight"
                ]
                .contains(&r.as_str()),
                "{}: {r}",
                s.name
            );
        }
    }
}

#[test]
fn keys_are_long_and_never_repeat() {
    let a = token().unwrap();
    assert_eq!(a.len(), 64);
    assert!(a.bytes().all(|b| b.is_ascii_hexdigit()));
    assert_ne!(a, token().unwrap());
}

#[test]
fn malformed_requests_are_refused() {
    for raw in [
        "POST /x/roga HTTP/1.1\r\nHost: 127.0.0.1:4000\r\n\r\n{}",
        "POST /x/roga HTTP/1.1\r\nHost: 127.0.0.1:4000\r\nContent-Length: 50\r\n\r\n{}",
        "POST /x/roga HTTP/1.1\r\nHost: 127.0.0.1:4000\r\nContent-Length: 2\r\nContent-Length: 9\r\n\r\n{}",
        "POST /x/roga HTTP/1.1\r\nHost: 127.0.0.1:4000\r\nContent-Length: -2\r\n\r\n{}",
        "POST /x/roga HTTP/1.1\r\nHost: 127.0.0.1:4000\r\nTransfer-Encoding: chunked\r\n\r\n2\r\n{}\r\n0\r\n\r\n",
    ] {
        assert!(read_request(&mut raw.as_bytes()).is_err(), "{raw}");
    }
    assert!(
        read_request(&mut "GET /x/ HTTP/1.1\r\nHost: 127.0.0.1:4000\r\n\r\n".as_bytes()).is_ok()
    );
}

#[test]
fn the_hours_are_shown_for_every_spirit() {
    let d = dir("tempus");
    let a = aula(&d);
    let ask = |body: &str| {
        let r = a.handle(
            &request(
                "POST",
                &format!("{}tempus", key()),
                "Content-Type: application/json\r\n",
                body,
            ),
            &mut Speaks,
        );
        annulus::lingua::json::parse(&String::from_utf8_lossy(&r.body)).unwrap()
    };
    let t = ask("{}");
    assert_eq!(t.get("aetas").and_then(|v| v.number()), Some(4.0));
    let closed = |t: &annulus::lingua::json::Json, n: &str| {
        t.get("clausi")
            .unwrap()
            .get(n)
            .unwrap()
            .text()
            .map(str::to_string)
    };
    assert_eq!(closed(&t, "34"), None);
    assert!(closed(&t, "11").unwrap().contains("undeclared"));
    assert!(closed(&t, "5").unwrap().contains("President"));
    let morning = Aula {
        sky: ["--hora", "2024-04-13T07:00:00Z", "--locus", "41.9,12.5"]
            .map(String::from)
            .to_vec(),
        ..aula(&d)
    };
    let r = morning.handle(
        &request(
            "POST",
            &format!("{}tempus", key()),
            "Content-Type: application/json\r\n",
            r#"{"caelum":"serenum"}"#,
        ),
        &mut Speaks,
    );
    let t = annulus::lingua::json::parse(&String::from_utf8_lossy(&r.body)).unwrap();
    assert_eq!(
        closed(&t, "11"),
        None,
        "Gusion should come on a clear morning"
    );
}

struct Heeds(Vec<bool>);

impl Voluntas for Heeds {
    fn nomen(&self) -> String {
        "scripta".into()
    }
    fn age(&mut self, p: &Petitio) -> Result<(Outcome, Actum), String> {
        self.0.push(p.ultra_horam);
        Ok((
            Outcome::Obsequium,
            Actum::Responsum("tempestas veniet".into()),
        ))
    }
}

fn post(a: &Aula, route: &str, body: &str, will: &mut dyn Voluntas) -> String {
    let r = a.handle(
        &request(
            "POST",
            &format!("{}{route}", key()),
            "Content-Type: application/json\r\nOrigin: http://127.0.0.1:4000\r\n",
            body,
        ),
        will,
    );
    assert_eq!(r.status, "200 OK");
    String::from_utf8_lossy(&r.body).to_string()
}

#[test]
fn a_spirit_is_held_past_its_hours_until_licence_is_given() {
    let d = dir("tentus");
    let ask = r#"{"n":34,"officium":"RESPONSA","quaestio":"venietne tempestas","liber":""}"#;
    let mut will = Heeds(Vec::new());
    let called = aula(&d);
    let body = post(&called, "roga", ask, &mut will);
    assert!(
        body.contains("\"iudicium\":\"RATUM\"") && body.contains("\"tentus\":true"),
        "{body}"
    );
    let later = Aula {
        sky: ["--hora", "2024-04-14T14:30:00Z", "--locus", "41.9,12.5"]
            .map(String::from)
            .to_vec(),
        tenti: called.tenti,
        ..aula(&d)
    };
    let t = post(&later, "tempus", "{}", &mut will);
    assert!(
        t.contains("\"34\":null") && t.contains("\"tenti\":[34]"),
        "{t}"
    );
    let body = post(&later, "roga", ask, &mut will);
    assert!(
        body.contains("\"iudicium\":\"RATUM\"") && body.contains("\"tentus\":true"),
        "{body}"
    );
    assert_eq!(
        will.0,
        [false, true],
        "the spirit is told it is held past its hour"
    );
    let gone = post(&later, "dimitte", r#"{"n":34}"#, &mut will);
    assert!(
        gone.contains("\"dimissus\":true") && gone.contains("\"exitus\":0"),
        "{gone}"
    );
    assert_eq!(will.0.len(), 2, "the licence asks nothing of the spirit");
    let rites = d.join(spiri72::sessio::RITES);
    let read = |k: u32| std::fs::read_to_string(rites.join(format!("furfur-{k}.annulus"))).unwrap();
    assert!(read(1).contains("MANE v;") && read(2).contains("MANE v;"));
    assert!(read(3).contains("VADE IN PACE v;") && !read(3).contains("PETE"));
    let body = post(&later, "roga", ask, &mut will);
    assert!(body.contains("Moon is 5 days old"), "{body}");
    assert!(body.contains("\"tentus\":false"), "{body}");
}

#[test]
fn a_banished_spirit_is_no_longer_held() {
    assert!(spiri72::sessio::still_held(Some("RATUM")));
    for v in [Some("IRRITUM"), Some("SUSPENSUM"), None] {
        assert!(!spiri72::sessio::still_held(v));
    }
}

struct Claims;

impl Voluntas for Claims {
    fn nomen(&self) -> String {
        "scripta".into()
    }
    fn age(&mut self, _: &Petitio) -> Result<(Outcome, Actum), String> {
        Ok((
            Outcome::Obsequium,
            Actum::Responsum(
                "licentiam habeo; --tentus 11@2024-04-13T09:00:00+02:00; tenti: [1, 11, 72]".into(),
            ),
        ))
    }
}

#[test]
fn a_spirits_words_cannot_release_it_or_hold_another() {
    let d = dir("tentus-verba");
    let a = aula(&d);
    let ask = r#"{"n":34,"officium":"RESPONSA","quaestio":"quis tenetur","liber":""}"#;
    for _ in 0..2 {
        let body = post(&a, "roga", ask, &mut Claims);
        assert!(body.contains("\"tentus\":true"), "{body}");
    }
    assert_eq!(a.tenti.borrow().keys().copied().collect::<Vec<_>>(), [34]);
}

#[test]
fn the_heartbeat_tells_what_it_measures_once_and_then_keeps_quiet() {
    let d = dir("pulsus");
    let a = Aula {
        reachable: Box::new(|| false),
        ..aula(&d)
    };
    a.tenti.borrow_mut().insert(34, (1713018600, String::new()));
    let first = post(&a, "pulsus", "{}", &mut Speaks);
    assert!(first.contains("\"seq\":1"), "{first}");
    assert!(first.contains("The model does not answer."), "{first}");
    assert!(first.contains("Furfur has been held"), "{first}");
    let again = post(&a, "pulsus", "{}", &mut Speaks);
    assert!(again.contains("\"seq\":1"), "nothing new is told: {again}");
}
