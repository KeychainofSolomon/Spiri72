//! Starting Spiri72 with nothing typed: the wand is found on its stick, and the model
//! server is started from the paths remembered in `spiri72.ini`, asked for once.

use std::fs;
use std::io::{BufRead, Write};
use std::net::{SocketAddr, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

/// Where the paths to the model server and the model are remembered, beside the program.
pub const INI: &str = "spiri72.ini";

/// The wand's folder and file on its stick (as `virga` lays them out).
const VIRGA_DIR: &str = "VIRGA";
const VIRGA_FILE: &str = "virga.txt";

/// The plugged-in wands: every drive whose root holds `VIRGA\virga.txt`.
pub fn wands() -> Vec<PathBuf> {
    let mut roots: Vec<PathBuf> = Vec::new();
    if cfg!(windows) {
        for l in b'A'..=b'Z' {
            roots.push(PathBuf::from(format!("{}:\\", l as char)));
        }
    } else {
        for base in ["/media", "/run/media", "/mnt", "/Volumes"] {
            let Ok(rd) = fs::read_dir(base) else { continue };
            for e in rd.flatten() {
                let p = e.path();
                roots.push(p.clone());
                if let Ok(inner) = fs::read_dir(&p) {
                    roots.extend(inner.flatten().map(|e| e.path()));
                }
            }
        }
    }
    roots
        .into_iter()
        .filter(|r| r.join(VIRGA_DIR).join(VIRGA_FILE).is_file())
        .collect()
}

/// The remembered paths: `llama-server = …` and `model = …`, one per line.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Ini {
    pub server: Option<PathBuf>,
    pub model: Option<PathBuf>,
    /// The eyes: a vision model (Qwen2.5-VL 3B) and its vision file (mmproj).
    pub eyes: Option<PathBuf>,
    pub eyes_mmproj: Option<PathBuf>,
    /// The Operator chose to go without sight.
    pub blind: bool,
    /// Seconds between the heartbeat's messages at least (`heartbeat = 60`); never under 30.
    pub heartbeat: Option<u64>,
}

pub fn read_ini(text: &str) -> Ini {
    let mut ini = Ini::default();
    for line in text.lines() {
        let Some((k, v)) = line.split_once('=') else {
            continue;
        };
        let v = PathBuf::from(clean_path(v));
        match k.trim() {
            "llama-server" => ini.server = Some(v),
            "model" => ini.model = Some(v),
            "eyes" => ini.eyes = Some(v),
            "eyes-mmproj" => ini.eyes_mmproj = Some(v),
            "sight" => ini.blind = v.as_os_str() == "none",
            "heartbeat" => ini.heartbeat = v.to_str().and_then(|s| s.trim().parse().ok()),
            _ => {}
        }
    }
    ini
}

pub fn write_ini(ini: &Ini) -> String {
    let mut s =
        String::from("# Spiri72 remembers here where the model server and the model are.\n");
    if let Some(p) = &ini.server {
        s.push_str(&format!("llama-server = {}\n", p.display()));
    }
    if let Some(p) = &ini.model {
        s.push_str(&format!("model = {}\n", p.display()));
    }
    if let Some(p) = &ini.eyes {
        s.push_str(&format!("eyes = {}\n", p.display()));
    }
    if let Some(p) = &ini.eyes_mmproj {
        s.push_str(&format!("eyes-mmproj = {}\n", p.display()));
    }
    if ini.blind {
        s.push_str("sight = none\n");
    }
    if let Some(n) = ini.heartbeat {
        s.push_str(&format!("heartbeat = {n}\n"));
    }
    s
}

/// A path as typed, pasted or dropped into the window: trimmed, and unquoted.
pub fn clean_path(s: &str) -> String {
    let t = s.trim();
    let t = t
        .strip_prefix('"')
        .and_then(|t| t.strip_suffix('"'))
        .unwrap_or(t);
    let t = t
        .strip_prefix('\'')
        .and_then(|t| t.strip_suffix('\''))
        .unwrap_or(t);
    t.trim().to_string()
}

fn ask_path(
    input: &mut dyn BufRead,
    out: &mut dyn Write,
    what: &str,
    ok: impl Fn(&Path) -> bool,
) -> Option<PathBuf> {
    loop {
        let _ = write!(out, "{what}\n> ");
        let _ = out.flush();
        let mut line = String::new();
        if input.read_line(&mut line).ok()? == 0 {
            return None;
        }
        let p = PathBuf::from(clean_path(&line));
        if p.as_os_str().is_empty() {
            return None;
        }
        if ok(&p) {
            return Some(p);
        }
        let _ = writeln!(
            out,
            "That is not it: {} was not found. Try again, or press Enter alone to go on without.",
            p.display()
        );
    }
}

