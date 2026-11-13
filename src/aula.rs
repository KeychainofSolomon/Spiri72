use std::cell::RefCell;
use std::collections::BTreeMap;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use annulus::horae;
use annulus::lingua::exsequi::Voluntas;
use annulus::lingua::json::{self, Json};
use annulus::lingua::registra::{goetia, Spirit};

use crate::fors;
use crate::llama::quote;
use crate::persona::entry;
use crate::scientia::Consultatio;
use crate::sessio::{
    ask_once_judged, give_licence, held_sky, kept_sound, still_held, telling_offices, unbindable,
};
use crate::sigilla::SIGILLA;

const PAGE: &str = include_str!("../aula/index.html");
const MAX_HEAD: usize = 16 * 1024;
const MAX_BODY: usize = 64 * 1024;

/// The one-time key of the binding page: 256 bits from the system's own
/// cryptographic source.
pub fn token() -> Result<String, String> {
    fors::hex::<32>()
}

pub struct Request {
    pub method: String,
    pub path: String,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

impl Request {
    fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(name))
            .map(|(_, v)| v.as_str())
    }
}

pub fn read_request(stream: &mut dyn Read) -> Result<Request, String> {
    let mut raw = Vec::new();
    let mut buf = [0u8; 4096];
    let split = loop {
        if let Some(i) = raw.windows(4).position(|w| w == b"\r\n\r\n") {
            break i;
        }
        if raw.len() > MAX_HEAD {
            return Err("headers too long".into());
        }
        let n = stream.read(&mut buf).map_err(|e| e.to_string())?;
        if n == 0 {
            return Err("closed".into());
        }
        raw.extend_from_slice(&buf[..n]);
    };
    let head = String::from_utf8_lossy(&raw[..split]).to_string();
    let mut lines = head.lines();
    let mut first = lines.next().unwrap_or_default().split_whitespace();
    let method = first.next().unwrap_or_default().to_string();
    let path = first.next().unwrap_or_default().to_string();
    let headers: Vec<(String, String)> = lines
        .filter_map(|l| l.split_once(':'))
        .map(|(k, v)| (k.trim().to_string(), v.trim().to_string()))
        .collect();
    let lengths: Vec<&String> = headers
        .iter()
        .filter(|(k, _)| k.eq_ignore_ascii_case("content-length"))
        .map(|(_, v)| v)
        .collect();
    if headers
        .iter()
        .any(|(k, _)| k.eq_ignore_ascii_case("transfer-encoding"))
    {
        return Err("chunked requests are refused".into());
    }
    let length: usize = match lengths.as_slice() {
        [] if method == "GET" => 0,
        [one] => one.parse().map_err(|_| "a bad length".to_string())?,
        _ => return Err("the length is missing or given twice".into()),
    };
    // a file handed from the page may be large; every other request stays small
    let limit = if path.ends_with("/affer") {
        crate::visus::MAX_FILE
    } else {
        MAX_BODY
    };
    if length > limit {
        return Err("body too long".into());
    }
    let mut body = raw[split + 4..].to_vec();
    while body.len() < length {
        let n = stream.read(&mut buf).map_err(|e| e.to_string())?;
        if n == 0 {
            break;
        }
        body.extend_from_slice(&buf[..n]);
    }
    if body.len() < length {
        return Err("the body was cut short".into());
    }
    body.truncate(length);
    Ok(Request {
        method,
        path,
        headers,
        body,
    })
}

pub struct Answer {
    pub status: &'static str,
    pub kind: &'static str,
    pub body: Vec<u8>,
}

fn answer(status: &'static str, kind: &'static str, body: impl Into<Vec<u8>>) -> Answer {
    Answer {
        status,
        kind,
        body: body.into(),
    }
}

fn refused(why: &str) -> Answer {
    answer(
        "403 Forbidden",
        "text/plain; charset=utf-8",
        why.to_string(),
    )
}

pub fn metal(rank: &str) -> &'static str {
    match rank {
        "king" => "gold",
        "duke" => "copper",
        "prince" => "tin",
        "marquis" => "silver",
        "president" => "mercury",
        "earl" => "copper silver",
        _ => "lead",
    }
}

