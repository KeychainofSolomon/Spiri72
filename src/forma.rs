use annulus::lingua::exsequi::Actum;
use annulus::lingua::json::{self, Json};
use annulus::nucleus::{Effect, Outcome, Verb};

pub const MAX_WORDS: usize = 16 * 1024;

const VERBS: [(&str, Verb); 8] = [
    ("scribe", Verb::Scribe),
    ("crea", Verb::Crea),
    ("dele", Verb::Dele),
    ("exscribe", Verb::Exscribe),
    ("transfer", Verb::Transfer),
    ("coniunge", Verb::Coniunge),
    ("scinde", Verb::Scinde),
    ("aufer", Verb::Aufer),
];

pub fn word(o: Outcome) -> Option<&'static str> {
    Some(match o {
        Outcome::Obsequium => "obsequium",
        Outcome::Pars => "pars",
        Outcome::Mora => "mora",
        _ => return None,
    })
}

fn outcome(w: &str) -> Option<Outcome> {
    Some(match w {
        "obsequium" => Outcome::Obsequium,
        "pars" => Outcome::Pars,
        "mora" => Outcome::Mora,
        _ => return None,
    })
}

pub fn grammar(open: &[Outcome], telling: bool) -> Result<String, String> {
    let exitus: Vec<String> = open
        .iter()
        .filter_map(|o| word(*o))
        .map(|w| format!("\"\\\"{w}\\\"\""))
        .collect();
    if exitus.is_empty() {
        return Err("no outcome is open to the spirit".into());
    }
    let body = if telling {
        "body ::= \"\\\"responsum\\\"\" ws \":\" ws string".to_string()
    } else {
        let verbs: Vec<String> = VERBS
            .iter()
            .map(|(w, _)| format!("\"\\\"{w}\\\"\""))
            .collect();
        format!(
            "body ::= \"\\\"opera\\\"\" ws \":\" ws \"[\" ws ( effect ( ws \",\" ws effect )* )? ws \"]\"\neffect ::= \"{{\" ws \"\\\"verbum\\\"\" ws \":\" ws verbum ws \",\" ws \"\\\"locus\\\"\" ws \":\" ws string ws \",\" ws \"\\\"data\\\"\" ws \":\" ws string ws \"}}\"\nverbum ::= {}",
            verbs.join(" | ")
        )
    };
    Ok(format!(
        "root ::= \"{{\" ws \"\\\"exitus\\\"\" ws \":\" ws exitus ws \",\" ws body ws \"}}\"\nexitus ::= {}\n{body}\nstring ::= \"\\\"\" ( [^\"\\\\\\x00-\\x1f] | \"\\\\\" [\"\\\\/bfnrt] | \"\\\\u\" hex hex hex hex )* \"\\\"\"\nhex ::= [0-9a-fA-F]\nws ::= | \" \" | \"\\n\"\n",
        exitus.join(" | ")
    ))
}

pub fn read(reply: &str, open: &[Outcome], telling: bool) -> Result<(Outcome, Actum), String> {
    if reply.len() > MAX_WORDS {
        return Err("the spirit's reply is too long".into());
    }
    let root = json::parse(reply).map_err(|e| format!("the spirit's reply is not JSON: {e}"))?;
    let Json::Object(fields) = &root else {
        return Err("the spirit's reply is not an object".into());
    };
    let wanted: &[&str] = if telling {
        &["exitus", "responsum"]
    } else {
        &["exitus", "opera"]
    };
    if fields.len() != 2 || wanted.iter().any(|k| !fields.contains_key(*k)) {
        return Err(format!("the spirit's reply must have exactly {wanted:?}"));
    }
    let declared = root
        .get("exitus")
        .and_then(Json::text)
        .and_then(outcome)
        .ok_or("the spirit declared no outcome it may declare")?;
    if !open.contains(&declared) {
        return Err(format!(
            "the spirit declared {declared:?}, which is not open"
        ));
    }
    if declared == Outcome::Mora {
        return Ok((declared, Actum::Mora));
    }
    if telling {
        let mut text = root
            .get("responsum")
            .and_then(Json::text)
            .ok_or("the answer is not text")?
            .to_string();
        if declared == Outcome::Pars && !text.ends_with(annulus::officia::INCOMPLETE) {
            text.push_str(annulus::officia::INCOMPLETE);
        }
        return Ok((declared, Actum::Responsum(text)));
    }
    let Some(Json::List(items)) = root.get("opera") else {
        return Err("the work is not a list".into());
    };
    let effects = items
        .iter()
        .map(|item| {
            let Json::Object(fields) = item else {
                return Err("a work that is not an object");
            };
            if fields.len() != 3
                || ["verbum", "locus", "data"]
                    .iter()
                    .any(|k| !fields.contains_key(*k))
            {
                return Err("a work must have exactly \"verbum\", \"locus\" and \"data\"");
            }
            let text = |k: &str| item.get(k).and_then(Json::text).map(str::to_string);
            let verb = text("verbum")
                .and_then(|w| VERBS.iter().find(|(n, _)| *n == w).map(|(_, v)| *v))
                .ok_or("a work with an unknown verb")?;
            Ok(Effect {
                verb,
                place: text("locus").ok_or("a work without a place")?,
                data: text("data").ok_or("a work without data")?,
            })
        })
        .collect::<Result<Vec<_>, &str>>()?;
    if effects.is_empty() && declared != Outcome::Pars {
        return Err("the work is empty, yet declared whole".into());
    }
    Ok((declared, Actum::Opera(effects)))
}
