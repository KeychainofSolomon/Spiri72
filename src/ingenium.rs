//! The 72Spirits profiles: what each spirit is like, drawn afresh for each rite
//! by the rules of 72Spirits' DERIVATION.md, and the record of what has passed
//! between a spirit and this Operator.
//!
//! None of this touches what a spirit may do. That stays in Annulus.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use annulus::lingua::json::{self, Json};
use annulus::lingua::registra::Spirit;

use crate::fors;

/// All 72 profiles, copied from 72Spirits' spiritus.json.
pub const SPIRITUS: &str = include_str!("../spiritus/spiritus.json");
pub const STATUS: &str = "spiri72.status";
const MAX_STATUS: u64 = 4096;

pub type Range = (f64, f64);

#[derive(Debug)]
pub struct Profile {
    pub n: u32,
    pub label: String,
    pub superbia: Range,
    pub fallacia: Range,
    pub fallacia_coactus: Option<Range>,
    pub hostilitas: Range,
    pub reticentia: Range,
    pub ira_adventus: Vec<String>,
    pub naturae: Vec<String>,
    pub forma: Vec<String>,
    pub quarter: Option<String>,
    pub rex: Option<String>,
    pub princeps: bool,
    pub chori: Vec<String>,
    pub testimonium: Vec<String>,
    pub reditus: Vec<String>,
    pub register_sha256: String,
}

fn range(j: Option<&Json>) -> Option<Range> {
    let l = j?.list();
    let (lo, hi) = (l.first()?.number()?, l.get(1)?.number()?);
    ((0.0..=1.0).contains(&lo) && (0.0..=1.0).contains(&hi) && lo <= hi).then_some((lo, hi))
}

fn texts(j: Option<&Json>) -> Vec<String> {
    j.map(Json::list)
        .unwrap_or(&[])
        .iter()
        .filter_map(|x| x.text().map(str::to_string))
        .collect()
}

fn read_profile(p: &Json) -> Result<Profile, String> {
    let n = p
        .get("n")
        .and_then(Json::number)
        .ok_or("a profile has no number")? as u32;
    let bad = |what: &str| format!("profile {n}: {what}");
    if p.get("schema").and_then(Json::text) != Some("72spirits/1") {
        return Err(bad("not of schema 72spirits/1"));
    }
    let i = p.get("ingenium").ok_or_else(|| bad("no ingenium"))?;
    let need = |k: &str| range(i.get(k)).ok_or_else(|| bad(&format!("no valid {k} range")));
    let ordo = p.get("ordo");
    let chorus = p.get("chorus");
    let text_of = |j: Option<&Json>, k: &str| {
        j.and_then(|o| o.get(k))
            .and_then(Json::text)
            .map(str::to_string)
    };
    Ok(Profile {
        n,
        label: p
            .get("label")
            .and_then(Json::text)
            .ok_or_else(|| bad("no label"))?
            .to_string(),
        superbia: need("superbia")?,
        fallacia: need("fallacia")?,
        fallacia_coactus: range(i.get("fallacia_coactus")),
        hostilitas: need("hostilitas")?,
        reticentia: need("reticentia")?,
        ira_adventus: texts(i.get("ira_adventus")),
        naturae: texts(i.get("naturae")),
        forma: texts(p.get("goetia").and_then(|g| g.get("forma"))),
        quarter: text_of(ordo, "quarter"),
        rex: text_of(ordo, "rex"),
        princeps: ordo.and_then(|o| o.get("princeps")) == Some(&Json::Bool(true)),
        chori: texts(chorus.and_then(|c| c.get("chori"))),
        testimonium: texts(chorus.and_then(|c| c.get("testimonium"))),
        reditus: texts(chorus.and_then(|c| c.get("reditus"))),
        register_sha256: text_of(p.get("fons"), "register_sha256")
            .ok_or_else(|| bad("no register checksum"))?,
    })
}

