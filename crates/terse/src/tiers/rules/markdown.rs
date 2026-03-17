use fancy_regex::Regex as FancyRegex;
use once_cell::sync::Lazy;
use regex::Regex;

static HR: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?m)^[ \t]*[-*_]{3,}[ \t]*$").unwrap());
static TABLE_SEP: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?m)^\|[\s\-:|]+\|\s*$").unwrap());
static HEADING: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?m)^#{1,6} ").unwrap());
static BOLD: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\*\*([^*\n]+)\*\*").unwrap());
// Italic: requires lookahead/lookbehind — use fancy-regex
static ITALIC: Lazy<FancyRegex> =
    Lazy::new(|| FancyRegex::new(r"(?<!\*)\*([^\s*\n][^*\n]*?)\*(?!\*)").unwrap());
static EM_DASH: Lazy<Regex> =
    Lazy::new(|| Regex::new(r" \u{2014} ").unwrap());
static CURLY_DOUBLE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"[\u{201C}\u{201D}]").unwrap());
static CURLY_SINGLE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"[\u{2018}\u{2019}]").unwrap());

pub fn strip_markdown_formatting(text: &str) -> String {
    let result = HR.replace_all(text, "");
    let result = TABLE_SEP.replace_all(&result, "");
    let result = HEADING.replace_all(&result, "");
    let result = BOLD.replace_all(&result, "$1");
    let result = ITALIC.replace_all(&result, "$1");
    let result = EM_DASH.replace_all(&result, ", ");
    let result = CURLY_DOUBLE.replace_all(&result, "\"");
    let result = CURLY_SINGLE.replace_all(&result, "'");
    result.into_owned()
}