fn texts(n: u32, key: &str, with_kind: bool) -> String {
    let items: Vec<String> = entry(n)
        .and_then(|e| e.get(key))
        .map(Json::list)
        .unwrap_or(&[])
        .iter()
        .filter_map(|x| {
            let text = quote(x.get("text").and_then(Json::text)?);
            Some(match (with_kind, x.get("kind").and_then(Json::text)) {
                (true, Some(k)) => format!("{{\"kind\":{},\"text\":{text}}}", quote(k)),
                (true, None) => format!("{{\"text\":{text}}}"),
                (false, _) => text,
            })
        })
        .collect();
    format!("[{}]", items.join(","))
}

fn list(items: &[String]) -> String {
    format!(
        "[{}]",
        items.iter().map(|s| quote(s)).collect::<Vec<_>>().join(",")
    )
}

pub fn spiritus_json() -> String {
    let items: Vec<String> = goetia()
        .spirits
        .iter()
        .map(|s| {
            format!(
                "{{\"n\":{},\"name\":{},\"ranks\":{},\"legions\":{},\"offices\":{},\"harm\":{},\"restraints\":{},\"veracity\":{},\"condition\":{},\"telling\":{},\"closed\":{}}}",
                s.n,
                quote(&s.name),
                list(&s.ranks),
                s.legions,
                texts(s.n, "offices", true),
                texts(s.n, "malice", false),
                texts(s.n, "counterbalances", false),
                quote(&s.veracity),
                quote(&s.veracity_condition),
                list(&telling_offices(s)),
                unbindable(s).map(|w| quote(&w)).unwrap_or_else(|| "null".into()),
            )
        })
        .collect();
    format!("[{}]", items.join(","))
}

pub fn sigilla_json() -> String {
    let items: Vec<String> = SIGILLA.iter().map(|s| quote(s.trim())).collect();
    format!("[{}]", items.join(","))
}

pub struct Responsum {
    pub iudicium: Option<String>,
    pub veritas: Option<String>,
    pub verba: Option<String>,
}

pub fn read_output(spirit: &Spirit, out: &str) -> Responsum {
    let mut lines = out.lines().map(str::trim_end).filter(|l| !l.is_empty());
    let iudicium = lines
        .next()
        .filter(|l| ["RATUM", "SUSPENSUM", "IRRITUM"].contains(l))
        .map(str::to_string);
    let prefix = format!("[{}, ", spirit.name);
    let mut veritas = None;
    let mut words = Vec::new();
    for line in lines {
        match line.strip_prefix(&prefix).and_then(|r| r.split_once("] ")) {
            Some((v, w)) => {
                veritas.get_or_insert_with(|| v.to_string());
                words.push(w.to_string());
            }
            None => words.push(line.to_string()),
        }
    }
    let verba = (!words.is_empty()).then(|| words.join("\n"));
    Responsum {
        iudicium,
        veritas,
        verba,
    }
}

fn first_error(log: &str) -> String {
    log.lines()
        .find(|l| {
            !l.starts_with("audit:")
                && !l.starts_with("awaits")
                && !l.starts_with("written:")
                && !l.starts_with("deleted:")
        })
        .unwrap_or("The rite was refused or broken off.")
        .trim_start_matches("error: ")
        .to_string()
}

fn sky_of(root: &Json, base: &[String]) -> Result<Vec<String>, String> {
    let mut flags: Vec<(String, String)> = base
        .chunks(2)
        .filter_map(|c| Some((c.first()?.clone(), c.get(1)?.clone())))
        .collect();
    let mut set = |k: &str, v: String| {
        flags.retain(|(f, _)| f != k);
        flags.push((k.to_string(), v));
    };
    if let Some(l) = root
        .get("locus")
        .and_then(Json::text)
        .filter(|l| !l.trim().is_empty())
    {
        horae::parse_locus(l)
            .ok_or("the place must be latitude,longitude, for example 52.52,13.40")?;
        set("--locus", l.trim().to_string());
    }
    match root.get("caelum").and_then(Json::text).unwrap_or("") {
        "" => {}
        c @ ("serenum" | "turbidum") => set("--caelum", c.to_string()),
        _ => return Err("the sky must be serenum or turbidum".into()),
    }
    Ok(flags.into_iter().flat_map(|(k, v)| [k, v]).collect())
}

pub fn caelum_of(flags: &[String]) -> horae::Caelum {
    let mut c = horae::Caelum::default();
    for pair in flags.chunks(2) {
        match (pair[0].as_str(), pair.get(1)) {
            ("--hora", Some(v)) => c.hora = horae::parse_hora(v),
            ("--locus", Some(v)) => c.locus = horae::parse_locus(v),
            ("--caelum", Some(v)) => c.serenum = Some(v == "serenum"),
            _ => {}
        }
    }
    c.hora = Some(c.hora.unwrap_or_else(horae::now));
    c
}

