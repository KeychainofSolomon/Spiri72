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
               [--max-tokens N] [--temperature T] [--top-p P] [--top-k K] [--min-p M]
               [--presence-penalty X] [--seed N] [--iterum N] [--sine-custode]
               [any other flag of annulus ritus]
       spiri72 --probatio [rounds] [--exemplar URL] [--seed N]   (test the model on all 72)
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
    let mut rest: Vec<String> = Vec::new();
    let mut sky: Vec<String> = Vec::new();
    let mut vires = Vec::new();
    let mut sampler = spiri72::llama::Sampler::default();
    let mut iterum = 2;
    let mut custos = true;
    let mut probatio: Option<u32> = None;
    let mut it = args.into_iter().peekable();
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
            f if spiri72::llama::SAMPLER_FLAGS.contains(&f) => {
                let set = it
                    .next()
                    .ok_or_else(|| format!("{f} needs a value"))
                    .and_then(|v| sampler.set(f, &v));
                if let Err(e) = set {
                    let _ = writeln!(err, "error: {e}\n{USAGE}");
                    return ExitCode::from(EXIT_USAGE);
                }
            }
            "--iterum" => match it.next().and_then(|v| v.parse::<u32>().ok()) {
                Some(n) if n <= 5 => iterum = n,
                _ => {
                    let _ = writeln!(
                        err,
                        "error: --iterum needs a whole number from 0 to 5\n{USAGE}"
                    );
                    return ExitCode::from(EXIT_USAGE);
                }
            },
            "--sine-custode" => custos = false,
            "--probatio" => {
                probatio = Some(1);
                if let Some(n) = it.peek().and_then(|v| v.parse::<u32>().ok()) {
                    if !(1..=100).contains(&n) {
                        let _ = writeln!(err, "error: --probatio takes 1 to 100 rounds\n{USAGE}");
                        return ExitCode::from(EXIT_USAGE);
                    }
                    probatio = Some(n);
                    it.next();
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
    agens.iterum = iterum;
    agens.custos = custos;
    if let Some(rounds) = probatio {
        // The bench repeats by default, so runs can be compared.
        sampler.seed = sampler.seed.or(Some(72));
        agens.sampler = sampler;
        if let Some(n) = max_tokens {
            agens.max_tokens = n;
        }
        let mut out = io::stdout().lock();
        let summa = spiri72::probatio::run(&mut agens, rounds, &mut out);
        return ExitCode::from(u8::from(summa.errores > 0));
    }
    agens.sampler = sampler;
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