/// Makes sure a model answers at `address`. If none does, starts `llama-server` with the
/// remembered model, asking once for the two paths (drag the files into the window) and
/// remembering them in `ini_path`. Returns the server it started, to be stopped at the end.
pub fn wake_model(
    address: SocketAddr,
    ini_path: &Path,
    input: &mut dyn BufRead,
    out: &mut dyn Write,
) -> Option<Child> {
    let answers = || TcpStream::connect_timeout(&address, Duration::from_millis(400)).is_ok();
    if answers() {
        return None;
    }
    let mut ini = fs::read_to_string(ini_path)
        .map(|t| read_ini(&t))
        .unwrap_or_default();
    let mut changed = false;
    if !ini.server.as_deref().is_some_and(Path::is_file) {
        let _ = writeln!(
            out,
            "No model answers yet, and Spiri72 can start one for you."
        );
        ini.server = ask_path(
            input,
            out,
            "Drag llama-server.exe (from the llama.cpp folder) into this window, then press Enter:",
            Path::is_file,
        );
        changed = true;
    }
    if ini.server.is_some() && !ini.model.as_deref().is_some_and(Path::is_file) {
        ini.model = ask_path(
            input,
            out,
            "Now drag the model file (it ends in .gguf) into this window, then press Enter:",
            Path::is_file,
        );
        changed = true;
    }
    let (Some(server), Some(model)) = (&ini.server, &ini.model) else {
        let _ = writeln!(
            out,
            "Going on without a model: start llama-server yourself, and the page will find it."
        );
        return None;
    };
    if changed {
        match fs::write(ini_path, write_ini(&ini)) {
            Ok(()) => {
                let _ = writeln!(out, "Remembered in {} for next time.", ini_path.display());
            }
            Err(e) => {
                let _ = writeln!(
                    out,
                    "Could not remember the paths in {}: {e}",
                    ini_path.display()
                );
            }
        }
    }
    start(
        server,
        &[OsArg::P(model.clone())],
        address,
        &ini_path.with_file_name("spiri72.model.log"),
        "Waking the model",
        8192,
        MODEL_CACHE_MIB,
        out,
    )
}

/// The environment variable by which llama-server takes `--cache-ram` (MiB).
pub const CACHE_RAM: &str = "LLAMA_ARG_CACHE_RAM";
/// The most ordinary memory the model's server may spend on remembered prompts.
pub const MODEL_CACHE_MIB: u32 = 256;
/// The eyes remember no prompts: each image is seen once.
pub const EYES_CACHE_MIB: u32 = 0;
/// The eyes' context. One image can cost several thousand tokens by itself, and the
/// eyes must still have room for the request and for what they write of it.
pub const EYES_CONTEXT: u32 = 8192;
/// The environment variable by which llama-server takes `--image-max-tokens`.
pub const IMAGE_MAX_TOKENS: &str = "LLAMA_ARG_IMAGE_MAX_TOKENS";
/// The most tokens one image may cost: with the longest request and reply the eyes
/// are asked for (about 1,700 tokens), it fits `EYES_CONTEXT`. A larger image is
/// scaled down by the server, not refused.
pub const EYES_IMAGE_TOKENS: u32 = 4096;

enum OsArg {
    P(PathBuf),
    S(&'static str),
}

/// Starts `server -m …` (with any further arguments) on `address`, logging to `log_path`,
/// and waits until it answers.
#[allow(clippy::too_many_arguments)]
fn start(
    server: &Path,
    model: &[OsArg],
    address: SocketAddr,
    log_path: &Path,
    what: &str,
    context: u32,
    cache_mib: u32,
    out: &mut dyn Write,
) -> Option<Child> {
    let answers = || TcpStream::connect_timeout(&address, Duration::from_millis(400)).is_ok();
    let log = fs::File::create(log_path).ok();
    let mut cmd = Command::new(server);
    cmd.arg("-m");
    for a in model {
        match a {
            OsArg::P(p) => cmd.arg(p),
            OsArg::S(s) => cmd.arg(s),
        };
    }
    cmd.args([
        "--host",
        "127.0.0.1",
        "--port",
        &address.port().to_string(),
        "-c",
        &context.to_string(),
    ])
    // llama-server keeps old prompts in ordinary memory, up to 8 GB unless told
    // otherwise. Named by its environment variable, not a flag, so that an older
    // build, which does not know it, still starts.
    .env(CACHE_RAM, cache_mib.to_string())
    .env(IMAGE_MAX_TOKENS, EYES_IMAGE_TOKENS.to_string())
    .stdin(Stdio::null());
    match log.and_then(|l| l.try_clone().ok().map(|c| (l, c))) {
        Some((a, b)) => {
            cmd.stdout(a).stderr(b);
        }
        None => {
            cmd.stdout(Stdio::null()).stderr(Stdio::null());
        }
    }
    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(e) => {
            let _ = writeln!(out, "Could not start {}: {e}", server.display());
            return None;
        }
    };
    let _ = write!(out, "{what} (this can take a minute)");
    let _ = out.flush();
    let begun = Instant::now();
    while begun.elapsed() < Duration::from_secs(300) {
        if let Ok(Some(status)) = child.try_wait() {
            let _ = writeln!(
                out,
                "\nIt stopped ({status}). See {} for why.",
                log_path.display()
            );
            return None;
        }
        if answers() {
            let _ = writeln!(out, " ready.");
            return Some(child);
        }
        std::thread::sleep(Duration::from_secs(1));
        let _ = write!(out, ".");
        let _ = out.flush();
    }
    let _ = writeln!(out, "\nStill loading; the page will show when it is ready.");
    Some(child)
}

