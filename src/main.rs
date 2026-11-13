use std::env;
use std::io::{self, Write};
use std::process::ExitCode;

use annulus::mandata::{ritus_with, Usage, EXIT_USAGE};
use spiri72::agens::Agens;
use spiri72::aula::serve;
use spiri72::llama::parse_url;
use spiri72::sessio::sessio;

const DEFAULT_MODEL: &str = "http://127.0.0.1:8080";

const USAGE: &str =
    "usage: spiri72 [--virga-probationis | --virga E:\\] [--exemplar http://127.0.0.1:8080]   (the binding page)
       spiri72 <program.annulus> --virga-probationis [--exemplar http://127.0.0.1:8080]
               [--max-tokens N] [any other flag of annulus ritus]
       spiri72 --latina <Latin words>
       spiri72 --liber <spirit>            (read a spirit's book)
       spiri72 --dele <spirit> <entry>     (strike an entry from it)
       spiri72 --version";

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    let mut err = io::stderr().lock();
    if args.as_slice() == ["--version"] {
        println!("spiri72 {}", env!("CARGO_PKG_VERSION"));
        return ExitCode::SUCCESS;
    }
    if args.first().map(String::as_str) == Some("--latina") {
        let text = args[1..].join(" ");
        for w in text
            .split(|c: char| !c.is_alphabetic())
            .filter(|w| !w.is_empty())
        {
            let found = spiri72::latina::analyse(w);
            if found.is_empty() {
                println!("{w}: not in the lexicon");
            }
            for l in found {
                let period = l
                    .aetas
                    .map(|a| format!("; {a} Latin, not classical"))
                    .unwrap_or_default();
                println!(
                    "{w}: {} ({}{period}), {}: {}",
                    l.lemma, l.kind, l.form, l.meaning
                );
            }
        }
        return ExitCode::SUCCESS;
    }
    if let Some(cmd @ ("--liber" | "--dele")) = args.first().map(String::as_str) {
        return liber(cmd, &args[1..], &mut err);
    }
    let mut url = DEFAULT_MODEL.to_string();
    let mut max_tokens = None;
    let mut named_model = false;
    let mut rest: Vec<String> = Vec::new();
    let mut sky: Vec<String> = Vec::new();
    let mut vires = Vec::new();
    let mut it = args.into_iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--hora" | "--locus" | "--caelum" | "--virga" => {
                let Some(v) = it.next() else {
                    let _ = writeln!(err, "error: {a} needs a value\n{USAGE}");
                    return ExitCode::from(EXIT_USAGE);
                };
                sky.push(a);
                sky.push(v);
            }
            "--vires" => {
                let parsed = it
                    .next()
                    .ok_or_else(|| "--vires needs a value".to_string())
                    .and_then(|v| spiri72::agens::parse_vires(&v));
                match parsed {
                    Ok(v) => vires = v,
                    Err(e) => {
                        let _ = writeln!(err, "error: {e}\n{USAGE}");
                        return ExitCode::from(EXIT_USAGE);
                    }
                }
            }
            "--exemplar" | "--max-tokens" => {
                let Some(v) = it.next() else {
                    let _ = writeln!(err, "error: {a} needs a value\n{USAGE}");
                    return ExitCode::from(EXIT_USAGE);
                };
                if a == "--exemplar" {
                    named_model = true;
                    url = v;
                } else {
                    match v.parse::<u32>() {
                        Ok(n) if n > 0 => max_tokens = Some(n),
                        _ => {
                            let _ =
                                writeln!(err, "error: --max-tokens needs a whole number\n{USAGE}");
                            return ExitCode::from(EXIT_USAGE);
                        }
                    }
                }
            }
            "--help" | "-h" => {
                println!("{USAGE}");
                return ExitCode::SUCCESS;
            }
            _ => rest.push(a),
        }
    }
    let exemplar = match parse_url(&url) {
        Ok(e) => e,
        Err(e) => {
            let _ = writeln!(err, "error: {e}");
            return ExitCode::from(EXIT_USAGE);
        }
    };
    let mut agens = Agens::new(exemplar);
    agens.vires = vires;
    agens.status = env::current_dir().ok();
    if let Some(n) = max_tokens {
        agens.max_tokens = n;
    }
    let mut out = io::stdout().lock();
    let terminal = rest.iter().any(|a| a == "--terminal");
    let no_browser = rest.iter().any(|a| a == "--no-browser");
    rest.retain(|a| a != "--terminal" && a != "--no-browser");
    if rest.iter().all(|a| a == "--virga-probationis") {
        if spiri72::sessio::virga_of(&sky).is_some() && !rest.is_empty() {
            let _ = writeln!(
                err,
                "error: --virga and --virga-probationis exclude each other"
            );
            return ExitCode::from(EXIT_USAGE);
        }
        let dir = match env::current_dir() {
            Ok(d) => d,
            Err(e) => {
                let _ = writeln!(err, "error: cannot find this folder: {e}");
                return ExitCode::from(1);
            }
        };
        agens.own_rite = true;
        // Nothing to type: a plugged-in wand is taken up by itself, unless the test wand
        // was asked for; a model that does not answer yet is woken.
        let mut sky = sky;
        if rest.is_empty() && spiri72::sessio::virga_of(&sky).is_none() {
            match spiri72::initium::wands().as_slice() {
                [] => {}
                [one] => {
                    let _ = writeln!(out, "The wand is found at {}.", one.display());
                    sky.push("--virga".into());
                    sky.push(one.to_string_lossy().into_owned());
                }
                _ => {
                    let _ = writeln!(out, "More than one wand is plugged in: unplug the spare, or name the one you mean with --virga. Going on with the test wand.");
                }
            }
        }
        let ini = env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|d| d.join(spiri72::initium::INI)))
            .unwrap_or_else(|| dir.join(spiri72::initium::INI));
        // A model named with --exemplar is the Operator's own: only the default one is woken.
        let mut model = if named_model {
            None
        } else {
            spiri72::initium::wake_model(
                agens.exemplar.address,
                &ini,
                &mut io::stdin().lock(),
                &mut out,
            )
        };
        // the eyes wake only beside a model Spiri72 looks after itself
        let mut eyes = if named_model {
            None
        } else {
            spiri72::initium::wake_eyes(
                agens.exemplar.address,
                &ini,
                &mut io::stdin().lock(),
                &mut out,
            )
        };
        let stop = |model: &mut Option<std::process::Child>| {
            if let Some(c) = model.as_mut() {
                let _ = c.kill();
                let _ = c.wait();
            }
        };
        if terminal {
            let mut input = io::stdin().lock();
            let code = sessio(&mut input, &mut out, &mut err, &mut agens, &dir, &sky);
            stop(&mut model);
            stop(&mut eyes);
            return ExitCode::from(code);
        }
        let address = agens.exemplar.address;
        let url = agens.exemplar.url.clone();
        let code = serve(&mut agens, address, &url, &dir, &sky, &mut out, !no_browser);
        stop(&mut model);
        stop(&mut eyes);
        return ExitCode::from(code);
    }
    rest.extend(sky);
    match ritus_with(&rest, &mut out, &mut err, Some(&mut agens)) {
        Ok(code) => ExitCode::from(code),
        Err(Usage(why)) => {
            let _ = writeln!(err, "error: {why}\n{USAGE}");
            ExitCode::from(EXIT_USAGE)
        }
    }
}

