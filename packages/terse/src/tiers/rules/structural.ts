// Structural meta-commentary: sentences whose sole purpose is to announce what immediately follows.
// The trailing colon is the signal — it guarantees the payload is present; only the frame is stripped.
//
// Examples:
//   "Here is the solution:"             → ''   (solution follows inline)
//   "Below are the steps:"              → ''   (steps follow)
//   "The following code demonstrates:"  → ''   (code follows)
//   "I've outlined the changes below:"  → ''   (changes follow)
//   "To summarize, the fix is X."       → ''   (summary follows)
//   "In conclusion: use indexes."       → ''   (conclusion follows)
//
// Applied to assistant turns only; users rarely write this way.

const STRUCTURAL: RegExp[] = [
  // ── Announcement frames (require colon — payload always follows) ──────────────

  // "Here is/are/was/were X:" and "Here's X:"
  /(?<=(?:^|[.!?])\s{0,3})Here(?:'s| (?:is|are|was|were)) [^:\n]{0,80}:\s*/gim,

  // "Below is/are X:"
  /(?<=(?:^|[.!?])\s{0,3})Below (?:is|are) [^:\n]{0,80}:\s*/gim,

  // "The following X:" / "The following is/are X:"
  /(?<=(?:^|[.!?])\s{0,3})The following\s+[^:\n]{0,80}:\s*/gim,

  // "I've/I have outlined/listed/summarized/described/detailed/provided X:"
  /(?<=(?:^|[.!?])\s{0,3})I(?:'ve| have) (?:outlined|listed|summarized|described|detailed|provided) [^:\n]{0,80}:\s*/gim,

  // ── Section labels ────────────────────────────────────────────────────────────

  // Closing/summary frames — always discourse markers at sentence start;
  // colon/comma optional since lookbehind already guards against mid-sentence.
  /(?<=(?:^|[.!?])\s{0,3})(?:To summarize|In summary|In conclusion|To conclude|To recap|To sum up|In short|In brief|In closing|To wrap up)[,:]?\s*/gim,

  // Opening frames — colon/comma REQUIRED to rule out purpose clauses
  // ("To begin the loop…" must not match; "To begin, …" safely does).
  /(?<=(?:^|[.!?])\s{0,3})To (?:begin|start)(?:\s+with)?[,:]\s*/gim,
  /(?<=(?:^|[.!?])\s{0,3})First things first[,:]\s*/gim,

  // Explanation frames — colon/comma REQUIRED ("To clarify the issue…" must not match)
  /(?<=(?:^|[.!?])\s{0,3})(?:To explain|To clarify|To illustrate)[,:]\s*/gim,

  // Rephrasing frames — always discourse markers (no purpose-clause ambiguity)
  /(?<=(?:^|[.!?])\s{0,3})(?:Put differently|Put simply|Simply put)[,:]?\s*/gim,

  // Terse content labels — colon/comma REQUIRED
  /(?<=(?:^|[.!?])\s{0,3})(?:TL;DR|Bottom line|Note|Key takeaways?)[,:]\s*/gim,
];

export function removeStructuralMetaCommentary(text: string): string {
  let result = text;
  for (const p of STRUCTURAL) result = result.replace(p, '');
  return result;
}
