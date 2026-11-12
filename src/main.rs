use std::env;
use std::io::{self, Write};
use std::process::ExitCode;

use annulus::mandata::{ritus_with, Usage, EXIT_USAGE};
use spiri72::agens::Agens;
use spiri72::llama::parse_url;
use spiri72::sessio::sessio;

const DEFAULT_MODEL: &str = "http://127.0.0.1:8080";

const USAGE: &str =
    "usage: spiri72 <program.annulus> --virga-probationis [--exemplar http://127.0.0.1:8080]
               [--max-tokens N] [any other flag of annulus ritus]
       spiri72 --version";

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    let mut err = io::stderr().lock();
    if args.as_slice() == ["--version"] {
        println!("spiri72 {}", env!("CARGO_PKG_VERSION"));
        return ExitCode::SUCCESS;
    }
    let mut url = DEFAULT_MODEL.to_string();
    let mut max_tokens = None;
    let mut rest = Vec::new();
    let mut it = args.into_iter();
    while let Some(a) = it.next() {
        match a.as_str() {
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
    if let Some(n) = max_tokens {
        agens.max_tokens = n;
    }
    let mut out = io::stdout().lock();
    if rest.iter().all(|a| a == "--virga-probationis") {
        let dir = match env::current_dir() {
            Ok(d) => d,
            Err(e) => {
                let _ = writeln!(err, "error: cannot find this folder: {e}");
                return ExitCode::from(1);
            }
        };
        let mut input = io::stdin().lock();
        return ExitCode::from(sessio(&mut input, &mut out, &mut err, &mut agens, &dir));
    }
    match ritus_with(&rest, &mut out, &mut err, Some(&mut agens)) {
        Ok(code) => ExitCode::from(code),
        Err(Usage(why)) => {
            let _ = writeln!(err, "error: {why}\n{USAGE}");
            ExitCode::from(EXIT_USAGE)
        }
    }
}
