use annulus::lingua::exsequi::{Actum, Petitio, Voluntas};
use annulus::nucleus::Outcome;
use annulus::officia::{signature, Class};

use std::path::PathBuf;

use crate::custos;
use crate::forma;
use crate::fors;
use crate::ingenium::{self, Status};
use crate::llama::{Exemplar, Rogatio, Sampler};
use crate::memoria;
use crate::persona;

/// What the custos and the retries saw, counted over the life of an Agens.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Notae {
    /// Every request sent to the model, retries included.
    pub rogationes: u32,
    /// Replies that could not be read (cut short, or not the form asked).
    pub lapsus: u32,
    /// Replies the custos faulted.
    pub vitia: u32,
    /// Answers given on with a fault still in them, after every try.
    pub relicta: u32,
    /// The faults in the last answer given on, if any.
    pub ultima: Vec<String>,
    /// The tokens spent, as the server counts them.
    pub usus: crate::llama::Usus,
}

pub struct Agens {
    pub exemplar: Exemplar,
    pub max_tokens: u32,
    pub sampler: Sampler,
    /// How many more times a faulted or unreadable reply is asked again.
    pub iterum: u32,
    /// Whether the custos checks what the spirit says of itself.
    pub custos: bool,
    /// Whether an answer given on with a fault is told on stderr.
    pub loquax: bool,
    pub notae: Notae,
    pub vires: Vec<(String, f32)>,
    /// Where the record of each spirit's calls and judgements is kept, the
    /// Operator's folder. Without it nothing is counted.
    pub status: Option<PathBuf>,
    /// The rites are Spiri72's own, where every COACTUS condition is met.
    pub own_rite: bool,
}

impl Agens {
    pub fn new(exemplar: Exemplar) -> Self {
        Agens {
            exemplar,
            max_tokens: 1024,
            sampler: Sampler::default(),
            iterum: 2,
            custos: true,
            loquax: true,
            notae: Notae::default(),
            vires: Vec::new(),
            status: None,
            own_rite: false,
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
        } else if p.officium == "DOCTRINA" {
            forma::grammar_doctrina(&p.aperta, p.quaestio.unwrap_or(""))?
        } else if telling {
            forma::grammar(&p.aperta, true)?
        } else {
            forma::grammar_working(&p.aperta, p.officium, p.modus, p.loci)?
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
        let user = persona::user_remembering(p, telling, &fors::hex::<8>()?, memoria.as_deref());
        // Only plain speech is the custos's to read: the past and the
        // measures are checked by the angel, and works are not speech.
        let speech = telling && !virtutes && p.officium != "PRAETERITA";
        let tries = 1 + self.iterum;
        let first = self.notae.rogationes;
        let mut faulted: Option<((Outcome, Actum), Vec<String>)> = None;
        let mut unread: Option<String> = None;
        for attempt in 0..tries {
            let mut sampler = self.sampler.clone();
            // A fixed seed still differs from try to try, or every try
            // would give the same words.
            sampler.seed = sampler.seed.map(|s| s.wrapping_add(attempt));
            self.notae.rogationes += 1;
            let asked = self.exemplar.ask_counted(&Rogatio {
                system: &system,
                user: &user,
                grammar: &grammar,
                max_tokens: self.max_tokens,
                sampler: &sampler,
            });
            let reply = match asked {
                Ok(r) => {
                    self.notae.usus.add(&r);
                    r.content
                }
                Err(e) if attempt == 0 => return Err(e),
                // A retry that cannot reach the model ends the tries; what
                // was wrong with the earlier reply is what is told.
                Err(_) => break,
            };
            let read = if virtutes {
                forma::read_virtutes(&reply, &p.aperta)
            } else if p.officium == "DOCTRINA" {
                forma::read_doctrina(&reply, &p.aperta, p.quaestio.unwrap_or(""))
            } else {
                forma::read(&reply, &p.aperta, telling)
            };
            let answer = match read {
                Ok(a) => a,
                Err(e) => {
                    self.notae.lapsus += 1;
                    unread = Some(e);
                    continue;
                }
            };
            let faults = match (&answer.1, speech && self.custos) {
                (Actum::Responsum(text), true) => custos::examine(p.spirit, text),
                _ => Vec::new(),
            };
            if faults.is_empty() {
                self.notae.ultima.clear();
                return Ok(answer);
            }
            self.notae.vitia += 1;
            faulted = Some((answer, faults));
        }
        match (faulted, unread) {
            // Every try was faulted: the last is given on as it is, for the
            // spirit's words are never changed or withheld.
            (Some((answer, faults)), _) => {
                self.notae.relicta += 1;
                if self.loquax {
                    eprintln!(
                        "custos: {} {} (after {} tries; given on unchanged)",
                        p.spirit.name,
                        faults.join("; "),
                        self.notae.rogationes - first
                    );
                }
                self.notae.ultima = faults;
                Ok(answer)
            }
            (None, Some(e)) => Err(format!(
                "{e} (after {} tries)",
                self.notae.rogationes - first
            )),
            (None, None) => Err("the model gave nothing".into()),
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
