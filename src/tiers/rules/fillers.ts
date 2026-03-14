// Filler words removed only at sentence-start position.
// Mid-sentence removal is too risky without POS tagging (handled in NLP tier).

const SENTENCE_START_FILLERS =
  /(?<=(?:^|[.!?])\s{0,3})(basically|essentially|simply|obviously|clearly|literally|honestly|actually),?\s+/gim;

export function removeFillers(text: string): string {
  return text.replace(SENTENCE_START_FILLERS, "");
}
