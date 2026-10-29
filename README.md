<div align="center">

# Sema

**"say" in Swahili**

*Offline Swahili understanding and Swahili speech, in both directions.*

[![license](https://img.shields.io/badge/code-MIT-blue.svg)](LICENSE)
[![data license](https://img.shields.io/badge/data-CC%20BY--SA%204.0-orange.svg)](DATA-LICENSE)
[![language](https://img.shields.io/badge/rust-stable-DEA584.svg)](https://rustup.rs)
[![version](https://img.shields.io/badge/version-v0.2.0-blue.svg)](Cargo.toml)
[![tests](https://img.shields.io/badge/tests-58%2F58%20green-00A651.svg)](src/lib.rs)

</div>

---

Your model barely speaks Swahili. Your users only speak Swahili.

Small LLMs are overwhelmingly English-trained. Fine-tuning is expensive.
Translation APIs need the cloud. **Sema** takes the boring, deterministic
route that actually works on a phone with no signal, and it now runs the
whole round trip: Swahili in, meaning out, meaning back in, Swahili out.

```
                 anchor            your model          render / terms
Swahili input ----------> English ----------> structured -------> Swahili
                  (Sema)      skeleton      reasons here       templates    output

                           (never back-translate prose)
```

New here? Start with the mini-course: **[learn/](learn/)** -- first
principles to advanced in six short lessons, with interactive pages
(course hub, verb explorer, hover glossary), guides, references, and
examples.

## What's in the box

| | |
|---|---|
| **19,717 lemmas** | nouns / verbs / adjectives / more -- distilled from Wiktionary |
| **Morphology-aware** | `umefikia` -> `fikia` (*arrive at*, root `-fika`) via language affix rules |
| **Verb decomposition** | `hawatakujibu` -> negation, subject, tense, object, root (`morph` command) |
| **Cascade resolution** | three racers compete per word; fastest good-enough parse wins (`racer`) |
| **Negation gate** | negated verbs beat noun hits (`sili` -> *I not eat*, not "seal"); explicit `[?word]` |
| **Role tagging + render** | sentence S/V/O/M roles and gloss reverse-index (`role_tagger`, `render` examples) |
| **Back-pass synthesis** | structured intent -> exact Swahili verb in microseconds (`synth` command) |
| **Derivations + harmony** | passive, applicative, causative, reciprocal, stative; `-esha` harmony, `-iwa` short stems |
| **Noun-class agreement** | every synthesis names the agreeing class (`C7` -> class 7 `ki/vi`, `C1..C18`) |
| **Term & ID protection** | `src/terms.rs`: lock drug names, doses (`500 mg`), numbers as `[#T..]`/`[#N..]` before LLM |
| **Drop-in dictionaries** | TSV dictionaries for clinical and core vocabulary (`data/terms.*.tsv`) |
| **JSON CLI** | anchor, morph, linearize, rolodex, synth, race, terms, bench over stdout (`sema_cc`) |
| **Zero dependencies** | pure Rust, no network, no cloud, no API keys, no models |
| **Self-contained** | 100% offline, standalone crate with zero external private dependencies |

## Quickstart

```bash
git clone https://github.com/Nama-ResearchLab/Sema
cd Sema
cargo test                                 # 58 tests + doctests green
cargo run --release --example acceptance
```

Real output:

```
SW: umefikia wapi?
   -> umefikia[fikia:verb 'Applicative form of -fika: to lodge at, ... arrive at' root=-fika]
   -> wapi[wapi:adv 'where']
   -> ?[?? unresolved]
```

Forward (understand one verb):

```bash
cargo run --bin sema_cc -- morph --word hawatakujibu
# {"negation":"ha","object":"ku","root":"jibu","subject":"wa","tense":"ta",...}
```

Backward (say one verb):

```bash
cargo run --bin sema_cc -- synth --subj 1SG --tense PAST --root fanya --neg
# {"surface":"sikufanya","slots":[...],"subject_class":null,...}
```

Round trip (apart and back together):

```bash
cargo run --bin sema_cc -- synth --subj 3PL --tense FUT --root jibu --obj ku --neg
# {"surface":"hawatakujibu",...}
```

Use it as a library:

```rust
let lex = sema::Lexicon::load("data/swahili.distilled.jsonl")?;

// exact hit
lex.skeleton_for("habari");      // habari[noun] 'news'

// affix-resolved verb form
lex.skeleton_for("unasema");     // unasema -> sema[verb] 'to say, speak'

// decompose to parts
let m = sema::linearizer::decompose_verb("sikufanya");
// subject "ni", tense "ku", negation present, root "fanya"

// render parts back to an exact verb
let e = sema::backpass::engine();
let intent = sema::backpass::intent_from_parts(&e, Some("ni"), Some("ku"), true, None, "fanya");
e.synthesize(&intent).surface;    // "sikufanya"
```

## The double-pass pattern

Sema is deliberately **not** a translator. It is the anchor layer of an
architecture for serving low-resource-language users with strong-in-English
models:

1. **Anchor** -- Sema turns Swahili text into an English semantic skeleton
2. **Reason** -- your LLM works in its strongest language
3. **Render** -- emit answers from *bilingual templates/fields*, so medical
   facts, dosage numbers and warnings never pass through lossy back-translation

The forward half lives here (lexicon, decomposer, linearizer, racer). The
render half runs on data tables through the back-pass engine: every verb
fills `[NEG][SUBJ][TENSE][OBJ][ROOT][DERIV][MOOD]` from TOML, with zero
language rules in code. Fused negation (`si-`, `hawa-`), TAM overrides
(`li->ku`, `me->ja`), intrusive `-ku-`, glides, derivations, harmony, and
noun-class labels all come from data.

## Measured, not promised

- **58/58 tests green** + doctest passing cleanly.
- **Negation Gate**: 26/26 negated sentences preserve an explicit `NOT` token; unresolved words emitted as `[?word]`.
- **Back-pass**: 100/100 gold intent benchmark exact; 10/10 derivations with vowel harmony (applicative, causative, stative, reciprocal, passive).
- **End-to-end pairs**: 100/100 complete pairs (`benchmark/pairs/pairs100.csv`) verified end-to-end through anchor -> model standin -> term protection -> synthesis.
- **Round-trip**: 500/500 gold verbs plus tricky cases (monosyllabic stems, subjunctives, glides, derivations, class concords `C1..C18`) decompose and re-synthesize exactly.

## Sub-1B Clinical Workflow: Term & ID Protection

Small models frequently flip negations, hallucinate dosages, or corrupt drug names. Sema's **Term Replacer** (`src/terms.rs`) locks clinical vocabulary, numbers, and dosages behind opaque IDs:

```
Input: "Give two tablets of paracetamol 500 mg twice a day. Do not give aspirin."
  │
  ▼ [protect] (matches data/terms.*.tsv)
"Give [#T1] of paracetamol [#N1] [#N2] twice a day. Do not give aspirin."
  │
  ▼ [LLM reasons in English]
  ▼ [restore with noun-class agreement]
"Mpe mtoto vidonge viwili vya paracetamol 500 mg mara mbili kwa siku. Usimpe aspirin."
```

CLI test:
```bash
cargo run --bin sema_cc -- terms \
  --dict data/terms.clinical.tsv,data/terms.core.tsv \
  --text "Give two tablets of paracetamol 500 mg"
```

## Companion repositories

- [sema-kesho](https://github.com/Nama-ResearchLab/sema-kesho) -- Internal production integration and research laboratory: Jericho desktop bridge, 400-prompt neural-vs-symbolic eval harness, 46M-case torture sweeps, and raw dictionary ingest pipelines.
- [Project Jericho](https://github.com/noelchesco5/jericho) -- desktop app wiring both directions with user-chosen local models.

## Adding a language

Wiktionary covers thousands of languages and [kaikki.org](https://kaikki.org)
publishes per-language dumps. The pipeline is language-agnostic:

```bash
# 1. grab a dump (example: Yoruba)
wget https://kaikki.org/dictionary/Yoruba/kaikki.org-dictionary-Yoruba.jsonl

# 2. distill
cargo run --release --bin sema-distill -- kaikki.org-dictionary-Yoruba.jsonl yoruba.distilled.jsonl

# 3. add an affix table (see src/affix/sw.toml as template) -- done
```

Affix rules live in TOML files (`src/affix/`), not in code. Contributions
for new languages = one data file + one affix table.

## Regenerating / verifying the Swahili data

```bash
wget https://kaikki.org/dictionary/Swahili/kaikki.org-dictionary-Swahili.jsonl
cargo run --release --bin sema-distill -- kaikki.org-dictionary-Swahili.jsonl data/swahili.distilled.jsonl
cargo test          # unit gates
cargo run --release --example acceptance   # coverage harness
```

Current acceptance coverage: **20/24 tokens (83%)** across a mixed Swahili
acceptance set; **20/21 (95%)** with proper names and punctuation excluded by
design (the two "?" tokens and the proper name "saida" are not in the shipped
19,717-lemma distilled lexicon and correctly return None, per the
honest-unresolved rule).

## Attribution (required)

The distilled lexicon derives from [Wiktionary](https://en.wiktionary.org),
extracted via [kaikki.org](https://kaikki.org). Both the raw dumps and the
distilled files are licensed **CC BY-SA 4.0** -- see
[DATA-LICENSE](DATA-LICENSE). If your product ships `swahili.distilled.jsonl`
(or derivatives), you must credit Wiktionary contributors and keep the
share-alike terms. Code is MIT -- see [LICENSE](LICENSE).

We are grateful to the Wiktionary contributor community and to Tatu Ylonen's
kaikki.org project for making low-resource language data accessible.

## Status & roadmap

- [x] v0.1 -- Swahili lexicon + naive two-slot verb morphology
- [x] sentence-level role tagging (POS-based S/V/O/M tagging)
- [x] render layer (gloss reverse-index for bilingual output)
- [x] forward pipeline: decomposition (`linearizer`), cascade resolution
      (`racer`), synthesis intents (`rolodex`), JSON CLI (`sema_cc`)
- [x] back-pass wired in: data-driven synthesis (`backpass`), `synth`
      command, round-trip checks, noun-class reporting
- [x] learn mini-course: first principles to advanced (`learn/`)
- [ ] richer Swahili morphology (object markers, locative infixes, negation slots)
- [ ] EN->SW direction (harvested from kaikki English `translations` fields)
- [ ] second language proof (community pick: Hausa? Luganda?)

Sema is developed as part of **Eden** (Nama Research Lab) -- an edge-AI
runtime stack bringing useful AI to low-end devices, offline.

## Research findings (Aug 2026)

Tested the anchor→reason→render pipeline with qwen2.5:0.5b and 1.5b on
a 100-sentence Swahili grammar set.

**Key finding:** At model sizes >= 1.5B, the model handles Swahili
directly without anchoring help. Sema's value is in:

1. **RAG lemmatization** -- agglutinative Swahili breaks TF-IDF;
   Sema's affix stripping fixes this (10/10 vs 7/10 retrieval).
2. **Render layer** -- bilingual output via gloss reverse-index.
3. **Low-resource models** -- at 0.5B, glosses help slightly but the
   model still can't compose responses.

Results and architecture documented in the companion project:
[Project Jericho](https://github.com/noelchesco5/jericho/blob/main/docs/RESEARCH.md)

## License

Code: MIT -- [LICENSE](LICENSE).
Data: CC BY-SA 4.0 -- [DATA-LICENSE](DATA-LICENSE).
