import { describe, expect, test } from 'bun:test';
import { compress, TIERS } from '@src/index.ts';
import type { CompressConfig } from '@src/index.ts';

const rules: CompressConfig = { tiers: [TIERS.RULES], tokenMethod: 'chars' };
const assistant = (text: string) => compress(text, 'assistant', rules).text;
const user      = (text: string) => compress(text, 'user',      rules).text;

describe('markdown — horizontal dividers', () => {
  test('strips --- on its own line', () =>
    expect(assistant('First point.\n---\nSecond point.')).toBe('First point. Second point.'));

  test('strips *** on its own line', () =>
    expect(assistant('First point.\n***\nSecond point.')).toBe('First point. Second point.'));

  test('strips ___ on its own line', () =>
    expect(assistant('First point.\n___\nSecond point.')).toBe('First point. Second point.'));

  test('strips divider with surrounding spaces', () =>
    expect(assistant('A\n  ---  \nB')).toBe('A B'));

  test('strips longer dividers (----)', () =>
    expect(assistant('A\n-----\nB')).toBe('A B'));
});

describe('markdown — table separator rows', () => {
  test('strips | --- | --- | row', () =>
    expect(assistant('| Col A | Col B |\n| --- | --- |\n| val | val |'))
      .toBe('| Col A | Col B | | val | val |'));

  test('strips aligned separator row (| :--- | ---: |)', () =>
    expect(assistant('| H |\n| :---: |\n| v |')).toBe('| H | | v |'));
});

describe('markdown — heading markers', () => {
  test('strips # heading', () =>
    expect(assistant('# Introduction\nSome text.')).toBe('Introduction Some text.'));

  test('strips ## heading', () =>
    expect(assistant('## Overview\nDetails here.')).toBe('Overview Details here.'));

  test('strips ### heading', () =>
    expect(assistant('### Step 1\nDo this.')).toBe('Step 1 Do this.'));

  test('strips up to ###### heading', () =>
    expect(assistant('###### Deep\nText.')).toBe('Deep Text.'));

  test('preserves heading text content', () => {
    const result = assistant('## Key Concepts\nExplanation follows.');
    expect(result).toContain('Key Concepts');
  });
});

describe('markdown — bold', () => {
  test('strips **bold** markers', () =>
    expect(assistant('This is **important**.')).toBe('This is important.'));

  test('strips **bold** mid-sentence', () =>
    expect(assistant('Use **const** not var.')).toBe('Use const not var.'));

  test('strips multiple **bold** in one string', () =>
    expect(assistant('**One** and **two**.')).toBe('One and two.'));

  test('preserves bold text content', () => {
    const result = assistant('Always **validate input**.');
    expect(result).toContain('validate input');
    expect(result).not.toContain('**');
  });
});

describe('markdown — italic', () => {
  test('strips *italic* markers', () =>
    expect(assistant('The *key* insight is locality.')).toBe('The key insight is locality.'));

  test('does NOT strip list bullet (* item)', () =>
    expect(assistant('* run tests\n* deploy')).toContain('* run'));

  test('does NOT strip * in math context (no closing *)', () =>
    expect(assistant('O(n*log n) complexity')).toContain('n*log'));
});

describe('markdown — em-dash', () => {
  test('converts spaced em-dash to comma', () =>
    expect(assistant('Use const — not var.')).toBe('Use const, not var.'));

  test('handles multiple em-dashes', () =>
    expect(assistant('A — B — C.')).toBe('A, B, C.'));
});

describe('markdown — curly quotes', () => {
  test('converts left double curly quote to straight', () =>
    expect(assistant('\u201CHello\u201D world.')).toBe('"Hello" world.'));

  test('converts right single curly quote (apostrophe) to straight', () =>
    expect(assistant('It\u2019s fine.')).toBe("It's fine."));

  test('converts left single curly quote to straight', () =>
    expect(assistant('\u2018quoted\u2019.')).toBe("'quoted'."));
});

describe('markdown — applies to user turns too', () => {
  test('strips bold in user messages', () =>
    expect(user('My code uses **async/await**.')).toBe('My code uses async/await.'));

  test('strips divider in user messages', () =>
    expect(user('Context:\n---\nQuestion here.')).toBe('Context: Question here.'));
});

describe('markdown — does not touch protected blocks', () => {
  test('preserves **bold** inside fenced code block', () => {
    const text = 'Outside.\n```\n**bold inside code**\n```';
    expect(assistant(text)).toContain('**bold inside code**');
  });

  test('preserves **bold** inside inline code', () =>
    expect(assistant('Use `**bold**` for emphasis.')).toContain('`**bold**`'));
});

import { readFileSync } from 'fs';
import { resolve } from 'path';

interface CompressCase {
  id: string; description: string;
  role: 'user' | 'assistant'; tiers: string[]; input: string; output: string;
}

describe('markdown — fixture contract', () => {
  const cases: CompressCase[] = JSON.parse(
    readFileSync(resolve(import.meta.dir, '../../../../../fixtures/rules/markdown.json'), 'utf8')
  );
  for (const c of cases) {
    const cfg = {
      tiers: c.tiers.map(t => TIERS[t.toUpperCase() as keyof typeof TIERS]),
      tokenMethod: 'chars' as const,
    };
    test(c.id, () => expect(compress(c.input, c.role, cfg).text).toBe(c.output));
  }
});
