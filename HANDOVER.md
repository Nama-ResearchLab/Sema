# SEMA â€” HANDOVER DOCUMENT

> Read this fully before touching anything. Last updated: 2026-08-22.

## 1. What Sema is (and deliberately is NOT)

Sema ("say" in Swahili) is an **offline semantic anchoring layer** for
low-resource languages, starting with Swahili. It resolves surface words to
lemmas + parts of speech + English glosses + derivational roots.

**It is NOT a translator and must not become one.** The architectural stance
(called "double-pass") is:

```
Swahili input â”€anchor(Sema)â†’ English skeleton â”€(host app's LLM reasons in EN)â”€>
structured output â”€render from bilingual templatesâ†’ Swahili output
```

Back-translating free prose is the thing we refuse to do â€” it loses clinical
facts, numbers, and negation. If a feature request pulls toward "just
translate it", push back and point them here.

Origin: Sema was extracted from the private Eden project
(`github.com/Nama-ResearchLab/EDENN`, dir `eden/bethlehem/src/lexicon.rs` + distill tooling).
Eden is an edge LLM runtime; Sema is the language layer that makes small
models usable for Swahili speakers on offline devices.

## 2. Repo layout

```
sema/
â”œâ”€â”€ Cargo.toml               edition 2024, deps: serde, serde_json, toml
â”œâ”€â”€ LICENSE                  MIT (code only)
â”œâ”€â”€ DATA-LICENSE             CC BY-SA 4.0 notice (data files) â€” LEGALLY REQUIRED
â”œâ”€â”€ README.md                public face; keep the attribution section intact
â”œâ”€â”€ data/
â”‚   â””â”€â”€ swahili.distilled.jsonl   19,717 lemmas, ~1.8MB, COMMITTED to git
â”œâ”€â”€ src/
â”‚   â”œâ”€â”€ lib.rs               crate docs + re-exports
â”‚   â”œâ”€â”€ lexicon.rs           core: LexEntry, Lexicon, Skeleton, resolver,
â”‚   â”‚                        clean_wiki(), root_from_glosses(), segment_words()
â”‚   â”œâ”€â”€ affix/
â”‚   â”‚   â”œâ”€â”€ mod.rs           AffixTable (TOML-driven stem-candidate generator)
â”‚   â”‚   â””â”€â”€ sw.toml          Swahili subject/tense prefixes (embedded via include_str!)
â”‚   â””â”€â”€ bin/
â”‚       â””â”€â”€ sema-distill.rs  raw kaikki JSONL -> distilled format
â””â”€â”€ examples/
    â””â”€â”€ acceptance.rs        runs real sentences, prints anchor coverage
```

## 3. Current state (v0.1.0)

Working:
- Exact-lemma lookup, inflected-forms index (`f:` field of entries), naive
  two-slot verb morphology (subject prefix + optional tense marker)
- Wiki-markup cleaning in the distiller ([[..]], {{..}})
- Form-of-gloss demotion (lemma-quality preference on duplicate words)
- Tests: `cargo test` â†’ 5 pass. Acceptance harness â†’ **88% word coverage** (21/24) on six mixed Swahili sentences. The 3 misses: proper name
  ("saida"), "?" punctuation â€” by design, not bugs.

Known limitations (the honest list):
1. Morphology model is shallow: no object markers (-m-, -wa- infixes), no
   negation slot beyond bare "ha", no relative infixes, no -po/-ko/-mo
   locative disambiguation. `hawapo` currently resolves via strip to
   "wapo" which happens to be right, but by accident of the table.
2. Gloss quality is Wiktionary-grade uneven (e.g. `mdogo` gives
   'younger brother' before the 'small' adjective sense). No sense ranking yet.
3. Only SWâ†’EN direction exists. ENâ†’SW requires harvesting translations
   (see Â§5 roadmap item 2).
4. `amina` resolves to intj 'amen' before verb 'believe' â€” POS-priority
   ranking doesn't exist yet.

## 4. Non-negotiables / conventions

- **Dual licensing is legal structure, not decoration.** Code MIT,
  data CC BY-SA 4.0 (Wiktionary origin via kaikki.org). Never merge them
  into one license file. Any new data artifact ships under CC BY-SA with
  provenance documented in `lexica`-style README or DATA-LICENSE.
- Keep the runtime dependency tree tiny (serde/serde_json/toml only).
  No regex crate, no network calls in the library.
- Affix rules are DATA (TOML), not code. A new language must never require
  editing the resolver.
- Errors are honest: unresolved words return None, not guesses.
- Test gate before any push: `cargo test && cargo run --release --example acceptance`
  (coverage must not drop below current baseline without explicit note).

## 5. Roadmap (ordered, with notes)

1. **Richer Swahili morphology (v0.2)**: extend sw.toml schema to slots
   `[neg][subject][tense][object]stem` + locative suffixes; keep candidate
   generation longest-first; add regression fixtures for hawapo/hakuna/
   watoto-wa classes.
2. **ENâ†’SW back direction**: harvest pairs from
   `kaikki.org-dictionary-English.jsonl` entries where
   `translations[].lang_code == "sw"` â†’ build `english.distilled.jsonl`
   with `g` = Swahili renderings. NOTE: as of handover, a background
   download of this multi-GB dump may still be running at
   `C:\Users\hp\Documents\eden\lexica\english_kaikki.jsonl`
   (resumable: `curl.exe -sL -C - <url>` â€” URL in eden git history).
3. **Sense ranking**: prefer non-form-of, shorter, capitalized-consistent
   glosses; consider frequency signal from raw dump head_templates.
4. **Second language proof** (community ask): Hausa or Luganda end-to-end.
   This validates the "adding a language = data + TOML" claim publicly.
5. **crates.io publish** once v0.2 lands (needs the above morphology bump
   so first public impression isn't the naive table).

## 6. Regenerating data

```bash
# raw dump (74MB for Swahili)
curl -L https://kaikki.org/dictionary/Swahili/kaikki.org-dictionary-Swahili.jsonl -o sw.jsonl
cargo run --release --bin sema-distill -- sw.jsonl data/swahili.distilled.jsonl
cargo test && cargo run --release --example acceptance
```

Distiller contract: JSONL in â†’ JSONL out, records
`{"w","p","g":[â‰¤3],"r"?,"f":[â‰¤8]}`. Keep the format stable; consumers exist
outside this repo (Eden/Bethlehem).

## 7. Deploy mechanics (for this machine)

- Git identity used: name `noelchesco5`,
  email `noelchesco5@users.noreply.github.com`
- Remote: `https://github.com/Nama-ResearchLab/Sema` (public)
- Pushes were made with a temporary classic PAT provided by the owner;
  if expired, ask owner for a fresh token with Contents:write. Pattern used:
  temporarily embed token in remote URL, push, immediately
  `git remote set-url origin <clean-url>` to scrub it.
- NEVER commit tokens, raw dumps, or target/.

## 8. Style notes from the author session

- Comments exist where intent isn't obvious from code; don't add narrational
  comments.
- The README's voice matters to the owner: direct, honest numbers, no hype.
  The "Your model barely speaks Swahili." hook stays unless he says otherwise.
- If you rename/move things, grep EDENN repo too â€” Bethlehem consumes Sema's
  distilled format (`eden/bethlehem/src/lexicon.rs` is the internal twin;
  plan is for Eden to depend on Sema as a path/git dependency later).
