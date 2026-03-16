// Filler words/phrases removed only at sentence-start position.
// Mid-sentence removal is too risky without POS tagging (handled in NLP tier).
//
// Inclusion criteria: safe to strip when the word adds rhetorical color but does not change
// the truth value or uncertainty of the claim that follows.
//
// Intentionally excluded (qualify the claim — carry semantic weight):
//   theoretically, technically, realistically, ideally, presumably, supposedly, arguably(*),
//   apparently — these signal caveat, uncertainty, or ideal-vs-actual distinctions.
//   (*) arguably is borderline but often signals genuine dispute; left out to be safe.
//
// "of course" is a two-word phrase but safe at sentence start: "Of course, this works." → "this works."

const SENTENCE_START_FILLERS =
  /(?<=(?:^|[.!?])\s{0,3})(basically|essentially|simply|obviously|clearly|literally|honestly|actually|frankly|truthfully|personally|genuinely|admittedly|undoubtedly|certainly|naturally|importantly|notably|interestingly|surprisingly|unsurprisingly|thankfully|fortunately|unfortunately|luckily|of course),?\s+/gim;

export function removeFillers(text: string): string {
  return text.replace(SENTENCE_START_FILLERS, "");
}
