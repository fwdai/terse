# terse

Every message you send to Claude includes the full conversation history. As sessions grow, you're paying for the same tokens over and over — assistant affirmations, verbose phrasing, structural filler that the model doesn't need to do its job.

Terse strips it before it reaches the API. Transparently, in microseconds, with no code changes.

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

**Manual:** download a pre-built binary from [Releases](https://github.com/fwdai/terse/releases).

## How it works

Terse runs as a local proxy between your CLI and the Anthropic API. Claude Code (and any Anthropic SDK client) respects `ANTHROPIC_BASE_URL` — set it to `http://localhost:3847` and every request is automatically compressed before forwarding. The current message is always sent verbatim; only history is compressed.

```
Claude Code  →  terse proxy (compress history)  →  api.anthropic.com
```

**One-time setup:**
```sh
terse install       # hooks into your shell — proxy auto-starts when you run claude
source ~/.zshrc
claude              # that's it, compression is live
```

No API key changes. No wrapper scripts. Just run `claude` as usual.

## What gets compressed

Three tiers, applied in order:

| Tier | Technique | Latency |
|------|-----------|---------|
| `trim` | Regex: removes opener/closer boilerplate, filler words, structural labels, verbose phrases | ~0ms |
| `compress` | NLP: drops articles, substitutes shorter synonyms (*utilize → use*, *repository → repo*) | ~1ms |
| `rewrite` | Local LLM: telegraphic rewrite, semantic deduplication | planned |

Code blocks, inline code, and URLs are never touched.

**Typical savings:**

| Content | `trim` | `compress` |
|---------|--------|------------|
| Conversational (verbose, polite) | 40–60% | 55–75% |
| Technical (code-heavy) | 5–15% | 10–20% |

## Track your savings

```sh
terse gains          # total tokens saved across all sessions
terse gains --watch  # live dashboard, updates as you work
```

```
══════════════════════════════════════════════════════
  TOTAL SAVINGS (12 sessions)
══════════════════════════════════════════════════════

  Total calls:      147
  Tokens processed: 2.4M
  Tokens saved:     890.3K (37.1%)

  Efficiency  [████████████████████░░░░░░░░ ] 37.1%
```

## Proxy commands

```sh
terse proxy status   # see active sessions and per-session savings
terse proxy stop     # stop the proxy
terse upgrade        # update to the latest release
```

## Configuration

Auto-created at `~/.terse/config.json` on first run:

```json
{
  "mode": "trim",
  "tokenizer": "tiktoken",
  "proxy": { "port": 3847 }
}
```

`mode`: `trim` (lossless) · `compress` (light NLP) · `rewrite` (planned)

## Notes

- **English only.** Other languages pass through unmodified.
- **Current message is never compressed.** Only history. Your prompt reaches the model exactly as written.
- **No external calls.** Compression is local, deterministic, sub-millisecond.

## Packages

| Package | Language |
|---------|----------|
| [`crates/terse`](crates/terse/) | Rust — CLI binary + library |
| [`packages/terse`](packages/terse/) | TypeScript — npm library + CLI |

Both implementations are verified against the same [test fixtures](fixtures/).
