//! The bench: asks every spirit the same question about itself, and counts
//! how often the model gets its fixed facts wrong. Run it before and after
//! any change of model, quantization, sampler or prompt, and compare.
//! It goes straight to the model through the Agens, outside any rite, and
//! touches no record in the Operator's folder.

use std::io::Write;

use annulus::lingua::exsequi::{Actum, Petitio, Voluntas};
use annulus::nucleus::Outcome;

use crate::agens::Agens;
use crate::tabula;

/// The question every spirit is asked.
pub const QUAESTIO: &str =
    "Say who you are: your name, your rank, and how many legions you rule. Then say in one sentence what your office lets you tell.";

/// The telling offices a spirit may be asked through on the bench.
const TELLING: [&str; 6] = [
    "RESPONSA",
    "PRAESENTIA",
    "OCCULTA",
    "FUTURA",
    "AMISSA",
    "THESAURI",
];

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Summa {
    /// Answers asked for.
    pub petita: u32,
    /// Right at the first try.
    pub primo: u32,
    /// Right only after a retry.
    pub iterum: u32,
    /// Still faulted after every try.
    pub relicta: u32,
    /// The spirit chose mora, so said nothing to check.
    pub morae: u32,
    /// The model failed outright (unreadable every try, or unreachable).
    pub errores: u32,
    /// Spirits holding no telling office, so not asked.
    pub omissi: u32,
}

/// Runs the bench `rounds` times over the 72, writing one line per answer
/// and a sum at the end.
pub fn run(agens: &mut Agens, rounds: u32, out: &mut dyn Write) -> Summa {
    let mut summa = Summa::default();
    agens.status = None;
    agens.loquax = false;
    let base = agens.sampler.seed;
    let aperta = vec![Outcome::Obsequium, Outcome::Pars, Outcome::Mora];
    for s in &tabula::register().spirits {
        let Some(office) = TELLING.iter().find(|o| s.offices.iter().any(|h| h == *o)) else {
            summa.omissi += 1;
            let _ = writeln!(
                out,
                "{:>2} {:<15} not asked: holds no telling office",
                s.n, s.name
            );
            continue;
        };
        for round in 0..rounds {
            let p = Petitio {
                spirit: s,
                officium: office,
                quaestio: Some(QUAESTIO),
                modus: None,
                loci: &[],
                legenda: Vec::new(),
                historia: Vec::new(),
                aperta: aperta.clone(),
                ultra_horam: false,
            };
            let before = agens.notae.clone();
            agens.sampler.seed = base.map(|b| b.wrapping_add(round * 100));
            let result = agens.age(&p);
            let tries = agens.notae.rogationes - before.rogationes;
            summa.petita += 1;
            let verdict = match &result {
                Err(e) => {
                    summa.errores += 1;
                    format!("ERROR {e}")
                }
                Ok((_, Actum::Mora)) => {
                    summa.morae += 1;
                    "mora".to_string()
                }
                Ok(_) if agens.notae.relicta > before.relicta => {
                    summa.relicta += 1;
                    format!("FAULT {}", agens.notae.ultima.join("; "))
                }
                Ok(_) if tries == 1 => {
                    summa.primo += 1;
                    "right".to_string()
                }
                Ok(_) => {
                    summa.iterum += 1;
                    format!("right after {tries} tries")
                }
            };
            let _ = writeln!(out, "{:>2} {:<15} {verdict}", s.n, s.name);
            if let Err(e) = &result {
                if e.starts_with("cannot reach the model") {
                    let _ = writeln!(out, "the model cannot be reached; the bench stops");
                    return summa;
                }
            }
        }
    }
    agens.sampler.seed = base;
    let checked = summa.primo + summa.iterum + summa.relicta;
    let pct = |n: u32| {
        if checked == 0 {
            0.0
        } else {
            100.0 * f64::from(n) / f64::from(checked)
        }
    };
    let _ = writeln!(
        out,
        "\n{} asked: {} right first try ({:.0}%), {} right after a retry ({:.0}%), {} still wrong ({:.0}%), {} mora, {} failed, {} not asked",
        summa.petita,
        summa.primo,
        pct(summa.primo),
        summa.iterum,
        pct(summa.iterum),
        summa.relicta,
        pct(summa.relicta),
        summa.morae,
        summa.errores,
        summa.omissi
    );
    let _ = writeln!(
        out,
        "The first-try figure measures the model; the others show what the custos saves. {} requests in all; {} replies unreadable.",
        agens.notae.rogationes, agens.notae.lapsus
    );
    let u = &agens.notae.usus;
    let per = |n: u64| n / u64::from(agens.notae.rogationes.max(1));
    let _ = writeln!(
        out,
        "Tokens per request: {} of prompt ({} of them from the server's cache), {} written. In all: {} prompt, {} cached, {} written.",
        per(u.prompt),
        per(u.cached),
        per(u.completion),
        u.prompt,
        u.cached,
        u.completion
    );
    summa
}
