import { describe, expect, test } from 'bun:test';
import { compress, TIERS } from '../../../src/index.ts';
import type { CompressConfig } from '../../../src/index.ts';

const rules: CompressConfig = { tiers: [TIERS.RULES], tokenMethod: 'chars' };
const assistant = (text: string, cfg = rules) => compress(text, 'assistant', cfg).text;
const user      = (text: string, cfg = rules) => compress(text, 'user',      cfg).text;

describe('rules tier — structural meta-commentary', () => {
  // ── Announcement frames ───────────────────────────────────────────────────────
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

  // ── Section labels — closing/summary ─────────────────────────────────────────
  describe('section labels — closing/summary', () => {
    test('strips "To summarize, "', () =>
      expect(assistant('To summarize, the fix is on line 42.')).toBe('The fix is on line 42.'));

    test('strips "In summary:"', () =>
      expect(assistant('In summary: use a cache.')).toBe('Use a cache.'));

    test('strips "In conclusion,"', () =>
      expect(assistant('In conclusion, avoid globals.')).toBe('Avoid globals.'));

    test('strips "To conclude:"', () =>
      expect(assistant('To conclude: always write tests.')).toBe('Always write tests.'));

    test('strips "To recap,"', () =>
      expect(assistant('To recap, we removed the N+1 query.')).toBe('We removed the N+1 query.'));

    test('strips "To sum up:"', () =>
      expect(assistant('To sum up: three bugs were fixed.')).toBe('Three bugs were fixed.'));

    test('strips "In short,"', () =>
      expect(assistant('In short, the loop is O(n).')).toBe('The loop is O(n).'));

    test('strips "In brief:"', () =>
      expect(assistant('In brief: deploy the patch.')).toBe('Deploy the patch.'));

    test('strips "In closing,"', () =>
      expect(assistant('In closing, good luck with the refactor.')).toBe('Good luck with the refactor.'));

    test('strips "To wrap up:"', () =>
      expect(assistant('To wrap up: the PR is ready to merge.')).toBe('The PR is ready to merge.'));

    test('does NOT strip "in short" mid-sentence', () =>
      expect(assistant('He explained it in short form.')).toBe('He explained it in short form.'));
  });

  // ── Section labels — opening frames ──────────────────────────────────────────
  describe('section labels — opening frames', () => {
    test('strips "To begin,"', () =>
      expect(assistant('To begin, set up your environment.')).toBe('Set up your environment.'));

    test('strips "To start:"', () =>
      expect(assistant('To start: install the dependencies.')).toBe('Install the dependencies.'));

    test('strips "To start with,"', () =>
      expect(assistant('To start with, understand the data model.')).toBe('Understand the data model.'));

    test('strips "To begin with:"', () =>
      expect(assistant('To begin with: run the migrations.')).toBe('Run the migrations.'));

    test('strips "First things first:"', () =>
      expect(assistant('First things first: back up the database.')).toBe('Back up the database.'));

    test('does NOT strip "To begin" without comma/colon', () =>
      expect(assistant('To begin the process, open the terminal.')).toBe('To begin the process, open the terminal.'));

    test('does NOT strip "To start" without comma/colon', () =>
      expect(assistant('To start the server, run npm start.')).toBe('To start the server, run npm start.'));
  });

  // ── Section labels — explanation frames ──────────────────────────────────────
  describe('section labels — explanation frames', () => {
    test('strips "To explain:"', () =>
      expect(assistant('To explain: closures capture their enclosing scope.')).toBe('Closures capture their enclosing scope.'));

    test('strips "To clarify,"', () =>
      expect(assistant('To clarify, the error only occurs on write.')).toBe('The error only occurs on write.'));

    test('strips "To illustrate:"', () =>
      expect(assistant('To illustrate: consider a sorted array.')).toBe('Consider a sorted array.'));

    test('does NOT strip "To clarify" without comma/colon', () =>
      expect(assistant('To clarify the confusion, read the docs.')).toBe('To clarify the confusion, read the docs.'));
  });

  // ── Section labels — rephrasing frames ───────────────────────────────────────
  describe('section labels — rephrasing frames', () => {
    test('strips "Put differently,"', () =>
      expect(assistant('Put differently, the function is not pure.')).toBe('The function is not pure.'));

    test('strips "Put simply:"', () =>
      expect(assistant('Put simply: the cache is cold.')).toBe('The cache is cold.'));

    test('strips "Simply put,"', () =>
      expect(assistant('Simply put, the tests are broken.')).toBe('The tests are broken.'));
  });

  // ── Section labels — terse content labels ────────────────────────────────────
  describe('section labels — terse content labels', () => {
    test('strips "TL;DR:"', () =>
      expect(assistant('TL;DR: the fix is on line 42.')).toBe('The fix is on line 42.'));

    test('strips "Bottom line:"', () =>
      expect(assistant('Bottom line: use indexes.')).toBe('Use indexes.'));

    test('strips "Note:"', () =>
      expect(assistant('Note: this only affects Windows.')).toBe('This only affects Windows.'));

    test('strips "Key takeaway:"', () =>
      expect(assistant('Key takeaway: always validate input.')).toBe('Always validate input.'));

    test('strips "Key takeaways:"', () =>
      expect(assistant('Key takeaways: test, document, review.')).toBe('Test, document, review.'));

    test('does NOT strip "Note" without colon/comma', () =>
      expect(assistant('Note the performance implications carefully.')).toBe('Note the performance implications carefully.'));
  });
});