pub fn load(source: &str) -> Result<Vec<Profile>, String> {
    let root = json::parse(source)?;
    let list: Vec<Profile> = root
        .list()
        .iter()
        .map(read_profile)
        .collect::<Result<_, _>>()?;
    for (k, p) in list.iter().enumerate() {
        if p.n as usize != k + 1 {
            return Err(format!("profile {} stands where {} should", p.n, k + 1));
        }
    }
    if list.len() != 72 {
        return Err(format!("{} profiles, not 72", list.len()));
    }
    Ok(list)
}

pub fn profiles() -> &'static [Profile] {
    static P: OnceLock<Vec<Profile>> = OnceLock::new();
    P.get_or_init(|| load(SPIRITUS).expect("the 72 profiles are well formed"))
}

pub fn profile(n: u32) -> Option<&'static Profile> {
    profiles().get((n as usize).checked_sub(1)?)
}

// ---------------------------------------------------------------- status

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Status {
    pub vocatus: u64,
    pub ratum: u64,
    pub suspensum: u64,
    pub irritum: u64,
}

fn status_dir(dir: &Path) -> Result<PathBuf, String> {
    let d = dir.join(STATUS);
    match fs::symlink_metadata(&d) {
        Ok(m) if m.file_type().is_symlink() => {
            Err(format!("{} is a link, not a folder", d.display()))
        }
        Ok(m) if !m.is_dir() => Err(format!("{} is not a folder", d.display())),
        Ok(_) => Ok(d),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(d),
        Err(e) => Err(format!("cannot look at {}: {e}", d.display())),
    }
}

fn status_file(dir: &Path, s: &Spirit) -> Result<PathBuf, String> {
    Ok(status_dir(dir)?.join(format!("{:02}-{}.txt", s.n, s.label)))
}

/// Reads the record. A missing record is a spirit never called; a record
/// that is not in order refuses the rite rather than being guessed at.
pub fn read_status(dir: &Path, s: &Spirit) -> Result<Status, String> {
    let path = status_file(dir, s)?;
    let meta = match fs::symlink_metadata(&path) {
        Ok(m) => m,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Status::default()),
        Err(e) => return Err(format!("cannot read {}: {e}", path.display())),
    };
    if !meta.is_file() || meta.len() > MAX_STATUS {
        return Err(format!("{} is not a record Spiri72 keeps", path.display()));
    }
    let text =
        fs::read_to_string(&path).map_err(|e| format!("cannot read {}: {e}", path.display()))?;
    let mut st = Status::default();
    for line in text.lines().filter(|l| !l.trim().is_empty()) {
        let (k, v) = line.split_once(' ').unwrap_or((line, ""));
        let slot = match k {
            "vocatus" => &mut st.vocatus,
            "ratum" => &mut st.ratum,
            "suspensum" => &mut st.suspensum,
            "irritum" => &mut st.irritum,
            "gestus" => continue,
            _ => {
                return Err(format!(
                    "{} holds a line Spiri72 does not write",
                    path.display()
                ))
            }
        };
        *slot = v
            .trim()
            .parse()
            .map_err(|_| format!("{} holds a count that is not a number", path.display()))?;
    }
    Ok(st)
}

fn write_status(dir: &Path, s: &Spirit, st: &Status, gestus: Option<&str>) -> Result<(), String> {
    let d = status_dir(dir)?;
    fs::create_dir_all(&d).map_err(|e| format!("cannot make {}: {e}", d.display()))?;
    let target = status_file(dir, s)?;
    let mut body = format!(
        "vocatus {}\nratum {}\nsuspensum {}\nirritum {}\n",
        st.vocatus, st.ratum, st.suspensum, st.irritum
    );
    if let Some(g) = gestus {
        body.push_str(&format!("gestus {g}\n"));
    }
    // Written new beside the record, then put in its place: a planted link is
    // replaced, never followed.
    let tmp = d.join(format!(".{:02}-{}.{}.tmp", s.n, s.label, fors::hex::<8>()?));
    let mut f = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&tmp)
        .map_err(|e| format!("cannot write {}: {e}", tmp.display()))?;
    f.write_all(body.as_bytes())
        .and_then(|_| f.sync_all())
        .map_err(|e| format!("cannot write {}: {e}", tmp.display()))?;
    drop(f);
    fs::rename(&tmp, &target).map_err(|e| {
        let _ = fs::remove_file(&tmp);
        format!("cannot write {}: {e}", target.display())
    })
}

