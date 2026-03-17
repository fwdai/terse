use once_cell::sync::Lazy;
use regex::Regex;

pub static USER_OPENERS: Lazy<Vec<Regex>> = Lazy::new(|| {
    let patterns = [
        r"(?i)^Good (morning|afternoon|evening)[,!.]?\s*",
        r"(?i)^(Howdy|Greetings)[,!.]?\s*",
        r"(?i)^(Hey|Hi|Hello)\s+\w+[,.!]\s*",
        r"(?i)^(Hey|Hi|Hello)[,!.]?\s+",
        r"(?i)^(Okay|OK|Alright|Right|Well|Now)[,.]\s*",
        r"(?i)^So[,.]\s*",
        r"(?i)^Great[,.]\s*",
        r"(?i)^(Wait|Oh)[,!.]?\s*",
        r"(?i)^(Hmm+|Hm)[,!.\u{2026}]?\s*",
        r"(?i)^(Also|Additionally)[,.]\s*",
        r"(?i)^And[,.]\s*",
        r"(?i)^Quick question[,:!]?\s*",
        r"(?i)^Just a quick (question|one)[,:!]?\s*",
        r"(?i)^(One|Another) (more|quick) (thing|question)[,:!]?\s*",
        r"(?i)^Follow[- ]?up (question\s*)?[,:!]?\s*",
        r"(?i)^(Newbie|Beginner|Novice|Silly|Dumb|Stupid|Random|Side|Hypothetical) question[,:!]?\s*",
        r"(?i)^Please[,.]?\s+",
        r"(?i)^Kindly[,.]?\s+",
        r"(?i)^(Sorry|Apologies)[,.]?\s+if this is( a)? (dumb|silly|basic|stupid|obvious) question[,.]?\s*(but\s+)?",
        r"(?i)^(Sorry|Apologies)[,.]?\s+for (the |a )?(dumb|silly|basic|stupid) question[,.]?\s*(but\s+)?",
        r"(?i)^This might be obvious[,.]?\s+(but\s+)?",
        r"(?i)^I know this (might|may|could) be (obvious|simple|basic|silly|dumb)[,.]?\s*(but\s+)?",
        r"(?i)^Forgive me if (I['\u{2019}]m wrong|this is obvious)[,.]?\s+(but\s+)?",
        r"(?i)^Not sure if this is (relevant|the right place)[,.]?\s+(but\s+)?",
        r"(?i)^I (might|may|could) be (wrong|mistaken)[,.]?\s+(but\s+)?",
        r"(?i)^Correct me if I['\u{2019}]m wrong[,.]?\s+(but\s+)?",
        r"(?i)^I['\u{2019}]m (probably |likely )?(wrong|mistaken)[,.]?\s+(but\s+)?",
        r"(?i)^I['\u{2019}]m not sure (if|whether) this is (right|correct|relevant)[,.]?\s*(but\s+)?",
        r"(?i)^I (have|had) a (quick\s+)?(question|query)[,.]?\s*(about|on|regarding)?\s*",
        r"(?i)^If you have (a moment|a minute|time|a chance)[,.]?\s*(could|can|would) you\s+",
        r"(?i)^When(ever)? you (have|get) (a moment|a minute|a chance)[,.]?\s*(could|can|would) you\s+",
        r"(?i)^Do you mind\s+",
        r"(?i)^(Could|May) I ask you to\s+",
        r"(?i)^I['\u{2019}]d appreciate (it )?if you could\s+",
        r"(?i)^Do you think you could\s+",
        r"(?i)^Would it be possible( for you)? to\s+",
        r"(?i)^I(?:['\u{2019}]m| was| am) (just )?wondering if you could\s+",
        r"(?i)^I was hoping (you could|you can)\s+",
        r"(?i)^I['\u{2019}]m hoping (you could|you can)\s+",
        r"(?i)^(Can|Could) you (please\s+)?(help me\s+)?",
        r"(?i)^Would you (be able to|mind\s+)",
        r"(?i)^I(['\u{2019}]d like| want| need)( you)? to\s+",
        r"(?i)^Is it possible( for you)? to\s+",
    ];
    patterns
        .iter()
        .map(|p| Regex::new(p).expect("invalid user opener regex"))
        .collect()
});

