export function normalizeWhitespace(text: string): string {
  return text
    .replace(/\n+/g, ' ')     // all newlines → space (target is machine reader, not human)
    .replace(/[ \t]+/g, ' ')  // multiple spaces/tabs → one
    .replace(/\.{3}/g, '…')   // ... → single ellipsis char (1 char vs 3)
    .replace(/!{2,}/g, '!')
    .replace(/\?{2,}/g, '?')
    .trim();
}
