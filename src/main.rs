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
    "usage: spiri72 <program.annulus> --virga-probationis [--exemplar http://127.0.0.1:8080]
               [--max-tokens N] [any other flag of annulus ritus]
       spiri72 --latina <Latin words>
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
                println!("{w}: not in the classical lexicon");
            }
            for l in found {
                println!("{w}: {} ({}), {}: {}", l.lemma, l.kind, l.form, l.meaning);
            }
        }
        return ExitCode::SUCCESS;
    }
    let mut url = DEFAULT_MODEL.to_string();
    let mut max_tokens = None;
    let mut rest: Vec<String> = Vec::new();
    let mut sky: Vec<String> = Vec::new();
    let mut vires = Vec::new();
    let mut it = args.into_iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--hora" | "--locus" | "--caelum" => {
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
        let dir = match env::current_dir() {
            Ok(d) => d,
            Err(e) => {
                let _ = writeln!(err, "error: cannot find this folder: {e}");
                return ExitCode::from(1);
            }
        };
        agens.own_rite = true;
        if terminal {
            let mut input = io::stdin().lock();
            return ExitCode::from(sessio(
                &mut input, &mut out, &mut err, &mut agens, &dir, &sky,
            ));
        }
        let address = agens.exemplar.address;
        let url = agens.exemplar.url.clone();
        return ExitCode::from(serve(
            &mut agens,
            address,
            &url,
            &dir,
            &sky,
            &mut out,
            !no_browser,
        ));
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
