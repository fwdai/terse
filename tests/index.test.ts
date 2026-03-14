import { describe, expect, test } from 'bun:test';
import { compress, compressHistory, TIERS, DEFAULT_CONFIG } from '../src/index.ts';
import type { CompressConfig } from '../src/index.ts';

const rules: CompressConfig = { tiers: [TIERS.RULES], tokenMethod: 'chars' };
const nlp:   CompressConfig = { tiers: [TIERS.NLP],   tokenMethod: 'chars' };
const both:  CompressConfig = { tiers: [TIERS.RULES, TIERS.NLP], tokenMethod: 'chars' };

// Shorthand helpers
const assistant = (text: string, cfg = rules) => compress(text, 'assistant', cfg).text;
const user      = (text: string, cfg = rules) => compress(text, 'user',      cfg).text;
const savings   = (text: string, role: 'user' | 'assistant' = 'assistant', cfg = rules) =>
  compress(text, role, cfg).savedPercent;

// ── Rules tier ────────────────────────────────────────────────────────────────

describe('rules tier — assistant boilerplate', () => {
  describe('openers', () => {
    test('removes "Certainly!"', () =>
      expect(assistant('Certainly! Here is the answer.')).toBe('Here is the answer.'));

    test('removes "Sure,"', () =>
      expect(assistant('Sure, here is what you need.')).toBe('Here is what you need.'));

    test('removes "Great question!"', () =>
      expect(assistant("Great question! Let's explore this.")).toBe("Let's explore this."));

    test("removes \"I'd be happy to help\"", () =>
      expect(assistant("I'd be happy to help. Here is the answer.")).toBe('Here is the answer.'));

    test("removes \"I'm glad to assist\"", () =>
      expect(assistant("I'm glad to assist. Here's what I found.")).toBe("Here's what I found."));

    test('removes "As an AI language model"', () =>
      expect(assistant('As an AI language model, I cannot browse the web.'))
        .toBe('I cannot browse the web.'));

    test('removes "Thank you for your question"', () =>
      expect(assistant('Thank you for your question. The answer is 42.'))
        .toBe('The answer is 42.'));
  });

  describe('closers', () => {
    test('removes "I hope this helps!"', () =>
      expect(assistant('The answer is 42. I hope this helps!')).toBe('The answer is 42.'));

    test('removes "Let me know if you have any questions."', () =>
      expect(assistant('Use recursion here. Let me know if you have any questions.'))
        .toBe('Use recursion here.'));

    test('removes "Feel free to ask"', () =>
      expect(assistant('That covers the basics. Feel free to ask if you need more details.'))
        .toBe('That covers the basics.'));

    test('removes "Is there anything else"', () =>
      expect(assistant('Done. Is there anything else I can help you with?'))
        .toBe('Done.'));

    test('removes opener and closer together', () =>
      expect(assistant("Certainly! The answer is 42. I hope this helps!"))
        .toBe('The answer is 42.'));
  });
});

