use annulus::lingua::exsequi::{Actum, Petitio, Voluntas};
use annulus::nucleus::Outcome;
use annulus::officia::{signature, Class};

use crate::forma;
use crate::llama::{Exemplar, Rogatio};
use crate::persona;

pub struct Agens {
    pub exemplar: Exemplar,
    pub max_tokens: u32,
    pub temperature: f32,
    pub vires: Vec<(String, f32)>,
}

impl Agens {
    pub fn new(exemplar: Exemplar) -> Self {
        Agens {
            exemplar,
            max_tokens: 1024,
            temperature: 0.7,
            vires: Vec::new(),
        }
    }
}

impl Voluntas for Agens {
    fn nomen(&self) -> String {
        let steered: Vec<String> = self.vires.iter().map(|(n, k)| format!("{n} {k}")).collect();
        if steered.is_empty() {
            format!("a local model at {}", self.exemplar.url)
        } else {
            format!(
                "a local model at {}, steered by {}",
                self.exemplar.url,
                steered.join(", ")
            )
        }
    }

    fn age(&mut self, p: &Petitio) -> Result<(Outcome, Actum), String> {
        let virtutes = p.officium == "VIRTUTES";
        let telling = signature(p.officium).map(|s| s.class) == Some(Class::Telling);
        let grammar = if virtutes {
            forma::grammar_virtutes(&p.aperta)?
        } else if p.officium == "PRAETERITA" {
            forma::grammar_praeterita(&p.aperta)?
        } else {
            forma::grammar(&p.aperta, telling)?
        };
        let system = persona::system(p);
        let user = persona::user(p, telling);
        let reply = self.exemplar.ask(&Rogatio {
            system: &system,
            user: &user,
            grammar: &grammar,
            max_tokens: self.max_tokens,
            temperature: self.temperature,
        })?;
        if virtutes {
            forma::read_virtutes(&reply, &p.aperta)
        } else {
            forma::read(&reply, &p.aperta, telling)
        }
    }
}

pub const VIRES: [&str; 3] = ["ligatus", "superbia", "fallax"];

pub fn parse_vires(s: &str) -> Result<Vec<(String, f32)>, String> {
    s.split(',')
        .map(|part| {
            let (name, scale) = part
                .trim()
                .split_once(':')
                .ok_or_else(|| format!("{part:?} must be name:scale, like ligatus:0.6"))?;
            if !VIRES.contains(&name) {
                return Err(format!("{name} is not one of {}", VIRES.join(", ")));
            }
            let k: f32 = scale
                .parse()
                .map_err(|_| format!("{scale:?} is not a number"))?;
            if !(-2.0..=2.0).contains(&k) {
                return Err(format!("the scale {k} is outside -2 to 2"));
            }
            Ok((name.to_string(), k))
        })
        .collect()
}
