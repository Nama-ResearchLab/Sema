<div align="center">

# Sema

**"say" in Swahili**

*Offline semantic anchoring for low-resource languages.*

[![license](https://img.shields.io/badge/code-MIT-blue.svg)](LICENSE)
[![data license](https://img.shields.io/badge/data-CC%20BY--SA%204.0-orange.svg)](DATA-LICENSE)
[![language](https://img.shields.io/badge/rust-stable-DEA584.svg)](https://rustup.rs)

</div>

---

Your model barely speaks Swahili. Your users only speak Swahili.

Small LLMs are overwhelmingly English-trained. Fine-tuning is expensive.
Translation APIs need the cloud. **Sema** takes the boring, deterministic
route that actually works on a phone with no signal:

```
                anchor            your model          render
Swahili input ----------> English ----------> structured -------> Swahili
                 (Sema)      skeleton      reasons here       templates    output

                          (never back-translate prose)
```

Sema provides step 1: it resolves surface words to lemmas, parts of speech,
English glosses, and derivational roots -- fully offline, in microseconds,
from a 1.8 MB data file.

## What's in the box

| | |
|---|---|
| **19,717 lemmas** | nouns / verbs / adjectives / more -- distilled from Wiktionary |
| **Morphology-aware** | `umefikia` -> `fikia` (*arrive at*, root `-fika`) via language affix rules |
| **Zero dependencies at runtime** | no network, no API keys, no models |
| **1.8 MB** | ships inside any app; regenerable from raw dumps |

## Quickstart

New here? Start with the mini-course: **[learn/](learn/)** -- first
principles to advanced in six short lessons, with interactive pages
(course hub, verb explorer), guides, references, and examples.

```bash
git clone https://github.com/Nama-ResearchLab/Sema
cd Sema
cargo run --release --example acceptance
```

Real output:

```
SW: umefikia wapi?
   -> umefikia[fikia:verb 'Applicative form of -fika: to lodge at, ... arrive at' root=-fika]
   -> wapi[wapi:adv 'where']
   -> ?[?? unresolved]
```

Use it as a library:

```rust
let lex = sema::Lexicon::load("data/swahili.distilled.jsonl")?;

// exact hit
lex.skeleton_for("habari");      // habari[noun] 'news'

// affix-resolved verb form
lex.skeleton_for("unasema");     // unasema -> sema[verb] 'to say, speak'
```

## The double-pass pattern

Sema is deliberately **not** a translator. It is the anchor layer of an
architecture for serving low-resource-language users with strong-in-English
models:

1. **Anchor** -- Sema turns Swahili text into an English semantic skeleton
2. **Reason** -- your LLM works in its strongest language
3. **Render** -- emit answers from *bilingual templates/fields*, so medical
   facts, dosage numbers and warnings never pass through lossy back-translation

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

Current acceptance coverage: **88% of content words resolved** across a
mixed Swahili acceptance set (proper names and punctuation excluded by design).

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
