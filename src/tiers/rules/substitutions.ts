// Phrase-level substitution vocabulary.
// Entries are applied in order — put longer/more specific patterns first.
// Format: [pattern, replacement] — replacement '' means delete entirely.
//
// Intentionally excluded:
//   "as a result" (mid-sentence form too ambiguous — position-dependent meaning)
//   "be in a position to" → "can" (breaks "should be in a position to" → "should can")
//   contractions (do not → don't) — handled better in NLP tier
//   "in fact", "indeed" — too load-bearing mid-sentence to strip safely

export const SUBSTITUTIONS: [RegExp, string][] = [

  // ── Purpose clauses ("in order to" and variants) ────────────────────────────
  [/\bwith the intention of\b/gi,              'to'],
  [/\bwith the goal of\b/gi,                   'to'],
  [/\bwith the aim of\b/gi,                    'to'],
  [/\bin an effort to\b/gi,                    'to'],
  [/\bin an attempt to\b/gi,                   'to'],
  [/\bso as to\b/gi,                           'to'],
  [/\bin order to\b/gi,                        'to'],
  [/\bin order for\b/gi,                       'for'],
  [/\bfor the sake of\b/gi,                    'for'],
  [/\bfor the purpose of\b/gi,                 'for'],

  // ── Ability / possibility ────────────────────────────────────────────────────
  [/\bhas the ability to\b/gi,                 'can'],
  [/\bhave the ability to\b/gi,                'can'],
  [/\bis able to\b/gi,                         'can'],
  [/\bare able to\b/gi,                        'can'],
  [/\bwas able to\b/gi,                        'could'],
  [/\bwere able to\b/gi,                       'could'],
  [/\bit is possible that\b/gi,                'maybe'],
  [/\bthere is a possibility that\b/gi,        'maybe'],

  // ── Causal / concessive connectives ─────────────────────────────────────────
  [/\bin spite of the fact that\b/gi,          'although'],
  [/\bregardless of the fact that\b/gi,        'although'],
  [/\bdespite the fact that\b/gi,              'although'],
  [/\bin light of the fact that\b/gi,          'since'],
  [/\bgiven the fact that\b/gi,                'since'],
  [/\bowing to the fact that\b/gi,             'because'],
  [/\bdue to the fact that\b/gi,               'because'],
  [/\bon the grounds that\b/gi,                'because'],
  [/\bfor the reason that\b/gi,                'because'],
  [/\bas a result of\b/gi,                     'due to'],
  [/\bin the event that\b/gi,                  'if'],
  [/\bon the other hand\b/gi,                  'however'],

  // ── Temporal / frequency ────────────────────────────────────────────────────
  [/\bat this point in time\b/gi,              'now'],
  [/\bat the present time\b/gi,                'now'],
  [/\bat the end of the day\b/gi,              'ultimately'],
  [/\bfor the time being\b/gi,                 'for now'],
  [/\bin the near future\b/gi,                 'soon'],
  [/\bin the meantime\b/gi,                    'meanwhile'],
  [/\bsooner or later\b/gi,                    'eventually'],
  [/\bat some point\b/gi,                      'eventually'],
  [/\bfrom time to time\b/gi,                  'sometimes'],
  [/\bat all times\b/gi,                       'always'],
  [/\bprior to\b/gi,                           'before'],
  [/\bsubsequent to\b/gi,                      'after'],
  [/\bon a daily basis\b/gi,                   'daily'],
  [/\bon a weekly basis\b/gi,                  'weekly'],
  [/\bon a monthly basis\b/gi,                 'monthly'],
  [/\bon a regular basis\b/gi,                 'regularly'],

  // ── Quantity / scope ─────────────────────────────────────────────────────────
  [/\ba significant number of\b/gi,            'many'],
  [/\ba large number of\b/gi,                  'many'],
  [/\ba wide variety of\b/gi,                  'many'],
  [/\ba wide range of\b/gi,                    'many'],
  [/\ba variety of\b/gi,                       'various'],
  [/\bthe majority of\b/gi,                    'most'],
  [/\beach and every\b/gi,                     'every'],
  [/\bfor the most part\b/gi,                  'mostly'],
  [/\bby and large\b/gi,                       'mostly'],
  [/\bwith the exception of\b/gi,              'except'],
  [/\bin addition to\b/gi,                     'beyond'],   // must precede "in addition"
  [/\bin addition\b/gi,                        'also'],

  // ── Connectives / discourse ──────────────────────────────────────────────────
  [/\bin close proximity to\b/gi,              'near'],
  [/\bwhen it comes to\b/gi,                   'for'],
  [/\bby means of\b/gi,                        'via'],
  [/\bin the process of\b/gi,                  'while'],
  [/\bon the basis of\b/gi,                    'based on'],
  [/\bwith regard(s)? to\b/gi,                 're:'],
  [/\bwith respect to\b/gi,                    're:'],
  [/\bin regard to\b/gi,                       're:'],
  [/\bin terms of\b/gi,                        'for'],
  [/\bin the same way\b/gi,                    'similarly'],
  [/\bin a similar manner\b/gi,                'similarly'],
  [/\bin a similar way\b/gi,                   'similarly'],
  [/\bthe way in which\b/gi,                   'how'],
  [/\bthe reason why\b/gi,                     'why'],
  [/\bas well as\b/gi,                         'and'],
  [/\bfirst and foremost\b/gi,                 'first'],
  [/\bfirst of all\b/gi,                       'first'],
  [/\blast but not least\b/gi,                 'finally'],

  // ── Action nominalizations → verb ────────────────────────────────────────────
  [/\bmakes? improvements to\b/gi,             'improve'],
  [/\bmakes? modifications to\b/gi,            'modify'],
  [/\bmakes? adjustments to\b/gi,              'adjust'],
  [/\bmakes? updates to\b/gi,                  'update'],
  [/\bmakes? changes to\b/gi,                  'change'],
  [/\bmake a recommendation\b/gi,              'recommend'],
  [/\bmake a suggestion\b/gi,                  'suggest'],
  [/\bmake a decision\b/gi,                    'decide'],
  [/\bmake use of\b/gi,                        'use'],
  [/\bhad an (impact|effect) on\b/gi,          'affected'],
  [/\bhas an (impact|effect) on\b/gi,          'affects'],
  [/\bhave an (impact|effect) on\b/gi,         'affect'],
  [/\btake a look at\b/gi,                     'check'],   // must precede "take a look"
  [/\bhave a look at\b/gi,                     'check'],
  [/\btake a look\b/gi,                        'check'],
  [/\btake into consideration\b/gi,            'consider'],
  [/\btake into account\b/gi,                  'consider'],
  [/\bgive consideration to\b/gi,              'consider'],
  [/\bkeep in mind\b/gi,                       'note'],
  [/\bbear in mind\b/gi,                       'note'],
  [/\bprovide assistance( to)?\b/gi,           'help'],
  [/\bcome to the conclusion\b/gi,             'conclude'],
  [/\breach a conclusion\b/gi,                 'conclude'],
  [/\bcome to an agreement\b/gi,               'agree'],

  // ── Zero-value discourse markers → delete entirely ───────────────────────────
  [/\bas (previously |already )?mentioned( (?:above|before|earlier))?[,]?\s*/gi, ''],
  [/\bas noted( (?:above|before|earlier))?[,]?\s*/gi,                            ''],
  [/\bit (is|should be) (worth noting|noted) that[,]?\s*/gi,                     ''],
  [/\bit is important to note that[,]?\s*/gi,                                     ''],
  [/\bit['']s worth (noting|mentioning) that[,]?\s*/gi,                           ''],
  [/\bit is (?:clear|evident|obvious) that[,]?\s*/gi,                             ''],
  [/\bas you can see[,]?\s*/gi,                                                    ''],
  [/\bit goes without saying( that)?[,]?\s*/gi,                                   ''],
  [/\bneedless to say[,]?\s*/gi,                                                   ''],
  [/\bas a matter of fact[,]?\s*/gi,                                               ''],
  [/\bin actual fact[,]?\s*/gi,                                                    ''],
  [/\bthe fact that\b/gi,                                                          'that'],

  // ── Standard abbreviations ───────────────────────────────────────────────────
  [/\bthat is to say\b/gi,                     'i.e.'],
  [/\bin other words\b/gi,                     'i.e.'],
  [/\bfor example\b/gi,                        'e.g.'],
  [/\bfor instance\b/gi,                       'e.g.'],
  [/\bet cetera\b/gi,                          'etc.'],
  [/\band so (on|forth)\b/gi,                  'etc.'],
  [/\bas opposed to\b/gi,                      'vs.'],
  [/\bcompared (?:to|with)\b/gi,               'vs.'],
  [/\bversus\b/gi,                             'vs.'],
  [/\bapproximately\s*/gi,                     '~'],
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
