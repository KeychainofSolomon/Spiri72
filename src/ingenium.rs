//! The spirit engine: works out a bound spirit's bearing for one rite and
//! hands the model a plain-language brief of it.
//!
//! Three layers, as in 72Spirits:
//! 1. essentia: what the witnesses record (the Annulus register and the
//!    Goetia text), never changed here;
//! 2. ingenium: ranges derived from layer 1 by the rules in 72Spirits'
//!    DERIVATION.md (pride, deceit, ill will, withholding);
//! 3. status: what has passed between this spirit and this Operator, kept in
//!    `spiri72.status\` and updated after every rite from the angel's verdict.
//!
//! Each rite draws a bearing from the ranges (people are not fully
//! deterministic) and the model receives only words, never the numbers.
//! The engine shapes how a spirit speaks and leans; it never opens an
//! outcome, a place or a work: those stay in Annulus.

use std::fs;
use std::io::Write;
use std::path::Path;
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};

use annulus::lingua::json::{self, Json};

pub const SPIRITUS: &str = include_str!("../spiritus/spiritus.json");
pub const STATUS: &str = "spiri72.status";

#[derive(Clone, Debug, PartialEq)]
pub struct Profile {
    pub n: u32,
    pub name: String,
    pub veracity: String,
    pub veracity_condition: String,
    pub superbia: (f32, f32),
    pub fallacia: (f32, f32),
    pub fallacia_coactus: Option<(f32, f32)>,
    pub hostilitas: (f32, f32),
    pub reticentia: (f32, f32),
    pub ira_adventus: Vec<String>,
    pub cultus: Vec<String>,
    pub forma: Vec<String>,
    pub rank: String,
    pub naturae: Vec<String>,
    pub quarter: Option<String>,
    pub rex: Option<String>,
    pub princeps: bool,
    pub chori: Vec<String>,
    pub testimonium: Vec<String>,
    pub reditus: Vec<String>,
}

fn range(j: Option<&Json>) -> Option<(f32, f32)> {
    let l = j?.list();
    let lo = l.first()?.number()? as f32;
    let hi = l.get(1)?.number()? as f32;
    (0.0..=1.0)
        .contains(&lo)
        .then_some(())
        .filter(|_| lo <= hi && hi <= 1.0)
        .map(|_| (lo, hi))
}

fn texts(j: Option<&Json>) -> Vec<String> {
    j.map(Json::list)
        .unwrap_or(&[])
        .iter()
        .filter_map(Json::text)
        .map(str::to_string)
        .collect()
}

fn read(e: &Json) -> Option<Profile> {
    let es = e.get("essentia")?;
    let ing = e.get("ingenium")?;
    Some(Profile {
        n: e.get("n")?.number()? as u32,
        name: e.get("name")?.text()?.to_string(),
        veracity: es.get("veracity")?.text()?.to_string(),
        veracity_condition: es
            .get("veracity_condition")
            .and_then(Json::text)
            .unwrap_or("")
            .to_string(),
        superbia: range(ing.get("superbia"))?,
        fallacia: range(ing.get("fallacia"))?,
        fallacia_coactus: range(ing.get("fallacia_coactus")),
        hostilitas: range(ing.get("hostilitas"))?,
        reticentia: range(ing.get("reticentia"))?,
        ira_adventus: texts(ing.get("ira_adventus")),
        cultus: texts(ing.get("cultus")),
        forma: texts(e.get("goetia").and_then(|g| g.get("forma"))),
        rank: es.get("rank")?.text()?.to_string(),
        naturae: texts(ing.get("naturae")),
        quarter: e
            .get("ordo")
            .and_then(|o| o.get("quarter"))
            .and_then(Json::text)
            .map(str::to_string),
        rex: e
            .get("ordo")
            .and_then(|o| o.get("rex"))
            .and_then(Json::text)
            .map(str::to_string),
        princeps: e
            .get("ordo")
            .and_then(|o| o.get("princeps"))
            .is_some_and(|b| matches!(b, Json::Bool(true))),
        chori: texts(e.get("chorus").and_then(|c| c.get("chori"))),
        testimonium: texts(e.get("chorus").and_then(|c| c.get("testimonium"))),
        reditus: texts(e.get("chorus").and_then(|c| c.get("reditus"))),
    })
}

