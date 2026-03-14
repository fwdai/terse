import type { TokenMethod } from './types.ts';

let gptEncode: ((text: string) => number[]) | null = null;
try {
  const mod = await import('gpt-tokenizer');
  gptEncode = mod.encode;
} catch { /* not installed; 'gpt' tokenMethod will throw if used */ }

export function estimateTokens(text: string, method: TokenMethod = 'chars'): number {
  if (method === 'gpt') {
    if (!gptEncode) throw new Error('gpt-tokenizer not installed. Run: bun add gpt-tokenizer');
    return gptEncode(text).length;
  }
  return Math.ceil(text.length / 4);
}
