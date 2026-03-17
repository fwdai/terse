import { describe, expect, test } from "bun:test";
import { compress, TIERS } from "@src/index.ts";
import type { CompressConfig } from "@src/index.ts";

const both: CompressConfig = {
  tiers: [TIERS.RULES, TIERS.NLP],
  tokenMethod: "chars",
};
const assistant = (text: string, cfg = both) =>
  compress(text, "assistant", cfg).text;

describe("non-Latin bail-out", () => {
  test("passes through Arabic text unchanged", () =>
    expect(assistant("مرحبا، كيف حالك؟")).toBe("مرحبا، كيف حالك؟"));

  test("passes through Chinese text unchanged", () =>
    expect(assistant("你好，请问有什么可以帮助你的吗？")).toBe("你好，请问有什么可以帮助你的吗？"));

  test("passes through Japanese text unchanged", () =>
    expect(assistant("もちろんです！お役に立てて光栄です。")).toBe("もちろんです！お役に立てて光栄です。"));

  test("passes through Ukrainian (Cyrillic) text unchanged", () =>
    expect(assistant("Звичайно! Ось відповідь на ваше запитання.")).toBe("Звичайно! Ось відповідь на ваше запитання."));

  test("Polish (Latin + diacritics) passes Latin check, no rules fire, returns unchanged", () =>
    expect(assistant("Oczywiście! Chętnie Ci pomogę. Mam nadzieję, że to pomoże!")).toBe("Oczywiście! Chętnie Ci pomogę. Mam nadzieję, że to pomoże!"));
});

describe("protected blocks", () => {
  test("does not compress content inside fenced code blocks", () => {
    const text = "Here is an example:\n```\nfor example in order to\n```";
    expect(assistant(text)).toContain("for example in order to");
  });

  test("does not compress inline code", () => {
    const text = "Call `getValueForExample()` to retrieve it.";
    expect(assistant(text)).toContain("`getValueForExample()`");
  });

  test("does not compress URLs", () => {
    const url = "https://example.com/in-order-to/basically/test";
    const text = `See the docs at ${url} for details.`;
    expect(assistant(text)).toContain(url);
  });

  test("compresses text around code blocks but not inside", () => {
    const text =
      "Certainly! For example:\n```\nconst x = 1;\n```\nI hope this helps!";
    const result = assistant(text);
    expect(result).not.toContain("Certainly");
    expect(result).not.toContain("I hope this helps");
    expect(result).toContain("const x = 1;");
  });
});

import { readFileSync } from 'fs';
import { resolve } from 'path';

interface CompressCase {
  id: string; description: string;
  role: 'user' | 'assistant'; tiers: string[]; input: string; output: string;
}

describe('pipeline — fixture contract', () => {
  const cases: CompressCase[] = JSON.parse(
    readFileSync(resolve(import.meta.dir, '../../../fixtures/pipeline/protected-blocks.json'), 'utf8')
  );
  for (const c of cases) {
    const cfg = {
      tiers: c.tiers.map(t => TIERS[t.toUpperCase() as keyof typeof TIERS]),
      tokenMethod: 'chars' as const,
    };
    test(c.id, () => expect(compress(c.input, c.role, cfg).text).toBe(c.output));
  }
});