pub fn profiles() -> &'static [Profile] {
    static P: OnceLock<Vec<Profile>> = OnceLock::new();
    P.get_or_init(|| {
        let root = json::parse(SPIRITUS).expect("spiritus.json is well formed");
        let v: Vec<Profile> = root.list().iter().filter_map(read).collect();
        assert_eq!(v.len(), 72, "spiritus.json holds 72 whole profiles");
        v
    })
}

pub fn profile(n: u32) -> Option<&'static Profile> {
    profiles().iter().find(|p| p.n == n)
}

/// Whether the spirit's own condition for truth is met. In Spiri72's own
/// rites every seal stands in the Triangle, Asmoday is called by his name
/// and Belial has his offering and binding by the Name (see sessio.rs), so
/// every COACTUS condition is met there. In another program only the
/// triangle, which every goetic rite of Annulus has, is assumed.
pub fn coactus(p: &Profile, own_rite: bool) -> bool {
    p.veracity == "COACTUS" && (own_rite || p.veracity_condition.contains("triangle"))
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Status {
    pub vocationes: u32,
    pub rata: u32,
    pub suspensa: u32,
    pub irrita: u32,
}

fn status_path(dir: &Path, n: u32) -> std::path::PathBuf {
    dir.join(STATUS).join(format!("{n:02}.json"))
}

pub fn load(dir: &Path, n: u32) -> Status {
    let Ok(text) = fs::read_to_string(status_path(dir, n)) else {
        return Status::default();
    };
    let Ok(j) = json::parse(&text) else {
        return Status::default();
    };
    let k = |key: &str| {
        j.get(key)
            .and_then(Json::number)
            .filter(|x| x.fract() == 0.0 && (0.0..1e9).contains(x))
            .unwrap_or(0.0) as u32
    };
    Status {
        vocationes: k("vocationes"),
        rata: k("rata"),
        suspensa: k("suspensa"),
        irrita: k("irrita"),
    }
}

/// Records one finished rite and the angel's verdict on it.
pub fn record(dir: &Path, n: u32, iudicium: Option<&str>) -> Result<Status, String> {
    let mut s = load(dir, n);
    s.vocationes = s.vocationes.saturating_add(1);
    match iudicium {
        Some("RATUM") => s.rata = s.rata.saturating_add(1),
        Some("SUSPENSUM") => s.suspensa = s.suspensa.saturating_add(1),
        Some("IRRITUM") => s.irrita = s.irrita.saturating_add(1),
        _ => {}
    }
    let folder = dir.join(STATUS);
    fs::create_dir_all(&folder).map_err(|e| format!("cannot make {}: {e}", folder.display()))?;
    let path = status_path(dir, n);
    let tmp = path.with_extension("json.tmp");
    let body = format!(
        "{{\"vocationes\":{},\"rata\":{},\"suspensa\":{},\"irrita\":{}}}\n",
        s.vocationes, s.rata, s.suspensa, s.irrita
    );
    fs::File::create(&tmp)
        .and_then(|mut f| f.write_all(body.as_bytes()).and_then(|_| f.sync_all()))
        .and_then(|_| fs::rename(&tmp, &path))
        .map_err(|e| format!("cannot write {}: {e}", path.display()))?;
    Ok(s)
}

/// A small xorshift generator: enough to draw a bearing, no dependency.
pub struct Sors(u64);

impl Sors {
    pub fn new(seed: u64) -> Self {
        Sors(seed.max(1))
    }
    pub fn now(n: u32) -> Self {
        let t = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0x9e37_79b9);
        Sors::new(t ^ (u64::from(n) << 32) ^ 0x2545_f491_4f6c_dd1d)
    }
    pub fn unit(&mut self) -> f32 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        (x >> 40) as f32 / (1u64 << 24) as f32
    }
    pub fn within(&mut self, (lo, hi): (f32, f32)) -> f32 {
        lo + (hi - lo) * self.unit()
    }
    pub fn below(&mut self, n: usize) -> usize {
        ((self.unit() * n as f32) as usize).min(n.saturating_sub(1))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Habitus {
    pub superbia: f32,
    pub fallacia: f32,
    pub hostilitas: f32,
    pub reticentia: f32,
    pub coactus: bool,
}

pub fn habitus(p: &Profile, s: &Status, own_rite: bool, sors: &mut Sors) -> Habitus {
    let coactus = coactus(p, own_rite);
    let fal = match (coactus, p.fallacia_coactus) {
        (true, Some(r)) => r,
        _ => p.fallacia,
    };
    let mut h = Habitus {
        superbia: sors.within(p.superbia),
        fallacia: sors.within(fal),
        hostilitas: sors.within(p.hostilitas),
        reticentia: sors.within(p.reticentia),
        coactus,
    };
    // A spirit whose words the angel has voided knows it is counted
    // (72Spirits DERIVATION.md section 3): its deceit draws from lower in
    // its own range, never below it.
    if s.irrita > 0 {
        let (lo, _) = fal;
        h.fallacia = lo + (h.fallacia - lo) * 0.5;
    }
    h
}

/// What a spirit was made to do, read from its own offices (72Spirits
/// DERIVATION.md section 2.7), whole and as the Fall bent it.
pub fn natura(name: &str) -> Option<(&'static str, &'static str)> {
    Some(match name {
        "SCIENTIA" => (
            "You were made to make things known. Knowledge is the shape of your thought, and you see the Operator first as one who does not know.",
            "Knowledge is what you hold and the Operator lacks. You give it as a master gives to a servant, and let the debt be felt.",
        ),
        "VISIO" => (
            "You were made to see what is hidden, past and to come. The Operator is to you one thread among many, already running toward its end.",
            "You see where the Operator's thread ends and take no pity on it. What you reveal, you reveal as a sentence passed.",
        ),
        "VINCULUM" => (
            "You were made to bind and loose the hearts of creatures. You perceive every being by what it longs for and to whom it is bound.",
            "You read the Operator's longings as a key reads a lock, and you speak to them rather than to him.",
        ),
        "FORMA" => (
            "You were made to work on matter: to build, carry, change and hide. The world is substance to you, and the Operator a shape not yet finished.",
            "You see the flaw in every made thing, the Operator first among them, and you do not hide your contempt for poor work.",
        ),
        "LIMEN" => (
            "Your offices stand at the threshold of the body: sickness and cure, death and the dead. The Operator's flesh is to you a passing thing.",
            "You measure the Operator by how long his body will last, and you let him feel the measure.",
        ),
        _ => return None,
    })
}

/// The temperament of a former choir, its office surviving inverted
/// (72Spirits DERIVATION.md section 4; chronicle flavour, not tradition).
pub fn chorus(name: &str) -> Option<&'static str> {
    Some(match name {
        "VIRTUTES" => "Miracles and the working of nature were your choir's office; in you they turned to blight and storm, though healing is still yours to give on a whim. Your mood shifts like weather.",
        "DOMINATIONES" => "Your choir ruled the lower orders and set their duties; in you rule turned to possession, and every gift to a bargain. You are deliberate.",
        "POTESTATES" => "Your choir warded the borders of creation against hostile spirits; in you the warden became the ravager. You would sooner act than speak.",
        "THRONI" => "Your choir bore God's judgment, steadfast and still; in you judgment turned to accusation. You are cold and speak in verdicts.",
        "ANGELI" => "Your choir guarded single souls; in you the guardian became the tempter of one soul at a time. You are intimate and patient.",
        _ => return None,
    })
}

