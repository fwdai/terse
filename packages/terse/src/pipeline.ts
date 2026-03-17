import type { Tier, CompressConfig, CompressResult, Message, MessageWithStats, HistoryResult } from './types.ts';
import { TIERS }           from './config.ts';
import { estimateTokens }  from './tokens.ts';
import { applyRules }      from './tiers/rules/index.ts';
import { applyNlp }        from './tiers/nlp/index.ts';
import { applyLlm }        from './tiers/llm/index.ts';

// ── Protected block masking ───────────────────────────────────────────────────
// Code blocks, inline code, and URLs are replaced with placeholders before any
// compression pass and restored afterward — they must never be touched.

export function maskProtected(text: string): { masked: string; blocks: string[] } {
  const blocks: string[] = [];
  const masked = text
    .replace(/```[\s\S]*?```/g, (m) => { blocks.push(m); return `\x00B${blocks.length - 1}\x00`; })
    .replace(/`[^`\n]+`/g,     (m) => { blocks.push(m); return `\x00B${blocks.length - 1}\x00`; })
    .replace(/https?:\/\/\S+/g, (m) => { blocks.push(m); return `\x00B${blocks.length - 1}\x00`; });
  return { masked, blocks };
}

export function unmaskProtected(text: string, blocks: string[]): string {
  return text.replace(/\x00B(\d+)\x00/g, (_, i) => blocks[parseInt(i)]);
}

// ── Tier registry ─────────────────────────────────────────────────────────────
// To add a new tier: implement applyX(text, role) → string and add it here.

const TIER_FNS: Record<Tier, (text: string, role: string) => string> = {
  [TIERS.RULES]: applyRules,
  [TIERS.NLP]:   (text) => applyNlp(text),
  [TIERS.LLM]:   applyLlm,
};

// ── Core pipeline ─────────────────────────────────────────────────────────────

export function runCompress(
  text: string,
  role: 'user' | 'assistant',
  config: CompressConfig,
): CompressResult {
  const { tiers, tokenMethod } = config;

  const originalTokens = estimateTokens(text, tokenMethod);

  // Fast bail-out: no Latin characters → not English, nothing to compress
  if (!/[a-zA-Z]/.test(text)) return { text, originalTokens, compressedTokens: originalTokens, savedTokens: 0, savedPercent: 0 };

  const { masked, blocks } = maskProtected(text);

  let result = masked;
  for (const tier of tiers) {
    const fn = TIER_FNS[tier];
    if (!fn) throw new Error(`Unknown tier: "${tier}". Valid: ${Object.values(TIERS).join(', ')}`);
    result = fn(result, role);
  }

  result = unmaskProtected(result, blocks);

  const compressedTokens = estimateTokens(result, tokenMethod);
  const savedTokens      = originalTokens - compressedTokens;
  const savedPercent     = originalTokens > 0 ? Math.round((savedTokens / originalTokens) * 100) : 0;

  return { text: result, originalTokens, compressedTokens, savedTokens, savedPercent };
}

function isCompressible(msg: Message): msg is Message & { role: 'user' | 'assistant'; content: string } {
  return (msg.role === 'user' || msg.role === 'assistant') && typeof msg.content === 'string';
}

export function runCompressHistory(
  messages: Message[],
  config: CompressConfig,
): HistoryResult {
  let totalOriginal   = 0;
  let totalCompressed = 0;

  const compressed = messages.map((msg): MessageWithStats => {
    if (!isCompressible(msg)) {
      // Pass through: null content, array content, tool/system roles — no compression, no crash
      return { ...msg, _stats: { originalTokens: 0, compressedTokens: 0, savedTokens: 0, savedPercent: 0 } };
    }

    const result = runCompress(msg.content, msg.role, config);
    totalOriginal   += result.originalTokens;
    totalCompressed += result.compressedTokens;
    return {
      ...msg,
      content: result.text,
      _stats: {
        originalTokens:   result.originalTokens,
        compressedTokens: result.compressedTokens,
        savedTokens:      result.savedTokens,
        savedPercent:     result.savedPercent,
      },
    };
  });

  const totalSaved        = totalOriginal - totalCompressed;
  const totalSavedPercent = totalOriginal > 0 ? Math.round((totalSaved / totalOriginal) * 100) : 0;

  return {
    messages: compressed,
    stats: {
      totalOriginalTokens:   totalOriginal,
      totalCompressedTokens: totalCompressed,
      totalSavedTokens:      totalSaved,
      totalSavedPercent,
    },
  };
}
