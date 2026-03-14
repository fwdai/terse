// Phrase-level substitution vocabulary.
// Entries are applied in order — put longer/more specific patterns first.
// Format: [pattern, replacement] — replacement '' means delete entirely.

export const SUBSTITUTIONS: [RegExp, string][] = [
  // Verbosity → concise equivalents
  [/\bin order to\b/gi,                          'to'],
  [/\bdue to the fact that\b/gi,                 'because'],
  [/\bat this point in time\b/gi,                'now'],
  [/\bin the event that\b/gi,                    'if'],
  [/\bis able to\b/gi,                           'can'],
  [/\bare able to\b/gi,                          'can'],
  [/\bwas able to\b/gi,                          'could'],
  [/\bwere able to\b/gi,                         'could'],
  [/\bhas the ability to\b/gi,                   'can'],
  [/\bhave the ability to\b/gi,                  'can'],
  [/\bprior to\b/gi,                             'before'],
  [/\bsubsequent to\b/gi,                        'after'],
  [/\bwith the exception of\b/gi,                'except'],
  [/\bon a regular basis\b/gi,                   'regularly'],
  [/\bat the present time\b/gi,                  'now'],
  [/\bin the near future\b/gi,                   'soon'],
  [/\bin spite of the fact that\b/gi,            'although'],
  [/\bregardless of the fact that\b/gi,          'although'],
  [/\bfor the purpose of\b/gi,                   'for'],
  [/\bin the process of\b/gi,                    'while'],
  [/\bfirst and foremost\b/gi,                   'first'],
  [/\blast but not least\b/gi,                   'finally'],
  [/\ba large number of\b/gi,                    'many'],
  [/\ba significant number of\b/gi,              'many'],
  [/\bthe majority of\b/gi,                      'most'],
  [/\bin close proximity to\b/gi,                'near'],
  [/\bwith regard(s)? to\b/gi,                   're:'],
  [/\bwith respect to\b/gi,                      're:'],
  [/\bin terms of\b/gi,                          'for'],
  [/\bas well as\b/gi,                           'and'],
  [/\bit is possible that\b/gi,                  'maybe'],
  [/\bthere is a possibility that\b/gi,          'maybe'],
  [/\bmake a decision\b/gi,                      'decide'],
  [/\bmake use of\b/gi,                          'use'],
  [/\bprovide assistance( to)?\b/gi,             'help'],
  [/\bgive consideration to\b/gi,                'consider'],
  [/\btake into consideration\b/gi,              'consider'],
  [/\bcome to the conclusion\b/gi,               'conclude'],
  [/\bcome to an agreement\b/gi,                 'agree'],
  [/\breach a conclusion\b/gi,                   'conclude'],

  // Zero-value discourse markers → delete entirely
  [/\bit (is|should be) (worth noting|noted) that[,]?\s*/gi, ''],
  [/\bit is important to note that[,]?\s*/gi,    ''],
  [/\bit['']s worth (noting|mentioning) that[,]?\s*/gi, ''],
  [/\bneedless to say[,]?\s*/gi,                 ''],
  [/\bas a matter of fact[,]?\s*/gi,             ''],
  [/\bin actual fact[,]?\s*/gi,                  ''],
  [/\bit goes without saying( that)?[,]?\s*/gi,  ''],
  [/\bthe fact that\b/gi,                        'that'],

  // Standard abbreviations
  [/\bfor example\b/gi,                          'e.g.'],
  [/\bthat is to say\b/gi,                       'i.e.'],
  [/\bin other words\b/gi,                       'i.e.'],
  [/\bet cetera\b/gi,                            'etc.'],
  [/\band so (on|forth)\b/gi,                    'etc.'],
  [/\bversus\b/gi,                               'vs.'],
  [/\bapproximately\s*/gi,                       '~'],
];

export function applySubstitutions(text: string): string {
  let result = text;
  for (const [pattern, replacement] of SUBSTITUTIONS) {
    result = result.replace(pattern, replacement);
  }
  // A deletion may have left the first letter lowercase — re-capitalize
  if (/^[a-z]/.test(result)) {
    result = result.charAt(0).toUpperCase() + result.slice(1);
  }
  return result;
}
