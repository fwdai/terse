import type View from 'compromise/view/three';

// ── Types ────────────────────────────────────────────────────────────────────

export type Tier        = 'rules' | 'nlp' | 'llm';
export type TokenMethod = 'chars' | 'gpt';

export interface CompressConfig {
  tiers:       Tier[];
  tokenMethod: TokenMethod;
}

export interface CompressResult {
  text:             string;
  originalTokens:   number;
  compressedTokens: number;
  savedTokens:      number;
  savedPercent:     number;
}

export interface Message {
  role:    'user' | 'assistant';
  content: string;
  [key: string]: unknown;
}

export interface MessageWithStats extends Message {
  _stats: Omit<CompressResult, 'text'>;
}

export interface HistoryResult {
  messages: MessageWithStats[];
  stats: {
    totalOriginalTokens:   number;
    totalCompressedTokens: number;
    totalSavedTokens:      number;
    totalSavedPercent:     number;
  };
}

// ── Optional dependencies (lazy-loaded) ──────────────────────────────────────

let gptEncode: ((text: string) => number[]) | null = null;
try {
  const mod = await import('gpt-tokenizer');
  gptEncode = mod.encode;
} catch { /* not installed; 'gpt' tokenMethod will throw if used */ }

let nlp: ((text: string) => View) | null = null;
try {
  const mod = await import('compromise');
  nlp = mod.default as (text: string) => View;
} catch { /* not installed; 'nlp' tier will throw if used */ }

// ── Config ───────────────────────────────────────────────────────────────────

export const TIERS = {
  RULES: 'rules' as const,
  NLP:   'nlp'   as const,
  LLM:   'llm'   as const,
};

export const DEFAULT_CONFIG: CompressConfig = {
  tiers:       [TIERS.RULES],
  tokenMethod: 'chars',
};

// ── Token counting ────────────────────────────────────────────────────────────

function estimateTokens(text: string, method: TokenMethod = 'chars'): number {
  if (method === 'gpt') {
    if (!gptEncode) throw new Error('gpt-tokenizer not installed. Run: bun add gpt-tokenizer');
    return gptEncode(text).length;
  }
  return Math.ceil(text.length / 4);
}

// ── Code/URL masking ──────────────────────────────────────────────────────────

