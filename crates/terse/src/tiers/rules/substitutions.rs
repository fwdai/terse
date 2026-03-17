use once_cell::sync::Lazy;
use regex::Regex;

/// Each entry: (pattern, replacement)
/// Patterns use word boundaries and case-insensitive flag where needed.
pub static SUBSTITUTIONS: Lazy<Vec<(Regex, &'static str)>> = Lazy::new(|| {
    let entries: &[(&str, &str)] = &[
        // Purpose clauses
        (r"(?i)\bwith the intention of\b", "to"),
        (r"(?i)\bwith the goal of\b", "to"),
        (r"(?i)\bwith the aim of\b", "to"),
        (r"(?i)\bin an effort to\b", "to"),
        (r"(?i)\bin an attempt to\b", "to"),
        (r"(?i)\bso as to\b", "to"),
        (r"(?i)\bin order to\b", "to"),
        (r"(?i)\bin order for\b", "for"),
        (r"(?i)\bfor the sake of\b", "for"),
        (r"(?i)\bfor the purpose of\b", "for"),
        (r"(?i)\bprovided that\b", "if"),

        // Ability
        (r"(?i)\bhas the ability to\b", "can"),
        (r"(?i)\bhave the ability to\b", "can"),
        (r"(?i)\bis able to\b", "can"),
        (r"(?i)\bare able to\b", "can"),
        (r"(?i)\bwas able to\b", "could"),
        (r"(?i)\bwere able to\b", "could"),
        (r"(?i)\bit is possible that\b", "maybe"),
        (r"(?i)\bthere is a possibility that\b", "maybe"),

        // Causal
        (r"(?i)\bin spite of the fact that\b", "although"),
        (r"(?i)\bregardless of the fact that\b", "although"),
        (r"(?i)\bdespite the fact that\b", "although"),
        (r"(?i)\bin light of the fact that\b", "since"),
        (r"(?i)\bgiven the fact that\b", "since"),
        (r"(?i)\bowing to the fact that\b", "because"),
        (r"(?i)\bdue to the fact that\b", "because"),
        (r"(?i)\bon the grounds that\b", "because"),
        (r"(?i)\bfor the reason that\b", "because"),
        (r"(?i)\bas a consequence of\b", "due to"),
        (r"(?i)\bas a result of\b", "due to"),
        (r"(?i)\bin the event that\b", "if"),
        (r"(?i)\bin contrast (?:to|with)\b", "unlike"),
        (r"(?i)\bin comparison (?:to|with)\b", "vs."),
        (r"(?i)\bon the other hand\b", "however"),

        // Temporal
        (r"(?i)\bat this point in time\b", "now"),
        (r"(?i)\bat the present time\b", "now"),
        (r"(?i)\bat the moment\b", "now"),
        (r"(?i)\bat the end of the day\b", "ultimately"),
        (r"(?i)\bin the long run\b", "ultimately"),
        (r"(?i)\bfor the foreseeable future\b", "for now"),
        (r"(?i)\bfor the time being\b", "for now"),
        (r"(?i)\bin the near future\b", "soon"),
        (r"(?i)\bin the future\b", "later"),
        (r"(?i)\bin the meantime\b", "meanwhile"),
        (r"(?i)\bsooner or later\b", "eventually"),
        (r"(?i)\bat some point\b", "eventually"),
        (r"(?i)\bfrom time to time\b", "sometimes"),
        (r"(?i)\bat all times\b", "always"),
        (r"(?i)\bprior to\b", "before"),
        (r"(?i)\bsubsequent to\b", "after"),
        (r"(?i)\bon a daily basis\b", "daily"),
        (r"(?i)\bon a weekly basis\b", "weekly"),
        (r"(?i)\bon a monthly basis\b", "monthly"),
        (r"(?i)\bon a regular basis\b", "regularly"),

        // Quantity
        (r"(?i)\bthe vast majority of\b", "most"),
        (r"(?i)\ba significant number of\b", "many"),
        (r"(?i)\ba large number of\b", "many"),
        (r"(?i)\ba(?:n increasing| growing) number of\b", "more"),
        (r"(?i)\ba number of\b", "several"),
        (r"(?i)\ba wide variety of\b", "many"),
        (r"(?i)\ba wide range of\b", "many"),
        (r"(?i)\ba variety of\b", "various"),
        (r"(?i)\ba handful of\b", "few"),
        (r"(?i)\bthe majority of\b", "most"),
        (r"(?i)\beach and every\b", "every"),
        (r"(?i)\bfor the most part\b", "mostly"),
        (r"(?i)\bby and large\b", "mostly"),
        (r"(?i)\bwith the exception of\b", "except"),
        (r"(?i)\bin addition to\b", "beyond"),
        (r"(?i)\bin addition\b", "also"),

        // Connectives
        (r"(?i)\bin close proximity to\b", "near"),
        (r"(?i)\bwhen it comes to\b", "for"),
        (r"(?i)\bby means of\b", "via"),
        (r"(?i)\bin the process of\b", "while"),
        (r"(?i)\bon the basis of\b", "based on"),
        (r"(?i)\bwith regards? to\b", "re:"),
        (r"(?i)\bwith respect to\b", "re:"),
        (r"(?i)\bin regard to\b", "re:"),
        (r"(?i)\bin terms of\b", "for"),
        (r"(?i)\bin the same way\b", "similarly"),
        (r"(?i)\bin a similar manner\b", "similarly"),
        (r"(?i)\bin a similar way\b", "similarly"),
        (r"(?i)\bthe way in which\b", "how"),
        (r"(?i)\bthe reason why\b", "why"),
        (r"(?i)\bas well as\b", "and"),
        (r"(?i)\bfirst and foremost\b", "first"),
        (r"(?i)\bfirst of all\b", "first"),
        (r"(?i)\blast but not least\b", "finally"),
        (r"(?i)\bthere(?:'s|\u{2019}s| is) no need (to|for)\b", "no need $1"),
        (r"(?i)\bin any case\b", "anyway"),
        (r"(?i)\bin that case\b", "then"),

        // Action nominalizations
        (r"(?i)\bmakes? improvements to\b", "improve"),
        (r"(?i)\bmakes? modifications to\b", "modify"),
        (r"(?i)\bmakes? adjustments to\b", "adjust"),
        (r"(?i)\bmakes? updates to\b", "update"),
        (r"(?i)\bmakes? changes to\b", "change"),
        (r"(?i)\bmake a recommendation\b", "recommend"),
        (r"(?i)\bmake a suggestion\b", "suggest"),
        (r"(?i)\bmake a decision\b", "decide"),
        (r"(?i)\bmake use of\b", "use"),
        (r"(?i)\bhad an (?:impact|effect) on\b", "affected"),
        (r"(?i)\bhas an (?:impact|effect) on\b", "affects"),
        (r"(?i)\bhave an (?:impact|effect) on\b", "affect"),
        (r"(?i)\btake a look at\b", "check"),
        (r"(?i)\bhave a look at\b", "check"),
        (r"(?i)\btake a look\b", "check"),
        (r"(?i)\btake into consideration\b", "consider"),
        (r"(?i)\btake into account\b", "consider"),
        (r"(?i)\bgive consideration to\b", "consider"),
        (r"(?i)\bkeep in mind\b", "note"),
        (r"(?i)\bbear in mind\b", "note"),
        (r"(?i)\btake note of\b", "note"),
        (r"(?i)\btake advantage of\b", "use"),
        (r"(?i)\btake care of\b", "handle"),
        (r"(?i)\bkeep track of\b", "track"),
        (r"(?i)\bget rid of\b", "remove"),
        (r"(?i)\bgive rise to\b", "cause"),
        (r"(?i)\bprovide assistance(?: to)?\b", "help"),
        (r"(?i)\bcome to the conclusion\b", "conclude"),
        (r"(?i)\breach a conclusion\b", "conclude"),
        (r"(?i)\bcome to an agreement\b", "agree"),
        (r"(?i)\ball you (?:need|have) to do is ", ""),
        (r"(?i)\bwhat you (?:need|have) to do is ", ""),
        (r"(?i)\bgo ahead and ", ""),

        // Zero-value markers
        (r"(?i)\bas (?:previously |already )?mentioned(?:\s+(?:above|before|earlier))?\b[,.]?\s*", ""),
        (r"(?i)\bas noted(?:\s+(?:above|before|earlier))?\b[,.]?\s*", ""),
        (r"(?i)\bas we (?:discussed|noted|covered|mentioned)(?:\s+(?:above|before|earlier))?\b[,.]?\s*", ""),
        (r"(?i)\bit (?:is|should be) (?:worth noting|noted) that\b", ""),
        (r"(?i)\bit is important to note that\b", ""),
        (r"(?i)\bit['\u{2019}]s worth (?:noting|mentioning) that\b", ""),
        (r"(?i)\bit is (?:clear|evident|obvious) that\b", ""),
        (r"(?i)\bas you can see\b[,.]?\s*", ""),
        (r"(?i)\bit goes without saying(?: that)?\b[,.]?\s*", ""),
        (r"(?i)\bneedless to say\b[,.]?\s*", ""),
        (r"(?i)\bas a matter of fact\b[,.]?\s*", ""),
        (r"(?i)\bin actual fact\b[,.]?\s*", ""),
        (r"(?i)\bthat being said\b[,.]?\s*", ""),
        (r"(?i)\bwith that said\b[,.]?\s*", ""),
        (r"(?i)\bhaving said that\b[,.]?\s*", ""),
        (r"(?i)\bwhen all is said and done\b", "ultimately"),
        (r"(?i)\blong story short\b[,.]?\s*", ""),
        (r"(?i)\bto make a long story short\b[,.]?\s*", ""),
        (r"(?i)\bthe bottom line is that[,:]?\s*", ""),
        (r"(?i)\bthe bottom line is[,:]?\s*", ""),
        (r"(?i)\bsuffice it to say(?: that)?\b[,.]?\s*", ""),
        (r"(?i)\bto put it simply\b[,.]?\s*", ""),
        (r"(?i)\bwith (?:this|that) in mind\b[,.]?\s*", ""),
        (r"(?i)\bthe fact that\b", "that"),

        // Standard abbreviations
        (r"(?i)\bthat is to say\b", "i.e."),
        (r"(?i)\bin other words\b", "i.e."),
        (r"(?i)\bfor example\b", "e.g."),
        (r"(?i)\bfor instance\b", "e.g."),
        (r"(?i)\bet cetera\b", "etc."),
        (r"(?i)\band so (?:on|forth)\b", "etc."),
        (r"(?i)\bas opposed to\b", "vs."),
        (r"(?i)\bcompared (?:to|with)\b", "vs."),
        (r"(?i)\bversus\b", "vs."),
        (r"(?i)\bapproximately ", "~"),
    ];

    entries
        .iter()
        .map(|(pat, rep)| {
            let re = Regex::new(pat).expect("invalid substitution regex");
            (re, *rep)
        })
        .collect()
});

fn capitalize_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

pub fn apply_substitutions(text: &str) -> String {
    let mut result = text.to_string();
    for (pat, rep) in SUBSTITUTIONS.iter() {
        result = pat.replace_all(&result, *rep).into_owned();
    }
    // Trim leading space a deleted phrase may have left, then re-capitalize if needed.
    // Matches TS: `if (/^[a-z]/.test(result)) result = result[0].toUpperCase() + result.slice(1)`
    let trimmed = result.trim_start().to_string();
    if trimmed.chars().next().map(|c| c.is_lowercase()).unwrap_or(false) {
        capitalize_first(&trimmed)
    } else {
        trimmed
    }
}
