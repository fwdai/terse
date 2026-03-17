export type Tier        = 'rules' | 'nlp' | 'llm';
export type TokenMethod = 'chars' | 'tiktoken';

export interface CompressConfig {
  tiers:       Tier[];
  tokenMethod: TokenMethod;
}

export interface CompressResult {
  text:             string;
  originalTokens:   number;
  compressedTokens: number;
  savedTokens:      number;
  savedPercent:     number;
}

// Minimal shape for Anthropic API content blocks (multimodal, tool_use, tool_result, etc.)
export type ContentBlock = { type: string; [key: string]: unknown };

export interface Message {
  role:    string;  // 'user' | 'assistant' | 'tool' | 'system' | ...
  content: string | null | ContentBlock[];
  [key: string]: unknown;
}

export interface MessageWithStats extends Message {
  _stats: Omit<CompressResult, 'text'>;
}

export interface HistoryResult {
  messages: MessageWithStats[];
  stats: {
    totalOriginalTokens:   number;
    totalCompressedTokens: number;
    totalSavedTokens:      number;
    totalSavedPercent:     number;
  };
}
