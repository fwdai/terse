// Boilerplate patterns for user turns.
// Add new openers/closers here as they're discovered.

export const USER_OPENERS: RegExp[] = [
  // Greetings — longer/name-taking forms first so "Hi Claude," beats "Hi,"
  /^Good (morning|afternoon|evening)[,!.]?\s*/i,
  /^(Howdy|Greetings)[,!.]?\s*/i,
  /^(Hey|Hi|Hello)\s+\w+[,.!]\s*/i,          // "Hi Claude, " / "Hey there, "
  /^(Hey|Hi|Hello)[,!.]?\s+/i,               // "Hi, " / "Hey! " / "Hello "

  // Short acknowledgment / continuation starters
  /^(Okay|OK|Alright|Right|Well|Now)[,.]\s*/i,
  /^So[,.]\s*/i,
  /^Great[,.]\s*/i,                           // "Great, can you also..." (acknowledgment, not compliment)
  /^(Wait|Oh)[,!.]?\s*/i,
  /^(Hmm+|Hm)[,!.…]?\s*/i,
  /^(Also|Additionally)[,.]\s*/i,
  /^And[,.]\s*/i,

  // Quick/follow-up question labels
  /^Quick question[,:!]?\s*/i,
  /^Just a quick (question|one)[,:!]?\s*/i,
  /^(One|Another) (more|quick) (thing|question)[,:!]?\s*/i,
  /^Follow[- ]?up (question\s*)?[,:!]?\s*/i,
  /^(Newbie|Beginner|Novice|Silly|Dumb|Stupid|Random|Side|Hypothetical) question[,:!]?\s*/i,

  /^Please[,.]?\s+/i,
  /^Kindly[,.]?\s+/i,

  // Self-deprecating hedges
  /^(Sorry|Apologies)[,.]?\s+if this is( a)? (dumb|silly|basic|stupid|obvious) question[,.]?\s*(but\s+)?/i,
  /^(Sorry|Apologies)[,.]?\s+for (the |a )?(dumb|silly|basic|stupid) question[,.]?\s*(but\s+)?/i,
  /^This might be obvious[,.]?\s+(but\s+)?/i,
  /^I know this (might|may|could) be (obvious|simple|basic|silly|dumb)[,.]?\s*(but\s+)?/i,
  /^Forgive me if (I['']m wrong|this is obvious)[,.]?\s+(but\s+)?/i,
  /^Not sure if this is (relevant|the right place)[,.]?\s+(but\s+)?/i,
  /^I (might|may|could) be (wrong|mistaken)[,.]?\s+(but\s+)?/i,
  /^Correct me if I['']m wrong[,.]?\s+(but\s+)?/i,
  /^I['']m (probably |likely )?(wrong|mistaken)[,.]?\s+(but\s+)?/i,
  /^I['']m not sure (if|whether) this is (right|correct|relevant)[,.]?\s*(but\s+)?/i,

  // "I have a question about X" → keep X
  /^I (have|had) a (quick\s+)?(question|query)[,.]?\s*(about|on|regarding)?\s*/i,

  // Availability-conditional requests — always polite filler in LLM context
  /^If you have (a moment|a minute|time|a chance)[,.]?\s*(could|can|would) you\s+/i,
  /^When(ever)? you (have|get) (a moment|a minute|a chance)[,.]?\s*(could|can|would) you\s+/i,

  // Request preambles — remainder is re-capitalized after removal
  /^Do you mind\s+/i,
  /^(Could|May) I ask you to\s+/i,
  /^I['']d appreciate (it )?if you could\s+/i,
  /^Do you think you could\s+/i,
  /^Would it be possible( for you)? to\s+/i,
  /^I(?:['']m| was| am) (just )?wondering if you could\s+/i,
  /^I was hoping (you could|you can)\s+/i,
  /^I['']m hoping (you could|you can)\s+/i,
  /^(Can|Could) you (please\s+)?(help me\s+)?/i,
  /^Would you (be able to|mind\s+)/i,
  /^I(['']d like| want| need)( you)? to\s+/i,
  /^Is it possible( for you)? to\s+/i,
];

// [,]? — consumes a preceding comma but NOT a period (preserves sentence-ending punctuation)
export const USER_CLOSERS: RegExp[] = [
  // Gratitude — longer/specific forms first to prevent partial matches
  /[,]?\s*[Mm]any [Tt]hanks[.!]?\s*$/i,
  /[,]?\s*[Mm]uch appreciated[.!]?\s*$/i,
  /[,]?\s*[Cc]heers[.!]?\s*$/i,
  /[,]?\s*[Tt]hank(s| you)( (very much|so much|a lot|in advance|again|for everything))?[.!]?\s*$/i,
  /[,]?\s*[Ii] (really |truly |greatly )?appreciate (it|your (help|time|assistance|patience))[.!]?\s*$/i,
  /[,]?\s*[Tt]hanks for (your (help|time|assistance|patience)|helping( me)?)[.!]?\s*$/i,

  // Hope closers — user wrapping up context/code they've provided
  /[,]?\s*[Hh]ope (this|that)\s+[^.!?]{0,60}[.!?]?\s*$/i,

  // Offering more info / follow-up — generalized "Let me know if ..."
  /[,]?\s*([Pp]lease\s+|[Jj]ust\s+)?[Ll]et me know if\s+[^.!?]{0,80}[.!?]?\s*$/i,
  /[,]?\s*[Hh]appy to (provide more (details|context|info(rmation)?)|elaborate|clarify|share more)[^.!?]{0,60}[.!?]?\s*$/i,
  /[,]?\s*[Ff]eel free to (ask|let me know)[^.]*[.!]?\s*$/i,

  // Apology closers
  /[,]?\s*[Ss]orry (for (the )?(long|lengthy) (message|post|explanation|question)|if (that['']?s|this is) (confusing|unclear|too long))[.!]?\s*$/i,
];

// After stripping "Can you help ", "figuring out" → "Figure out"
const GERUND_TO_IMPERATIVE = /^([a-z]+ing)\b/;

export function removeUserBoilerplate(text: string): string {
  let result = text;

  for (const p of USER_CLOSERS) result = result.replace(p, '');

  for (const p of USER_OPENERS) {
    const stripped = result.replace(p, '').trimStart();
    if (stripped !== result.trimStart()) {
      result = stripped.charAt(0).toUpperCase() + stripped.slice(1);
    }
  }

  result = result.replace(GERUND_TO_IMPERATIVE, (gerund) => {
    const stem = gerund.replace(/ing$/, '');
    return stem.charAt(0).toUpperCase() + stem.slice(1);
  });

  return result.trim();
}
