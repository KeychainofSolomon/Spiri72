//! The heartbeat (`pulsus`): facts measured by the program, never guessed by a model.
//!
//! The page asks for a beat now and then. Each time, Spiri72 measures what it can see:
//! whether the model and the eyes answer, whether the wand's stick is still in, how long
//! each held spirit has been held and whether its hour has passed, and how long the
//! binding has left. A message goes out only when those facts have changed, and never more
//! often than once per gap (at least 30 seconds, `heartbeat = N` in `spiri72.ini`).
//! Changes that come within the gap are gathered into the next message, so nothing is
//! lost and nothing is repeated.

use std::time::{Duration, Instant};

/// The shortest gap between two messages, whatever is asked.
pub const MIN_GAP: Duration = Duration::from_secs(30);

pub struct Pulsus {
    gap: Duration,
    last_sent: Option<Instant>,
    told: Vec<String>,
    /// How many messages have gone out; the page shows a message once per number.
    pub seq: u64,
    pub message: Option<String>,
}

impl Pulsus {
    /// A heartbeat whose messages are at least `gap_secs` apart (never under 30).
    pub fn new(gap_secs: u64) -> Pulsus {
        Pulsus {
            gap: Duration::from_secs(gap_secs).max(MIN_GAP),
            last_sent: None,
            told: Vec::new(),
            seq: 0,
            message: None,
        }
    }

    pub fn gap(&self) -> Duration {
        self.gap
    }

    /// Takes the facts measured now. Gives a new message only if they differ from the
    /// last ones told and the gap has passed since the last message.
    pub fn beat(&mut self, now: Instant, facts: Vec<String>) -> Option<&str> {
        if facts == self.told {
            return None;
        }
        if self
            .last_sent
            .is_some_and(|t| now.saturating_duration_since(t) < self.gap)
        {
            return None;
        }
        self.last_sent = Some(now);
        self.seq += 1;
        self.message = Some(if facts.is_empty() {
            "All is as it should be.".to_string()
        } else {
            facts.join(" ")
        });
        self.told = facts;
        self.message.as_deref()
    }
}

/// How long, in plain words, rounded down to the quarter hour so it changes rarely.
pub fn held_for(secs: i64) -> String {
    let quarters = secs.max(0) / 900;
    let (h, m) = (quarters / 4, (quarters % 4) * 15);
    match (h, m) {
        (0, 0) => "less than a quarter of an hour".into(),
        (0, m) => format!("{m} minutes"),
        (h, 0) => format!("{h} hour{}", if h == 1 { "" } else { "s" }),
        (h, m) => format!("{h} hour{} {m} minutes", if h == 1 { "" } else { "s" }),
    }
}

/// Whether the wand's stick is in: its `VIRGA` folder can be seen.
pub fn wand_present(stick: &std::path::Path) -> bool {
    stick.join("VIRGA").is_dir()
}
