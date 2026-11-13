use std::io::{Read, Write};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, TcpStream};
use std::time::Duration;

use annulus::lingua::json::{self, Json};

pub const MAX_REPLY_BYTES: usize = 4 * 1024 * 1024;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Exemplar {
    pub address: SocketAddr,
    pub url: String,
}

pub fn parse_url(url: &str) -> Result<Exemplar, String> {
    let refuse = |why: &str| format!("the model's address {url:?} is refused: {why}");
    let rest = url
        .strip_prefix("http://")
        .ok_or_else(|| refuse("only http:// on this computer"))?;
    let authority = rest.strip_suffix('/').unwrap_or(rest);
    if authority.contains('/') || authority.contains('@') || authority.contains('?') {
        return Err(refuse("give only the host and port"));
    }
    let (host, port) = authority
        .rsplit_once(':')
        .ok_or_else(|| refuse("the port is missing"))?;
    let port: u16 = port
        .parse()
        .ok()
        .filter(|p| *p != 0)
        .ok_or_else(|| refuse("the port is not a number"))?;
    let ip = match host {
        "localhost" | "127.0.0.1" => IpAddr::V4(Ipv4Addr::LOCALHOST),
        "[::1]" => IpAddr::V6(Ipv6Addr::LOCALHOST),
        _ => return Err(refuse("the model must run on this computer")),
    };
    Ok(Exemplar {
        address: SocketAddr::new(ip, port),
        url: format!("http://{authority}"),
    })
}

pub fn quote(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

#[derive(Clone, Debug)]
pub struct Rogatio<'a> {
    pub system: &'a str,
    pub user: &'a str,
    pub grammar: &'a str,
    pub max_tokens: u32,
    pub temperature: f32,
}

impl Rogatio<'_> {
    pub fn body(&self) -> String {
        format!(
            "{{\"messages\":[{{\"role\":\"system\",\"content\":{}}},{{\"role\":\"user\",\"content\":{}}}],\"grammar\":{},\"max_tokens\":{},\"temperature\":{},\"stream\":false,\"chat_template_kwargs\":{{\"enable_thinking\":false}}}}",
            quote(self.system),
            quote(self.user),
            quote(self.grammar),
            self.max_tokens,
            self.temperature
        )
    }
}

fn dechunk(mut body: &[u8]) -> Result<Vec<u8>, String> {
    let mut out = Vec::new();
    loop {
        let end = body
            .windows(2)
            .position(|w| w == b"\r\n")
            .ok_or("a broken chunked reply")?;
        let size = std::str::from_utf8(&body[..end])
            .ok()
            .and_then(|s| usize::from_str_radix(s.split(';').next()?.trim(), 16).ok())
            .ok_or("a broken chunk size")?;
        body = &body[end + 2..];
        if size == 0 {
            return Ok(out);
        }
        if body.len() < size + 2 {
            return Err("a chunk is cut short".into());
        }
        out.extend_from_slice(&body[..size]);
        body = &body[size + 2..];
    }
}

pub fn read_reply(raw: &[u8]) -> Result<String, String> {
    let split = raw
        .windows(4)
        .position(|w| w == b"\r\n\r\n")
        .ok_or("the model's server sent no headers")?;
    let head = String::from_utf8_lossy(&raw[..split]).to_string();
    let mut lines = head.lines();
    let status = lines.next().unwrap_or_default();
    let code = status.split_whitespace().nth(1).unwrap_or_default();
    let mut body = raw[split + 4..].to_vec();
    let chunked = lines.any(|l| {
        l.to_ascii_lowercase()
            .replace(' ', "")
            .starts_with("transfer-encoding:chunked")
    });
    if chunked {
        body = dechunk(&body)?;
    }
    let text = String::from_utf8(body).map_err(|_| "the model's reply is not UTF-8")?;
    if code != "200" {
        let short = annulus::lingua::exsequi::visible(&text.chars().take(300).collect::<String>())
            .replace('\n', " ");
        return Err(format!("the model's server answered {status}: {short}"));
    }
    let root = json::parse(&text).map_err(|e| format!("the model's reply is not JSON: {e}"))?;
    root.get("choices")
        .map(Json::list)
        .and_then(|c| c.first())
        .and_then(|c| c.get("message"))
        .and_then(|m| m.get("content"))
        .and_then(Json::text)
        .map(str::to_string)
        .ok_or_else(|| "the model's reply has no message".into())
}

impl Exemplar {
    pub fn ask(&self, rogatio: &Rogatio) -> Result<String, String> {
        self.ask_body(&rogatio.body())
    }

    /// Sends a ready chat request body and reads the message of its reply.
    pub fn ask_body(&self, body: &str) -> Result<String, String> {
        let request = format!(
            "POST /v1/chat/completions HTTP/1.1\r\nHost: {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            self.address,
            body.len()
        );
        let mut stream = TcpStream::connect_timeout(&self.address, Duration::from_secs(10))
            .map_err(|e| format!("cannot reach the model at {}: {e}", self.url))?;
        stream
            .set_read_timeout(Some(Duration::from_secs(900)))
            .and_then(|_| stream.set_write_timeout(Some(Duration::from_secs(60))))
            .map_err(|e| e.to_string())?;
        stream
            .write_all(request.as_bytes())
            .map_err(|e| format!("cannot send to the model: {e}"))?;
        let mut raw = Vec::new();
        stream
            .take(MAX_REPLY_BYTES as u64 + 1)
            .read_to_end(&mut raw)
            .map_err(|e| format!("cannot read the model's reply: {e}"))?;
        if raw.len() > MAX_REPLY_BYTES {
            return Err("the model's reply is too long".into());
        }
        read_reply(&raw)
    }
}
