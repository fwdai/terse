// Structural meta-commentary: sentences whose sole purpose is to announce what immediately follows.
// The trailing colon is the signal — it guarantees the payload is present; only the frame is stripped.
//
// Examples:
//   "Here is the solution:"             → ''   (solution follows inline)
//   "Below are the steps:"              → ''   (steps follow)
//   "The following code demonstrates:"  → ''   (code follows)
//   "I've outlined the changes below:"  → ''   (changes follow)
//
// Applied to assistant turns only; users rarely write this way.

const STRUCTURAL: RegExp[] = [
  // "Here is/are/was/were X:" and "Here's X:"
  /(?<=(?:^|[.!?])\s{0,3})Here(?:'s| (?:is|are|was|were)) [^:\n]{0,80}:\s*/gim,

  // "Below is/are X:"
  /(?<=(?:^|[.!?])\s{0,3})Below (?:is|are) [^:\n]{0,80}:\s*/gim,

  // "The following X:" / "The following is/are X:"
  /(?<=(?:^|[.!?])\s{0,3})The following\s+[^:\n]{0,80}:\s*/gim,

  // "I've/I have outlined/listed/summarized/described/detailed/provided X:"
  /(?<=(?:^|[.!?])\s{0,3})I(?:'ve| have) (?:outlined|listed|summarized|described|detailed|provided) [^:\n]{0,80}:\s*/gim,
];

export function removeStructuralMetaCommentary(text: string): string {
  let result = text;
  for (const p of STRUCTURAL) result = result.replace(p, '');
  return result;
}
