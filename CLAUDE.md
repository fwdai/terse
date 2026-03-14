# Context Compressor

Tool that reduces token usage in LLM conversation history by removing unnecessary tokens from past messages. Target: machine reader (the model), not humans — output doesn't need to be readable.

## Stack

**Runtime:** Bun. Run everything with `bun`, not `node`.
**Language:** TypeScript. No build step — Bun executes `.ts` directly.

## Files

- `index.ts` — core library (ESM, top-level await for optional deps)
- `cli.ts` — CLI interface
- `tsconfig.json` — `moduleResolution: bundler`, `types: bun-types`
- `package.json` — optional deps: `compromise`, `gpt-tokenizer`; dev: `bun-types`, `typescript`

## Architecture

Pipeline per message: **mask protected** → tiers in order → **unmask protected**

Protected (never touched): fenced code blocks, inline code, URLs — replaced with `\x00Bn\x00` placeholders, restored after.

### Config

```js
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

### Tier 1: Rules (`applyRules`)

- `removeAssistantBoilerplate` — strips openers (Certainly!, Great question!, I'd be happy to...) and closers (I hope this helps!, Let me know if...)
- `removeUserBoilerplate` — strips politeness (Please, Thank you), request preambles (Can you, Could you, I'd like you to, I was wondering if), self-deprecating hedges (Sorry if this is a dumb question). Re-capitalizes remainder. Gerund→imperative fix for leftovers.
- `applySubstitutions` — ~50-entry phrase dict: "in order to"→"to", "due to the fact that"→"because", "for example"→"e.g.", zero-value markers deleted, etc.
- `removeFillers` — sentence-start only (safe without POS): basically, essentially, obviously, etc.
- `normalizeWhitespace` — newlines→space, multi-space→one, `...`→`…`

Typical savings: 30–50% on conversational text.

### Tier 2: NLP (`applyNlp`)

Requires `npm install compromise`. Lazy-loaded; module works without it.

- Drops all determiners (`#Determiner`: a, an, the) — safe for machine reader
- Drops intensifier adverbs (`very`, `really`, `extremely`...) globally, POS-verified
- Verb synonyms (16 entries): utilize→use, attempt→try, obtain→get, demonstrate→show, construct→build, etc.
- Noun synonyms (11 entries): functionality→feature, repository→repo, documentation→docs, configuration→config, etc.

Additive on top of rules. Typical additional savings: 15–25%.

### Tier 3: LLM (future)

Planned techniques (discussed, not built):
- **Telegraphic rewrite** — drop function words (articles, copulas, auxiliaries, prepositions) via small local LLM; LLM judges which are safe to drop given context
- **Token-optimal synonym selection** — LLM picks shortest-tokenizing synonym
- **Symbolic notation** — prose→symbols where unambiguous (X→Y, X←Y, ≈, ≠, ∴)
- **Predictability-based pruning** — score each token by perplexity, remove below threshold (information-theoretic; fundamentally requires LM)
- **Cross-message dedup** — replace repeated phrases with back-references

## Public API

```js
import { compress, compressHistory, TIERS, DEFAULT_CONFIG } from './index.js';

// Single message
const { text, originalTokens, compressedTokens, savedTokens, savedPercent }
  = compress(text, role, { tiers: [TIERS.RULES, TIERS.NLP], tokenMethod: 'chars' });

// History [{role, content}]
const { messages, stats } = compressHistory(messages, config);
// messages[n]._stats has per-message stats; stats has totals
```

## CLI

```bash
echo "text" | bun cli.ts --role assistant --tiers rules,nlp
echo "text" | bun cli.ts --tokens gpt
bun cli.ts history.json --tiers rules,nlp --tokens gpt
bun cli.ts --diff < input.txt      # show what changed
```

## Key design decisions

- **Role-aware**: boilerplate patterns differ per role; rules and nlp tiers both receive `role`
- **Additive tiers**: each tier receives previous tier's output; order in config array matters
- **Optional deps lazy-loaded** via top-level `await import()` in try/catch — module loads fine without `compromise` or `gpt-tokenizer`; throws clear install message only when that tier/method is actually used
- **Never touch code**: masking happens before any tier, restore happens after all tiers
- **chars/4 default**: avoids network call and external dep; use `gpt` method when accuracy matters

## What's not built yet

- LLM tier implementation (needs model integration — llama.cpp, ollama, or API)
- Cross-message deduplication (repeated phrases across turns → back-references)
- POS-based copula/auxiliary dropping (riskier than determiners; needs LLM or careful dep parsing)
- Tokenization-aware synonym discovery (script to build synonym map sorted by token count)
- Tests
