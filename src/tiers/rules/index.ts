import { removeAssistantBoilerplate }      from './assistant.ts';
import { removeStructuralMetaCommentary } from './structural.ts';
import { removeUserBoilerplate }           from './user.ts';
import { applySubstitutions }              from './substitutions.ts';
import { removeFillers }                   from './fillers.ts';
import { stripMarkdownFormatting }         from './markdown.ts';
import { normalizeWhitespace }             from './whitespace.ts';

export function applyRules(text: string, role: string): string {
  let result = text;
  if (role === 'assistant') result = removeAssistantBoilerplate(result);
  if (role === 'assistant') result = removeStructuralMetaCommentary(result);
  if (role === 'user')      result = removeUserBoilerplate(result);
  result = applySubstitutions(result);
  result = removeFillers(result);
  result = stripMarkdownFormatting(result);
  result = normalizeWhitespace(result);
  return result;
}
