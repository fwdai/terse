use once_cell::sync::Lazy;
use regex::Regex;

// These patterns detect sentence-start phrases without lookbehind.
// Strategy: capture the preceding sentence-end + optional whitespace as group 1,
// then replace the whole match with just group 1 (restoring the sentence boundary).
// Pattern: (^|[.!?])\s{0,3} followed by the phrase to strip, then :\s*
//
// We use (?m) for multiline so ^ matches start of each line.
pub static STRUCTURAL: Lazy<Vec<Regex>> = Lazy::new(|| {
    let patterns = [
        // Here's / Here is/are/was/were <noun phrase>:
        r#"(?im)((?:^|[.!?])\s{0,3})Here(?:'s|\u{2019}s| (?:is|are|was|were)) [^:\n]{0,80}:\s*"#,
        // Below is/are <noun phrase>:
        r#"(?im)((?:^|[.!?])\s{0,3})Below (?:is|are) [^:\n]{0,80}:\s*"#,
        // The following ...:
        r#"(?im)((?:^|[.!?])\s{0,3})The following\s+[^:\n]{0,80}:\s*"#,
        // I've/I have outlined/listed/...:
        r#"(?im)((?:^|[.!?])\s{0,3})I(?:'ve|\u{2019}ve| have) (?:outlined|listed|summarized|described|detailed|provided) [^:\n]{0,80}:\s*"#,
        // To summarize / In summary / etc.
        r#"(?im)((?:^|[.!?])\s{0,3})(?:To summarize|In summary|In conclusion|To conclude|To recap|To sum up|In short|In brief|In closing|To wrap up)[,:]?\s*"#,
        // To begin/start:
        r#"(?im)((?:^|[.!?])\s{0,3})To (?:begin|start)(?:\s+with)?[,:]\s*"#,
        // First things first:
        r#"(?im)((?:^|[.!?])\s{0,3})First things first[,:]\s*"#,
        // To explain/clarify/illustrate:
        r#"(?im)((?:^|[.!?])\s{0,3})(?:To explain|To clarify|To illustrate)[,:]\s*"#,
        // Put differently / simply put:
        r#"(?im)((?:^|[.!?])\s{0,3})(?:Put differently|Put simply|Simply put)[,:]?\s*"#,
        // TL;DR / Bottom line / Note / Key takeaways:
        r#"(?im)((?:^|[.!?])\s{0,3})(?:TL;DR|Bottom line|Note|Key takeaways?)[,:]\s*"#,
    ];
    patterns
        .iter()
        .map(|p| Regex::new(p).expect("invalid structural regex"))
        .collect()
});

pub fn remove_structural_meta_commentary(text: &str) -> String {
    let mut result = text.to_string();
    for pat in STRUCTURAL.iter() {
        // Replace with just the captured sentence-boundary prefix (group 1)
        result = pat.replace_all(&result, "$1").into_owned();
    }
    result
}