pub fn tempus(body: &[u8], base: &[String], tenti: &BTreeMap<u32, (i64, String)>) -> Answer {
    let root = std::str::from_utf8(body)
        .ok()
        .and_then(|t| json::parse(t).ok())
        .unwrap_or(Json::Null);
    let flags = match sky_of(&root, base) {
        Ok(f) => f,
        Err(e) => {
            return answer(
                "400 Bad Request",
                "application/json",
                format!("{{\"error\":{}}}", quote(&e)),
            )
        }
    };
    let c = caelum_of(&flags);
    let t = c.hora.unwrap_or(0);
    let open: Vec<String> = goetia()
        .spirits
        .iter()
        .map(|s| {
            let held = horae::Caelum {
                tentus: tenti.get(&s.n).map(|(t0, _)| (s.n as i64, *t0)),
                ..c
            };
            let why =
                unbindable(s).or_else(|| horae::admit_spiritum(s.n as i64, &s.ranks, &held).err());
            format!(
                "\"{}\":{}",
                s.n,
                why.map(|w| quote(&w)).unwrap_or_else(|| "null".into())
            )
        })
        .collect();
    answer(
        "200 OK",
        "application/json",
        format!(
            "{{\"hora\":{},\"aetas\":{},\"clausi\":{{{}}},\"tenti\":[{}]}}",
            quote(&horae::iso(t)),
            horae::aetas_lunae(t as f64),
            open.join(","),
            tenti
                .keys()
                .map(|n| n.to_string())
                .collect::<Vec<_>>()
                .join(",")
        ),
    )
}

pub fn roga(
    body: &[u8],
    dir: &Path,
    sky: &[String],
    tenti: &RefCell<BTreeMap<u32, (i64, String)>>,
    will: &mut dyn Voluntas,
    consulta: &Arc<Mutex<Option<Consultatio>>>,
) -> Answer {
    let bad = |why: &str| {
        answer(
            "400 Bad Request",
            "application/json",
            format!("{{\"exitus\":2,\"error\":{}}}", quote(why)),
        )
    };
    let Ok(text) = std::str::from_utf8(body) else {
        return bad("the request is not text");
    };
    let Ok(root) = json::parse(text) else {
        return bad("the request is not JSON");
    };
    let field = |k: &str| root.get(k).and_then(Json::text).unwrap_or("").to_string();
    let Some(n) = root.get("n").and_then(Json::number) else {
        return bad("no spirit was named");
    };
    let Some(spirit) = goetia().spirits.iter().find(|s| s.n as f64 == n) else {
        return bad("no spirit of the 72 has that number");
    };
    if let Some(why) = unbindable(spirit) {
        return bad(&why);
    }
    let office = field("officium");
    if !telling_offices(spirit).contains(&office) {
        return bad("that office is not one this spirit can be charged through");
    }
    let question = field("quaestio").trim().to_string();
    if question.is_empty() || question.chars().count() > 2000 {
        return bad("the question must be between 1 and 2000 characters");
    }
    let sky = match sky_of(&root, sky) {
        Ok(f) => f,
        Err(e) => return bad(&e),
    };
    let liber = field("liber").trim().replace('\\', "/");
    let handed: Vec<String> = if liber.is_empty() {
        Vec::new()
    } else {
        vec![liber]
    };
    let mut out = Vec::new();
    let mut err = Vec::new();
    let held = tenti.borrow().get(&spirit.n).map(|(t0, _)| *t0);
    let (sitting, t0) = held_sky(spirit, &sky, held);
    let (code, ritus, verdict) = ask_once_judged(
        spirit, &office, &question, &handed, dir, &sitting, will, &mut out, &mut err,
    );
    let read = consulta.lock().ok().and_then(|mut g| g.take());
    let scientia = match &read {
        Some(c) => format!(
            "{{\"liber\":{},\"terminus\":{},\"textus\":{}}}",
            quote(&c.liber),
            quote(&c.terminus),
            quote(&c.textus)
        ),
        None => "null".to_string(),
    };
    let tentus = still_held(verdict.as_deref());
    if tentus {
        tenti.borrow_mut().insert(spirit.n, (t0, office.clone()));
    } else {
        tenti.borrow_mut().remove(&spirit.n);
    }
    let ritus = ritus.filter(|r| kept_sound(dir, r));
    let out = String::from_utf8_lossy(&out).to_string();
    let log = String::from_utf8_lossy(&err).to_string();
    let r = read_output(spirit, &out);
    let opt = |v: &Option<String>| v.as_deref().map(quote).unwrap_or_else(|| "null".into());
    let error = if r.iudicium.is_none() {
        quote(&first_error(&log))
    } else {
        "null".into()
    };
    answer(
        "200 OK",
        "application/json",
        format!(
            "{{\"exitus\":{code},\"iudicium\":{},\"veritas\":{},\"verba\":{},\"error\":{error},\"acta\":{},\"ritus\":{},\"tentus\":{tentus},\"dimissus\":{},\"scientia\":{scientia}}}",
            opt(&r.iudicium),
            opt(&r.veritas),
            opt(&r.verba),
            quote(&log),
            opt(&ritus),
            held.is_some() && !tentus
        ),
    )
}

