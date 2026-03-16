import { describe, expect, test } from 'bun:test';
import { compress, TIERS } from '../../../src/index.ts';
import type { CompressConfig } from '../../../src/index.ts';

const rules: CompressConfig = { tiers: [TIERS.RULES], tokenMethod: 'chars' };
const assistant = (text: string, cfg = rules) => compress(text, 'assistant', cfg).text;

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
