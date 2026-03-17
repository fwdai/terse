// LLM tier — planned techniques:
//   - Telegraphic rewrite: drop function words (articles, copulas, auxiliaries)
//   - Token-optimal synonym selection: pick shortest-tokenizing equivalent
//   - Symbolic notation: prose → symbols where unambiguous (X→Y, X←Y, ≈, ≠, ∴)
//   - Predictability pruning: remove tokens below perplexity threshold
//
// Implementation will use a local model via llama-cpp-2 or candle (Rust rewrite).

export function applyLlm(_text: string, _role: string): string {
  throw new Error('LLM tier not yet implemented');
}
