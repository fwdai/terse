// Boilerplate patterns for user turns.
// Add new openers/closers here as they're discovered.

export const USER_OPENERS: RegExp[] = [
  /^Please[,.]?\s+/i,
  /^Kindly[,.]?\s+/i,

  // Self-deprecating hedges
  /^(Sorry|Apologies)[,.]?\s+if this is( a)? (dumb|silly|basic|stupid|obvious) question[,.]?\s*(but\s+)?/i,
  /^(Sorry|Apologies)[,.]?\s+for (the |a )?(dumb|silly|basic|stupid) question[,.]?\s*(but\s+)?/i,
  /^This might be obvious[,.]?\s+but\s*/i,
  /^Forgive me if (I['']m wrong|this is obvious)[,.]?\s+but\s*/i,
  /^Not sure if this is (relevant|the right place)[,.]?\s+but\s*/i,

  // "I have a question about X" → keep X
  /^I (have|had) a (quick\s+)?(question|query)[,.]?\s*(about|on|regarding)?\s*/i,

  // Request preambles — remainder is re-capitalized after removal
  /^(Can|Could) you (please\s+)?(help me\s+)?/i,
  /^Would you (be able to|mind\s+)/i,
  /^I(['']d like| want| need)( you)? to\s+/i,
  /^I was wondering if you could\s+/i,
  /^Is it possible( for you)? to\s+/i,
];

// [,]? — consumes a preceding comma but NOT a period (preserves sentence-ending punctuation)
export const USER_CLOSERS: RegExp[] = [
  /[,]?\s*[Tt]hank(s| you)( (so much|a lot|in advance))?[.!]?\s*$/i,
  /[,]?\s*[Ii] appreciate (it|your help|your (time|assistance))[.!]?\s*$/i,
  /[,]?\s*[Tt]hanks for (your help|helping( me)?)[.!]?\s*$/i,
  /[,]?\s*[Pp]lease let me know if you need (more|any) (info|information|details|clarification)[.!]?\s*$/i,
  /[,]?\s*[Ff]eel free to (ask|let me know)[^.]*[.!]?\s*$/i,
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
