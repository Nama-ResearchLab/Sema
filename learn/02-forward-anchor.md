# Lesson 2: forward anchoring (Swahili to English)

Goal: understand how a surface word becomes a meaning skeleton, and run
it yourself. Assumes Lesson 1 vocabulary (lemma, affix, concord, gloss).

## 1. The two data sources

The forward pass uses no model, only two files:

1. The distilled lexicon (`data/swahili.distilled.jsonl`, 1.7 MB).
   One JSON object per line, 19,717 lemmas distilled from Wiktionary.
   Anatomy of one line:

       {"w":"fikia", "p":"verb", "g":["to arrive at"], "r":"-fika"}

   "w" is the headword, "p" the part of speech, "g" the English
   glosses, "r" the derivational root. This file is CC BY-SA 4.0, so
   credit Wiktionary contributors if you ship it.

2. The affix table (`src/affix/sw.toml`). Lists of subject concords,
   tense markers, object infixes, and stripping rules. Small, readable,
   editable without touching Rust.

## 2. Worked example: "umefikia"

Step 1, exact lookup: "umefikia" is not a headword, so lookup misses.
Step 2, strip affixes using the table: "u" (you) + "me" (perfect) +
"fikia". Step 3, "fikia" hits the lexicon: verb, "to arrive at",
root "-fika". Result, the skeleton:

    umefikia[fikia:verb 'to arrive at' root=-fika]

If nothing resolves, the word lands on the "unresolved" list instead
of being guessed. Unknown words are reported, never invented.

## 3. Whole verbs, decomposed

`decompose_verb` splits an agglutinated verb into named parts:

    hawatakujibu -> negation "ha", subject "wa", tense "ta",
                     object "ku", root "jibu"

Read it as: not (ha) + they (wa) + will (ta) + you (ku) + answer
(jibu) = "they will not answer you". Try shorter ones and read them
the same way: "sikufanya" (si + ku + fanya), "anakula" (a + na + la),
"nitakuja" (ni + ta + ja).

## 4. Sentence roles in one rule

Part of speech plus position gives subject, verb, object, modifier:

- First noun or pronoun before any verb: subject.
- The verb itself.
- Nouns after the verb: objects.
- Adjectives and adverbs: modifiers.

So "umefikia wapi?" tags as verb (umefikia) plus modifier (wapi).
The example program prints exactly this. See `examples/role_tagger.rs`.

## 5. Try it

From the repository root:

```bash
cargo run --bin sema_cc -- morph --word hawatakujibu
# {"negation":"ha","object":"ku","root":"jibu","subject":"wa","tense":"ta",...}

cargo run --example role_tagger
# verb: umefikia (to lodge at, to overcome, to arrive at) | adv: wapi (where)
```

What to notice: the JSON names the parts; the example prints the
human summary. Both come from the same library calls your own code
can use (`Lexicon::skeleton_for`, `decompose_verb`).

## 6. Speed and failure modes

Resolution runs in microseconds per word. It fails in exactly two
honest ways: the word is unknown (reported in "unresolved"), or the
gloss is noisy Wiktionary boilerplate (cleaned aggressively; what
cannot be cleaned is dropped, never shown to the model).

Next: [Lesson 3: back-pass rendering](03-backpass-render.md).
