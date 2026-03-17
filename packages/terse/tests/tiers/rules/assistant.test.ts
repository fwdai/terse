import { describe, expect, test } from 'bun:test';
import { compress, TIERS } from '@src/index.ts';
import type { CompressConfig } from '@src/index.ts';
import { readFileSync } from 'fs';
import { resolve } from 'path';

const rules: CompressConfig = { tiers: [TIERS.RULES], tokenMethod: 'chars' };
const assistant = (text: string, cfg = rules) => compress(text, 'assistant', cfg).text;

describe('rules tier — assistant boilerplate', () => {
  describe('openers', () => {
    test('removes "Certainly!"', () =>
      expect(assistant('Certainly! Here is the answer.')).toBe('Here is the answer.'));

    test('removes "Sure,"', () =>
      expect(assistant('Sure, here is what you need.')).toBe('Here is what you need.'));

    test('removes "Great question!"', () =>
      expect(assistant("Great question! Let's explore this.")).toBe("Let's explore this."));

    test("removes \"I'd be happy to help\"", () =>
      expect(assistant("I'd be happy to help. Here is the answer.")).toBe('Here is the answer.'));

    test("removes \"I'm glad to assist\"", () =>
      expect(assistant("I'm glad to assist. Here's what I found.")).toBe("Here's what I found."));

    test('removes "As an AI language model"', () =>
      expect(assistant('As an AI language model, I cannot browse the web.'))
        .toBe('I cannot browse the web.'));

    test('removes "Thank you for your question"', () =>
      expect(assistant('Thank you for your question. The answer is 42.'))
        .toBe('The answer is 42.'));

    // Short affirmations
    test('removes "No problem!"', () =>
      expect(assistant('No problem! Here is how to fix it.')).toBe('Here is how to fix it.'));

    test('removes "Got it."', () =>
      expect(assistant('Got it. Here is the updated version.')).toBe('Here is the updated version.'));

    test('removes "Understood."', () =>
      expect(assistant('Understood. I will avoid globals.')).toBe('I will avoid globals.'));

    test('removes "Perfect!"', () =>
      expect(assistant('Perfect! That is the right approach.')).toBe('That is the right approach.'));

    // Acknowledgment phrases
    test('removes "I understand."', () =>
      expect(assistant('I understand. The issue is in the parser.')).toBe('The issue is in the parser.'));

    test('removes "I understand what you\'re asking."', () =>
      expect(assistant("I understand what you're asking. The answer is 42.")).toBe('The answer is 42.'));

    test('removes "I see what you mean."', () =>
      expect(assistant('I see what you mean. The loop is off by one.')).toBe('The loop is off by one.'));

    test('removes "That makes sense."', () =>
      expect(assistant('That makes sense. Here is a better approach.')).toBe('Here is a better approach.'));

    test("removes \"You're absolutely right.\"", () =>
      expect(assistant("You're absolutely right. We should use a set instead.")).toBe('We should use a set instead.'));

    // "Let me X." preambles
    test('removes "Let me help you with that."', () =>
      expect(assistant('Let me help you with that. The fix is on line 5.')).toBe('The fix is on line 5.'));

    test('removes "Let me walk you through this."', () =>
      expect(assistant('Let me walk you through this. First, install the deps.')).toBe('First, install the deps.'));

    test('removes "Let me explain."', () =>
      expect(assistant('Let me explain. Closures capture variables by reference.')).toBe('Closures capture variables by reference.'));

    test('removes "Let me break this down."', () =>
      expect(assistant('Let me break this down. There are three steps.')).toBe('There are three steps.'));

    // Agentic acknowledgment
    test('removes "I\'ll help you with that."', () =>
      expect(assistant("I'll help you with that. Here is the updated code.")).toBe('Here is the updated code.'));

    test("removes \"I can help.\"", () =>
      expect(assistant('I can help. The problem is a missing null check.')).toBe('The problem is a missing null check.'));

    // Analysis starters
    test('removes "Based on what you\'ve described,"', () =>
      expect(assistant("Based on what you've described, the issue is a race condition.")).toBe('The issue is a race condition.'));

    test('removes "Looking at your code,"', () =>
      expect(assistant('Looking at your code, the loop runs one extra time.')).toBe('The loop runs one extra time.'));

    test('removes "To answer your question,"', () =>
      expect(assistant('To answer your question, yes it is safe to call this twice.')).toBe('Yes it is safe to call this twice.'));

    // Safety: does NOT strip substantive "I understand X"
    test('does NOT strip "I understand Rust\'s ownership model" — substantive', () =>
      expect(assistant("I understand Rust's ownership model. It ensures memory safety."))
        .toBe("I understand Rust's ownership model. It ensures memory safety."));
  });

  describe('closers', () => {
    test('removes "I hope this helps!"', () =>
      expect(assistant('The answer is 42. I hope this helps!')).toBe('The answer is 42.'));

    test('removes "Let me know if you have any questions."', () =>
      expect(assistant('Use recursion here. Let me know if you have any questions.'))
        .toBe('Use recursion here.'));

    test('removes "Feel free to ask"', () =>
      expect(assistant('That covers the basics. Feel free to ask if you need more details.'))
        .toBe('That covers the basics.'));

    test('removes "Is there anything else"', () =>
      expect(assistant('Done. Is there anything else I can help you with?'))
        .toBe('Done.'));

    test('removes "Does that make sense?"', () =>
      expect(assistant('Use an index on the foreign key. Does that make sense?'))
        .toBe('Use an index on the foreign key.'));

    test('removes "Does this help?"', () =>
      expect(assistant('Set the timeout to 30s. Does this help?'))
        .toBe('Set the timeout to 30s.'));

    test('removes "Hope that makes sense!"', () =>
      expect(assistant('The closure captures x by reference. Hope that makes sense!'))
        .toBe('The closure captures x by reference.'));

    test('removes "That should do it!"', () =>
      expect(assistant('Add a null check on line 5. That should do it!'))
        .toBe('Add a null check on line 5.'));

    test('removes "Give that a try!"', () =>
      expect(assistant('Run npm install first. Give that a try!'))
        .toBe('Run npm install first.'));

    test('removes "Shall I proceed?"', () =>
      expect(assistant('I can refactor the whole module. Shall I proceed?'))
        .toBe('I can refactor the whole module.'));

    test('removes "Let me know your thoughts."', () =>
      expect(assistant('Use a trie for prefix lookups. Let me know your thoughts.'))
        .toBe('Use a trie for prefix lookups.'));

    test('removes "Let me know how it goes."', () =>
      expect(assistant('Restart the service. Let me know how it goes.'))
        .toBe('Restart the service.'));

    test('removes "Let me know if that works."', () =>
      expect(assistant('Add a null check. Let me know if that works.'))
        .toBe('Add a null check.'));

    test('removes "Feel free to reach out."', () =>
      expect(assistant('That covers the basics. Feel free to reach out.'))
        .toBe('That covers the basics.'));

    test('removes "If you have any questions, just ask."', () =>
      expect(assistant('The fix is straightforward. If you have any questions, just ask.'))
        .toBe('The fix is straightforward.'));

    test('removes "Happy to help further!"', () =>
      expect(assistant('That should resolve the issue. Happy to help further!'))
        .toBe('That should resolve the issue.'));

    test('removes "Hope this clarifies things."', () =>
      expect(assistant('Closures capture variables by reference. Hope this clarifies things.'))
        .toBe('Closures capture variables by reference.'));

    test('removes opener and closer together', () =>
      expect(assistant("Certainly! The answer is 42. I hope this helps!"))
        .toBe('The answer is 42.'));
  });
});

interface CompressCase {
  id: string; description: string;
  role: 'user' | 'assistant'; tiers: string[]; input: string; output: string;
}

describe('assistant — fixture contract', () => {
  const cases: CompressCase[] = JSON.parse(
    readFileSync(resolve(import.meta.dir, '../../../../../fixtures/rules/assistant.json'), 'utf8')
  );
  for (const c of cases) {
    const cfg = {
      tiers: c.tiers.map(t => TIERS[t.toUpperCase() as keyof typeof TIERS]),
      tokenMethod: 'chars' as const,
    };
    test(c.id, () => expect(compress(c.input, c.role, cfg).text).toBe(c.output));
  }
});