function maskProtected(text: string): { masked: string; blocks: string[] } {
  const blocks: string[] = [];
  const masked = text
    .replace(/```[\s\S]*?```/g, (m) => { blocks.push(m); return `\x00B${blocks.length - 1}\x00`; })
    .replace(/`[^`\n]+`/g,     (m) => { blocks.push(m); return `\x00B${blocks.length - 1}\x00`; })
    .replace(/https?:\/\/\S+/g, (m) => { blocks.push(m); return `\x00B${blocks.length - 1}\x00`; });
  return { masked, blocks };
}

function unmaskProtected(text: string, blocks: string[]): string {
  return text.replace(/\x00B(\d+)\x00/g, (_, i) => blocks[parseInt(i)]);
}

// ── TIER 1: Rules ─────────────────────────────────────────────────────────────

const ASSISTANT_OPENERS: RegExp[] = [
  /^(Certainly|Sure|Absolutely|Of course|Indeed)[!,.]?\s*/i,
  /^Great (question|point)[!,.]?\s*/i,
  /^(That['']s a great|What a great) (question|point)[!,.]?\s*/i,
  /^I['']d be (happy|glad|delighted) to (help|assist)[^.]*[.!]\s*/i,
  /^I['']m (happy|glad) to (help|assist)[^.]*[.!]\s*/i,
  /^Thank you for (your question|asking|reaching out)[^.]*[.!]\s*/i,
  /^As an AI (language model|assistant)[,.]?\s*/i,
];

const ASSISTANT_CLOSERS: RegExp[] = [
  /\s*I hope this (helps|answers)[^.]*[.!]?\s*$/i,
  /\s*Hope that helps[.!]?\s*$/i,
  /\s*Let me know if you (have|need)[^.]*[.!?]\s*$/i,
  /\s*Feel free to ask[^.]*[.!?]\s*$/i,
  /\s*Is there anything else[^.]*[?!]\s*$/i,
  /\s*Don['']t hesitate to[^.]*[.!]\s*$/i,
  /\s*Please (let me know|feel free)[^.]*[.!]\s*$/i,
];

function removeAssistantBoilerplate(text: string): string {
  let result = text;
  for (const p of ASSISTANT_OPENERS) result = result.replace(p, '');
  for (const p of ASSISTANT_CLOSERS) result = result.replace(p, '');
  return result.trim();
}

const USER_OPENERS: RegExp[] = [
  /^Please[,.]?\s+/i,
  /^Kindly[,.]?\s+/i,
  /^(Sorry|Apologies)[,.]?\s+if this is( a)? (dumb|silly|basic|stupid|obvious) question[,.]?\s*(but\s+)?/i,
  /^(Sorry|Apologies)[,.]?\s+for (the |a )?(dumb|silly|basic|stupid) question[,.]?\s*(but\s+)?/i,
  /^This might be obvious[,.]?\s+but\s*/i,
  /^Forgive me if (I['']m wrong|this is obvious)[,.]?\s+but\s*/i,
  /^Not sure if this is (relevant|the right place)[,.]?\s+but\s*/i,
  /^I (have|had) a (quick\s+)?(question|query)[,.]?\s*(about|on|regarding)?\s*/i,
  /^(Can|Could) you (please\s+)?(help me\s+)?/i,
  /^Would you (be able to|mind\s+)/i,
  /^I(['']d like| want| need)( you)? to\s+/i,
  /^I was wondering if you could\s+/i,
  /^Is it possible( for you)? to\s+/i,
];

const GERUND_TO_IMPERATIVE = /^([a-z]+ing)\b/;

const USER_CLOSERS: RegExp[] = [
  // [,]? — consumes a preceding comma but NOT a period (preserves sentence-ending punctuation)
  /[,]?\s*[Tt]hank(s| you)( (so much|a lot|in advance))?[.!]?\s*$/i,
  /[,]?\s*[Ii] appreciate (it|your help|your (time|assistance))[.!]?\s*$/i,
  /[,]?\s*[Tt]hanks for (your help|helping( me)?)[.!]?\s*$/i,
  /[,]?\s*[Pp]lease let me know if you need (more|any) (info|information|details|clarification)[.!]?\s*$/i,
  /[,]?\s*[Ff]eel free to (ask|let me know)[^.]*[.!]?\s*$/i,
];

function removeUserBoilerplate(text: string): string {
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

const SUBSTITUTIONS: [RegExp, string][] = [
  [/\bin order to\b/gi,                          'to'],
  [/\bdue to the fact that\b/gi,                 'because'],
  [/\bat this point in time\b/gi,                'now'],
  [/\bin the event that\b/gi,                    'if'],
  [/\bis able to\b/gi,                           'can'],
  [/\bare able to\b/gi,                          'can'],
  [/\bwas able to\b/gi,                          'could'],
  [/\bwere able to\b/gi,                         'could'],
  [/\bhas the ability to\b/gi,                   'can'],
  [/\bhave the ability to\b/gi,                  'can'],
  [/\bprior to\b/gi,                             'before'],
  [/\bsubsequent to\b/gi,                        'after'],
  [/\bwith the exception of\b/gi,                'except'],
  [/\bon a regular basis\b/gi,                   'regularly'],
  [/\bat the present time\b/gi,                  'now'],
  [/\bin the near future\b/gi,                   'soon'],
  [/\bin spite of the fact that\b/gi,            'although'],
  [/\bregardless of the fact that\b/gi,          'although'],
  [/\bfor the purpose of\b/gi,                   'for'],
  [/\bin the process of\b/gi,                    'while'],
  [/\bfirst and foremost\b/gi,                   'first'],
  [/\blast but not least\b/gi,                   'finally'],
  [/\ba large number of\b/gi,                    'many'],
  [/\ba significant number of\b/gi,              'many'],
  [/\bthe majority of\b/gi,                      'most'],
  [/\bin close proximity to\b/gi,                'near'],
  [/\bwith regard(s)? to\b/gi,                   're:'],
  [/\bwith respect to\b/gi,                      're:'],
  [/\bin terms of\b/gi,                          'for'],
  [/\bas well as\b/gi,                           'and'],
  [/\bit is possible that\b/gi,                  'maybe'],
  [/\bthere is a possibility that\b/gi,          'maybe'],
  [/\bmake a decision\b/gi,                      'decide'],
  [/\bmake use of\b/gi,                          'use'],
  [/\bprovide assistance( to)?\b/gi,             'help'],
  [/\bgive consideration to\b/gi,                'consider'],
  [/\btake into consideration\b/gi,              'consider'],
  [/\bcome to the conclusion\b/gi,               'conclude'],
  [/\bcome to an agreement\b/gi,                 'agree'],
  [/\breach a conclusion\b/gi,                   'conclude'],
  [/\bit (is|should be) (worth noting|noted) that[,]?\s*/gi, ''],
  [/\bit is important to note that[,]?\s*/gi,    ''],
  [/\bit['']s worth (noting|mentioning) that[,]?\s*/gi, ''],
  [/\bneedless to say[,]?\s*/gi,                 ''],
  [/\bas a matter of fact[,]?\s*/gi,             ''],
  [/\bin actual fact[,]?\s*/gi,                  ''],
  [/\bit goes without saying( that)?[,]?\s*/gi,  ''],
  [/\bthe fact that\b/gi,                        'that'],
  [/\bfor example\b/gi,                          'e.g.'],
  [/\bthat is to say\b/gi,                       'i.e.'],
  [/\bin other words\b/gi,                       'i.e.'],
  [/\bet cetera\b/gi,                            'etc.'],
  [/\band so (on|forth)\b/gi,                    'etc.'],
  [/\bversus\b/gi,                               'vs.'],
  [/\bapproximately\s*/gi,                       '~'],
];

function applySubstitutions(text: string): string {
  let result = text;
  for (const [pattern, replacement] of SUBSTITUTIONS) {
    result = result.replace(pattern, replacement);
  }
  // A deletion may have left the first letter lowercase — re-capitalize
  if (/^[a-z]/.test(result)) {
    result = result.charAt(0).toUpperCase() + result.slice(1);
  }
  return result;
}

const SENTENCE_START_FILLERS =
  /(?<=(?:^|[.!?])\s{0,3})(basically|essentially|simply|obviously|clearly|literally|honestly),?\s+/gim;

function removeFillers(text: string): string {
  return text.replace(SENTENCE_START_FILLERS, '');
}

function normalizeWhitespace(text: string): string {
  return text
    .replace(/\n+/g, ' ')
    .replace(/[ \t]+/g, ' ')
    .replace(/\.{3}/g, '…')
    .replace(/!{2,}/g, '!')
    .replace(/\?{2,}/g, '?')
    .trim();
}

function applyRules(text: string, role: string): string {
  let result = text;
  if (role === 'assistant') result = removeAssistantBoilerplate(result);
  if (role === 'user')      result = removeUserBoilerplate(result);
  result = applySubstitutions(result);
  result = removeFillers(result);
  result = normalizeWhitespace(result);
  return result;
}

// ── TIER 2: NLP ───────────────────────────────────────────────────────────────

const NLP_VERB_SYNONYMS: [string, string][] = [
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

const NLP_NOUN_SYNONYMS: [string, string][] = [
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

function applyNlp(text: string): string {
  if (!nlp) throw new Error('compromise not installed. Run: bun add compromise');

  const doc = nlp(text);

  // Drop articles only (a, an, the) — demonstratives (this/that/these/those) carry meaning
  doc.match('(a|an|the)').remove();
  // @ts-expect-error compromise types incomplete
  doc.match('#Adverb (very|really|quite|extremely|highly|truly|absolutely|utterly|incredibly)').remove();
  doc.match('(very|really|quite|extremely|highly|truly|absolutely|utterly|incredibly) #Adjective').remove('#Adverb');

  for (const [from, to] of NLP_VERB_SYNONYMS) {
    doc.verbs().match(from).replaceWith(to);
  }
  for (const [from, to] of NLP_NOUN_SYNONYMS) {
    doc.nouns().match(from).replaceWith(to);
  }

  return doc.text().replace(/[ \t]+/g, ' ').trim();
}

// ── TIER 3: LLM (future) ──────────────────────────────────────────────────────

function applyLlm(_text: string, _role: string): string {
  throw new Error('LLM tier not yet implemented');
}

// ── Pipeline ──────────────────────────────────────────────────────────────────

const TIER_FNS: Record<Tier, (text: string, role: string) => string> = {
  rules: applyRules,
  nlp:   (text) => applyNlp(text),
  llm:   applyLlm,
};

// ── Public API ────────────────────────────────────────────────────────────────

export function compress(
  text: string,
  role: 'user' | 'assistant' = 'assistant',
  config: CompressConfig = DEFAULT_CONFIG,
): CompressResult {
  const { tiers, tokenMethod } = config;

  const originalTokens = estimateTokens(text, tokenMethod);
  const { masked, blocks } = maskProtected(text);

  let result = masked;
  for (const tier of tiers) {
    const fn = TIER_FNS[tier];
    if (!fn) throw new Error(`Unknown tier: "${tier}". Valid: ${Object.values(TIERS).join(', ')}`);
    result = fn(result, role);
  }

  result = unmaskProtected(result, blocks);

  const compressedTokens = estimateTokens(result, tokenMethod);
  const savedTokens      = originalTokens - compressedTokens;
  const savedPercent     = originalTokens > 0 ? Math.round((savedTokens / originalTokens) * 100) : 0;

  return { text: result, originalTokens, compressedTokens, savedTokens, savedPercent };
}

export function compressHistory(
  messages: Message[],
  config: CompressConfig = DEFAULT_CONFIG,
): HistoryResult {
  let totalOriginal   = 0;
  let totalCompressed = 0;

  const compressed = messages.map((msg): MessageWithStats => {
    const result = compress(msg.content, msg.role, config);
    totalOriginal   += result.originalTokens;
    totalCompressed += result.compressedTokens;
    return {
      ...msg,
      content: result.text,
      _stats: {
        originalTokens:   result.originalTokens,
        compressedTokens: result.compressedTokens,
        savedTokens:      result.savedTokens,
        savedPercent:     result.savedPercent,
      },
    };
  });

  const totalSaved        = totalOriginal - totalCompressed;
  const totalSavedPercent = totalOriginal > 0 ? Math.round((totalSaved / totalOriginal) * 100) : 0;

  return {
    messages: compressed,
    stats: {
      totalOriginalTokens:   totalOriginal,
      totalCompressedTokens: totalCompressed,
      totalSavedTokens:      totalSaved,
      totalSavedPercent,
    },
  };
}
