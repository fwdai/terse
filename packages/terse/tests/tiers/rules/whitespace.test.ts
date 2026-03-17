import { describe, expect, test } from 'bun:test';
import { compress, TIERS } from '@src/index.ts';
import type { CompressConfig } from '@src/index.ts';

const rules: CompressConfig = { tiers: [TIERS.RULES], tokenMethod: 'chars' };
const assistant = (text: string, cfg = rules) => compress(text, 'assistant', cfg).text;

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

import { readFileSync } from 'fs';
import { resolve } from 'path';

interface CompressCase {
  id: string; description: string;
  role: 'user' | 'assistant'; tiers: string[]; input: string; output: string;
}

describe('whitespace — fixture contract', () => {
  const cases: CompressCase[] = JSON.parse(
    readFileSync(resolve(import.meta.dir, '../../../../../fixtures/rules/whitespace.json'), 'utf8')
  );
  for (const c of cases) {
    const cfg = {
      tiers: c.tiers.map(t => TIERS[t.toUpperCase() as keyof typeof TIERS]),
      tokenMethod: 'chars' as const,
    };
    test(c.id, () => expect(compress(c.input, c.role, cfg).text).toBe(c.output));
  }
});
