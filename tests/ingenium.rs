use std::fs;
use std::path::PathBuf;

use annulus::lingua::registra::{goetia, Spirit, GOETIA};
use annulus::nucleus::sha256::hash_hex;
use spiri72::ingenium::{
    self, brief, compelled, draw, profile, profiles, read_status, record_call, record_verdict,
    Status,
};
use spiri72::persona::entry;

fn spirit(n: u32) -> &'static Spirit {
    goetia().spirits.iter().find(|s| s.n == n).unwrap()
}

fn fixed(u: f64) -> impl FnMut() -> Result<f64, String> {
    move || Ok(u)
}

fn folder(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("spiri72-ing-{}-{name}", std::process::id()));
    let _ = fs::remove_dir_all(&d);
    fs::create_dir_all(&d).unwrap();
    d
}

#[test]
fn the_profiles_are_made_from_this_very_register() {
    let sum = hash_hex(GOETIA.as_bytes());
    assert_eq!(profiles().len(), 72);
    for p in profiles() {
        assert_eq!(
            p.register_sha256, sum,
            "profile {} was made from another register",
            p.n
        );
        assert_eq!(
            p.label,
            spirit(p.n).label,
            "profile {} names another spirit",
            p.n
        );
    }
}

#[test]
fn the_profiles_copy_the_witnesses_unchanged() {
    let root = annulus::lingua::json::parse(ingenium::SPIRITUS).unwrap();
    for p in root.list() {
        let n = p.get("n").and_then(|x| x.number()).unwrap() as u32;
        let ess = p.get("essentia").unwrap();
        let reg = entry(n).unwrap();
        if let annulus::lingua::json::Json::Object(map) = ess {
            for (k, v) in map {
                assert_eq!(
                    reg.get(k),
                    Some(v),
                    "spirit {n}: essentia.{k} differs from the register"
                );
            }
        } else {
            panic!("spirit {n} has no essentia");
        }
    }
}

#[test]
fn a_broken_profile_set_is_refused() {
    assert!(ingenium::load("[]").is_err());
    for (from, to) in [
        ("\"72spirits/1\"", "\"other\""),
        ("\"superbia\":[", "\"superbia\":[2.0,"),
        ("\"hostilitas\":[", "\"hostilitas\":\"fierce\",\"x\":["),
        ("\"n\":2,", "\"n\":3,"),
    ] {
        let tampered = ingenium::SPIRITUS.replacen(from, to, 1);
        assert_ne!(tampered, ingenium::SPIRITUS, "{from} not found");
        assert!(
            ingenium::load(&tampered).is_err(),
            "{from} -> {to} was accepted"
        );
    }
}

#[test]
fn every_draw_stays_within_its_range() {
    for p in profiles() {
        let s = spirit(p.n);
        for u in [0.0, 0.5, 0.999_999] {
            for own in [true, false] {
                let g = draw(p, s, own, &Status::default(), &mut fixed(u)).unwrap();
                let deceit = if g.coactus {
                    p.fallacia_coactus.unwrap()
                } else {
                    p.fallacia
                };
                for (x, (lo, hi)) in [
                    (g.superbia, p.superbia),
                    (g.fallacia, deceit),
                    (g.hostilitas, p.hostilitas),
                    (g.reticentia, p.reticentia),
                ] {
                    assert!(lo <= x && x <= hi, "spirit {}: {x} outside {lo}..{hi}", p.n);
                }
            }
        }
    }
}

#[test]
fn a_voided_spirit_lies_from_the_lower_half_and_never_below() {
    let p = profile(46).unwrap(); // Bifrons, TACITUS: 0.1..0.5
    let s = spirit(46);
    let counted = Status {
        irritum: 1,
        ..Status::default()
    };
    let high = draw(p, s, true, &counted, &mut fixed(0.999_999)).unwrap();
    let low = draw(p, s, true, &counted, &mut fixed(0.0)).unwrap();
    assert!(high.numeratus);
    assert!(high.fallacia <= (p.fallacia.0 + p.fallacia.1) / 2.0);
    assert_eq!(low.fallacia, p.fallacia.0);
    let free = draw(p, s, true, &Status::default(), &mut fixed(0.999_999)).unwrap();
    assert!(!free.numeratus);
    assert!(free.fallacia > (p.fallacia.0 + p.fallacia.1) / 2.0);
}

#[test]
fn compulsion_follows_the_witnesses() {
    // In Spiri72's own rites every COACTUS condition is met.
    for n in [32, 34, 44, 64, 68] {
        assert!(compelled(spirit(n), true), "{n}");
    }
    // Elsewhere only the triangle is assumed: Furfur, Shax and Haures, not
    // Asmoday (his name) or Belial (offerings, the divine power).
    for n in [34, 44, 64] {
        assert!(compelled(spirit(n), false), "{n}");
    }
    for n in [32, 68] {
        assert!(!compelled(spirit(n), false), "{n}");
    }
    assert!(!compelled(spirit(1), true), "Bael is not COACTUS");

    let p = profile(68).unwrap();
    let held = draw(
        p,
        spirit(68),
        true,
        &Status::default(),
        &mut fixed(0.999_999),
    )
    .unwrap();
    assert!(held.coactus && held.fallacia <= 0.15);
    let loose = draw(p, spirit(68), false, &Status::default(), &mut fixed(0.0)).unwrap();
    assert!(!loose.coactus && loose.fallacia >= 0.5);
}

