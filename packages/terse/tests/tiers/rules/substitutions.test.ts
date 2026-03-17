import { describe, expect, test } from 'bun:test';
import { compress, TIERS } from '@src/index.ts';
import type { CompressConfig } from '@src/index.ts';

const rules: CompressConfig = { tiers: [TIERS.RULES], tokenMethod: 'chars' };
const assistant = (text: string, cfg = rules) => compress(text, 'assistant', cfg).text;

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

  test('"provided that" → "if"', () =>
    expect(assistant('You can merge provided that all tests pass.')).toBe('You can merge if all tests pass.'));

  // ── Causal / concessive ───────────────────────────────────────────────────────
  test('"despite the fact that" → "although"', () =>
    expect(assistant('It works despite the fact that the input is malformed.'))
      .toBe('It works although the input is malformed.'));

  test('"as a result of" → "due to"', () =>
    expect(assistant('The build failed as a result of a missing dependency.'))
      .toBe('The build failed due to a missing dependency.'));

  test('"as a consequence of" → "due to"', () =>
    expect(assistant('The system is slow as a consequence of the N+1 queries.'))
      .toBe('The system is slow due to the N+1 queries.'));

  test('"on the other hand" → "however"', () =>
    expect(assistant('This is fast. On the other hand, it uses more memory.'))
      .toBe('This is fast. however, it uses more memory.'));

  test('"in contrast to" → "unlike"', () =>
    expect(assistant('In contrast to Python, JavaScript is single-threaded.'))
      .toBe('Unlike Python, JavaScript is single-threaded.'));

  test('"in comparison with" → "vs."', () =>
    expect(assistant('The new approach is faster in comparison with the old one.'))
      .toBe('The new approach is faster vs. the old one.'));

  // ── Temporal ─────────────────────────────────────────────────────────────────
  test('"at the end of the day" → "ultimately"', () =>
    expect(assistant('At the end of the day, correctness matters more than speed.'))
      .toBe('Ultimately, correctness matters more than speed.'));

  test('"for the time being" → "for now"', () =>
    expect(assistant('Use a placeholder for the time being.')).toBe('Use a placeholder for now.'));

  test('"at the moment" → "now"', () =>
    expect(assistant('The feature is not supported at the moment.')).toBe('The feature is not supported now.'));

  test('"for the foreseeable future" → "for now"', () =>
    expect(assistant('Keep the workaround for the foreseeable future.')).toBe('Keep the workaround for now.'));

  test('"in the future" → "later"', () =>
    expect(assistant('In the future, we can optimize this.')).toBe('Later, we can optimize this.'));

  test('"in the long run" → "ultimately"', () =>
    expect(assistant('In the long run, this saves memory.')).toBe('Ultimately, this saves memory.'));

  test('"from time to time" → "sometimes"', () =>
    expect(assistant('The cache is invalidated from time to time.')).toBe('The cache is invalidated sometimes.'));

  test('"on a daily basis" → "daily"', () =>
    expect(assistant('The job runs on a daily basis.')).toBe('The job runs daily.'));

  // ── Quantity / scope ──────────────────────────────────────────────────────────
  test('"a wide range of" → "many"', () =>
    expect(assistant('It supports a wide range of formats.')).toBe('It supports many formats.'));

  test('"the vast majority of" → "most"', () =>
    expect(assistant('The vast majority of requests succeed.')).toBe('Most requests succeed.'));

  test('"a number of" → "several"', () =>
    expect(assistant('There are a number of edge cases to handle.')).toBe('There are several edge cases to handle.'));

  test('"a growing number of" → "more"', () =>
    expect(assistant('A growing number of users are affected.')).toBe('More users are affected.'));

  test('"an increasing number of" → "more"', () =>
    expect(assistant('An increasing number of requests are failing.')).toBe('More requests are failing.'));

  test('"a handful of" → "few"', () =>
    expect(assistant('Only a handful of cases remain.')).toBe('Only few cases remain.'));

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

  test('"there is no need to" → "no need to"', () =>
    expect(assistant("There is no need to restart the server.")).toBe('No need to restart the server.'));

  test('"there\'s no need for" → "no need for"', () =>
    expect(assistant("There's no need for a full rebuild.")).toBe("No need for a full rebuild."));

  test('"in any case" → "anyway"', () =>
    expect(assistant('In any case, the tests should cover this.')).toBe('Anyway, the tests should cover this.'));

  test('"in that case" → "then"', () =>
    expect(assistant('In that case, use a different algorithm.')).toBe('Then, use a different algorithm.'));

  // ── Action nominalizations ────────────────────────────────────────────────────
  test('"makes changes to" → "change"', () =>
    expect(assistant('The migration makes changes to the schema.')).toBe('The migration change the schema.'));

  test('"take a look at" → "check"', () =>
    expect(assistant('Take a look at the error log.')).toBe('Check the error log.'));

  test('"have an impact on" → "affect"', () =>
    expect(assistant('This will have an impact on performance.')).toBe('This will affect performance.'));

  test('"has an effect on" → "affects"', () =>
    expect(assistant('Caching has an effect on latency.')).toBe('Caching affects latency.'));

  test('"take note of" → "note"', () =>
    expect(assistant('Take note of the order of operations.')).toBe('Note the order of operations.'));

  test('"take advantage of" → "use"', () =>
    expect(assistant('Take advantage of the built-in sort.')).toBe('Use the built-in sort.'));

  test('"take care of" → "handle"', () =>
    expect(assistant('The middleware will take care of authentication.')).toBe('The middleware will handle authentication.'));

  test('"keep track of" → "track"', () =>
    expect(assistant('Keep track of the connection count.')).toBe('Track the connection count.'));

  test('"get rid of" → "remove"', () =>
    expect(assistant('Get rid of the unused imports.')).toBe('Remove the unused imports.'));

  test('"give rise to" → "cause"', () =>
    expect(assistant('Circular imports give rise to subtle bugs.')).toBe('Circular imports cause subtle bugs.'));

  test('"go ahead and" → "" (agentic)', () =>
    expect(assistant("I'll go ahead and refactor the service layer.")).toBe("I'll refactor the service layer."));

  test('"all you need to do is" → ""', () =>
    expect(assistant('All you need to do is restart the server.')).toBe('Restart the server.'));

  test('"what you need to do is" → ""', () =>
    expect(assistant('What you need to do is update the config.')).toBe('Update the config.'));

  // ── Zero-value discourse markers ──────────────────────────────────────────────
  test('"it is clear that" → ""', () =>
    expect(assistant('It is clear that the loop is O(n).')).toBe('The loop is O(n).'));

  test('"as you can see" → ""', () =>
    expect(assistant('As you can see, the output is correct.')).toBe('The output is correct.'));

  test('"as mentioned" → ""', () =>
    expect(assistant('As mentioned, the fix is on line 42.')).toBe('The fix is on line 42.'));

  test('"as previously mentioned above" → ""', () =>
    expect(assistant('As previously mentioned above, avoid globals.')).toBe('Avoid globals.'));

  test('"that being said" → ""', () =>
    expect(assistant('That being said, the fix is straightforward.')).toBe('The fix is straightforward.'));

  test('"with that said" → ""', () =>
    expect(assistant('With that said, let us proceed.')).toBe('Let us proceed.'));

  test('"having said that" → ""', () =>
    expect(assistant('Having said that, the tests are still failing.')).toBe('The tests are still failing.'));

  test('"when all is said and done" → "ultimately"', () =>
    expect(assistant('When all is said and done, simplicity wins.')).toBe('Ultimately, simplicity wins.'));

  test('"long story short" → ""', () =>
    expect(assistant('Long story short, the deploy failed.')).toBe('The deploy failed.'));

  test('"the bottom line is that" → ""', () =>
    expect(assistant('The bottom line is that performance matters.')).toBe('Performance matters.'));

  test('"the bottom line is" → ""', () =>
    expect(assistant('The bottom line is: use indexes.')).toBe('Use indexes.'));

  test('"suffice it to say" → ""', () =>
    expect(assistant('Suffice it to say, the API is unstable.')).toBe('The API is unstable.'));

  test('"to put it simply" → ""', () =>
    expect(assistant('To put it simply, the loop is O(n²).')).toBe('The loop is O(n²).'));

  test('"with this in mind" → ""', () =>
    expect(assistant('With this in mind, consider using a cache.')).toBe('Consider using a cache.'));

  test('"as we discussed" → ""', () =>
    expect(assistant('As we discussed, the timeout is 30s.')).toBe('The timeout is 30s.'));

  test('"as we noted earlier" → ""', () =>
    expect(assistant('As we noted earlier, avoid globals.')).toBe('Avoid globals.'));

  // ── Standard abbreviations ────────────────────────────────────────────────────
  test('"for instance" → "e.g."', () =>
    expect(assistant('Use a short-circuit, for instance an early return.')).toBe('Use a short-circuit, e.g. an early return.'));

  test('"as opposed to" → "vs."', () =>
    expect(assistant('Use composition as opposed to inheritance.')).toBe('Use composition vs. inheritance.'));

  test('"compared to" → "vs."', () =>
    expect(assistant('This is faster compared to the naive approach.')).toBe('This is faster vs. the naive approach.'));
});

import { readFileSync } from 'fs';
import { resolve } from 'path';

interface CompressCase {
  id: string; description: string;
  role: 'user' | 'assistant'; tiers: string[]; input: string; output: string;
}

describe('substitutions — fixture contract', () => {
  const cases: CompressCase[] = JSON.parse(
    readFileSync(resolve(import.meta.dir, '../../../../../fixtures/rules/substitutions.json'), 'utf8')
  );
  for (const c of cases) {
    const cfg = {
      tiers: c.tiers.map(t => TIERS[t.toUpperCase() as keyof typeof TIERS]),
      tokenMethod: 'chars' as const,
    };
    test(c.id, () => expect(compress(c.input, c.role, cfg).text).toBe(c.output));
  }
});
