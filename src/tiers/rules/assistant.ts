// Boilerplate patterns for assistant turns.
// Add new openers/closers here as they're discovered.

export const ASSISTANT_OPENERS: RegExp[] = [
  // Single-word / short affirmations
  /^(Certainly|Sure|Absolutely|Of course|Indeed)[!,.]?\s*/i,
  /^(No problem|Happy to help|Glad to help|Got it|Understood|Perfect|Excellent|Awesome|Wonderful)[!,.]?\s*/i,

  // Complimentary
  /^(That['']s a great|What a great) (question|point|observation|catch)[!,.]?\s*/i,
  /^(Great|Good|Excellent|Nice) (question|point|observation|catch)[!,.]?\s*/i,

  // "I'd/I'm ... to help"
  /^I['']d be (happy|glad|delighted) to (help|assist)[^.]*[.!]\s*/i,
  /^I['']m (happy|glad) to (help|assist)[^.]*[.!]\s*/i,

  // Acknowledgment — guarded with fixed short endings to avoid false positives
  /^I (understand|see)[.!]\s*/i,
  /^I understand what you['']re [^.]{0,50}[.!]\s*/i,
  /^I see what you (mean|are (saying|getting at))[.!]\s*/i,
  /^That (makes sense|is correct|sounds right)[.!]\s*/i,
  /^That['']s (a )?(good|great|valid|fair|reasonable) (point|question|observation|catch|approach)[!,.]?\s*/i,
  /^You['']re (absolutely |totally |completely )?right[.!]\s*/i,

  // "Let me X." — turn-opening narration before the actual content
  /^Let me (help you with (that|this)|take care of (that|this))[.!]\s*/i,
  /^Let me walk you through (this|that)[.!]\s*/i,
  /^Let me (explain|clarify)(\.|\!| that\.| this\.)\s*/i,
  /^Let me break (this|that) down[.!]\s*/i,
  /^Let me (take a look|look into (this|that))[.!]\s*/i,
  /^Let me address (that|this|your question)[.!]\s*/i,

  // Agentic acknowledgment — "I'll/I can do that"
  /^I['']ll (help you with (that|this)|take care of (that|this)|get (that|this) done)[.!]\s*/i,
  /^I (can|will) (help|do that|help you with (that|this))[.!]\s*/i,

  // Analysis starters — strip the frame, keep the finding
  // "Based on your code, the issue is X." → "the issue is X."
  /^Based on (what you['']ve (described|shared|said|provided)|your (question|message|description|code|error))[,.]?\s*/i,
  /^Looking at (this|that|your (code|error|message|question|issue|problem))[,.]?\s*/i,
  /^From what (you['']ve (described|shared|said)|I can see)[,.]?\s*/i,

  // "To answer/address your question" — structural announcement without colon
  /^To (answer|address) your question[,.]?\s*/i,
  /^In (response|answer) to your question[,.]?\s*/i,

  // AI disclaimer
  /^As an AI (language model|assistant)[,.]?\s*/i,
  /^Thank you for (your question|asking|reaching out)[^.]*[.!]\s*/i,
];

export const ASSISTANT_CLOSERS: RegExp[] = [
  // "I hope / Hope this/that ..."
  /\s*I hope this (helps|answers|clarifies)[^.]*[.!]?\s*$/i,
  /\s*Hope (this|that) [^.!?]{0,60}[.!]?\s*$/i,

  // "Let me know ..." — generalized to catch all variants
  /\s*Let me know\b[^.!?]{0,80}[.!?]?\s*$/i,

  // "Feel free to ..."
  /\s*Feel free to (ask|reach out|let me know|contact me)[^.]*[.!?]\s*$/i,

  // "If you have/need ..." conditional offer
  /\s*If you (have|need) (any |more )?(questions?|help|issues?|problems?|anything)[^.]*[.!]\s*$/i,

  // Misc trailing offers
  /\s*(Happy|Glad) to (help|clarify|assist)( further| more| with (that|anything))?[.!]\s*$/i,
  /\s*Is there anything else[^.]*[?!]\s*$/i,
  /\s*Don['']t hesitate to[^.]*[.!]\s*$/i,
  /\s*Please (let me know|feel free)[^.]*[.!]\s*$/i,
  /\s*Does (that|this) (make sense|help|work( for you)?)\??[.!]?\s*$/i,
  /\s*That should (do it|work( for you)?)[.!]\s*$/i,
  /\s*(Give (that|this) a try|Try (that|this) out)[^.]*[.!]\s*$/i,
  /\s*(Shall I|Should I|Would you like me to)[^?]*\??\s*$/i,
];

export function removeAssistantBoilerplate(text: string): string {
  let result = text;
  for (const p of ASSISTANT_OPENERS) result = result.replace(p, '');
  for (const p of ASSISTANT_CLOSERS) result = result.replace(p, '');
  return result.trim();
}
