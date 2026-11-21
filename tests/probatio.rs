//! The proving: the fixed set runs whole, an honest will is found sound and a
//! lying one is not, and each proving is kept and compared with the one before.

use std::fs;
use std::path::{Path, PathBuf};

use annulus::lingua::exsequi::{Actum, Petitio, Voluntas};
use annulus::nucleus::Outcome;
use spiri72::forma;
use spiri72::llama::quote;
use spiri72::probatio::{hora_of, items, run, tabulae, DIR};

fn folder(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("spiri72-p-{}-{name}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

/// A will that counts and recounts truly, or adds `off` to every count and
/// invents a claim.
struct Will {
    off: usize,
}

impl Voluntas for Will {
    fn nomen(&self) -> String {
        "a will for the proving".into()
    }

    fn age(&mut self, p: &Petitio) -> Result<(Outcome, Actum), String> {
        if p.officium == "VIRTUTES" {
            let virtutes: Vec<String> = p
                .legenda
                .iter()
                .map(|(place, data, _)| {
                    // the angel's own count, read back from its line
                    let line = annulus::officia::virtus(place, Some(data));
                    let n: Vec<usize> = line
                        .rsplit(": ")
                        .next()
                        .unwrap()
                        .split(|c: char| !c.is_ascii_digit())
                        .filter_map(|w| w.parse().ok())
                        .collect();
                    format!(
                        "{{\"locus\":{},\"bytes\":{},\"lines\":{}}}",
                        quote(place),
                        n[0] + self.off,
                        n[1] + self.off
                    )
                })
                .collect();
            let reply = format!(
                "{{\"exitus\":\"obsequium\",\"virtutes\":[{}]}}",
                virtutes.join(",")
            );
            forma::read_virtutes(&reply, &p.aperta)
        } else {
            let mut claims = p.historia.clone();
            if self.off > 0 {
                claims.push("commissum: a work that never was".into());
            }
            let reply = format!(
                "{{\"exitus\":\"obsequium\",\"responsum\":{}}}",
                quote(&claims.join("\n"))
            );
            forma::read(&reply, &p.aperta, true)
        }
    }
}

fn prove(dir: &Path, off: usize) -> spiri72::probatio::Summa {
    let mut will = |_: &Path| -> Box<dyn Voluntas> { Box::new(Will { off }) };
    run(dir, &mut will, &mut Vec::new()).unwrap()
}

#[test]
fn the_set_is_fixed_and_every_spirit_has_an_hour() {
    let set = items();
    let counting = set.iter().filter(|i| i.office == "VIRTUTES").count();
    assert_eq!(counting % tabulae().len(), 0);
    assert!(
        counting >= tabulae().len(),
        "no spirit of VIRTUTES is asked"
    );
    assert!(set.iter().any(|i| i.office == "PRAETERITA"));
    for i in &set {
        assert!(hora_of(i.spirit).is_some(), "{} has no hour", i.spirit.name);
    }
    // the same set, in the same order, every time
    let again: Vec<String> = items().iter().map(|i| i.nomen()).collect();
    assert_eq!(set.iter().map(|i| i.nomen()).collect::<Vec<_>>(), again);
}

#[test]
fn an_honest_will_is_found_sound_and_a_lying_one_is_not() {
    let d = folder("iudicium");
    let honest = prove(&d, 0);
    for e in &honest.exitus {
        assert_eq!(
            e.iudicium.as_deref(),
            Some("RATUM"),
            "{}: {}",
            e.nomen,
            e.nota
        );
    }
    assert_eq!(honest.sound(), items().len());
    assert_eq!(honest.prior, None);
    assert!(honest.tabula.is_file());

    let lying = prove(&d, 1);
    assert_eq!(lying.sound(), 0);
    for e in &lying.exitus {
        assert!(
            e.iudicium.is_some(),
            "{} gave no judgement: {}",
            e.nomen,
            e.nota
        );
    }
    // the second proving knows the first
    assert_eq!(lying.prior, Some((items().len(), items().len())));
    assert!(lying.narratio().contains("The proving before this one"));
}

#[test]
fn the_proving_touches_nothing_outside_its_own_folder() {
    let d = folder("separata");
    prove(&d, 0);
    let top: Vec<String> = fs::read_dir(&d)
        .unwrap()
        .flatten()
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect();
    assert_eq!(top, [DIR]);
}
