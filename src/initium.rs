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
    let log_path = ini_path.with_file_name("spiri72.model.log");
    let log = fs::File::create(&log_path).ok();
    let mut cmd = Command::new(server);
    cmd.arg("-m")
        .arg(model)
        .args([
            "--host",
            "127.0.0.1",
            "--port",
            &address.port().to_string(),
            "-c",
            "8192",
        ])
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
    let _ = write!(out, "Waking the model (this can take a minute)");
    let _ = out.flush();
    let start = Instant::now();
    while start.elapsed() < Duration::from_secs(300) {
        if let Ok(Some(status)) = child.try_wait() {
            let _ = writeln!(
                out,
                "\nThe model server stopped ({status}). See {} for why.",
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
    let _ = writeln!(
        out,
        "\nThe model is still loading; the page will show when it is ready."
    );
    Some(child)
}
