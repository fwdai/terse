# Context Compressor

Tool that reduces token usage in LLM conversation history by removing unnecessary tokens from past messages. Target: machine reader (the model), not humans — output doesn't need to be readable.

## Stack

**Runtime:** Bun. Run everything with `bun`, not `node`.
**Language:** TypeScript. No build step — Bun executes `.ts` directly.

## File structure

```
src/
  index.ts                  ← public API only (compress, compressHistory) — keep thin
  types.ts                  ← all TS interfaces and type aliases
  config.ts                 ← TIERS constant, DEFAULT_CONFIG
  tokens.ts                 ← estimateTokens, gptEncode lazy load
  pipeline.ts               ← maskProtected, unmaskProtected, TIER_FNS, runCompress/runCompressHistory
  tiers/
    rules/
      index.ts              ← applyRules() — orchestrates sub-passes in order
      assistant.ts          ← ASSISTANT_OPENERS/CLOSERS + removeAssistantBoilerplate
      user.ts               ← USER_OPENERS/CLOSERS + removeUserBoilerplate
      substitutions.ts      ← SUBSTITUTIONS vocab + applySubstitutions
      fillers.ts            ← SENTENCE_START_FILLERS + removeFillers
      whitespace.ts         ← normalizeWhitespace
    nlp/
      index.ts              ← applyNlp(), nlp lazy load
      vocab.ts              ← VERB_SYNONYMS, NOUN_SYNONYMS, INTENSIFIERS
    llm/
      index.ts              ← applyLlm() stub + planned techniques
bin/
  cli.ts                    ← CLI entry point
tests/
  index.test.ts             ← bun test (79 tests)
```

**Extensibility hooks:**
- New substitution phrase → `tiers/rules/substitutions.ts`
- New boilerplate pattern → `tiers/rules/assistant.ts` or `user.ts`
- New NLP synonym → `tiers/nlp/vocab.ts`
- New rules pass → new file under `tiers/rules/`, import in `tiers/rules/index.ts`
- New tier → new folder under `tiers/`, add to `TIER_FNS` in `pipeline.ts`

## Architecture

Pipeline per message: **mask protected** → tiers in order → **unmask protected**

Protected (never touched): fenced code blocks, inline code, URLs — replaced with `\x00Bn\x00` placeholders, restored after all tiers.

### Config

```ts
{ tiers: ['rules', 'nlp'], tokenMethod: 'chars' }
```

`tokenMethod`: `'chars'` (length/4, default) or `'gpt'` (gpt-tokenizer, optional dep).

### TIERS

```
TIERS.RULES  — rule-based, zero latency, zero deps
TIERS.NLP    — POS-based via `compromise`, optional dep
TIERS.LLM    — stubbed, throws; future tier
```

Tiers applied in declared order, each receives output of previous.

### Tier 1: Rules

- `removeAssistantBoilerplate` — strips openers (Certainly!, Great question!, I'd be happy to...) and closers (I hope this helps!, Let me know if...)
- `removeUserBoilerplate` — strips politeness (Please, Thank you), request preambles (Can you, Could you, I'd like you to, I was wondering if), self-deprecating hedges (Sorry if this is a dumb question). Re-capitalizes remainder. Gerund→imperative fix for leftovers. USER_CLOSERS use `[,]?` not `[,.]?` — preserves sentence-ending periods.
- `applySubstitutions` — ~50-entry phrase dict: "in order to"→"to", "due to the fact that"→"because", "for example"→"e.g.", zero-value markers deleted, etc. Re-capitalizes if deletion lowercased first letter.
- `removeFillers` — sentence-start only (safe without POS): basically, essentially, obviously, etc.
- `normalizeWhitespace` — newlines→space, multi-space→one, `...`→`…`

Typical savings: 30–50% on conversational text.

### Tier 2: NLP

Requires `bun add compromise`. Lazy-loaded; module works without it.

- Drops articles only (`a`, `an`, `the`) — demonstratives (`this`/`that`/`these`/`those`) preserved, they carry meaning
- Drops intensifier adverbs (`very`, `really`, `extremely`...) when modifying adjectives, POS-verified
- Verb synonyms (16 entries): utilize→use, attempt→try, obtain→get, demonstrate→show, construct→build, etc.
- Noun synonyms (11 entries): functionality→feature, repository→repo, documentation→docs, configuration→config, etc.

Additive on top of rules. Typical additional savings: 15–25%.

### Tier 3: LLM (future)

Planned techniques (discussed, not built):
- **Telegraphic rewrite** — drop function words (copulas, auxiliaries, prepositions) via small local LLM
- **Token-optimal synonym selection** — pick shortest-tokenizing equivalent
- **Symbolic notation** — prose→symbols where unambiguous (X→Y, X←Y, ≈, ≠, ∴)
- **Predictability-based pruning** — remove tokens below perplexity threshold
- **Cross-message dedup** — replace repeated phrases with back-references

Implementation: local model via `llama-cpp-2` or `candle` (Rust rewrite). The inference engine compiles into the binary; model weights (~1-3B, Q4 GGUF) are external files loaded at runtime.

## Public API

```ts
import { compress, compressHistory, TIERS, DEFAULT_CONFIG } from './src/index.ts';

// Single message
const { text, originalTokens, compressedTokens, savedTokens, savedPercent }
  = compress(text, role, { tiers: [TIERS.RULES, TIERS.NLP], tokenMethod: 'chars' });

// History [{role, content}]
const { messages, stats } = compressHistory(messages, config);
// messages[n]._stats has per-message stats; stats has totals
```

## CLI

```bash
echo "text" | bun bin/cli.ts --role assistant --tiers rules,nlp
echo "text" | bun bin/cli.ts --tokens gpt
bun bin/cli.ts history.json --tiers rules,nlp --tokens gpt
bun bin/cli.ts --diff < input.txt
```

## Key design decisions

- **Role-aware**: boilerplate patterns differ per role; rules and nlp tiers both receive `role`
- **Additive tiers**: each tier receives previous tier's output; order in config array matters
- **Optional deps lazy-loaded** via top-level `await import()` in try/catch — module loads fine without `compromise` or `gpt-tokenizer`; throws clear install message only when that tier/method is actually used
- **Never touch code**: masking happens before any tier, restore happens after all tiers
- **chars/4 default**: avoids network call and external dep; use `gpt` method when accuracy matters
- **Articles only in NLP tier**: `doc.match('(a|an|the)').remove()` not `doc.remove('#Determiner')` — demonstratives carry meaning

## Known limitations / show stoppers

- **English only**: all patterns, substitutions, and `compromise` are English. No language detection — other languages get no compression at best, corruption at worst.
- **Semantic loss unmeasurable**: no feedback loop to verify compression doesn't degrade downstream model output. Works well in practice but has no theoretical guarantee.

## What's not built yet

- LLM tier (local model via llama-cpp-2 or candle — planned as part of Rust rewrite)
- Language detection before applying English rules
- Cross-message deduplication
- Tokenization-aware synonym discovery
- POS-based copula/auxiliary dropping (riskier; leave for LLM tier)
