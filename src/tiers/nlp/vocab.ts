// NLP synonym vocabularies — POS-verified substitutions (applied via compromise).
// Verbs are only substituted when tagged as verbs; nouns when tagged as nouns.
// Add entries freely — POS tagging prevents cross-category false matches.

// Longer/academic verb → shorter common equivalent
export const VERB_SYNONYMS: [string, string][] = [
  ['utilize',     'use'],
  ['initiate',    'start'],
  ['terminate',   'stop'],
  ['obtain',      'get'],
  ['demonstrate', 'show'],
  ['construct',   'build'],
  ['attempt',     'try'],
  ['require',     'need'],
  ['indicate',    'show'],
  ['determine',   'find'],
  ['commence',    'start'],
  ['accomplish',  'do'],
  ['perform',     'do'],
  ['modify',      'change'],
  ['purchase',    'buy'],
  ['request',     'ask'],
];

// Longer/formal noun → shorter common equivalent
export const NOUN_SYNONYMS: [string, string][] = [
  ['functionality', 'feature'],
  ['individual',    'person'],
  ['assistance',    'help'],
  ['modification',  'change'],
  ['configuration', 'config'],
  ['repository',    'repo'],
  ['documentation', 'docs'],
  ['application',   'app'],
  ['parameter',     'param'],
  ['argument',      'arg'],
  ['component',     'part'],
];

// Intensifier adverbs that add no meaning before adjectives
export const INTENSIFIERS = [
  'very', 'really', 'quite', 'extremely', 'highly',
  'truly', 'absolutely', 'utterly', 'incredibly',
];