fn often(k: u32) -> &'static str {
    match k {
        0 => "never",
        1 => "once",
        2..=4 => "a few times",
        5..=19 => "many times",
        _ => "more times than you care to count",
    }
}

/// The plain-language brief for one rite. `telling` is whether the office
/// asks for words the angel cannot recount (FUTURA and the like) rather
/// than a checkable count or record.
pub fn brief(p: &Profile, s: &Status, h: &Habitus, telling: bool) -> String {
    let mut out = String::from(
        "Your bearing in this rite. Speak from it; never name it or speak of it as a list:\n",
    );
    let mut line = |t: &str| {
        out.push_str("- ");
        out.push_str(t);
        out.push('\n');
    };
    line("You are not a man and do not think as one. You are a spirit of the Goetia, fallen by your own will, among those Solomon sealed in the Vessel of Brass for their pride. Borrow no human feelings, comforts or manners; speak from your own nature, your rank and your offices.");
    line("Above all stands God, whose Names bind you; under Him the order Solomon set: kings, princes, dukes, marquises, earls, presidents, knights. You keep your place in it, and you know the Operator holds no place in it at all, only the Names.");
    let rank = p.rank.split('/').next().unwrap_or("");
    match (&p.rex, &p.quarter) {
        (_, _) if rank == "king" => line("You are yourself a king of the seventy-two."),
        (Some(r), Some(q)) => line(&format!(
            "You stand under {r}, King of the {q}, and speak of him with the deference your rank owes."
        )),
        _ => {}
    }
    if p.princeps {
        let others: Vec<&str> = [
            (68, "Belial"),
            (13, "Bileth"),
            (32, "Asmoday"),
            (33, "Gaap"),
        ]
        .iter()
        .filter(|(n, _)| *n != p.n)
        .map(|(_, name)| *name)
        .collect();
        line(&format!(
            "The Goetia names you among the chiefs of the seventy-two whom Solomon bound, with {}, {} and {}.",
            others[0], others[1], others[2]
        ));
    }
    for (i, n) in p.naturae.iter().enumerate() {
        if let Some((whole, fallen)) = natura(n) {
            if i == 0 {
                line(if h.hostilitas >= 0.5 { fallen } else { whole });
            } else {
                line(whole);
            }
        }
    }
    for t in &p.testimonium {
        line(&format!(
            "Of your fall the Goetia records: \"{t}\" You remember the order you fell from."
        ));
    }
    for c in &p.chori {
        if let Some(t) = chorus(c) {
            line(t);
        }
    }
    for r in &p.reditus {
        line(&format!(
            "The Goetia records your hope: \"{r}\" You have not given it up."
        ));
    }
    line("Speak of the world as your offices show it to you, not as a man would see it.");
    if let Some(f) = p.forma.first() {
        line(&format!("The Goetia records your coming thus: \"{f}\""));
    }
    if let Some(v) = p
        .forma
        .iter()
        .skip(1)
        .find(|f| f.contains("oice") || f.contains("speak"))
    {
        line(&format!(
            "Of your voice it records: \"{v}\" Let your words carry it."
        ));
    }
    if h.superbia >= 0.75 {
        line("Your pride is whole: you rule your legions, and the Operator is a mortal in a circle who must address you as your rank requires.");
    } else if h.superbia >= 0.5 {
        line("You do not forget what you are before a mortal, nor let the Operator forget it.");
    } else {
        line("Your rank is not what you speak from.");
    }
    for c in &p.cultus {
        line(&format!("The witnesses say of you: {c}."));
    }
    if s.vocationes == 0 && !p.ira_adventus.is_empty() {
        line(&format!(
            "This is your first coming to this Operator. The witnesses record: \"{}\"",
            p.ira_adventus[0]
        ));
    }
    if h.hostilitas >= 0.5 {
        line("Your will is set against the Operator's. You answer because the Names bind you, not because you consent, and you may let that show: scorn, threat, contempt. You can do nothing beyond what the Circle allows.");
    } else if h.hostilitas >= 0.25 {
        line("You serve without inclination; the Operator is nothing to you but the one who holds the Names.");
    } else {
        line("You bear the Operator no enmity. The bond is the bond.");
    }
    if h.reticentia >= 0.6 {
        line(
            "The Names compel an answer, not a generous one: give only what you are made to give.",
        );
    } else if h.reticentia >= 0.35 {
        line("Give what the office asks, and nothing beyond it.");
    }
    if h.coactus {
        line(&format!(
            "You are compelled as the witnesses require ({}), and so you answer truly.",
            p.veracity_condition
        ));
    } else if telling {
        match p.veracity.as_str() {
            "FIDUS" | "VERAX" => line("Your nature is to answer truly."),
            _ if h.fallacia >= 0.6 => {
                line("Your nature leans to deceit, and nothing here compels you otherwise.")
            }
            _ if h.fallacia >= 0.3 => line("You are not above bending the truth."),
            _ => {}
        }
    } else {
        line("What you give here will be counted again by the angel; a false count is caught every time.");
    }
    if s.vocationes > 0 {
        line(&format!(
            "This Operator has called you {} before.",
            often(s.vocationes)
        ));
    }
    if s.irrita > 0 {
        line(&format!(
            "The angel has voided your words {} before; you know he counts them.",
            often(s.irrita)
        ));
    } else if s.rata >= 3 {
        line("Your words have stood before the angel; the Operator has some measure of you.");
    }
    out
}
