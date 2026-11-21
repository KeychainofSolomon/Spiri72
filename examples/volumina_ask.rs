// usage: cargo run --example volumina_ask -- <signa> <office words> <book.txt:gate words> ... < questions
// Asks whole-book search the questions on stdin, one per line, over books given as
// pdftotext output (pages parted by form feeds). V=1 prints the start of each passage.
// For measuring only: the program itself reads PDFs with hayro.
use spiri72::volumina::*;
use std::io::BufRead;
fn main() {
    let a: Vec<String> = std::env::args().skip(1).collect();
    let signa: usize = a[0].parse().unwrap();
    let office = vec![a[1].clone()];
    let mut vols = Vec::new();
    for spec in &a[2..] {
        let (path, claves) = spec.split_once(':').unwrap();
        let text = std::fs::read_to_string(path).unwrap();
        let pages: Vec<Option<String>> = text
            .split('\u{c}')
            .map(|p| {
                (p.chars().filter(|c| c.is_alphanumeric()).count() >= 16).then(|| p.to_string())
            })
            .collect();
        let t = titulus(&format!(
            "# nomen: {path}\n# claves: {claves}\n# aetas: modern\n"
        ));
        let common = spiri72::concordantia::Concordantia::parse(
            spiri72::concordantia::SEMEN,
            &["name", "named", "names", "call", "called", "use", "used"],
        );
        let v = bind_with(path, &t, &pages, true, signa, &common);
        eprintln!(
            "{path}: {} pages, {} pictae, {} passages, {} tables set aside, concordance {}",
            v.paginae,
            v.pictae,
            v.passus(),
            v.tabulae,
            v.concordantia.len()
        );
        vols.push(v);
    }
    let verbose = std::env::var("V").is_ok();
    for q in std::io::stdin().lock().lines().map(Result::unwrap) {
        let got = consult_as(&vols, &office, &q, signa);
        let where_: Vec<String> = got
            .iter()
            .map(|l| format!("{} {}", l.liber.rsplit('/').next().unwrap(), l.locus))
            .collect();
        println!(
            "Q: {q}\n   -> {}",
            if where_.is_empty() {
                "nothing".into()
            } else {
                where_.join(" | ")
            }
        );
        if verbose {
            for l in &got {
                println!(
                    "      [{}] {}",
                    l.locus,
                    l.textus
                        .replace('\n', " ")
                        .chars()
                        .take(160)
                        .collect::<String>()
                );
            }
        }
    }
}