pub static USER_CLOSERS: Lazy<Vec<Regex>> = Lazy::new(|| {
    let patterns = [
        r"(?i)[,]?\s*[Mm]any [Tt]hanks[.!]?\s*$",
        r"(?i)[,]?\s*[Mm]uch appreciated[.!]?\s*$",
        r"(?i)[,]?\s*[Cc]heers[.!]?\s*$",
        r"(?i)[,]?\s*[Tt]hank(s| you)( (very much|so much|a lot|in advance|again|for everything))?[.!]?\s*$",
        r"(?i)[,]?\s*[Ii] (really |truly |greatly )?appreciate (it|your (help|time|assistance|patience))[.!]?\s*$",
        r"(?i)[,]?\s*[Tt]hanks for (your (help|time|assistance|patience)|helping( me)?)[.!]?\s*$",
        r"(?i)[,]?\s*[Hh]ope (this|that)\s+[^.!?]{0,60}[.!?]?\s*$",
        r"(?i)[,]?\s*([Pp]lease\s+|[Jj]ust\s+)?[Ll]et me know if\s+[^.!?]{0,80}[.!?]?\s*$",
        r"(?i)[,]?\s*[Hh]appy to (provide more (details|context|info(rmation)?)|elaborate|clarify|share more)[^.!?]{0,60}[.!?]?\s*$",
        r"(?i)[,]?\s*[Ff]eel free to (ask|let me know)[^.]*[.!]?\s*$",
        r"(?i)[,]?\s*[Ss]orry (for (the )?(long|lengthy) (message|post|explanation|question)|if (that['\u{2019}]?s|this is) (confusing|unclear|too long))[.!]?\s*$",
    ];
    patterns
        .iter()
        .map(|p| Regex::new(p).expect("invalid user closer regex"))
        .collect()
});

static GERUND_TO_IMPERATIVE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^([a-z]+ing)\b").expect("invalid gerund regex"));

fn capitalize_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

/// Naive English gerund→stem: strips "ing" with basic e-restoration.
/// e.g. "figuring" → "figure", "running" → "run", "making" → "make"
fn gerund_to_imperative(gerund: &str) -> String {
    let lower = gerund.to_lowercase();
    let stem_raw = lower.trim_end_matches("ing");

    // Double-consonant: "running" → "runn" → "run"
    let bytes = stem_raw.as_bytes();
    let stem = if bytes.len() >= 2 && bytes[bytes.len() - 1] == bytes[bytes.len() - 2] {
        let s = &stem_raw[..stem_raw.len() - 1];
        // Only de-double if it was a consonant (not vowel doubling)
        let last = s.chars().last().unwrap_or('x');
        if !"aeiou".contains(last) {
            s
        } else {
            stem_raw
        }
    } else {
        stem_raw
    };

    // If stem ends in consonant cluster that likely needs trailing 'e',
    // check if original verb likely had trailing 'e' (e.g. "figur" → "figure")
    // Simple heuristic: if stem ends in a consonant that forms common -e verbs
    let imperative = if !stem.ends_with('e') {
        let last = stem.chars().last().unwrap_or('x');
        // Common suffixes that need 'e' back: -t, -d, -v, -s, -z, -c, -g, -k, -l, -n, -r after certain patterns
        // We'll use a simple vowel-consonant-e heuristic
        let needs_e = matches!(last, 'v' | 'z' | 'c') || {
            let stem_bytes = stem.as_bytes();
            stem_bytes.len() >= 2
                && !"aeiou".contains(last)
                && "aeiou".contains(stem_bytes[stem_bytes.len() - 2] as char)
        };
        if needs_e {
            format!("{}e", stem)
        } else {
            stem.to_string()
        }
    } else {
        stem.to_string()
    };

    capitalize_first(&imperative)
}

pub fn remove_user_boilerplate(text: &str) -> String {
    let mut result = text.to_string();

    // Strip closers
    for pat in USER_CLOSERS.iter() {
        result = pat.replace(&result, "").into_owned();
    }

    // Strip openers — iterate all patterns, capitalizing after each match.
    // Matches TS: `const stripped = result.replace(p,'').trimStart(); if (stripped !== result.trimStart()) { result = stripped[0].toUpperCase() + stripped.slice(1); }`
    for pat in USER_OPENERS.iter() {
        let current = result.trim_start().to_string();
        let stripped = pat.replace(&current, "").into_owned();
        if stripped != current {
            result = capitalize_first(stripped.trim_start());
        }
    }

    // Gerund-to-imperative on the result
    if let Some(cap) = GERUND_TO_IMPERATIVE.captures(&result) {
        let gerund = &cap[1];
        let imperative = gerund_to_imperative(gerund);
        result = GERUND_TO_IMPERATIVE
            .replace(&result, imperative.as_str())
            .into_owned();
    }

    result.trim().to_string()
}
