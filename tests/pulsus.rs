use std::time::{Duration, Instant};

use spiri72::initium::read_ini;
use spiri72::pulsus::*;

fn facts(f: &[&str]) -> Vec<String> {
    f.iter().map(|s| s.to_string()).collect()
}

#[test]
fn a_message_goes_out_only_when_something_changed() {
    let mut p = Pulsus::new(30);
    let t = Instant::now();
    assert_eq!(p.beat(t, facts(&["a"])), Some("a"));
    for k in 1..100 {
        assert_eq!(p.beat(t + Duration::from_secs(k * 60), facts(&["a"])), None);
    }
    assert_eq!(p.seq, 1);
}

#[test]
fn never_more_than_one_message_per_gap_and_nothing_is_lost() {
    let mut p = Pulsus::new(30);
    let t = Instant::now();
    assert!(p.beat(t, facts(&["a"])).is_some());
    for k in 1..30 {
        let changed = facts(&[&format!("b{k}")]);
        assert_eq!(
            p.beat(t + Duration::from_secs(k), changed),
            None,
            "second {k}"
        );
    }
    // after the gap, the newest facts go out, once
    assert_eq!(
        p.beat(t + Duration::from_secs(30), facts(&["b29"])),
        Some("b29")
    );
    assert_eq!(p.seq, 2);
}

#[test]
fn the_gap_is_variable_but_never_under_thirty_seconds() {
    assert_eq!(Pulsus::new(5).gap(), Duration::from_secs(30));
    assert_eq!(Pulsus::new(0).gap(), Duration::from_secs(30));
    assert_eq!(Pulsus::new(90).gap(), Duration::from_secs(90));
    let mut p = Pulsus::new(90);
    let t = Instant::now();
    p.beat(t, facts(&["a"]));
    assert_eq!(p.beat(t + Duration::from_secs(60), facts(&["b"])), None);
    assert!(p.beat(t + Duration::from_secs(90), facts(&["b"])).is_some());
    assert_eq!(read_ini("heartbeat = 120\n").heartbeat, Some(120));
}

#[test]
fn all_well_is_said_once_when_the_trouble_ends() {
    let mut p = Pulsus::new(30);
    let t = Instant::now();
    p.beat(t, facts(&["The model does not answer."]));
    assert_eq!(
        p.beat(t + Duration::from_secs(40), vec![]),
        Some("All is as it should be.")
    );
}

#[test]
fn time_held_is_told_in_quarter_hours() {
    assert_eq!(held_for(60), "less than a quarter of an hour");
    assert_eq!(held_for(16 * 60), "15 minutes");
    assert_eq!(held_for(3600), "1 hour");
    assert_eq!(held_for(80 * 60), "1 hour 15 minutes");
    assert_eq!(held_for(2 * 3600 + 31 * 60), "2 hours 30 minutes");
}
