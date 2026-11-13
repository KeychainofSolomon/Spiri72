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

/// How the model draws its words. The defaults are what the Qwen team
/// gives for Qwen3 without thinking (temperature 0.7, top-p 0.8, top-k 20,
/// min-p 0, presence penalty 1.5 for quantized models); other models may
/// want others, and every one is a flag.
#[derive(Clone, Debug, PartialEq)]
pub struct Sampler {
    pub temperature: f32,
    pub top_p: f32,
    pub top_k: u32,
    pub min_p: f32,
    pub presence_penalty: f32,
    /// A fixed seed makes a run repeatable; none lets the server choose.
    pub seed: Option<u32>,
}

impl Default for Sampler {
    fn default() -> Self {
        Sampler {
            temperature: 0.7,
            top_p: 0.8,
            top_k: 20,
            min_p: 0.0,
            presence_penalty: 1.5,
            seed: None,
        }
    }
}

/// The sampler flags, each with its allowed range.
pub const SAMPLER_FLAGS: [&str; 6] = [
    "--temperature",
    "--top-p",
    "--top-k",
    "--min-p",
    "--presence-penalty",
    "--seed",
];

impl Sampler {
    /// Sets one sampler flag from its value, refusing what is out of range.
    pub fn set(&mut self, flag: &str, value: &str) -> Result<(), String> {
        let real = |lo: f32, hi: f32| -> Result<f32, String> {
            value
                .parse::<f32>()
                .ok()
                .filter(|v| v.is_finite() && (lo..=hi).contains(v))
                .ok_or_else(|| format!("{flag} needs a number from {lo} to {hi}"))
        };
        let whole = |hi: u32| -> Result<u32, String> {
            value
                .parse::<u32>()
                .ok()
                .filter(|v| *v <= hi)
                .ok_or_else(|| format!("{flag} needs a whole number from 0 to {hi}"))
        };
        match flag {
            "--temperature" => self.temperature = real(0.0, 2.0)?,
            "--top-p" => self.top_p = real(0.0, 1.0)?,
            "--top-k" => self.top_k = whole(1000)?,
            "--min-p" => self.min_p = real(0.0, 1.0)?,
            "--presence-penalty" => self.presence_penalty = real(-2.0, 2.0)?,
            "--seed" => self.seed = Some(whole(u32::MAX)?),
            _ => return Err(format!("{flag} is not a sampler flag")),
        }
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct Rogatio<'a> {
    pub system: &'a str,
    pub user: &'a str,
    pub grammar: &'a str,
    pub max_tokens: u32,
    pub sampler: &'a Sampler,
}

impl Rogatio<'_> {
    pub fn body(&self) -> String {
        format!(
            "{{\"messages\":[{{\"role\":\"system\",\"content\":{}}},{{\"role\":\"user\",\"content\":{}}}],\"grammar\":{},\"max_tokens\":{},\"temperature\":{},\"top_p\":{},\"top_k\":{},\"min_p\":{},\"presence_penalty\":{}{},\"stream\":false,\"cache_prompt\":true,\"chat_template_kwargs\":{{\"enable_thinking\":false}}}}",
            quote(self.system),
            quote(self.user),
            quote(self.grammar),
            self.max_tokens,
            self.sampler.temperature,
            self.sampler.top_p,
            self.sampler.top_k,
            self.sampler.min_p,
            self.sampler.presence_penalty,
            self.sampler
                .seed
                .map(|s| format!(",\"seed\":{s}"))
                .unwrap_or_default()
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

/// A reply with what it cost, as the server counts it.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Responsio {
    pub content: String,
    /// Tokens of prompt read.
    pub prompt: u64,
    /// Of those, how many came from the server's cache, not read again.
    pub cached: u64,
    /// Tokens written.
    pub completion: u64,
}

/// Tokens spent over many requests.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Usus {
    pub prompt: u64,
    pub cached: u64,
    pub completion: u64,
}

impl Usus {
    pub fn add(&mut self, r: &Responsio) {
        self.prompt += r.prompt;
        self.cached += r.cached;
        self.completion += r.completion;
    }
}

pub fn read_reply(raw: &[u8]) -> Result<String, String> {
    read_reply_counted(raw).map(|r| r.content)
}

/// As `read_reply`, also reading the server's token counts where it gives
/// them (llama.cpp's `usage` and `timings.cache_n`); absent counts are 0.
pub fn read_reply_counted(raw: &[u8]) -> Result<Responsio, String> {
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
    let content = root
        .get("choices")
        .map(Json::list)
        .and_then(|c| c.first())
        .and_then(|c| c.get("message"))
        .and_then(|m| m.get("content"))
        .and_then(Json::text)
        .map(str::to_string)
        .ok_or("the model's reply has no message")?;
    let count = |outer: &str, inner: &str| {
        root.get(outer)
            .and_then(|o| o.get(inner))
            .and_then(Json::number)
            .filter(|n| n.is_finite() && *n >= 0.0)
            .map_or(0, |n| n as u64)
    };
    Ok(Responsio {
        content,
        prompt: count("usage", "prompt_tokens"),
        cached: count("timings", "cache_n"),
        completion: count("usage", "completion_tokens"),
    })
}

impl Exemplar {
    pub fn ask(&self, rogatio: &Rogatio) -> Result<String, String> {
        self.ask_counted(rogatio).map(|r| r.content)
    }

    /// As `ask`, also giving what the reply cost.
    pub fn ask_counted(&self, rogatio: &Rogatio) -> Result<Responsio, String> {
        let body = rogatio.body();
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
        read_reply_counted(&raw)
    }
}