describe('rules tier — user boilerplate', () => {
  describe('politeness openers', () => {
    test('removes "Please,"', () =>
      expect(user('Please, write me a poem about the sea.')).toBe('Write me a poem about the sea.'));

    test('removes "Please" without comma', () =>
      expect(user('Please write a function that sorts an array.')).toBe('Write a function that sorts an array.'));

    test('removes "Kindly"', () =>
      expect(user('Kindly review this pull request.')).toBe('Review this pull request.'));
  });

  describe('request preambles', () => {
    test('removes "Can you"', () =>
      expect(user('Can you explain how closures work?')).toBe('Explain how closures work?'));

    test('removes "Could you please"', () =>
      expect(user('Could you please explain how React hooks work?')).toBe('Explain how React hooks work?'));

    test('removes "Would you be able to"', () =>
      expect(user('Would you be able to list the differences between TCP and UDP?'))
        .toBe('List the differences between TCP and UDP?'));

    test("removes \"I'd like you to\"", () =>
      expect(user("I'd like you to summarize this document.")).toBe('Summarize this document.'));

    test('removes "I need you to"', () =>
      expect(user('I need you to fix this bug in my code.')).toBe('Fix this bug in my code.'));

    test('removes "I was wondering if you could"', () =>
      expect(user('I was wondering if you could help me understand closures.'))
        .toBe('Help me understand closures.'));

    test('removes "Is it possible for you to"', () =>
      expect(user('Is it possible for you to generate a JSON schema for this?'))
        .toBe('Generate a JSON schema for this?'));

    test('converts leftover gerund to imperative', () =>
      expect(user('Can you help figuring out what is wrong here?'))
        .toBe('Help figuring out what is wrong here?'));
  });

  describe('self-deprecating hedges', () => {
    test('removes "Sorry if this is a dumb question"', () =>
      expect(user('Sorry if this is a dumb question, but what is a pointer?'))
        .toBe('What is a pointer?'));

    test('removes "Sorry for the dumb question"', () =>
      expect(user('Sorry for the dumb question, but how does memoization work?'))
        .toBe('How does memoization work?'));

    test('removes "This might be obvious, but"', () =>
      expect(user('This might be obvious, but why does this return undefined?'))
        .toBe('Why does this return undefined?'));
  });

  describe('politeness closers', () => {
    test('removes "Thank you"', () =>
      expect(user('Write me an article on LLMs. Thank you')).toBe('Write me an article on LLMs.'));

    test('removes "Thanks a lot"', () =>
      expect(user('Summarize this. Thanks a lot')).toBe('Summarize this.'));

    test('removes "I appreciate your time"', () =>
      expect(user('Review this PR. I appreciate your time.')).toBe('Review this PR.'));

    test('removes opener and closer together', () =>
      expect(user('Please write a sorting function. Thank you!')).toBe('Write a sorting function.'));
  });

  describe('preserved user content', () => {
    test('keeps "I think" — epistemic signal', () =>
      expect(user('I think the issue is on line 42.')).toBe('I think the issue is on line 42.'));

    test('keeps "I believe" — epistemic signal', () =>
      expect(user("I believe this might be a race condition.")).toBe("I believe this might be a race condition."));
  });
});

describe('rules tier — phrase substitutions', () => {
  test('"in order to" → "to"', () =>
    expect(assistant('Use a loop in order to iterate.')).toBe('Use a loop to iterate.'));

  test('"due to the fact that" → "because"', () =>
    expect(assistant('It failed due to the fact that the input was null.'))
      .toBe('It failed because the input was null.'));

  test('"prior to" → "before"', () =>
    expect(assistant('Prior to the refactor, tests were failing.')).toBe('Before the refactor, tests were failing.'));

  test('"is able to" → "can"', () =>
    expect(assistant('The function is able to handle null inputs.')).toBe('The function can handle null inputs.'));

  test('"for example" → "e.g."', () =>
    expect(assistant('Use a data structure, for example a hash map.')).toBe('Use a data structure, e.g. a hash map.'));

  test('"in other words" → "i.e."', () =>
    expect(assistant('It is idempotent, in other words calling it twice has the same effect.'))
      .toBe('It is idempotent, i.e. calling it twice has the same effect.'));

  test('"versus" → "vs."', () =>
    expect(assistant('Consider performance versus readability.')).toBe('Consider performance vs. readability.'));

  test('"approximately" → "~"', () =>
    expect(assistant('The query takes approximately 200ms.')).toBe('The query takes ~200ms.'));

  test('"take into consideration" → "consider"', () =>
    expect(assistant('You should take into consideration the edge cases.'))
      .toBe('You should consider the edge cases.'));

  test('zero-value markers deleted: "it is worth noting that"', () =>
    expect(assistant('It is worth noting that this is O(n).')).toBe('This is O(n).'));

  test('zero-value markers deleted: "needless to say"', () =>
    expect(assistant('Needless to say, tests should pass.')).toBe('Tests should pass.'));
});

