use once_cell::sync::Lazy;
use regex::Regex;

static NEWLINES: Lazy<Regex> = Lazy::new(|| Regex::new(r"\n+").unwrap());
static MULTI_SPACE: Lazy<Regex> = Lazy::new(|| Regex::new(r"[ \t]+").unwrap());
static ELLIPSIS: Lazy<Regex> = Lazy::new(|| Regex::new(r"\.{3}").unwrap());
static MULTI_BANG: Lazy<Regex> = Lazy::new(|| Regex::new(r"!{2,}").unwrap());
static MULTI_QUESTION: Lazy<Regex> = Lazy::new(|| Regex::new(r"\?{2,}").unwrap());

pub fn normalize_whitespace(text: &str) -> String {
    let result = NEWLINES.replace_all(text, " ");
    let result = MULTI_SPACE.replace_all(&result, " ");
    let result = ELLIPSIS.replace_all(&result, "\u{2026}");
    let result = MULTI_BANG.replace_all(&result, "!");
    let result = MULTI_QUESTION.replace_all(&result, "?");
    result.trim().to_string()
}
