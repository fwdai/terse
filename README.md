# terse

Strip boilerplate from LLM conversation history before sending it back to the model. Less noise, fewer tokens, same signal.

```
"Certainly! I'd be happy to help. In order to fix this issue,
 you should take into consideration the edge cases. I hope this helps!"

→  "Fix this issue, consider the edge cases."

  30 tokens → 12 tokens  (-60%)
```

## Install

**curl (macOS and Linux):**
```sh
curl -fsSL https://raw.githubusercontent.com/fwdai/terse/main/install.sh | bash
```

**cargo:**
```sh
cargo install terse
```

**Manual:** download a pre-built binary from [Releases](https://github.com/fwdai/terse/releases), extract, and place `terse` on your `$PATH`.

## Packages

| Package | Language | Description |
|---------|----------|-------------|
| [`packages/terse`](packages/terse/) | TypeScript (Bun) | npm library + CLI |
| [`crates/terse`](crates/terse/) | Rust | library + binary |

Shared test fixtures live in [`fixtures/`](fixtures/) — both implementations are verified against the same cases.

## Testing

```sh
make test        # run both suites
make test-ts     # TypeScript only (bun test)
make test-rust   # Rust only (cargo test)
```

## What it does

LLM conversations accumulate filler: opener affirmations (*Certainly! Great question!*), closer offers (*Let me know if you have any questions*), verbose phrasing (*due to the fact that* → *because*), and structural announcements (*Here is the solution:*). None of this helps the model on the next turn.

Terse removes it — deterministically, in microseconds, with no external calls.

**Three tiers, applied in order:**

| Tier | What it does | Requires |
|------|-------------|---------|
| `rules` | Regex patterns: boilerplate openers/closers, phrase substitutions, filler words, structural labels | nothing |
| `nlp` | POS-aware: drop articles, shorten synonyms (*utilize → use*, *repository → repo*) | `bun add compromise` (TS) |
| `llm` | Semantic rewrite via local model | planned |

Code blocks, inline code, and URLs are never touched.

## Typical savings

| Content type | Rules only | Rules + NLP |
|---|---|---|
| Conversational (polite, verbose) | 40–60% | 55–75% |
| Technical (code-heavy, terse) | 5–15% | 10–20% |

## Notes

- **English only.** No language detection — other languages pass through unchanged.
- **Target is the model, not humans.** Output is optimized for token efficiency, not readability.
- **No semantic loss measurement.** Works well in practice; no theoretical guarantee.