#[test]
fn ill_will_bends_the_first_nature_only() {
    let p = profile(13).unwrap(); // Beleth: ill will 0.4..0.6
    assert!(p.naturae.len() == 2 || p.naturae.len() == 1);
    let s = spirit(13);
    let calm = draw(p, s, true, &Status::default(), &mut fixed(0.0)).unwrap();
    let angry = draw(p, s, true, &Status::default(), &mut fixed(0.999_999)).unwrap();
    let calm_b = brief(p, &calm, &Status::default());
    let angry_b = brief(p, &angry, &Status::default());
    assert!(!calm_b.contains("the Fall bent it"), "{calm_b}");
    assert_eq!(angry_b.matches("the Fall bent it").count(), 1, "{angry_b}");
}

#[test]
fn the_brief_carries_the_spirits_place_fall_and_bearing() {
    let mid = |n: u32| {
        let p = profile(n).unwrap();
        let g = draw(p, spirit(n), true, &Status::default(), &mut fixed(0.5)).unwrap();
        brief(p, &g, &Status::default())
    };
    let bael = mid(1);
    assert!(
        bael.contains("Your quarter is the East, under its king Amaymon."),
        "{bael}"
    );
    assert!(bael.contains("He speaketh hoarsely."), "{bael}");
    assert!(bael.contains("The Operator holds no place in that order."));
    assert!(bael.contains("How you come to this rite:"));
    assert!(
        !bael.contains("Of the "),
        "Bael's choir is not given: {bael}"
    );

    let belial = mid(68);
    assert!(belial.contains("among the chiefs: Belial, Bileth, Asmoday and Gaap"));
    assert!(belial.contains("fell first from among the worthier sort"));
    assert!(belial.contains("compelled as the witnesses require"));
    assert!(
        !belial.contains("Of the Virtues"),
        "the Goetia gives Belial no choir"
    );

    let marchosias = mid(35);
    assert!(marchosias.contains("Of the Dominations:"), "{marchosias}");
    assert!(marchosias.contains("Seventh Throne"), "{marchosias}");

    let beleth = mid(13);
    assert!(
        beleth.contains("What the witnesses say of your fury:"),
        "{beleth}"
    );
    assert!(beleth.contains("Of the Powers:"), "{beleth}");
    assert_eq!(
        beleth.matches("layeth his courage").count(),
        1,
        "said twice: {beleth}"
    );
    assert!(
        beleth.contains("- He is a mighty King and terrible."),
        "{beleth}"
    );
}

#[test]
fn the_record_counts_calls_and_judgements() {
    let d = folder("record");
    let s = spirit(36);
    let p = profile(36).unwrap();
    let g = draw(p, s, true, &Status::default(), &mut fixed(0.5)).unwrap();
    assert_eq!(read_status(&d, s).unwrap(), Status::default());
    record_call(&d, s, &g).unwrap();
    record_verdict(&d, s, "IRRITUM").unwrap();
    record_call(&d, s, &g).unwrap();
    record_verdict(&d, s, "RATUM").unwrap();
    let st = read_status(&d, s).unwrap();
    assert_eq!(
        st,
        Status {
            vocatus: 2,
            ratum: 1,
            suspensum: 0,
            irritum: 1
        }
    );
    let text = fs::read_to_string(d.join("spiri72.status/36-stolas.txt")).unwrap();
    assert!(text.contains("gestus superbia"), "{text}");
    assert!(record_verdict(&d, s, "MALITIA").is_err());

    let b = brief(p, &draw(p, s, true, &st, &mut fixed(0.5)).unwrap(), &st);
    assert!(b.contains("called you 2 times before"), "{b}");
    assert!(b.contains("voided 1"), "{b}");
    assert!(b.contains("You know the angel counts"), "{b}");
    assert!(
        b.contains("the angel has found 1 sound and voided 1."),
        "{b}"
    );
    assert!(!b.contains("in suspense"), "{b}");
}

#[test]
fn a_record_out_of_order_refuses_the_rite() {
    let d = folder("disorder");
    let s = spirit(36);
    fs::create_dir_all(d.join("spiri72.status")).unwrap();
    for bad in ["vocatus many\n", "irritum 0\nfiat lux\n", &"x".repeat(5000)] {
        fs::write(d.join("spiri72.status/36-stolas.txt"), bad).unwrap();
        assert!(read_status(&d, s).is_err(), "{bad:?} was accepted");
    }
}

#[cfg(unix)]
#[test]
fn a_planted_link_is_never_followed() {
    use std::os::unix::fs::symlink;
    let s = spirit(36);
    let g = draw(
        profile(36).unwrap(),
        s,
        true,
        &Status::default(),
        &mut fixed(0.5),
    )
    .unwrap();

    // The record folder itself a link elsewhere: refused.
    let d = folder("link-dir");
    let elsewhere = folder("link-dir-elsewhere");
    symlink(&elsewhere, d.join("spiri72.status")).unwrap();
    assert!(record_call(&d, s, &g).is_err());
    assert!(fs::read_dir(&elsewhere).unwrap().next().is_none());

    // The record a link to another file: refused to read, never written through.
    let d = folder("link-file");
    let victim = d.join("victim.txt");
    fs::write(&victim, "the operator's own").unwrap();
    fs::create_dir_all(d.join("spiri72.status")).unwrap();
    symlink(&victim, d.join("spiri72.status/36-stolas.txt")).unwrap();
    assert!(record_call(&d, s, &g).is_err());
    assert_eq!(fs::read_to_string(&victim).unwrap(), "the operator's own");
}