/// Gives a held spirit licence to depart, at the Operator's word.
pub fn dimitte(
    body: &[u8],
    dir: &Path,
    sky: &[String],
    tenti: &RefCell<BTreeMap<u32, (i64, String)>>,
    will: &mut dyn Voluntas,
) -> Answer {
    let root = std::str::from_utf8(body)
        .ok()
        .and_then(|t| json::parse(t).ok())
        .unwrap_or(Json::Null);
    let n = root.get("n").and_then(Json::number);
    let Some(spirit) = n.and_then(|n| goetia().spirits.iter().find(|s| s.n as f64 == n)) else {
        return answer(
            "400 Bad Request",
            "application/json",
            "{\"error\":\"no spirit of the 72 has that number\"}",
        );
    };
    let was = tenti.borrow_mut().remove(&spirit.n);
    let mut exitus = 0;
    if let Some((t0, office)) = &was {
        let sky = sky_of(&root, sky).unwrap_or_else(|_| sky.to_vec());
        let (mut out, mut err) = (Vec::new(), Vec::new());
        exitus = give_licence(spirit, office, *t0, dir, &sky, will, &mut out, &mut err);
    }
    let was = was.is_some();
    answer(
        "200 OK",
        "application/json",
        format!(
            "{{\"dimissus\":{was},\"exitus\":{exitus},\"nomen\":{}}}",
            quote(&spirit.name)
        ),
    )
}

/// Takes a rite's sound answer into its spirit's book, at the Operator's click.
pub fn recipe(body: &[u8], dir: &Path) -> Answer {
    let reply = |status: &'static str, ok: bool, msg: &str| {
        answer(
            status,
            "application/json",
            format!("{{\"scriptum\":{ok},\"nuntius\":{}}}", quote(msg)),
        )
    };
    let ritus = std::str::from_utf8(body)
        .ok()
        .and_then(|t| json::parse(t).ok())
        .and_then(|j| j.get("ritus").and_then(Json::text).map(str::to_string));
    let Some(ritus) = ritus else {
        return reply("400 Bad Request", false, "no rite was named");
    };
    match crate::sessio::receive(dir, &ritus) {
        Ok(e) => reply(
            "200 OK",
            true,
            &format!("Written in the book as entry {}.", e.n),
        ),
        Err(e) => reply("400 Bad Request", false, &e),
    }
}

pub struct Aula<'a> {
    pub port: u16,
    pub token: String,
    pub dir: &'a Path,
    pub exemplar: String,
    pub sky: Vec<String>,
    pub reachable: Box<dyn Fn() -> bool + 'a>,
    /// The spirits' eyes (a vision model one port above), if they answer.
    pub eyes: Option<crate::llama::Exemplar>,
    /// The spirits held until the Operator gives licence to depart, each
    /// with the hour of its calling.
    pub tenti: RefCell<BTreeMap<u32, (i64, String)>>,
    /// The heartbeat: measured facts, at most one message per gap.
    pub pulsus: RefCell<crate::pulsus::Pulsus>,
    /// Where the will sets down the last library reading, for the page to show.
    pub consulta: Arc<Mutex<Option<Consultatio>>>,
    /// Whether the eyes have answered once, so their loss is told and their absence is not.
    pub eyes_seen: std::cell::Cell<bool>,
}

