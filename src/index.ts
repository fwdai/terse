export type { Tier, TokenMethod, CompressConfig, CompressResult, Message, MessageWithStats, HistoryResult } from './types.ts';
export { TIERS, DEFAULT_CONFIG } from './config.ts';

import type { CompressConfig, CompressResult, Message, HistoryResult } from './types.ts';
import { DEFAULT_CONFIG } from './config.ts';
import { runCompress, runCompressHistory } from './pipeline.ts';

export function compress(
  text: string,
  role: 'user' | 'assistant' = 'assistant',
  config: CompressConfig = DEFAULT_CONFIG,
): CompressResult {
  return runCompress(text, role, config);
}

export function compressHistory(
  messages: Message[],
  config: CompressConfig = DEFAULT_CONFIG,
): HistoryResult {
  return runCompressHistory(messages, config);
}
