# terse

Strip boilerplate from LLM conversation history before sending it back to the model. Less noise, fewer tokens, same signal.

```
"Certainly! I'd be happy to help. In order to fix this issue,
 you should take into consideration the edge cases. I hope this helps!"

→  "Fix this issue, consider the edge cases."

  30 tokens → 12 tokens  (-60%)
```

## What it does

LLM conversations accumulate filler: opener affirmations (*Certainly! Great question!*), closer offers (*Let me know if you have any questions*), verbose phrasing (*due to the fact that* → *because*), and structural announcements (*Here is the solution:*). None of this helps the model on the next turn.

Terse removes it — deterministically, in microseconds, with no external calls.

**Three tiers, applied in order:**

| Tier | What it does | Requires |
|------|-------------|---------|
| `rules` | Regex patterns: boilerplate openers/closers, phrase substitutions, filler words, structural labels | nothing |
| `nlp` | POS-aware: drop articles, shorten synonyms (*utilize → use*, *repository → repo*) | `bun add compromise` |
| `llm` | Semantic rewrite via local model | planned |

Code blocks, inline code, and URLs are never touched.

## Install

```sh
bun add terse
```

## API

```ts
import { compress, compressHistory, TIERS } from 'terse';

// Single message
const { text, savedPercent } = compress(
  "Certainly! I'd be happy to help with that.",
  'assistant',
);
// → { text: "Help with that.", savedPercent: 62 }

// Full conversation history
const { messages, stats } = compressHistory(messages, {
  tiers: [TIERS.RULES, TIERS.NLP],
  tokenMethod: 'chars',  // 'chars': fast, ~±20% | 'tiktoken': exact for GPT, proxy for Claude (±5-10%)
});
// messages[n]._stats — per-message savings
// stats.totalSavedPercent — aggregate
```

## CLI

```sh
# Stdin
echo "Certainly! I hope this helps!" | bun bin/cli.ts

# With diff output
echo "Could you please explain closures? Thank you!" | bun bin/cli.ts --role user --diff

# Compress a history JSON file
bun bin/cli.ts history.json --tiers rules,nlp

# Accurate token counts (GPT-4 exact; good proxy for Claude)
echo "some text" | bun bin/cli.ts --tokens tiktoken
```

## Typical savings

| Content type | Rules only | Rules + NLP |
|---|---|---|
| Conversational (polite, verbose) | 40–60% | 55–75% |
| Technical (code-heavy, terse) | 5–15% | 10–20% |

## Notes

- **English only.** No language detection — other languages pass through unchanged.
- **Target is the model, not humans.** Output is optimized for token efficiency, not readability.
- **No semantic loss measurement.** Works well in practice; no theoretical guarantee.
