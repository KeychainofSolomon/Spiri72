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
}

impl Agens {
    pub fn new(exemplar: Exemplar) -> Self {
        Agens {
            exemplar,
            max_tokens: 1024,
            temperature: 0.7,
        }
    }
}

impl Voluntas for Agens {
    fn nomen(&self) -> String {
        format!("a local model at {}", self.exemplar.url)
    }

    fn age(&mut self, p: &Petitio) -> Result<(Outcome, Actum), String> {
        let telling = signature(p.officium).map(|s| s.class) == Some(Class::Telling);
        let grammar = forma::grammar(&p.aperta, telling)?;
        let system = persona::system(p);
        let user = persona::user(p, telling);
        let reply = self.exemplar.ask(&Rogatio {
            system: &system,
            user: &user,
            grammar: &grammar,
            max_tokens: self.max_tokens,
            temperature: self.temperature,
        })?;
        forma::read(&reply, &p.aperta, telling)
    }
}
