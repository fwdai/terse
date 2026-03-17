pub mod vocab;

use once_cell::sync::Lazy;
use regex::Regex;
use vocab::{INTENSIFIERS, NOUN_SYNONYMS, VERB_SYNONYMS};

// Remove articles: a, an, the — word boundary followed by space then alpha.
// Capture the letter after the space so we can restore it (no lookahead in regex crate).
static ARTICLE_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?i)\b(?:a|an|the)\s+([A-Za-z])").unwrap());

static INTENSIFIER_RE: Lazy<Regex> = Lazy::new(|| {
    let alts = INTENSIFIERS.join("|");
    Regex::new(&format!(r"(?i)\b(?:{})\s+", alts)).unwrap()
});

/// Pre-compiled (long, short, Regex) triples for verb synonyms.
static VERB_SYNONYM_RES: Lazy<Vec<(Regex, &'static str)>> = Lazy::new(|| {
    VERB_SYNONYMS
        .iter()
        .map(|(long, short)| {
            let re = Regex::new(&format!(r"(?i)\b{}\b", regex::escape(long)))
                .expect("invalid verb synonym regex");
            (re, *short)
        })
        .collect()
});

/// Pre-compiled (long, short, Regex) triples for noun synonyms.
static NOUN_SYNONYM_RES: Lazy<Vec<(Regex, &'static str)>> = Lazy::new(|| {
    NOUN_SYNONYMS
        .iter()
        .map(|(long, short)| {
            let re = Regex::new(&format!(r"(?i)\b{}\b", regex::escape(long)))
                .expect("invalid noun synonym regex");
            (re, *short)
        })
        .collect()
});

pub fn apply_nlp(text: &str, _role: &str) -> String {
    let mut result = text.to_string();

    // Remove articles — keep the captured first letter of the following word
    result = ARTICLE_RE.replace_all(&result, "$1").into_owned();

    // Remove intensifiers
    result = INTENSIFIER_RE.replace_all(&result, "").into_owned();

    // Apply verb synonyms
    for (re, short) in VERB_SYNONYM_RES.iter() {
        result = re.replace_all(&result, *short).into_owned();
    }

    // Apply noun synonyms
    for (re, short) in NOUN_SYNONYM_RES.iter() {
        result = re.replace_all(&result, *short).into_owned();
    }

    result
}
