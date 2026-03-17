import { describe, expect, test } from 'bun:test';
import { compress, compressHistory, TIERS, DEFAULT_CONFIG } from '@src/index.ts';
import type { CompressConfig } from '@src/index.ts';

const rules: CompressConfig = { tiers: [TIERS.RULES], tokenMethod: 'chars' };
const both:  CompressConfig = { tiers: [TIERS.RULES, TIERS.NLP], tokenMethod: 'chars' };

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
