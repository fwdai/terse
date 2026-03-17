import type { CompressConfig } from './types.ts';

export const TIERS = {
  RULES: 'rules' as const,
  NLP:   'nlp'   as const,
  LLM:   'llm'   as const,
};

export const DEFAULT_CONFIG: CompressConfig = {
  tiers:       [TIERS.RULES],
  tokenMethod: 'chars',
};
