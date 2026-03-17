import type { TokenMethod } from './types.ts';

// 'tiktoken' — uses gpt-tokenizer (cl100k_base / tiktoken), exact for GPT-4 and GPT-3.5-turbo.
// Also a reliable proxy for Anthropic Claude models: token density is comparable for
// English prose (typically within 5-10%). No official Claude tokenizer is available
// outside the Anthropic API, so this is the best local option for Claude token counts.
let gptEncode: ((text: string) => number[]) | null = null;
try {
  const mod = await import('gpt-tokenizer');
  gptEncode = mod.encode;
} catch { /* not installed; 'tiktoken' tokenMethod will throw if used */ }

export function estimateTokens(text: string, method: TokenMethod = 'chars'): number {
  if (method === 'tiktoken') {
    if (!gptEncode) throw new Error('gpt-tokenizer not installed. Run: bun add gpt-tokenizer');
    return gptEncode(text).length;
  }
  // 'chars': fast zero-dependency approximation (~±20% vs real token count)
  return Math.ceil(text.length / 4);
}

// Accurate token count via the Anthropic API.
// Requires ANTHROPIC_API_KEY env var. Useful for auditing real savings against Claude billing.
export async function countTokensAnthropic(text: string): Promise<number> {
  const apiKey = process.env.ANTHROPIC_API_KEY;
  if (!apiKey) throw new Error('ANTHROPIC_API_KEY env var required for Anthropic token counting');

  const res = await fetch(
    "https://api.anthropic.com/v1/messages/count_tokens",
    {
      method: "POST",
      headers: {
        "content-type": "application/json",
        "x-api-key": apiKey,
        "anthropic-version": "2023-06-01",
      },
      body: JSON.stringify({
        model: "claude-opus-4-6",
        messages: [{ role: "user", content: text }],
      }),
    },
  );

  if (!res.ok) {
    const body = await res.text();
    throw new Error(`Anthropic token count API error (${res.status}): ${body}`);
  }

  const { input_tokens } = await res.json() as { input_tokens: number };
  return input_tokens;
}
