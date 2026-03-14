// Boilerplate patterns for assistant turns.
// Add new openers/closers here as they're discovered.

export const ASSISTANT_OPENERS: RegExp[] = [
  /^(Certainly|Sure|Absolutely|Of course|Indeed)[!,.]?\s*/i,
  /^Great (question|point)[!,.]?\s*/i,
  /^(That['']s a great|What a great) (question|point)[!,.]?\s*/i,
  /^I['']d be (happy|glad|delighted) to (help|assist)[^.]*[.!]\s*/i,
  /^I['']m (happy|glad) to (help|assist)[^.]*[.!]\s*/i,
  /^Thank you for (your question|asking|reaching out)[^.]*[.!]\s*/i,
  /^As an AI (language model|assistant)[,.]?\s*/i,
];

export const ASSISTANT_CLOSERS: RegExp[] = [
  /\s*I hope this (helps|answers)[^.]*[.!]?\s*$/i,
  /\s*Hope that helps[.!]?\s*$/i,
  /\s*Let me know if you (have|need)[^.]*[.!?]\s*$/i,
  /\s*Feel free to ask[^.]*[.!?]\s*$/i,
  /\s*Is there anything else[^.]*[?!]\s*$/i,
  /\s*Don['']t hesitate to[^.]*[.!]\s*$/i,
  /\s*Please (let me know|feel free)[^.]*[.!]\s*$/i,
];

export function removeAssistantBoilerplate(text: string): string {
  let result = text;
  for (const p of ASSISTANT_OPENERS) result = result.replace(p, '');
  for (const p of ASSISTANT_CLOSERS) result = result.replace(p, '');
  return result.trim();
}
