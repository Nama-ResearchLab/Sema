# References (what each one is good for)

## Data sources (where our words come from)

- Wiktionary via kaikki.org per-language dumps: the raw material for
  the 19,717-lemma lexicon. Good for: rebuilding or extending the
  dictionary. License CC BY-SA 4.0.
- FreeDict swh-eng: small Swahili-English dictionary in TEI XML. Good
  for: cross-checking glosses.
- Afri-Dict Swahili reverse: audited English-to-Swahili rows. Good
  for: the EN to SW direction.
- Helsinki Corpus of Swahili: large research corpus, background
  reading. Not bundled.

## Related systems (how others approach it)

- SALAMA, University of Helsinki (Hurskainen, since 1985): the most
  complete rule-based Swahili environment (analysis, translation,
  dictionary compilation). Closed source and commercial. Good for:
  understanding what 35 years of hand-written rules achieve, and why
  a data-driven approach differs.
- UralicNLP Swahili transducers: free offline analyser and generator
  files. Good for: comparing finite-state analysis with our table
  approach.
- Apertium swa / eng-swa: open rule-based tooling, Swahili coverage
  minimal (incubator). Good for: seeing how far community rule
  writing gets without sustained effort.
- SwaRegex (2022): verb segmentation with regular expressions,
  analysis only. Good for: the 10-slot verb template view.
- Masakhane (EMNLP 2020 Findings): participatory neural translation
  for African languages, Swahili baselines included, with explicit
  production limits. Good for: why sentence-level neural models are
  not the answer here.

## Companion repositories

- Sema-Tena: the data-driven back-pass synthesis engine, with full
  benchmark tables, torture results, and the neural comparison
  harness.
- Jericho: desktop host wiring both directions with user-chosen
  local models.

Back to the [course index](README.md).
