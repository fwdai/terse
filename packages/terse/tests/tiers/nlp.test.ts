import { describe, expect, test } from 'bun:test';
import { compress, TIERS } from '@src/index.ts';
import type { CompressConfig } from '@src/index.ts';

const rules: CompressConfig = { tiers: [TIERS.RULES], tokenMethod: 'chars' };
const nlp:   CompressConfig = { tiers: [TIERS.NLP],   tokenMethod: 'chars' };
const both:  CompressConfig = { tiers: [TIERS.RULES, TIERS.NLP], tokenMethod: 'chars' };
const assistant = (text: string, cfg = rules) => compress(text, 'assistant', cfg).text;

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

import { readFileSync } from 'fs';
import { resolve } from 'path';

interface CompressCase {
  id: string; description: string;
  role: 'user' | 'assistant'; tiers: string[]; input: string; output: string;
}

describe('nlp — fixture contract', () => {
  const cases: CompressCase[] = JSON.parse(
    readFileSync(resolve(import.meta.dir, '../../../../fixtures/nlp/nlp.json'), 'utf8')
  );
  for (const c of cases) {
    const cfg = {
      tiers: c.tiers.map(t => TIERS[t.toUpperCase() as keyof typeof TIERS]),
      tokenMethod: 'chars' as const,
    };
    test(c.id, () => expect(compress(c.input, c.role, cfg).text).toBe(c.output));
  }
});
