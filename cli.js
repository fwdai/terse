#!/usr/bin/env node
import { compress, compressHistory, TIERS, DEFAULT_CONFIG } from './index.js';
import { readFileSync } from 'fs';

const args = process.argv.slice(2);

function usage() {
  const validTiers = Object.values(TIERS).join(', ');
  console.error(`
Usage:
  # Compress text from stdin (default role: assistant, default tier: rules)
  echo "Certainly! I'd be happy to help..." | node cli.js

  # Specify role
  echo "some text" | node cli.js --role user

  # Specify compression tiers (comma-separated, applied in order)
  echo "some text" | node cli.js --tiers rules,nlp
  echo "some text" | node cli.js --tiers rules

  # Use gpt-tokenizer for token counts (requires: npm install gpt-tokenizer)
  echo "some text" | node cli.js --tokens gpt

  # Compress a JSON history file: [{role, content}, ...]
  node cli.js history.json
  node cli.js history.json --tiers rules,nlp --tokens gpt

  # Show diff alongside result
  node cli.js --diff < input.txt

  Valid tiers: ${validTiers}
`);
  process.exit(1);
}

function printStats({ savedTokens, savedPercent, originalTokens, compressedTokens }, config) {
  console.error(`\n── Stats ─────────────────────────────────`);
  console.error(`  Tiers:      ${config.tiers.join(' → ')}`);
  console.error(`  Tokens via: ${config.tokenMethod}`);
  console.error(`  Original:   ~${originalTokens} tokens`);
  console.error(`  Compressed: ~${compressedTokens} tokens`);
  console.error(`  Saved:      ~${savedTokens} tokens (${savedPercent}%)`);
  console.error(`──────────────────────────────────────────`);
}

function showDiff(original, compressed) {
  const origLines = original.split('\n');
  const compLines = compressed.split('\n');
  const maxLen = Math.max(origLines.length, compLines.length);

  console.error('\n── Diff (original → compressed) ──────────');
  for (let i = 0; i < maxLen; i++) {
    const o = origLines[i] ?? '';
    const c = compLines[i] ?? '';
    if (o !== c) {
      console.error(`- ${o}`);
      console.error(`+ ${c}`);
    }
  }
  console.error('──────────────────────────────────────────');
}

// ── Parse flags ───────────────────────────────────────────────────────────────

const tiersFlag   = args.indexOf('--tiers');
const tokensFlag  = args.indexOf('--tokens');
const roleFlag    = args.indexOf('--role');

const rawTiers    = tiersFlag  !== -1 ? args[tiersFlag  + 1] : null;
const tokenMethod = tokensFlag !== -1 ? args[tokensFlag + 1] : DEFAULT_CONFIG.tokenMethod;
const role        = roleFlag   !== -1 ? args[roleFlag   + 1] : 'assistant';
const showDiffFlag = args.includes('--diff');

const tiers = rawTiers ? rawTiers.split(',').map(t => t.trim()) : DEFAULT_CONFIG.tiers;

const validTierValues = Object.values(TIERS);
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

const config = { tiers, tokenMethod };

// ── JSON history file mode ────────────────────────────────────────────────────

const jsonFile = args.find((a) => a.endsWith('.json'));
if (jsonFile) {
  const messages = JSON.parse(readFileSync(jsonFile, 'utf8'));
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

let input = '';
process.stdin.setEncoding('utf8');
process.stdin.on('data', (chunk) => { input += chunk; });
process.stdin.on('end', () => {
  if (!input.trim()) usage();

  const result = compress(input.trim(), role, config);
  console.log(result.text);

  if (showDiffFlag) showDiff(input.trim(), result.text);
  printStats(result, config);
});
