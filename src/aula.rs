use std::collections::hash_map::RandomState;
use std::hash::{BuildHasher, Hasher};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::Path;
use std::time::{Duration, SystemTime};

use annulus::lingua::exsequi::Voluntas;
use annulus::lingua::json::{self, Json};
use annulus::lingua::registra::{goetia, Spirit};

use crate::llama::quote;
use crate::persona::entry;
use crate::sessio::{ask_once, telling_offices, unbindable};
use crate::sigilla::SIGILLA;

const PAGE: &str = include_str!("../aula/index.html");
const MAX_HEAD: usize = 16 * 1024;
const MAX_BODY: usize = 64 * 1024;

pub fn token() -> String {
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    (0..4u64)
        .map(|i| {
            let mut h = RandomState::new().build_hasher();
            h.write_u64(i);
            h.write_u128(now);
            h.write_u32(std::process::id());
            format!("{:016x}", h.finish())
        })
        .collect()
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
    let length: usize = headers
        .iter()
        .find(|(k, _)| k.eq_ignore_ascii_case("content-length"))
        .and_then(|(_, v)| v.parse().ok())
        .unwrap_or(0);
    if length > MAX_BODY {
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
    let rest: Vec<&str> = lines.collect();
    let rest = rest.join("\n");
    let prefix = format!("[{}, ", spirit.name);
    let (veritas, verba) = match rest.strip_prefix(&prefix).and_then(|r| r.split_once("] ")) {
        Some((v, words)) => (Some(v.to_string()), Some(words.to_string())),
        None if rest.is_empty() => (None, None),
        None => (None, Some(rest)),
    };
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

pub fn roga(body: &[u8], dir: &Path, will: &mut dyn Voluntas) -> Answer {
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
    let liber = field("liber").trim().replace('\\', "/");
    let handed: Vec<String> = if liber.is_empty() {
        Vec::new()
    } else {
        vec![liber]
    };
    let mut out = Vec::new();
    let mut err = Vec::new();
    let code = ask_once(
        spirit, &office, &question, &handed, dir, will, &mut out, &mut err,
    );
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
            "{{\"exitus\":{code},\"iudicium\":{},\"veritas\":{},\"verba\":{},\"error\":{error},\"acta\":{}}}",
            opt(&r.iudicium),
            opt(&r.veritas),
            opt(&r.verba),
            quote(&log)
        ),
    )
}

pub struct Aula<'a> {
    pub port: u16,
    pub token: String,
    pub dir: &'a Path,
    pub exemplar: String,
    pub reachable: Box<dyn Fn() -> bool + 'a>,
}

impl Aula<'_> {
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
                    "{{\"exemplar\":{},\"paratus\":{}}}",
                    quote(&self.exemplar),
                    (self.reachable)()
                ),
            ),
            ("POST", "roga") => {
                if req.header("content-type") != Some("application/json") {
                    return refused("wrong content type");
                }
                roga(&req.body, self.dir, will)
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
    out: &mut dyn Write,
    open: bool,
) -> u8 {
    let listener = match TcpListener::bind("127.0.0.1:0") {
        Ok(l) => l,
        Err(e) => {
            let _ = writeln!(out, "error: cannot open a port on this computer: {e}");
            return 1;
        }
    };
    let port = listener.local_addr().map(|a| a.port()).unwrap_or(0);
    let aula = Aula {
        port,
        token: token(),
        dir,
        exemplar: exemplar_url.to_string(),
        reachable: Box::new(move || {
            TcpStream::connect_timeout(&exemplar, Duration::from_millis(400)).is_ok()
        }),
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