describe('rules tier — filler words', () => {
  test('removes filler at sentence start', () =>
    expect(assistant('Basically, this is how it works.')).toBe('this is how it works.'));

  test('removes filler after sentence boundary', () =>
    expect(assistant('First point. Basically, the cache is warm.'))
      .toBe('First point. the cache is warm.'));

  test('does NOT remove filler mid-sentence', () =>
    expect(assistant('She basically ignored the warning.')).toBe('She basically ignored the warning.'));

  test('does NOT remove "clearly" when it is an adjective', () =>
    expect(assistant('The code is clearly wrong.')).toBe('The code is clearly wrong.'));
});

describe('rules tier — whitespace normalization', () => {
  test('collapses newlines to spaces', () =>
    expect(assistant('First point.\nSecond point.')).toBe('First point. Second point.'));

  test('collapses multiple newlines', () =>
    expect(assistant('Paragraph one.\n\n\nParagraph two.')).toBe('Paragraph one. Paragraph two.'));

  test('collapses multiple spaces', () =>
    expect(assistant('Too   many   spaces.')).toBe('Too many spaces.'));

  test('normalizes ellipsis', () =>
    expect(assistant('Hmm...')).toBe('Hmm…'));

  test('normalizes repeated punctuation', () =>
    expect(assistant('What!!!')).toBe('What!'));
});

// ── Protected blocks ──────────────────────────────────────────────────────────

describe('protected blocks', () => {
  test('does not compress content inside fenced code blocks', () => {
    const text = 'Here is an example:\n```\nfor example in order to\n```';
    expect(assistant(text, both)).toContain('for example in order to');
  });

  test('does not compress inline code', () => {
    const text = 'Call `getValueForExample()` to retrieve it.';
    expect(assistant(text, both)).toContain('`getValueForExample()`');
  });

  test('does not compress URLs', () => {
    const url = 'https://example.com/in-order-to/basically/test';
    const text = `See the docs at ${url} for details.`;
    expect(assistant(text, both)).toContain(url);
  });

  test('compresses text around code blocks but not inside', () => {
    const text = 'Certainly! For example:\n```\nconst x = 1;\n```\nI hope this helps!';
    const result = assistant(text, both);
    expect(result).not.toContain('Certainly');
    expect(result).not.toContain('I hope this helps');
    expect(result).toContain('const x = 1;');
  });
});

// ── NLP tier ──────────────────────────────────────────────────────────────────

describe('nlp tier — determiners', () => {
  test('drops "the" before common nouns', () =>
    expect(assistant('The function returns null.', nlp)).toBe('function returns null.'));

  test('drops "a" before common nouns', () =>
    expect(assistant('Use a hash map for lookups.', nlp)).toBe('Use hash map for lookups.'));

  test('drops "an" before vowel-initial nouns', () =>
    expect(assistant('Create an instance of the class.', nlp)).toBe('Create instance of class.'));
});

describe('nlp tier — synonym substitution', () => {
  // Note: NLP tier also drops articles (a/an/the), so expected strings reflect that

  test('verb: utilize → use (demonstratives like "this" are kept)', () =>
    expect(assistant('We should utilize this approach.', nlp)).toBe('We should use this approach.'));

  test('verb: attempt → try', () =>
    expect(assistant('The system will attempt to reconnect.', nlp)).toBe('system will try to reconnect.'));

  test('verb: obtain → get', () =>
    expect(assistant('You can obtain the token from the header.', nlp)).toBe('You can get token from header.'));

  test('verb: demonstrate → show', () =>
    expect(assistant('This example will demonstrate the pattern.', nlp)).toBe('This example will show pattern.'));

  test('noun: repository → repo', () =>
    expect(assistant('Clone the repository first.', nlp)).toBe('Clone repo first.'));

  test('noun: documentation → docs', () =>
    expect(assistant('Check the documentation for details.', nlp)).toBe('Check docs for details.'));

  test('noun: configuration → config', () =>
    expect(assistant('Update the configuration file.', nlp)).toBe('Update config file.'));
});

