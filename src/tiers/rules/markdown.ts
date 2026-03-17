export function stripMarkdownFormatting(text: string): string {
  return text
    // Horizontal rules on their own line (---, ***, ___)
    .replace(/^[ \t]*[-*_]{3,}[ \t]*$/gm, '')
    // Table separator rows (| --- | --- |)
    .replace(/^\|[\s\-:|]+\|\s*$/gm, '')
    // Heading markers (## Title → Title)
    .replace(/^#{1,6} /gm, '')
    // Bold (**text** → text)  —  ** can't appear in snake_case so no false positives
    .replace(/\*\*([^*\n]+)\*\*/g, '$1')
    // Italic (*text* → text)  —  lookbehind/lookahead avoids ** bold and * list bullets
    .replace(/(?<!\*)\*([^\s*\n][^*\n]*?)\*(?!\*)/g, '$1')
    // Spaced em-dash → comma-space (same semantic pause, 3 chars → 2)
    .replace(/ — /g, ', ')
    // Curly/smart quotes → ASCII
    .replace(/[\u201C\u201D]/g, '"')
    .replace(/[\u2018\u2019]/g, "'");
}