impl Aula<'_> {
    /// One beat of the heart: what is measured now, and a message if it is due.
    fn pulse(&self, body: &[u8]) -> Answer {
        use crate::pulsus::{held_for, wand_present};
        let root = std::str::from_utf8(body)
            .ok()
            .and_then(|t| json::parse(t).ok())
            .unwrap_or(Json::Null);
        let flags = sky_of(&root, &self.sky).unwrap_or_else(|_| self.sky.clone());
        let c = caelum_of(&flags);
        let now = c.hora.unwrap_or_else(horae::now);
        let mut facts = Vec::new();
        if !(self.reachable)() {
            facts.push("The model does not answer.".to_string());
        }
        let eyes_up = self.eyes.as_ref().is_some_and(|e| {
            TcpStream::connect_timeout(&e.address, Duration::from_millis(300)).is_ok()
        });
        if eyes_up {
            self.eyes_seen.set(true);
        } else if self.eyes_seen.get() {
            facts.push("The eyes no longer answer.".to_string());
        }
        if let Some(stick) = crate::sessio::virga_of(&self.sky) {
            if !wand_present(Path::new(stick)) {
                crate::sessio::lay_down_binding();
                facts.push(
                    "The wand's stick has been taken out: the binding is laid down.".to_string(),
                );
            }
        }
        for (n, (t0, _)) in self.tenti.borrow().iter() {
            let Some(s) = goetia().by_number(*n as i64) else {
                continue;
            };
            facts.push(format!("{} has been held {}.", s.name, held_for(now - t0)));
            let held = horae::Caelum {
                tentus: Some((*n as i64, *t0)),
                ..c
            };
            if horae::ultra_horam(*n as i64, &s.ranks, &held) {
                facts.push(format!("{}'s hour has passed; it grows restless.", s.name));
            }
        }
        if crate::sessio::binding_left().is_some_and(|l| l < Duration::from_secs(300)) {
            facts.push(
                "The binding is laid down within five minutes unless a question is asked."
                    .to_string(),
            );
        }
        let mut p = self.pulsus.borrow_mut();
        p.beat(std::time::Instant::now(), facts);
        answer(
            "200 OK",
            "application/json",
            format!(
                "{{\"seq\":{},\"nuntius\":{},\"intervallum\":{}}}",
                p.seq,
                p.message
                    .as_deref()
                    .map(quote)
                    .unwrap_or_else(|| "null".into()),
                p.gap().as_secs()
            ),
        )
    }

    pub fn handle(&self, req: &Request, will: &mut dyn Voluntas) -> Answer {
        let host = format!("127.0.0.1:{}", self.port);
        if req.header("host") != Some(host.as_str()) {
            return refused("wrong host");
        }
        if let Some(origin) = req.header("origin") {
            if origin != format!("http://{host}") {
                return refused("wrong origin");
            }
        }
        let prefix = format!("/{}/", self.token);
        let Some(route) = req.path.strip_prefix(&prefix) else {
            return refused("unknown key");
        };
        match (req.method.as_str(), route) {
            ("GET", "") => answer("200 OK", "text/html; charset=utf-8", PAGE),
            ("GET", "spiritus") => answer("200 OK", "application/json", spiritus_json()),
            ("GET", "sigilla") => answer("200 OK", "application/json", sigilla_json()),
            ("GET", "status") => answer(
                "200 OK",
                "application/json",
                format!(
                    "{{\"exemplar\":{},\"paratus\":{},\"visus\":{},\"virga\":{}}}",
                    quote(&self.exemplar),
                    (self.reachable)(),
                    self.eyes.as_ref().is_some_and(|e| {
                        TcpStream::connect_timeout(&e.address, Duration::from_millis(300)).is_ok()
                    }),
                    crate::sessio::virga_of(&self.sky)
                        .map(quote)
                        .unwrap_or_else(|| "null".into())
                ),
            ),
            ("POST", "roga") => {
                if req.header("content-type") != Some("application/json") {
                    return refused("wrong content type");
                }
                roga(
                    &req.body,
                    self.dir,
                    &self.sky,
                    &self.tenti,
                    will,
                    &self.consulta,
                )
            }
            ("POST", "affer") => {
                if req.header("content-type") != Some("application/octet-stream") {
                    return refused("wrong content type");
                }
                let name = req
                    .header("x-nomen")
                    .and_then(crate::visus::unpercent)
                    .unwrap_or_default();
                let reply =
                    |status: &'static str, body: String| answer(status, "application/json", body);
                match crate::visus::affer(self.dir, &name, &req.body, self.eyes.as_ref()) {
                    Ok((place, shown)) => reply(
                        "200 OK",
                        format!(
                            "{{\"locus\":{},\"nomen\":{}}}",
                            quote(&place),
                            quote(&shown)
                        ),
                    ),
                    Err(e) => reply("400 Bad Request", format!("{{\"error\":{}}}", quote(&e))),
                }
            }
            ("POST", "recipe") => {
                if req.header("content-type") != Some("application/json") {
                    return refused("wrong content type");
                }
                recipe(&req.body, self.dir)
            }
            ("POST", "tempus") => {
                if req.header("content-type") != Some("application/json") {
                    return refused("wrong content type");
                }
                tempus(&req.body, &self.sky, &self.tenti.borrow())
            }
            ("POST", "pulsus") => {
                if req.header("content-type") != Some("application/json") {
                    return refused("wrong content type");
                }
                self.pulse(&req.body)
            }
            ("POST", "dimitte") => {
                if req.header("content-type") != Some("application/json") {
                    return refused("wrong content type");
                }
                dimitte(&req.body, self.dir, &self.sky, &self.tenti, will)
            }
            _ => answer("404 Not Found", "text/plain; charset=utf-8", "not found"),
        }
    }
}

