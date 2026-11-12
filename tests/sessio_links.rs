#![cfg(unix)]

use std::fs;

use annulus::lingua::exsequi::{Actum, Petitio, Voluntas};
use annulus::lingua::registra::goetia;
use annulus::nucleus::Outcome;
use spiri72::sessio::{ask_once, RITES};

struct Silent;

impl Voluntas for Silent {
    fn nomen(&self) -> String {
        "silens".into()
    }
    fn age(&mut self, _: &Petitio) -> Result<(Outcome, Actum), String> {
        Ok((Outcome::Obsequium, Actum::Responsum("tacet".into())))
    }
}

#[test]
fn a_planted_link_at_the_next_rite_name_is_never_followed() {
    let dir = std::env::temp_dir().join(format!("spiri72-links-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join(RITES)).unwrap();
    let furfur = goetia().by_number(34).unwrap();
    let target = dir.join("victim.txt");
    std::os::unix::fs::symlink(
        &target,
        dir.join(RITES).join(format!("{}-1.annulus", furfur.label)),
    )
    .unwrap();
    let (mut out, mut err) = (Vec::new(), Vec::new());
    ask_once(
        furfur,
        "RESPONSA",
        "venietne tempestas",
        &[],
        &dir,
        &[],
        &mut Silent,
        &mut out,
        &mut err,
    );
    assert!(!target.exists(), "the link was followed");
    assert!(dir
        .join(RITES)
        .join(format!("{}-2.annulus", furfur.label))
        .exists());
}