describe('nlp tier — additive on top of rules', () => {
  test('rules + nlp stack correctly', () => {
    const text = 'Certainly! The function is attempting to utilize the configuration. I hope this helps!';
    const result = assistant(text, both);
    expect(result).not.toContain('Certainly');
    expect(result).not.toContain('I hope this helps');
    expect(result).not.toContain('utilize');
    expect(result).not.toContain('configuration');
    expect(result).toContain('use');
    expect(result).toContain('config');
  });
});

// ── Tiers config ──────────────────────────────────────────────────────────────

describe('config', () => {
  test('default config applies rules tier only', () => {
    const text = 'Certainly! The function returns null.';
    const result = compress(text, 'assistant', DEFAULT_CONFIG);
    expect(result.text).not.toContain('Certainly');
    // determiners still present (no NLP tier)
    expect(result.text).toContain('The');
  });

  test('unknown tier throws', () =>
    // @ts-expect-error — intentionally invalid tier
    expect(() => compress('hello', 'assistant', { tiers: ['magic'], tokenMethod: 'chars' })).toThrow());

  test('LLM tier throws not implemented', () =>
    expect(() => compress('hello', 'assistant', { tiers: [TIERS.LLM], tokenMethod: 'chars' })).toThrow('not yet implemented'));

  test('savedPercent is 0 for empty compression', () => {
    const result = compress('x', 'assistant', rules);
    expect(result.savedPercent).toBe(0);
  });

  test('originalTokens is always >= compressedTokens', () => {
    const text = 'Certainly! Due to the fact that you asked, in order to help, I hope this helps!';
    const result = compress(text, 'assistant', both);
    expect(result.originalTokens).toBeGreaterThanOrEqual(result.compressedTokens);
  });
});

// ── compressHistory ───────────────────────────────────────────────────────────

describe('compressHistory()', () => {
  const history = [
    { role: 'user' as const,      content: 'Please explain how closures work. Thank you!' },
    { role: 'assistant' as const, content: 'Certainly! A closure is a function that captures variables from its surrounding scope. I hope this helps!' },
    { role: 'user' as const,      content: 'Could you give me an example?' },
    { role: 'assistant' as const, content: 'Sure, for example: `const add = x => y => x + y`. Feel free to ask if you need more details.' },
  ];

  test('returns correct number of messages', () => {
    const { messages } = compressHistory(history, rules);
    expect(messages.length).toBe(4);
  });

  test('each message has _stats', () => {
    const { messages } = compressHistory(history, rules);
    for (const msg of messages) {
      expect(msg._stats).toBeDefined();
      expect(msg._stats.originalTokens).toBeGreaterThan(0);
      expect(msg._stats.savedPercent).toBeGreaterThanOrEqual(0);
    }
  });

  test('total stats are sum of individual message stats', () => {
    const { messages, stats } = compressHistory(history, rules);
    const sumOriginal   = messages.reduce((s, m) => s + m._stats.originalTokens, 0);
    const sumCompressed = messages.reduce((s, m) => s + m._stats.compressedTokens, 0);
    expect(stats.totalOriginalTokens).toBe(sumOriginal);
    expect(stats.totalCompressedTokens).toBe(sumCompressed);
  });

  test('original message objects are not mutated', () => {
    const original = history[0]!.content;
    compressHistory(history, rules);
    expect(history[0]!.content).toBe(original);
  });

  test('compresses user and assistant turns with role-appropriate rules', () => {
    const { messages } = compressHistory(history, rules);
    expect(messages[0]!.content).not.toContain('Please');
    expect(messages[0]!.content).not.toContain('Thank you');
    expect(messages[1]!.content).not.toContain('Certainly');
    expect(messages[1]!.content).not.toContain('I hope this helps');
    expect(messages[3]!.content).not.toContain('Sure');
    expect(messages[3]!.content).not.toContain('Feel free to ask');
  });

  test('preserves inline code in history', () => {
    const { messages } = compressHistory(history, both);
    expect(messages[3]!.content).toContain('`const add = x => y => x + y`');
  });

  test('totalSavedPercent > 0 for compressible history', () => {
    const { stats } = compressHistory(history, rules);
    expect(stats.totalSavedPercent).toBeGreaterThan(0);
  });
});