pub fn write_answer(stream: &mut dyn Write, a: &Answer) {
    let head = format!(
        "HTTP/1.1 {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nCache-Control: no-store\r\nX-Content-Type-Options: nosniff\r\nReferrer-Policy: no-referrer\r\nContent-Security-Policy: default-src 'none'; style-src 'unsafe-inline'; script-src 'unsafe-inline'; connect-src 'self'; img-src data:; frame-ancestors 'none'; base-uri 'none'; form-action 'none'\r\nConnection: close\r\n\r\n",
        a.status,
        a.kind,
        a.body.len()
    );
    let _ = stream.write_all(head.as_bytes());
    let _ = stream.write_all(&a.body);
}

pub fn open_browser(url: &str) {
    let _ = if cfg!(windows) {
        std::process::Command::new("rundll32")
            .args(["url.dll,FileProtocolHandler", url])
            .spawn()
    } else if cfg!(target_os = "macos") {
        std::process::Command::new("open").arg(url).spawn()
    } else {
        std::process::Command::new("xdg-open").arg(url).spawn()
    };
}

pub fn serve(
    will: &mut dyn Voluntas,
    exemplar: std::net::SocketAddr,
    exemplar_url: &str,
    dir: &Path,
    sky: &[String],
    out: &mut dyn Write,
    open: bool,
    consulta: Arc<Mutex<Option<Consultatio>>>,
) -> u8 {
    let listener = match TcpListener::bind("127.0.0.1:0") {
        Ok(l) => l,
        Err(e) => {
            let _ = writeln!(out, "error: cannot open a port on this computer: {e}");
            return 1;
        }
    };
    let port = listener.local_addr().map(|a| a.port()).unwrap_or(0);
    let token = match token() {
        Ok(t) => t,
        Err(e) => {
            let _ = writeln!(out, "error: no key for the binding page: {e}");
            return 1;
        }
    };
    let aula = Aula {
        port,
        token,
        dir,
        exemplar: exemplar_url.to_string(),
        sky: sky.to_vec(),
        reachable: Box::new(move || {
            TcpStream::connect_timeout(&exemplar, Duration::from_millis(400)).is_ok()
        }),
        tenti: RefCell::default(),
        pulsus: RefCell::new(crate::pulsus::Pulsus::new(
            std::env::current_exe()
                .ok()
                .and_then(|p| p.parent().map(|d| d.join(crate::initium::INI)))
                .and_then(|p| std::fs::read_to_string(p).ok())
                .and_then(|t| crate::initium::read_ini(&t).heartbeat)
                .unwrap_or(30),
        )),
        eyes_seen: std::cell::Cell::new(false),
        eyes: crate::llama::parse_url(exemplar_url)
            .ok()
            .and_then(|e| crate::visus::eyes_of(&e)),
        consulta,
    };
    let url = format!("http://127.0.0.1:{port}/{}/", aula.token);
    let _ = writeln!(
        out,
        "Spiri72 is open at\n  {url}\nKeep this window open while you work; close it to end."
    );
    let _ = out.flush();
    if open {
        open_browser(&url);
    }
    for stream in listener.incoming() {
        let Ok(mut stream) = stream else { continue };
        let _ = stream.set_read_timeout(Some(Duration::from_secs(10)));
        let Ok(req) = read_request(&mut stream) else {
            continue;
        };
        let a = aula.handle(&req, will);
        write_answer(&mut stream, &a);
    }
    0
}