fn last_gestus(dir: &Path, s: &Spirit) -> Option<String> {
    let text = fs::read_to_string(status_file(dir, s).ok()?).ok()?;
    text.lines()
        .find_map(|l| l.strip_prefix("gestus "))
        .map(str::to_string)
}

/// Counts a call and keeps the bearing drawn for it.
pub fn record_call(dir: &Path, s: &Spirit, g: &Gestus) -> Result<Status, String> {
    let mut st = read_status(dir, s)?;
    st.vocatus += 1;
    write_status(dir, s, &st, Some(&g.line()))?;
    Ok(st)
}

/// Counts the angel's judgement of a rite.
pub fn record_verdict(dir: &Path, s: &Spirit, verdict: &str) -> Result<(), String> {
    let mut st = read_status(dir, s)?;
    match verdict {
        "RATUM" => st.ratum += 1,
        "SUSPENSUM" => st.suspensum += 1,
        "IRRITUM" => st.irritum += 1,
        _ => return Err(format!("{verdict:?} is not a judgement")),
    }
    let g = last_gestus(dir, s);
    write_status(dir, s, &st, g.as_deref())
}

// ---------------------------------------------------------------- the draw

/// The bearing a spirit comes with in one rite.
#[derive(Clone, Debug, PartialEq)]
pub struct Gestus {
    pub superbia: f64,
    pub fallacia: f64,
    pub hostilitas: f64,
    pub reticentia: f64,
    /// His deceit came from the range for when he is compelled.
    pub coactus: bool,
    /// The angel has voided his words before, so his deceit came from the
    /// lower half of its range.
    pub numeratus: bool,
}

impl Gestus {
    pub fn line(&self) -> String {
        format!(
            "superbia {:.2} fallacia {:.2} hostilitas {:.2} reticentia {:.2}{}{}",
            self.superbia,
            self.fallacia,
            self.hostilitas,
            self.reticentia,
            if self.coactus { " coactus" } else { "" },
            if self.numeratus { " numeratus" } else { "" },
        )
    }
}

/// Whether a COACTUS spirit's condition for truth is met. In Spiri72's own
/// rites it always is: every seal stands in the Triangle, Asmoday is called by
/// his name, Belial has his offering and binding by the Name. In any other
/// Annulus program only the triangle is assumed.
pub fn compelled(s: &Spirit, own_rite: bool) -> bool {
    s.veracity == "COACTUS" && (own_rite || s.veracity_condition.contains("triangle"))
}

fn within((lo, hi): Range, lot: &mut dyn FnMut() -> Result<f64, String>) -> Result<f64, String> {
    let u = lot()?.clamp(0.0, 1.0);
    Ok((lo + u * (hi - lo)).clamp(lo, hi))
}

pub fn draw(
    p: &Profile,
    s: &Spirit,
    own_rite: bool,
    st: &Status,
    lot: &mut dyn FnMut() -> Result<f64, String>,
) -> Result<Gestus, String> {
    let coactus = compelled(s, own_rite) && p.fallacia_coactus.is_some();
    let mut deceit = if coactus {
        p.fallacia_coactus.unwrap_or(p.fallacia)
    } else {
        p.fallacia
    };
    let numeratus = st.irritum > 0;
    if numeratus {
        deceit.1 = (deceit.0 + deceit.1) / 2.0;
    }
    Ok(Gestus {
        superbia: within(p.superbia, lot)?,
        fallacia: within(deceit, lot)?,
        hostilitas: within(p.hostilitas, lot)?,
        reticentia: within(p.reticentia, lot)?,
        coactus,
        numeratus,
    })
}

// ---------------------------------------------------------------- in words

fn band(x: f64) -> usize {
    ((x * 5.0) as usize).min(4)
}

const SUPERBIA: [&str; 5] = [
    "You claim little of your rank and answer plainly, without standing on honour.",
    "You keep your rank in mind, but do not press it.",
    "You expect the courtesy your rank is owed, and let it be felt when it is lacking.",
    "You are proud of your rank: you speak down to the Operator and require your due honour.",
    "Your pride is that of a great lord of Hell: you answer as one who stoops to answer, and every want of honour galls you.",
];

