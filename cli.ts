#!/usr/bin/env bun
import { compress, compressHistory, TIERS, DEFAULT_CONFIG } from './index.ts';
import type { CompressConfig, CompressResult, Tier } from './index.ts';

const args = process.argv.slice(2);

function usage(): never {
  const validTiers = Object.values(TIERS).join(', ');
  console.error(`
Usage:
  # Compress text from stdin (default role: assistant, default tier: rules)
  echo "Certainly! I'd be happy to help..." | bun cli.ts

  # Specify role
  echo "some text" | bun cli.ts --role user

  # Specify compression tiers (comma-separated, applied in order)
  echo "some text" | bun cli.ts --tiers rules,nlp

  # Use gpt-tokenizer for token counts (requires: bun add gpt-tokenizer)
  echo "some text" | bun cli.ts --tokens gpt

  # Compress a JSON history file: [{role, content}, ...]
  bun cli.ts history.json --tiers rules,nlp

  # Show diff alongside result
  bun cli.ts --diff < input.txt

  Valid tiers: ${validTiers}
`);
  process.exit(1);
}

function printStats(result: CompressResult, config: CompressConfig): void {
  console.error(`\n── Stats ─────────────────────────────────`);
  console.error(`  Tiers:      ${config.tiers.join(' → ')}`);
  console.error(`  Tokens via: ${config.tokenMethod}`);
  console.error(`  Original:   ~${result.originalTokens} tokens`);
  console.error(`  Compressed: ~${result.compressedTokens} tokens`);
  console.error(`  Saved:      ~${result.savedTokens} tokens (${result.savedPercent}%)`);
  console.error(`──────────────────────────────────────────`);
}

function showDiff(original: string, compressed: string): void {
  console.error('\n── Diff (original → compressed) ──────────');
  console.error(`- ${original}`);
  console.error(`+ ${compressed}`);
  console.error('──────────────────────────────────────────');
}

// ── Parse flags ───────────────────────────────────────────────────────────────

const tiersFlag  = args.indexOf('--tiers');
const tokensFlag = args.indexOf('--tokens');
const roleFlag   = args.indexOf('--role');

const rawTiers    = tiersFlag  !== -1 ? args[tiersFlag  + 1] : null;
const tokenMethod = tokensFlag !== -1 ? args[tokensFlag + 1] : DEFAULT_CONFIG.tokenMethod;
const role        = roleFlag   !== -1 ? args[roleFlag   + 1] : 'assistant';
const showDiffFlag = args.includes('--diff');

const tiers: Tier[] = rawTiers
  ? rawTiers.split(',').map(t => t.trim() as Tier)
  : DEFAULT_CONFIG.tiers;

const validTierValues = Object.values(TIERS) as string[];
for (const tier of tiers) {
  if (!validTierValues.includes(tier)) {
    console.error(`Error: unknown tier "${tier}". Valid: ${validTierValues.join(', ')}`);
    usage();
  }
}

if (!['chars', 'gpt'].includes(tokenMethod)) {
  console.error(`Error: --tokens must be "chars" or "gpt"`);
  usage();
}

if (!['user', 'assistant'].includes(role)) {
  console.error(`Error: --role must be "user" or "assistant"`);
  usage();
}

const config: CompressConfig = {
  tiers,
  tokenMethod: tokenMethod as CompressConfig['tokenMethod'],
};

// ── JSON history file mode ────────────────────────────────────────────────────

const jsonFile = args.find((a) => a.endsWith('.json'));
if (jsonFile) {
  const messages = JSON.parse(await Bun.file(jsonFile).text());
  const { messages: compressed, stats } = compressHistory(messages, config);

  console.log(JSON.stringify(compressed, null, 2));

  console.error(`\n── History Stats ──────────────────────────`);
  console.error(`  Tiers:      ${config.tiers.join(' → ')}`);
  console.error(`  Messages:   ${messages.length}`);
  console.error(`  Original:   ~${stats.totalOriginalTokens} tokens`);
  console.error(`  Compressed: ~${stats.totalCompressedTokens} tokens`);
  console.error(`  Saved:      ~${stats.totalSavedTokens} tokens (${stats.totalSavedPercent}%)`);
  console.error(`──────────────────────────────────────────`);
  process.exit(0);
}

// ── Stdin mode ────────────────────────────────────────────────────────────────

const input = await Bun.stdin.text();
if (!input.trim()) usage();

const result = compress(input.trim(), role as 'user' | 'assistant', config);
console.log(result.text);

if (showDiffFlag) showDiff(input.trim(), result.text);
printStats(result, config);