/// The spirits' eyes, kept shut: where their server, model and vision file are, and the
/// server itself while it runs. They are opened for a look and shut after it, so their
/// memory is spent only while an image is being seen.
#[derive(Debug)]
pub struct Oculi {
    server: PathBuf,
    model: PathBuf,
    mmproj: PathBuf,
    address: SocketAddr,
    log: PathBuf,
    child: Option<Child>,
}

impl Oculi {
    fn answers(&self) -> bool {
        TcpStream::connect_timeout(&self.address, Duration::from_millis(400)).is_ok()
    }

    /// Opens the eyes if they are shut, and waits until they answer.
    pub fn open(&mut self) -> Result<(), String> {
        if self.answers() {
            return Ok(());
        }
        self.close();
        self.child = start(
            &self.server,
            &[
                OsArg::P(self.model.clone()),
                OsArg::S("--mmproj"),
                OsArg::P(self.mmproj.clone()),
            ],
            self.address,
            &self.log,
            "Opening the spirits' eyes",
            EYES_CONTEXT,
            EYES_CACHE_MIB,
            &mut std::io::sink(),
        );
        if self.answers() {
            Ok(())
        } else {
            self.close();
            Err(format!(
                "the eyes did not open; see {} for why",
                self.log.display()
            ))
        }
    }

    /// Shuts the eyes, if Spiri72 opened them, and gives their memory back.
    pub fn close(&mut self) {
        if let Some(mut c) = self.child.take() {
            let _ = c.kill();
            let _ = c.wait();
        }
    }
}

impl Drop for Oculi {
    fn drop(&mut self) {
        self.close();
    }
}

/// Makes the spirits' eyes ready one port above the model, from the remembered vision
/// model and its vision file, asking once (Enter alone: go without sight, and do not ask
/// again). The eyes are not opened here: they open when an image is handed. Eyes that
/// already answer are the Operator's own, and are left alone.
pub fn prepare_eyes(
    model_address: SocketAddr,
    ini_path: &Path,
    input: &mut dyn BufRead,
    out: &mut dyn Write,
) -> Option<Oculi> {
    let address = SocketAddr::new(model_address.ip(), model_address.port().checked_add(1)?);
    if TcpStream::connect_timeout(&address, Duration::from_millis(400)).is_ok() {
        return None;
    }
    let mut ini = fs::read_to_string(ini_path)
        .map(|t| read_ini(&t))
        .unwrap_or_default();
    if ini.blind {
        return None;
    }
    let server = ini.server.clone().filter(|p| p.is_file())?;
    let mut changed = false;
    if !ini.eyes.as_deref().is_some_and(Path::is_file) {
        let _ = writeln!(
            out,
            "The spirits can be given sight, to see the images you hand them."
        );
        ini.eyes = ask_path(input, out, "Drag the eyes' model (Qwen2.5-VL 3B, a .gguf) into this window, then press Enter (Enter alone: no sight):", Path::is_file);
        ini.blind = ini.eyes.is_none();
        changed = true;
    }
    if ini.eyes.is_some() && !ini.eyes_mmproj.as_deref().is_some_and(Path::is_file) {
        ini.eyes_mmproj = ask_path(input, out, "Now drag its vision file (its name begins with mmproj) into this window, then press Enter:", Path::is_file);
        ini.blind = ini.eyes_mmproj.is_none();
        changed = true;
    }
    if changed {
        let _ = fs::write(ini_path, write_ini(&ini));
    }
    let (Some(eyes), Some(mmproj)) = (ini.eyes, ini.eyes_mmproj) else {
        return None;
    };
    Some(Oculi {
        server,
        model: eyes,
        mmproj,
        address,
        log: ini_path.with_file_name("spiri72.eyes.log"),
        child: None,
    })
}
