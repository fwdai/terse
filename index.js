// ── Optional dependencies (lazy-loaded) ─────────────────────────────────────

let gptEncode = null;
try {
  const mod = await import('gpt-tokenizer');
  gptEncode = mod.encode;
} catch { /* not installed; 'gpt' tokenMethod will throw if used */ }

let nlp = null;
try {
  const mod = await import('compromise');
  nlp = mod.default;
} catch { /* not installed; 'nlp' tier will throw if used */ }

// ── Config ───────────────────────────────────────────────────────────────────

export const TIERS = {
  RULES: 'rules', // boilerplate removal, phrase substitutions, whitespace
  NLP:   'nlp',   // POS-based function word dropping, synonym optimization
  LLM:   'llm',   // telegraphic rewrite, semantic compression (future)
};

export const DEFAULT_CONFIG = {
  tiers:       [TIERS.RULES],
  tokenMethod: 'chars',        // 'chars' | 'gpt'
};

// ── Token counting ───────────────────────────────────────────────────────────

function estimateTokens(text, method = 'chars') {
  if (method === 'gpt') {
    if (!gptEncode) throw new Error('gpt-tokenizer not installed. Run: npm install gpt-tokenizer');
    return gptEncode(text).length;
  }
  return Math.ceil(text.length / 4);
}

// ── Code/URL masking ─────────────────────────────────────────────────────────

