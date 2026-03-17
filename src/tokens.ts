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
