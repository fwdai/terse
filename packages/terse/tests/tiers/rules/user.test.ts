import { describe, expect, test } from 'bun:test';
import { compress, TIERS } from '@src/index.ts';
import type { CompressConfig } from '@src/index.ts';

const rules: CompressConfig = { tiers: [TIERS.RULES], tokenMethod: 'chars' };
const user = (text: string, cfg = rules) => compress(text, 'user', cfg).text;

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

    test('removes "Thank you very much"', () =>
      expect(user('Fix this bug. Thank you very much!')).toBe('Fix this bug.'));

    test('removes "Thanks again"', () =>
      expect(user('Explain closures. Thanks again!')).toBe('Explain closures.'));

    test('removes "Many thanks"', () =>
      expect(user('Review this PR. Many thanks')).toBe('Review this PR.'));

    test('removes "Much appreciated"', () =>
      expect(user('Refactor this function. Much appreciated!')).toBe('Refactor this function.'));

    test('removes "Cheers"', () =>
      expect(user('Add error handling. Cheers!')).toBe('Add error handling.'));

    test('removes "I really appreciate it"', () =>
      expect(user('Debug this for me. I really appreciate it.')).toBe('Debug this for me.'));

    test('removes "I appreciate your time"', () =>
      expect(user('Review this PR. I appreciate your time.')).toBe('Review this PR.'));

    test('removes "Hope that makes sense"', () =>
      expect(user('Here is the stack trace. Hope that makes sense.')).toBe('Here is the stack trace.'));

    test('removes "Hope this helps clarify"', () =>
      expect(user('I added more context above. Hope this helps clarify!')).toBe('I added more context above.'));

    test('removes "Let me know if that\'s clear"', () =>
      expect(user("I've described the issue above. Let me know if that's clear.")).toBe("I've described the issue above."));

    test('removes "Just let me know if you need more context"', () =>
      expect(user('Here is my code. Just let me know if you need more context.'))
        .toBe('Here is my code.'));

    test('removes "Happy to elaborate"', () =>
      expect(user('That is the issue I am seeing. Happy to elaborate!')).toBe('That is the issue I am seeing.'));

    test('removes "Happy to provide more details"', () =>
      expect(user('The error only happens on prod. Happy to provide more details if needed.'))
        .toBe('The error only happens on prod.'));

    test('removes "Sorry for the long message"', () =>
      expect(user('Fix this function. Sorry for the long message!')).toBe('Fix this function.'));

    test('removes "Sorry if that\'s confusing"', () =>
      expect(user("I know this is complex. Sorry if that's confusing.")).toBe('I know this is complex.'));

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

    test('strips "Good morning, "', () =>
      expect(user('Good morning, can you review this PR?')).toBe('Review this PR?'));

    test('strips "Good afternoon! "', () =>
      expect(user('Good afternoon! What is a closure?')).toBe('What is a closure?'));

    test('strips "Howdy, "', () =>
      expect(user('Howdy, help me debug this.')).toBe('Help me debug this.'));

    test('strips "Greetings! "', () =>
      expect(user('Greetings! Explain async/await.')).toBe('Explain async/await.'));
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

    test('strips "Silly question:"', () =>
      expect(user('Silly question: what does NaN mean?')).toBe('What does NaN mean?'));

    test('strips "Hypothetical question:"', () =>
      expect(user('Hypothetical question: what would happen if we removed the index?'))
        .toBe('What would happen if we removed the index?'));
  });

  describe('acknowledgment / continuation starters (require comma)', () => {
    test('strips "Okay, "', () =>
      expect(user('Okay, can you also add logging?')).toBe('Also add logging?'));

    test('strips "Alright, "', () =>
      expect(user('Alright, now explain why this is slow.')).toBe('Now explain why this is slow.'));

    test('strips "Right, "', () =>
      expect(user('Right, so what should I change?')).toBe('So what should I change?'));

    test('strips "Well, "', () =>
      expect(user('Well, that makes sense but how do I fix it?')).toBe('That makes sense but how do I fix it?'));

    test('strips "Great, " — acknowledgment, not compliment', () =>
      expect(user('Great, can you also update the tests?')).toBe('Also update the tests?'));

    test('strips "Wait, "', () =>
      expect(user('Wait, that is not what I asked.')).toBe('That is not what I asked.'));

    test('strips "Oh, "', () =>
      expect(user('Oh, I see. Can you also explain X?')).toBe('I see. Can you also explain X?'));

    test('strips "Hmm, "', () =>
      expect(user('Hmm, I am not sure that is right.')).toBe('I am not sure that is right.'));

    test('does NOT strip "Great" without comma — may be content', () =>
      expect(user('Great work on the refactor!')).toBe('Great work on the refactor!'));
  });

  describe('additional hedges', () => {
    test('strips "I might be wrong, but"', () =>
      expect(user('I might be wrong, but this looks like a memory leak.')).toBe('This looks like a memory leak.'));

    test('strips "Correct me if I\'m wrong, but"', () =>
      expect(user("Correct me if I'm wrong, but the timeout should be 30s.")).toBe('The timeout should be 30s.'));

    test('strips "I\'m not sure if this is right, but"', () =>
      expect(user("I'm not sure if this is right, but I think we need to flush the cache."))
        .toBe('I think we need to flush the cache.'));

    test('strips "I know this might be obvious, but"', () =>
      expect(user('I know this might be obvious, but why does this return null?'))
        .toBe('Why does this return null?'));

    test('strips "I know this could be basic, but"', () =>
      expect(user('I know this could be basic, but explain what a pointer is.'))
        .toBe('Explain what a pointer is.'));

    test("strips \"Forgive me if I'm wrong, but\"", () =>
      expect(user("Forgive me if I'm wrong, but shouldn't this be async?"))
        .toBe("Shouldn't this be async?"));

    test('strips "Not sure if this is relevant, but"', () =>
      expect(user('Not sure if this is relevant, but the error appears after a restart.'))
        .toBe('The error appears after a restart.'));

    test("strips \"I'm probably wrong, but\"", () =>
      expect(user("I'm probably wrong, but this looks like a deadlock."))
        .toBe('This looks like a deadlock.'));
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

    test('strips "Do you mind explaining"', () =>
      expect(user('Do you mind explaining how garbage collection works?'))
        .toBe('Explaining how garbage collection works?'));

    test('strips "Could I ask you to"', () =>
      expect(user('Could I ask you to rewrite this function without globals?'))
        .toBe('Rewrite this function without globals?'));

    test("strips \"I'd appreciate it if you could\"", () =>
      expect(user("I'd appreciate it if you could add JSDoc comments."))
        .toBe('Add JSDoc comments.'));

    test("strips \"I'm hoping you could\"", () =>
      expect(user("I'm hoping you could help me understand monads."))
        .toBe('Help me understand monads.'));

    test("strips \"I'm wondering if you could\"", () =>
      expect(user("I'm wondering if you could generate a test suite for this module."))
        .toBe('Generate a test suite for this module.'));
  });

  describe('availability-conditional openers', () => {
    test('strips "If you have a moment, could you"', () =>
      expect(user('If you have a moment, could you review my PR?'))
        .toBe('Review my PR?'));

    test('strips "If you have time, can you"', () =>
      expect(user('If you have time, can you explain this error?'))
        .toBe('Explain this error?'));

    test('strips "Whenever you get a chance, can you"', () =>
      expect(user('Whenever you get a chance, can you update the docs?'))
        .toBe('Update the docs?'));

    test('strips "When you have a minute, would you"', () =>
      expect(user('When you have a minute, would you check this logic?'))
        .toBe('Check this logic?'));
  });

  describe('preserved user content', () => {
    test('keeps "I think" — epistemic signal', () =>
      expect(user('I think the issue is on line 42.')).toBe('I think the issue is on line 42.'));

    test('keeps "I believe" — epistemic signal', () =>
      expect(user("I believe this might be a race condition.")).toBe("I believe this might be a race condition."));
  });
});

import { readFileSync } from 'fs';
import { resolve } from 'path';

interface CompressCase {
  id: string; description: string;
  role: 'user' | 'assistant'; tiers: string[]; input: string; output: string;
}

describe('user — fixture contract', () => {
  const cases: CompressCase[] = JSON.parse(
    readFileSync(resolve(import.meta.dir, '../../../../../fixtures/rules/user.json'), 'utf8')
  );
  for (const c of cases) {
    const cfg = {
      tiers: c.tiers.map(t => TIERS[t.toUpperCase() as keyof typeof TIERS]),
      tokenMethod: 'chars' as const,
    };
    test(c.id, () => expect(compress(c.input, c.role, cfg).text).toBe(c.output));
  }
});
