import { describe, expect, test } from 'bun:test';
import { compress, TIERS } from '../../../src/index.ts';
import type { CompressConfig } from '../../../src/index.ts';

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