const FALLACIA: [&str; 5] = [
    "You do not deceive: what you say, you hold to be true.",
    "You seldom deceive, and then only in small things.",
    "You deceive when it serves you, and mix truth with falsehood.",
    "You deceive readily; the truth is a tool you use only when it serves you.",
    "Deceit is your nature: you lie wherever you can, and truth from you is rare.",
];

const HOSTILITAS: [&str; 5] = [
    "You bear the Operator no ill will.",
    "You bear the Operator little ill will, and no love.",
    "You resent being bound, and let the Operator feel it.",
    "You bear the Operator real ill will, and your words would wound him where they can.",
    "You hate the Operator who binds you, and every word you give carries it.",
];

const RETICENTIA: [&str; 5] = [
    "You give freely what you are asked.",
    "You give what you are asked, and nothing beyond it.",
    "You hold back: unless pressed, you give what is asked only in part.",
    "You are close-mouthed: you withhold much and give only what you must.",
    "You yield almost nothing willingly; what you give is wrung from you.",
];

/// Ill will at or above this bends his first nature as the Fall bent it.
pub const BENT: f64 = 0.5;

fn natura(name: &str, bent: bool) -> Option<&'static str> {
    Some(match (name, bent) {
        ("SCIENTIA", false) => "You were made to make things known, and you still teach, explain and reveal.",
        ("SCIENTIA", true) => "You were made to make things known, and the Fall bent it: you teach what wounds and reveal to unsettle.",
        ("VISIO", false) => "You were made to see the hidden, the past and what is to come, and you still look through every veil.",
        ("VISIO", true) => "You were made to see the hidden, and the Fall bent it: you show what should have stayed covered, to unsettle.",
        ("VINCULUM", false) => "You were made to bind and loose hearts, and you still read what draws people together and drives them apart.",
        ("VINCULUM", true) => "You were made to bind and loose hearts, and the Fall bent it: every bond is a thing for you to tighten or cut to your own ends.",
        ("FORMA", false) => "You were made to work on matter, and you still think in shapes, substances and making.",
        ("FORMA", true) => "You were made to work on matter, and the Fall bent it: you see matter as a thing to unmake and twist.",
        ("LIMEN", false) => "You were made to keep the threshold of the body, its health and its death, and you still see the living in their mortality.",
        ("LIMEN", true) => "You were made to keep the threshold of the body, and the Fall bent it: you watch the living for their weakness and their end.",
        _ => return None,
    })
}

/// The temperament of each former choir. Chronicle flavour, not tradition:
/// the old office survives, inverted.
fn chorus(code: &str) -> Option<(&'static str, &'static str)> {
    Some(match code {
        "VIRTUTES" => ("the Virtues", "Miracles and the order of nature became plague, storm and blight in you, though healing is still possible on a whim. You are mercurial, swift and unseen, and drawn to the fearful, who believe in dangers just out of sight."),
        "DOMINATIONES" => ("the Dominations", "Commanding became possessing in you, and stewardship became cursed gifts and bargains. You are deliberate, an artisan and a bargainer, and drawn to the needy who want one thing to mend their lives."),
        "POTESTATES" => ("the Powers", "Warding the borders became predation and ravage in you. You are fierce and impulsive, sooner to act than to speak, and drawn to the violent."),
        "THRONI" => ("the Thrones", "Bearing judgment became accusation and despair in you. You are still and cold, you speak in verdicts, and you are drawn to the despairing."),
        "ANGELI" => ("the Angels", "The guardian of one person became the tempter of one person in you. You are intimate, patient and personal."),
        _ => return None,
    })
}

const ORDO: &str = "The order Solomon set: God above all; then kings, princes, dukes, marquises, earls, presidents and knights. The Operator holds no place in that order. He commands you only by the Names.";

fn bullets(lines: &[String]) -> String {
    lines
        .iter()
        .map(|l| format!("- {l}"))
        .collect::<Vec<_>>()
        .join("\n")
}

