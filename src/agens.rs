use annulus::lingua::exsequi::{Actum, Petitio, Voluntas};
use annulus::nucleus::Outcome;
use annulus::officia::{signature, Class};

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crate::forma;
use crate::fors;
use crate::ingenium::{self, Status};
use crate::llama::{Exemplar, Rogatio};
use crate::memoria;
use crate::persona;
use crate::scientia::{self, Consultatio, Liber};

pub struct Agens {
    pub exemplar: Exemplar,
    pub max_tokens: u32,
    pub temperature: f32,
    pub vires: Vec<(String, f32)>,
    /// Where the record of each spirit's calls and judgements is kept, the
    /// Operator's folder. Without it nothing is counted.
    pub status: Option<PathBuf>,
    /// The rites are Spiri72's own, where every COACTUS condition is met.
    pub own_rite: bool,
    /// The knowledge libraries a spirit may consult, loaded once at the start.
    pub scientia: Vec<Liber>,
    /// Where the last library reading is set down for the page to show, if the
    /// page is watching. Cleared each speaking rite; set when something is read.
    pub consulta: Option<Arc<Mutex<Option<Consultatio>>>>,
}

impl Agens {
    pub fn new(exemplar: Exemplar) -> Self {
        Agens {
            exemplar,
            max_tokens: 1024,
            temperature: 0.7,
            vires: Vec::new(),
            status: None,
            own_rite: false,
            scientia: Vec::new(),
            consulta: None,
        }
    }
}

impl Voluntas for Agens {
    fn nomen(&self) -> String {
        let steered: Vec<String> = self.vires.iter().map(|(n, k)| format!("{n} {k}")).collect();
        let mut n = format!("a local model at {}", self.exemplar.url);
        if !steered.is_empty() {
            n.push_str(&format!(", steered by {}", steered.join(", ")));
        }
        n.push_str(", each spirit's bearing drawn for each rite from its 72Spirits profile");
        n
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
        let profile = ingenium::profile(p.spirit.n)
            .ok_or_else(|| format!("no profile for spirit {}", p.spirit.n))?;
        let before = match &self.status {
            Some(dir) => ingenium::read_status(dir, p.spirit)?,
            None => Status::default(),
        };
        let gestus = ingenium::draw(profile, p.spirit, self.own_rite, &before, &mut fors::unit)?;
        if let Some(dir) = &self.status {
            ingenium::record_call(dir, p.spirit, &gestus)?;
        }
        let mut brief = ingenium::brief(profile, &gestus, &before);
        if p.ultra_horam {
            brief.push_str(RESTLESS);
        }
        let system = persona::system(p, &brief);
        let memoria = match &self.status {
            Some(dir) => memoria::for_rite(dir, p.spirit, p.quaestio.unwrap_or(""))?,
            None => None,
        };
        // A speaking office (not the counting or the recounting of the past) may
        // draw on a knowledge library the spirit's inclination opens to it.
        let speaking = telling && !virtutes && p.officium != "PRAETERITA";
        let consulta = speaking
            .then(|| scientia::consult(&self.scientia, p.spirit.n, p.quaestio.unwrap_or("")))
            .flatten();
        if let Some(slot) = &self.consulta {
            *slot.lock().unwrap() = consulta.clone();
        }
        let heading = consulta
            .as_ref()
            .map(|c| format!("the {} library, on \u{201c}{}\u{201d}", c.liber, c.terminus));
        let scientia_arg = match (&heading, &consulta) {
            (Some(h), Some(c)) => Some((h.as_str(), c.textus.as_str())),
            _ => None,
        };
        let user = persona::user_remembering(
            p,
            telling,
            &fors::hex::<8>()?,
            memoria.as_deref(),
            scientia_arg,
        );
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

/// Told to a spirit held past its hours: still bound, and chafing at it.
pub const RESTLESS: &str = " Your hour is past, yet you are still held in the triangle: the magician has not given you licence to depart. You chafe at the bond, and press to be let go.";

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
