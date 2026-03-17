use once_cell::sync::Lazy;
use regex::Regex;

// Sentence-start filler words.
// Strategy: capture the preceding sentence-end + optional whitespace as group 1,
// then replace with just group 1 (preserving the sentence boundary).
// (?im): case-insensitive + multiline so ^ matches line starts.
static SENTENCE_START_FILLERS: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"(?im)((?:^|[.!?])\s{0,3})(basically|essentially|simply|obviously|clearly|literally|honestly|actually|frankly|truthfully|personally|genuinely|admittedly|undoubtedly|certainly|naturally|importantly|notably|interestingly|surprisingly|unsurprisingly|thankfully|fortunately|unfortunately|luckily|of course),?\s+"
    )
    .expect("invalid filler regex")
});

pub fn remove_fillers(text: &str) -> String {
    // Replace with group 1 (the sentence-start boundary) to preserve structure
    SENTENCE_START_FILLERS
        .replace_all(text, "$1")
        .into_owned()
}
