import { describe, expect, test } from 'bun:test';
import { compress, TIERS } from '../src/index.ts';
import type { CompressConfig } from '../src/index.ts';

const both: CompressConfig = { tiers: [TIERS.RULES, TIERS.NLP], tokenMethod: 'chars' };
const assistant = (text: string, cfg = both) => compress(text, 'assistant', cfg).text;

describe('protected blocks', () => {
  test('does not compress content inside fenced code blocks', () => {
    const text = 'Here is an example:\n```\nfor example in order to\n```';
    expect(assistant(text)).toContain('for example in order to');
  });

  test('does not compress inline code', () => {
    const text = 'Call `getValueForExample()` to retrieve it.';
    expect(assistant(text)).toContain('`getValueForExample()`');
  });

  test('does not compress URLs', () => {
    const url = 'https://example.com/in-order-to/basically/test';
    const text = `See the docs at ${url} for details.`;
    expect(assistant(text)).toContain(url);
  });

  test('compresses text around code blocks but not inside', () => {
    const text = 'Certainly! For example:\n```\nconst x = 1;\n```\nI hope this helps!';
    const result = assistant(text);
    expect(result).not.toContain('Certainly');
    expect(result).not.toContain('I hope this helps');
    expect(result).toContain('const x = 1;');
  });
});
