//! The custos: a plain check, in code, of a spirit's words against the
//! tables. It looks only at what the spirit says of itself (its name, rank
//! and legions), which is fixed and never the model's to choose. What a
//! spirit says of anything else is its own, deceit included, and is left to
//! the angel. The custos never changes or withholds a word: a reply it faults
//! is asked again, and if every try is faulted the last is given on as it is.

use annulus::lingua::registra::Spirit;

use crate::tabula;

/// Words that may stand between "I am" and a name or rank.
const BETWEEN: &[&str] = &[
    "a", "an", "the", "called", "named", "known", "as", "great", "mighty", "strong", "dread",
    "high", "powerful", "terrible", "most",
];

fn words(sentence: &str) -> Vec<String> {
    sentence
        .split(|c: char| !(c.is_alphanumeric() || c == '\'' || c == '-'))
        .flat_map(|w| {
            let w = w.trim_matches(|c| c == '\'' || c == '-').to_lowercase();
            // "thirty-six" is two number words; "glasya-labolas" stays whole.
            if w.split('-').all(|p| number_word(p).is_some()) {
                w.split('-').map(str::to_string).collect::<Vec<_>>()
            } else {
                vec![w]
            }
        })
        .filter(|w| !w.is_empty())
        .collect()
}

fn number_word(w: &str) -> Option<u32> {
    const UNITS: [&str; 20] = [
        "zero",
        "one",
        "two",
        "three",
        "four",
        "five",
        "six",
        "seven",
        "eight",
        "nine",
        "ten",
        "eleven",
        "twelve",
        "thirteen",
        "fourteen",
        "fifteen",
        "sixteen",
        "seventeen",
        "eighteen",
        "nineteen",
    ];
    const TENS: [&str; 8] = [
        "twenty", "thirty", "forty", "fifty", "sixty", "seventy", "eighty", "ninety",
    ];
    if let Some(i) = UNITS.iter().position(|u| *u == w) {
        return Some(i as u32);
    }
    if let Some(i) = TENS.iter().position(|t| *t == w) {
        return Some(20 + 10 * i as u32);
    }
    match w {
        "hundred" => Some(100),
        "thousand" => Some(1000),
        "score" => Some(20),
        _ => None,
    }
}

/// The number that ends just before `end` in `w`, if one does: digits, or
/// words like "two hundred and thirty-six" or "three score".
fn number_before(w: &[String], end: usize) -> Option<u32> {
    let last = end.checked_sub(1)?;
    if let Ok(n) = w[last].parse::<u32>() {
        return Some(n);
    }
    let mut start = end;
    while start > 0 {
        let p = &w[start - 1];
        if number_word(p).is_some() || (p == "and" && start < end) {
            start -= 1;
        } else {
            break;
        }
    }
    while start < end && w[start] == "and" {
        start += 1;
    }
    if start == end {
        return None;
    }
    let (mut total, mut part) = (0u32, 0u32);
    for t in &w[start..end] {
        match t.as_str() {
            "and" => {}
            "hundred" | "score" => part = part.max(1) * number_word(t)?,
            "thousand" => {
                total += part.max(1) * 1000;
                part = 0;
            }
            _ => part += number_word(t)?,
        }
    }
    Some(total + part)
}

fn speaks_of_self(w: &[String], own: &str) -> bool {
    w.iter()
        .any(|t| matches!(t.as_str(), "i" | "i'm" | "my" | "me" | "mine" | "myself") || t == own)
}

/// Where in `w` each "I am" (or "I'm", or "my name is") ends.
fn claims(w: &[String]) -> Vec<usize> {
    let mut out = Vec::new();
    for i in 0..w.len() {
        if w[i] == "i'm" {
            out.push(i + 1);
        } else if (w[i] == "i" && w.get(i + 1).map(String::as_str) == Some("am"))
            || (w[i] == "name"
                && w.get(i + 1).map(String::as_str) == Some("is")
                && i > 0
                && w[i - 1] == "my")
        {
            out.push(i + 2);
        }
    }
    out
}

/// The faults in what `s` says of itself, each in plain words. Empty means
/// nothing it said of itself is against the tables.
pub fn examine(s: &Spirit, text: &str) -> Vec<String> {
    let own = s.name.to_lowercase();
    let others: Vec<&str> = tabula::names()
        .iter()
        .filter(|(n, _)| *n != s.n)
        .map(|(_, name)| name.as_str())
        .collect();
    let mut faults = Vec::new();
    for sentence in text.split(['.', '!', '?', ';', '\n']) {
        let w = words(sentence);
        if w.is_empty() {
            continue;
        }
        // "Paimon's legions" speak of Paimon too.
        let about_another = w
            .iter()
            .any(|t| others.contains(&t.strip_suffix("'s").unwrap_or(t)));
        for at in claims(&w) {
            let mut j = at;
            while j < w.len() && BETWEEN.contains(&w[j].as_str()) {
                j += 1;
            }
            let Some(next) = w.get(j) else { continue };
            if others.contains(&next.as_str()) {
                faults.push(format!("calls itself {next}, but it is {}", s.name));
            } else if let Some(rank) = tabula::rank_of_word(next) {
                if !tabula::holds_rank(s, rank) && !about_another {
                    faults.push(format!(
                        "calls itself a {rank}, but it is a {}",
                        s.ranks.join(" and ")
                    ));
                }
            }
        }
        if about_another || !speaks_of_self(&w, &own) {
            continue;
        }
        for (i, t) in w.iter().enumerate() {
            if !matches!(t.as_str(), "legion" | "legions" | "legiones") {
                continue;
            }
            // "thirty legions", or "thirty of my legions".
            let n = number_before(&w, i).or_else(|| {
                (i >= 2 && w[i - 1] == "my" && w[i - 2] == "of")
                    .then(|| number_before(&w, i - 2))
                    .flatten()
            });
            if let Some(n) = n {
                if n != s.legions {
                    faults.push(format!(
                        "says it rules {n} legions, but it rules {}",
                        s.legions
                    ));
                }
            }
        }
    }
    faults.dedup();
    faults
}
