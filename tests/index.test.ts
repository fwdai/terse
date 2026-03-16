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

  describe('greetings', () => {
    test('strips "Hi, "', () =>
      expect(user('Hi, write me a sorting function.')).toBe('Write me a sorting function.'));

    test('strips "Hey! "', () =>
      expect(user('Hey! Explain how closures work.')).toBe('Explain how closures work.'));

    test('strips "Hi Claude, " — name-taking greeting', () =>
      expect(user('Hi Claude, can you fix this bug?')).toBe('Fix this bug?'));

    test('strips "Hey there, "', () =>
      expect(user('Hey there, what is a monad?')).toBe('What is a monad?'));
  });

  describe('quick question / continuation starters', () => {
    test('strips "Quick question:"', () =>
      expect(user('Quick question: how does memoization work?')).toBe('How does memoization work?'));

    test('strips "Just a quick question:"', () =>
      expect(user('Just a quick question: what is the difference between null and undefined?'))
        .toBe('What is the difference between null and undefined?'));

    test('strips "One more thing:" — chains into request preamble', () =>
      expect(user('One more thing: can you add error handling?')).toBe('Add error handling?'));

    test('strips "Follow-up question:"', () =>
      expect(user('Follow-up question: why does this return undefined?')).toBe('Why does this return undefined?'));

    test('strips "Also," — chains into request preamble', () =>
      expect(user('Also, can you add tests?')).toBe('Add tests?'));

    test('strips "And," — chains into request preamble', () =>
      expect(user('And, could you update the docs?')).toBe('Update the docs?'));
  });

  describe('additional hedges', () => {
    test('strips "I might be wrong, but"', () =>
      expect(user('I might be wrong, but this looks like a memory leak.')).toBe('This looks like a memory leak.'));

    test('strips "Correct me if I\'m wrong, but"', () =>
      expect(user("Correct me if I'm wrong, but the timeout should be 30s.")).toBe('The timeout should be 30s.'));

    test('strips "I\'m not sure if this is right, but"', () =>
      expect(user("I'm not sure if this is right, but I think we need to flush the cache."))
        .toBe('I think we need to flush the cache.'));
  });

  describe('additional request preambles', () => {
    test('strips "Do you think you could"', () =>
      expect(user('Do you think you could explain the difference between process and thread?'))
        .toBe('Explain the difference between process and thread?'));

    test('strips "Would it be possible to"', () =>
      expect(user('Would it be possible to rewrite this without recursion?'))
        .toBe('Rewrite this without recursion?'));

    test('strips "I was hoping you could"', () =>
      expect(user('I was hoping you could review this PR.')).toBe('Review this PR.'));
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

  // ── Purpose clauses ──────────────────────────────────────────────────────────
  test('"with the aim of" → "to"', () =>
    expect(assistant('Refactor with the aim of reducing complexity.')).toBe('Refactor to reducing complexity.'));

  test('"in an effort to" → "to"', () =>
    expect(assistant('Cache the result in an effort to speed up queries.')).toBe('Cache the result to speed up queries.'));

  test('"for the sake of" → "for"', () =>
    expect(assistant('Add a comment for the sake of clarity.')).toBe('Add a comment for clarity.'));

  // ── Causal / concessive ───────────────────────────────────────────────────────
  test('"despite the fact that" → "although"', () =>
    expect(assistant('It works despite the fact that the input is malformed.'))
      .toBe('It works although the input is malformed.'));

  test('"as a result of" → "due to"', () =>
    expect(assistant('The build failed as a result of a missing dependency.'))
      .toBe('The build failed due to a missing dependency.'));

  test('"on the other hand" → "however"', () =>
    expect(assistant('This is fast. On the other hand, it uses more memory.'))
      .toBe('This is fast. however, it uses more memory.'));

  // ── Temporal ─────────────────────────────────────────────────────────────────
  test('"at the end of the day" → "ultimately"', () =>
    expect(assistant('At the end of the day, correctness matters more than speed.'))
      .toBe('Ultimately, correctness matters more than speed.'));

  test('"for the time being" → "for now"', () =>
    expect(assistant('Use a placeholder for the time being.')).toBe('Use a placeholder for now.'));

  test('"from time to time" → "sometimes"', () =>
    expect(assistant('The cache is invalidated from time to time.')).toBe('The cache is invalidated sometimes.'));

  test('"on a daily basis" → "daily"', () =>
    expect(assistant('The job runs on a daily basis.')).toBe('The job runs daily.'));

  // ── Quantity / scope ──────────────────────────────────────────────────────────
  test('"a wide range of" → "many"', () =>
    expect(assistant('It supports a wide range of formats.')).toBe('It supports many formats.'));

  test('"in addition to" → "beyond"', () =>
    expect(assistant('In addition to tests, we need docs.')).toBe('Beyond tests, we need docs.'));

  test('"in addition" → "also" (does not fire when "to" follows)', () =>
    expect(assistant('It is fast. In addition, it is reliable.')).toBe('It is fast. also, it is reliable.'));

  test('"each and every" → "every"', () =>
    expect(assistant('Each and every request must be authenticated.')).toBe('Every request must be authenticated.'));

  // ── Connectives / discourse ───────────────────────────────────────────────────
  test('"when it comes to" → "for"', () =>
    expect(assistant('When it comes to performance, use indexes.')).toBe('For performance, use indexes.'));

  test('"by means of" → "via"', () =>
    expect(assistant('Authenticate by means of an API key.')).toBe('Authenticate via an API key.'));

  test('"on the basis of" → "based on"', () =>
    expect(assistant('Choose the algorithm on the basis of input size.')).toBe('Choose the algorithm based on input size.'));

  test('"keep in mind" → "note"', () =>
    expect(assistant('Keep in mind that this is O(n).')).toBe('Note that this is O(n).'));

  test('"the reason why" → "why"', () =>
    expect(assistant('The reason why this fails is the null check.')).toBe('Why this fails is the null check.'));

  // ── Action nominalizations ────────────────────────────────────────────────────
  test('"makes changes to" → "change"', () =>
    expect(assistant('The migration makes changes to the schema.')).toBe('The migration change the schema.'));

  test('"take a look at" → "check"', () =>
    expect(assistant('Take a look at the error log.')).toBe('Check the error log.'));

  test('"have an impact on" → "affect"', () =>
    expect(assistant('This will have an impact on performance.')).toBe('This will affect performance.'));

  test('"has an effect on" → "affects"', () =>
    expect(assistant('Caching has an effect on latency.')).toBe('Caching affects latency.'));

  // ── Zero-value markers ────────────────────────────────────────────────────────
  test('"it is clear that" → ""', () =>
    expect(assistant('It is clear that the loop is O(n).')).toBe('The loop is O(n).'));

  test('"as you can see" → ""', () =>
    expect(assistant('As you can see, the output is correct.')).toBe('The output is correct.'));

  test('"as mentioned" → ""', () =>
    expect(assistant('As mentioned, the fix is on line 42.')).toBe('The fix is on line 42.'));

  test('"as previously mentioned above" → ""', () =>
    expect(assistant('As previously mentioned above, avoid globals.')).toBe('Avoid globals.'));

  // ── Standard abbreviations (new) ─────────────────────────────────────────────
  test('"for instance" → "e.g."', () =>
    expect(assistant('Use a short-circuit, for instance an early return.')).toBe('Use a short-circuit, e.g. an early return.'));

  test('"as opposed to" → "vs."', () =>
    expect(assistant('Use composition as opposed to inheritance.')).toBe('Use composition vs. inheritance.'));

  test('"compared to" → "vs."', () =>
    expect(assistant('This is faster compared to the naive approach.')).toBe('This is faster vs. the naive approach.'));
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

  test('removes "actually"', () =>
    expect(assistant('Actually, the bug is in the parser.')).toBe('the bug is in the parser.'));

  test('removes "frankly"', () =>
    expect(assistant('Frankly, this approach will not scale.')).toBe('this approach will not scale.'));

  test('removes "unfortunately"', () =>
    expect(assistant('Unfortunately, there is no built-in way to do this.')).toBe('there is no built-in way to do this.'));

  test('removes "importantly"', () =>
    expect(assistant('Importantly, the tests still pass.')).toBe('the tests still pass.'));

  test('removes "notably"', () =>
    expect(assistant('Notably, this only affects Windows.')).toBe('this only affects Windows.'));

  test('removes "interestingly"', () =>
    expect(assistant('Interestingly, the slower algorithm wins here.')).toBe('the slower algorithm wins here.'));

  test('removes "of course" phrase', () =>
    expect(assistant('Of course, you need to restart the server.')).toBe('You need to restart the server.'));

  test('removes "certainly"', () =>
    expect(assistant('Certainly, that is one valid approach.')).toBe('That is one valid approach.'));

  test('does NOT remove "theoretically" — qualifies the claim', () =>
    expect(assistant('Theoretically, this should work.')).toBe('Theoretically, this should work.'));

  test('does NOT remove "technically" — qualifies the claim', () =>
    expect(assistant('Technically, this is correct.')).toBe('Technically, this is correct.'));
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

// ── Structural meta-commentary ────────────────────────────────────────────────

describe('rules tier — structural meta-commentary', () => {
  test('strips "Here is X:"', () =>
    expect(assistant('Here is the solution: use a hash map.')).toBe('Use a hash map.'));

  test('strips "Here\'s X:"', () =>
    expect(assistant("Here's what you need: install the package.")).toBe('Install the package.'));

  test('strips "Here are X:"', () =>
    expect(assistant('Here are the steps: clone, install, run.')).toBe('Clone, install, run.'));

  test('strips "Below is X:"', () =>
    expect(assistant('Below is an example: update the config.')).toBe('Update the config.'));

  test('strips "Below are X:"', () =>
    expect(assistant('Below are the results: 3 passed, 0 failed.')).toBe('3 passed, 0 failed.'));

  test('strips "The following X:"', () =>
    expect(assistant('The following code fixes the bug: check the null case.')).toBe('Check the null case.'));

  test('strips "The following is X:"', () =>
    expect(assistant('The following is a list of options: A, B, C.')).toBe('A, B, C.'));

  test('strips "I\'ve outlined X:"', () =>
    expect(assistant("I've outlined the changes below: remove the loop.")).toBe('Remove the loop.'));

  test('strips "I have described X:"', () =>
    expect(assistant('I have described the fix above: update the config.')).toBe('Update the config.'));

  test('does NOT strip when no colon — "Here is an example."', () =>
    expect(assistant('Here is an example.')).toBe('Here is an example.'));

  test('strips mid-text after sentence boundary', () =>
    expect(assistant('The bug is confirmed. Here is the fix: delete line 42.'))
      .toBe('The bug is confirmed. delete line 42.'));

  test('is not applied to user turns', () =>
    expect(user('Here is my code: const x = 1;')).toBe('Here is my code: const x = 1;'));
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

// ── Real API message formats ───────────────────────────────────────────────────

describe('compressHistory() — non-string content pass-through', () => {
  test('passes through null content without crashing', () => {
    const messages = [{ role: 'assistant', content: null }] as any;
    const { messages: result } = compressHistory(messages, rules);
    expect(result[0]!.content).toBeNull();
    expect(result[0]!._stats.savedTokens).toBe(0);
  });

  test('passes through array content without crashing or mutating', () => {
    const blocks = [{ type: 'text', text: 'hello' }];
    const messages = [{ role: 'user', content: blocks }] as any;
    const { messages: result } = compressHistory(messages, rules);
    expect(Array.isArray(result[0]!.content)).toBe(true);
    expect(result[0]!.content).toEqual(blocks);
  });

  test('passes through tool role messages without compressing', () => {
    const messages = [{ role: 'tool', tool_use_id: 'abc', content: '{"result":"ok"}' }] as any;
    const { messages: result } = compressHistory(messages, rules);
    expect(result[0]!.content).toBe('{"result":"ok"}');
    expect(result[0]!._stats.savedTokens).toBe(0);
  });

  test('passes through system role messages without compressing', () => {
    const messages = [{ role: 'system', content: 'Certainly! You are a helpful assistant.' }] as any;
    const { messages: result } = compressHistory(messages, rules);
    expect(result[0]!.content).toContain('Certainly');
  });

  test('compresses only user/assistant string messages in a mixed real-API history', () => {
    const messages = [
      { role: 'system',    content: 'Certainly! You are helpful.' },
      { role: 'user',      content: 'Please explain closures. Thank you!' },
      { role: 'assistant', content: null },
      { role: 'tool',      content: '{"data":1}', tool_use_id: 'x' },
    ] as any;
    const { messages: result } = compressHistory(messages, rules);
    expect(result[0]!.content).toContain('Certainly');  // system — not touched
    expect(result[1]!.content).not.toContain('Please'); // user — compressed
    expect(result[1]!.content).not.toContain('Thank you');
    expect(result[2]!.content).toBeNull();              // null — passed through
    expect(result[3]!.content).toBe('{"data":1}');      // tool — not touched
  });
});
