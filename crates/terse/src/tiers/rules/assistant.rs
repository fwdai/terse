use once_cell::sync::Lazy;
use regex::Regex;

pub static ASSISTANT_OPENERS: Lazy<Vec<Regex>> = Lazy::new(|| {
  let patterns = [
    r"(?i)^(Certainly|Sure|Absolutely|Of course|Indeed)[!,.]?\s*",
    r"(?i)^(No problem|Happy to help|Glad to help|Got it|Understood|Perfect|Excellent|Awesome|Wonderful)[!,.]?\s*",
    r"(?i)^(That['\u{2019}]s a great|What a great) (question|point|observation|catch)[!,.]?\s*",
    r"(?i)^(Great|Good|Excellent|Nice) (question|point|observation|catch)[!,.]?\s*",
    r"(?i)^I['\u{2019}]d be (happy|glad|delighted) to (help|assist)[^.]*[.!]\s*",
    r"(?i)^I['\u{2019}]m (happy|glad) to (help|assist)[^.]*[.!]\s*",
    r"(?i)^I (understand|see)[.!]\s*",
    r"(?i)^I understand what you['\u{2019}]re [^.]{0,50}[.!]\s*",
    r"(?i)^I see what you (mean|are (saying|getting at))[.!]\s*",
    r"(?i)^That (makes sense|is correct|sounds right)[.!]\s*",
    r"(?i)^That['\u{2019}]s (a )?(good|great|valid|fair|reasonable) (point|question|observation|catch|approach)[!,.]?\s*",
    r"(?i)^You['\u{2019}]re (absolutely |totally |completely )?right[.!]\s*",
    r"(?i)^Let me (help you with (that|this)|take care of (that|this))[.!]\s*",
    r"(?i)^Let me walk you through (this|that)[.!]\s*",
    r"(?i)^Let me (explain|clarify)(\.|\!| that\.| this\.)\s*",
    r"(?i)^Let me break (this|that) down[.!]\s*",
    r"(?i)^Let me (take a look|look into (this|that))[.!]\s*",
    r"(?i)^Let me address (that|this|your question)[.!]\s*",
    r"(?i)^I['\u{2019}]ll (help you with (that|this)|take care of (that|this)|get (that|this) done)[.!]\s*",
    r"(?i)^I (can|will) (help|do that|help you with (that|this))[.!]\s*",
    r"(?i)^Based on (what you['\u{2019}]ve (described|shared|said|provided)|your (question|message|description|code|error))[,.]?\s*",
    r"(?i)^Looking at (this|that|your (code|error|message|question|issue|problem))[,.]?\s*",
    r"(?i)^From what (you['\u{2019}]ve (described|shared|said)|I can see)[,.]?\s*",
    r"(?i)^To (answer|address) your question[,.]?\s*",
    r"(?i)^In (response|answer) to your question[,.]?\s*",
    r"(?i)^As an AI (language model|assistant)[,.]?\s*",
    r"(?i)^Thank you for (your question|asking|reaching out)[^.]*[.!]\s*",
  ];
  patterns
      .iter()
      .map(|p| Regex::new(p).expect("invalid assistant opener regex"))
      .collect()
});

pub static ASSISTANT_CLOSERS: Lazy<Vec<Regex>> = Lazy::new(|| {
  let patterns = [
    r"(?i)\s*I hope this (helps|answers|clarifies)[^.]*[.!]?\s*$",
    r"(?i)\s*Hope (this|that) [^.!?]{0,60}[.!]?\s*$",
    r"(?i)\s*Let me know\b[^.!?]{0,80}[.!?]?\s*$",
    r"(?i)\s*Feel free to (ask|reach out|let me know|contact me)[^.]*[.!?]\s*$",
    r"(?i)\s*If you (have|need) (any |more )?(questions?|help|issues?|problems?|anything)[^.]*[.!]\s*$",
    r"(?i)\s*(Happy|Glad) to (help|clarify|assist)( further| more| with (that|anything))?[.!]\s*$",
    r"(?i)\s*Is there anything else[^.]*[?!]\s*$",
    r"(?i)\s*Don['\u{2019}]t hesitate to[^.]*[.!]\s*$",
    r"(?i)\s*Please (let me know|feel free)[^.]*[.!]\s*$",
    r"(?i)\s*Does (that|this) (make sense|help|work( for you)?)\??[.!]?\s*$",
    r"(?i)\s*That should (do it|work( for you)?)[.!]\s*$",
    r"(?i)\s*(Give (that|this) a try|Try (that|this) out)[^.]*[.!]\s*$",
    r"(?i)\s*(Shall I|Should I|Would you like me to)[^?]*\??\s*$",
  ];
  patterns
    .iter()
    .map(|p| Regex::new(p).expect("invalid assistant closer regex"))
    .collect()
});

pub fn remove_assistant_boilerplate(text: &str) -> String {
  let mut result = text.to_string();

  // Strip closers first
  for pat in ASSISTANT_CLOSERS.iter() {
    result = pat.replace(&result, "").into_owned();
  }

  // Strip openers
  for pat in ASSISTANT_OPENERS.iter() {
    let stripped = pat.replace(&result, "").into_owned();
    if stripped != result {
        result = stripped;
        break;
    }
  }

  result
}
