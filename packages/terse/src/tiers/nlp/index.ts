import { VERB_SYNONYMS, NOUN_SYNONYMS, INTENSIFIERS } from './vocab.ts';

let nlp: ((text: string) => any) | null = null;
try {
  const mod = await import('compromise');
  nlp = mod.default;
} catch { /* not installed; NLP tier will throw if used */ }

export function applyNlp(text: string): string {
  if (!nlp) throw new Error('compromise not installed. Run: bun add compromise');

  const doc = nlp(text);
  const intensifierPattern = `(${INTENSIFIERS.join('|')})`;

  // Drop articles only (a/an/the) — demonstratives (this/that/these/those) carry meaning
  doc.match('(a|an|the)').remove();

  // Drop intensifier adverbs (very, really, extremely…) when modifying adjectives
  doc.match(`#Adverb ${intensifierPattern}`).remove();
  doc.match(`${intensifierPattern} #Adjective`).remove('#Adverb');

  // POS-verified synonym substitution
  for (const [from, to] of VERB_SYNONYMS) doc.verbs().match(from).replaceWith(to);
  for (const [from, to] of NOUN_SYNONYMS) doc.nouns().match(from).replaceWith(to);

  return doc.text().replace(/[ \t]+/g, ' ').trim();
}