function maskProtected(text) {
  const blocks = [];
  const masked = text
    .replace(/```[\s\S]*?```/g, (m) => { blocks.push(m); return `\x00B${blocks.length - 1}\x00`; })
    .replace(/`[^`\n]+`/g,     (m) => { blocks.push(m); return `\x00B${blocks.length - 1}\x00`; })
    .replace(/https?:\/\/\S+/g, (m) => { blocks.push(m); return `\x00B${blocks.length - 1}\x00`; });
  return { masked, blocks };
}

function unmaskProtected(text, blocks) {
  return text.replace(/\x00B(\d+)\x00/g, (_, i) => blocks[parseInt(i)]);
}

// ── TIER 1: Rules ────────────────────────────────────────────────────────────

// Boilerplate: assistant

const ASSISTANT_OPENERS = [
  /^(Certainly|Sure|Absolutely|Of course|Indeed)[!,.]?\s*/i,
  /^Great (question|point)[!,.]?\s*/i,
  /^(That['']s a great|What a great) (question|point)[!,.]?\s*/i,
  /^I['']d be (happy|glad|delighted) to (help|assist)[^.]*[.!]\s*/i,
  /^I['']m (happy|glad) to (help|assist)[^.]*[.!]\s*/i,
  /^Thank you for (your question|asking|reaching out)[^.]*[.!]\s*/i,
  /^As an AI (language model|assistant)[,.]?\s*/i,
];

const ASSISTANT_CLOSERS = [
  /\s*I hope this (helps|answers)[^.]*[.!]?\s*$/i,
  /\s*Hope that helps[.!]?\s*$/i,
  /\s*Let me know if you (have|need)[^.]*[.!?]\s*$/i,
  /\s*Feel free to ask[^.]*[.!?]\s*$/i,
  /\s*Is there anything else[^.]*[?!]\s*$/i,
  /\s*Don['']t hesitate to[^.]*[.!]\s*$/i,
  /\s*Please (let me know|feel free)[^.]*[.!]\s*$/i,
];

function removeAssistantBoilerplate(text) {
  let result = text;
  for (const p of ASSISTANT_OPENERS) result = result.replace(p, '');
  for (const p of ASSISTANT_CLOSERS) result = result.replace(p, '');
  return result.trim();
}

// Boilerplate: user

const USER_OPENERS = [
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

const USER_CLOSERS = [
  /[,.]?\s*[Tt]hank(s| you)( (so much|a lot|in advance))?[.!]?\s*$/i,
  /[,.]?\s*[Ii] appreciate (it|your help|your (time|assistance))[.!]?\s*$/i,
  /[,.]?\s*[Tt]hanks for (your help|helping( me)?)[.!]?\s*$/i,
  /[,.]?\s*[Pp]lease let me know if you need (more|any) (info|information|details|clarification)[.!]?\s*$/i,
  /[,.]?\s*[Ff]eel free to (ask|let me know)[^.]*[.!]?\s*$/i,
];

function removeUserBoilerplate(text) {
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

// Phrase substitutions

const SUBSTITUTIONS = [
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
  [/\bapproximately\b/gi,                        '~'],
];

function applySubstitutions(text) {
  let result = text;
  for (const [pattern, replacement] of SUBSTITUTIONS) {
    result = result.replace(pattern, replacement);
  }
  return result;
}

// Filler words — sentence-start only (safe without POS)
const SENTENCE_START_FILLERS =
  /(?<=(?:^|[.!?])\s{0,3})(basically|essentially|simply|obviously|clearly|literally|honestly),?\s+/gim;

function removeFillers(text) {
  return text.replace(SENTENCE_START_FILLERS, '');
}

function normalizeWhitespace(text) {
  return text
    .replace(/\n+/g, ' ')
    .replace(/[ \t]+/g, ' ')
    .replace(/\.{3}/g, '…')
    .replace(/!{2,}/g, '!')
    .replace(/\?{2,}/g, '?')
    .trim();
}

function applyRules(text, role) {
  let result = text;
  if (role === 'assistant') result = removeAssistantBoilerplate(result);
  if (role === 'user')      result = removeUserBoilerplate(result);
  result = applySubstitutions(result);
  result = removeFillers(result);
  result = normalizeWhitespace(result);
  return result;
}

// ── TIER 2: NLP ──────────────────────────────────────────────────────────────

// Verb synonyms — only substituted when word is used as a verb (POS-verified)
const NLP_VERB_SYNONYMS = [
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

// Noun synonyms — only substituted when word is used as a noun (POS-verified)
const NLP_NOUN_SYNONYMS = [
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

function applyNlp(text) {
  if (!nlp) throw new Error('compromise not installed. Run: npm install compromise');

  let doc = nlp(text);

  // Drop determiners (a, an, the) — safe for machine reader
  doc.remove('#Determiner');

  // Drop intensifier adverbs globally (POS-verified, unlike rules tier which only caught sentence-start)
  // Matches adverbs that modify adjectives/other adverbs, not verbs (e.g. "runs quickly" stays)
  doc.match('#Adverb (very|really|quite|extremely|highly|truly|absolutely|utterly|incredibly)').remove();
  doc.match('(very|really|quite|extremely|highly|truly|absolutely|utterly|incredibly) #Adjective').remove('#Adverb');

  // Synonym substitution with POS verification
  for (const [from, to] of NLP_VERB_SYNONYMS) {
    doc.verbs().match(from).replaceWith(to);
  }
  for (const [from, to] of NLP_NOUN_SYNONYMS) {
    doc.nouns().match(from).replaceWith(to);
  }

  // Clean up any double spaces left by removals
  return doc.text().replace(/[ \t]+/g, ' ').trim();
}

// ── TIER 3: LLM (future) ────────────────────────────────────────────────────
// Planned: telegraphic rewrite, semantic compression, cross-message dedup

function applyLlm(_text, _role) {
  throw new Error('LLM tier not yet implemented');
}

// ── Pipeline ─────────────────────────────────────────────────────────────────

const TIER_FNS = {
  [TIERS.RULES]: (text, role) => applyRules(text, role),
  [TIERS.NLP]:   (text)       => applyNlp(text),
  [TIERS.LLM]:   (text, role) => applyLlm(text, role),
};

// ── Public API ───────────────────────────────────────────────────────────────

/**
 * Compress a single message.
 *
 * @param {string} text
 * @param {'user' | 'assistant'} role
 * @param {object} config
 * @param {string[]} config.tiers - ordered list of tiers to apply, e.g. ['rules', 'nlp']
 * @param {'chars'|'gpt'} config.tokenMethod
 * @returns {{ text, originalTokens, compressedTokens, savedTokens, savedPercent }}
 */
export function compress(text, role = 'assistant', config = DEFAULT_CONFIG) {
  const { tiers = [TIERS.RULES], tokenMethod = 'chars' } = config;

  const originalTokens = estimateTokens(text, tokenMethod);
  const { masked, blocks } = maskProtected(text);

  let result = masked;
  for (const tier of tiers) {
    const fn = TIER_FNS[tier];
    if (!fn) throw new Error(`Unknown tier: "${tier}". Valid tiers: ${Object.values(TIERS).join(', ')}`);
    result = fn(result, role);
  }

  result = unmaskProtected(result, blocks);

  const compressedTokens = estimateTokens(result, tokenMethod);
  const savedTokens      = originalTokens - compressedTokens;
  const savedPercent     = originalTokens > 0 ? Math.round((savedTokens / originalTokens) * 100) : 0;

  return { text: result, originalTokens, compressedTokens, savedTokens, savedPercent };
}

/**
 * Compress an array of messages (conversation history).
 *
 * @param {Array<{ role: string, content: string }>} messages
 * @param {object} config
 * @returns {{ messages: Array, stats: object }}
 */
export function compressHistory(messages, config = DEFAULT_CONFIG) {
  let totalOriginal   = 0;
  let totalCompressed = 0;

  const compressed = messages.map((msg) => {
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