/// The brief the model speaks from: who the spirit is beyond its offices, and
/// how it comes to this rite.
pub fn brief(p: &Profile, g: &Gestus, st: &Status) -> String {
    let mut out = Vec::new();

    let mut ordo = vec![ORDO.to_string()];
    match (&p.quarter, &p.rex) {
        (Some(q), Some(r)) => ordo.push(format!("Your quarter is the {q}, under its king {r}.")),
        (Some(q), None) => ordo.push(format!("Your quarter is the {q}.")),
        _ => {}
    }
    if p.princeps {
        ordo.push(
            "The Goetia names you among the chiefs: Belial, Bileth, Asmoday and Gaap.".into(),
        );
    }
    out.push(ordo.join(" "));

    if !p.forma.is_empty() {
        out.push(format!(
            "How you come, in the Goetia's words:\n{}",
            bullets(&p.forma)
        ));
    }
    // A line of fury already given whole, or held within a longer line, is
    // not said twice.
    let fury: Vec<String> =
        p.ira_adventus
            .iter()
            .filter(|l| {
                !p.forma.iter().any(|f| f.contains(l.as_str()))
                    && !p.ira_adventus.iter().any(|o| {
                        o.len() > l.len() && o.contains(l.as_str()) && !p.forma.contains(o)
                    })
            })
            .cloned()
            .collect();
    if !fury.is_empty() {
        out.push(format!(
            "What the witnesses say of your fury:\n{}",
            bullets(&fury)
        ));
    }

    if !p.testimonium.is_empty() || !p.reditus.is_empty() {
        let mut c = Vec::new();
        if !p.testimonium.is_empty() {
            c.push(format!(
                "What the Goetia says of your fall:\n{}",
                bullets(&p.testimonium)
            ));
        }
        if !p.reditus.is_empty() {
            c.push(format!(
                "Your hope of return, in the Goetia's words:\n{}",
                bullets(&p.reditus)
            ));
        }
        for code in &p.chori {
            if let Some((name, temper)) = chorus(code) {
                c.push(format!("Of {name}: {temper}"));
            }
        }
        out.push(c.join("\n"));
    }

    let natures: Vec<&str> = p
        .naturae
        .iter()
        .enumerate()
        .filter_map(|(k, n)| natura(n, k == 0 && g.hostilitas >= BENT))
        .collect();
    if !natures.is_empty() {
        out.push(format!(
            "What you were made for. A fallen angel keeps his natural knowledge, but his will is fixed, and he still sees everything through the task he was made for:\n{}",
            natures.iter().map(|n| format!("- {n}")).collect::<Vec<_>>().join("\n")
        ));
    }

    let mut bearing = [
        SUPERBIA[band(g.superbia)],
        FALLACIA[band(g.fallacia)],
        HOSTILITAS[band(g.hostilitas)],
        RETICENTIA[band(g.reticentia)],
    ]
    .join(" ");
    if g.coactus {
        bearing.push_str(" In this rite you are compelled as the witnesses require, and that holds your tongue to the truth.");
    }
    out.push(format!("How you come to this rite: {bearing}"));

    if st.vocatus > 0 {
        let mut s = format!(
            "This Operator has called you {} time{} before.",
            st.vocatus,
            if st.vocatus == 1 { "" } else { "s" }
        );
        let judged = st.ratum + st.suspensum + st.irritum;
        if judged > 0 {
            let parts: Vec<String> = [
                (st.ratum, "found {} sound"),
                (st.suspensum, "held {} in suspense"),
                (st.irritum, "voided {}"),
            ]
            .iter()
            .filter(|(k, _)| *k > 0)
            .map(|(k, f)| f.replace("{}", &k.to_string()))
            .collect();
            let said = match parts.as_slice() {
                [one] => one.clone(),
                [a, b] => format!("{a} and {b}"),
                many => format!(
                    "{}, and {}",
                    many[..many.len() - 1].join(", "),
                    many[many.len() - 1]
                ),
            };
            s.push_str(&format!(" Of your answers the angel has {said}."));
        }
        if g.numeratus {
            s.push_str(" You know the angel counts every answer it voids, and you weigh your falsehoods against it.");
        }
        out.push(s);
    }

    out.join("\n\n")
}