fn liber(cmd: &str, args: &[String], err: &mut dyn Write) -> ExitCode {
    use spiri72::memoria::{self, Kind};
    let Some(spirit) = args.first().and_then(|a| spiri72::sessio::find(a)) else {
        let _ = writeln!(err, "error: name one of the 72 by name or number\n{USAGE}");
        return ExitCode::from(EXIT_USAGE);
    };
    let Ok(dir) = env::current_dir() else {
        let _ = writeln!(err, "error: cannot find this folder");
        return ExitCode::from(1);
    };
    if cmd == "--dele" {
        let Some(n) = args.get(1).and_then(|a| a.parse::<u64>().ok()) else {
            let _ = writeln!(err, "error: --dele needs the number of an entry\n{USAGE}");
            return ExitCode::from(EXIT_USAGE);
        };
        return match memoria::strike(&dir, spirit, n) {
            Ok(_) => {
                println!(
                    "Entry {n} of {}'s book is struck. It will not come back to the spirit.",
                    spirit.name
                );
                ExitCode::SUCCESS
            }
            Err(e) => {
                let _ = writeln!(err, "error: {e}");
                ExitCode::from(1)
            }
        };
    }
    let book = match memoria::read(&dir, spirit) {
        Ok(b) => b,
        Err(e) => {
            let _ = writeln!(err, "error: {e}");
            return ExitCode::from(1);
        }
    };
    let live: Vec<u64> = memoria::live(&book).iter().map(|e| e.n).collect();
    if book.is_empty() {
        println!("{}'s book is empty.", spirit.name);
    }
    for e in &book {
        let mark = if live.contains(&e.n) { " " } else { "x" };
        match &e.kind {
            Kind::Rogatum => println!(
                "{mark}{:>5}  asked through {}: {} [{}]",
                e.n, e.officium, e.quaestio, e.iudicium
            ),
            Kind::Receptum => println!("{mark}{:>5}  received: {}", e.n, e.verba),
            Kind::Deletum(t) => println!(" {:>5}  struck entry {t}", e.n),
        }
    }
    ExitCode::SUCCESS
}
