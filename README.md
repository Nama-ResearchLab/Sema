<div align="center">

# Sema

**Offline Swahili Language Engine, Concord Compiler, and Fact Guardian (`v0.2.5`)**

*Pure data-driven morphological synthesis, 18-class non-verbal concord, pure-Rust decomposition, deterministic clause composition, and zero-leak clinical fact shielding.*

[![license](https://img.shields.io/badge/code-MIT-blue.svg)](LICENSE)
[![data license](https://img.shields.io/badge/data-CC%20BY--SA%204.0-orange.svg)](DATA-LICENSE)
[![language](https://img.shields.io/badge/rust-2024%20edition-DEA584.svg)](https://rustup.rs)
[![version](https://img.shields.io/badge/version-v0.2.5-0D9488.svg)](Cargo.toml)
[![tests](https://img.shields.io/badge/tests-96%20passing-059669.svg)](tests/)
[![roundtrip](https://img.shields.io/badge/round--trip-500%2F500%20exact-059669.svg)](tests/v25_morph.rs)
[![fact guardian](https://img.shields.io/badge/adversarial%20leaks-0%2F24%20(0%25)-059669.svg)](tests/adversarial.rs)
[![gui](https://img.shields.io/badge/studio-local%20gui%20%2B%20mcp-7C3AED.svg)](src/bin/sema_gui.rs)

[Web Studio and Downloads](https://nama-researchlab.github.io/Sema/) | [Documentation](https://nama-ResearchLab.github.io/Sema/) | [Mini Course](learn/)

</div>

---

Your model barely speaks Swahili. Your users only speak Swahili.

Sema bridges the gap offline, deterministically, and without cloud calls or neural weights: all Swahili morphology, concord agreements, and phonological rules live in external data tables ([src/synth/sw.toml](src/synth/sw.toml)), leaving the Rust engine completely language-agnostic with only `serde`, `serde_json`, and `toml` as runtime dependencies.

> [!IMPORTANT]
> **Status:** Research engine, version `0.2.5` (`v0.2.5`). Sema is not validated for clinical use, medical diagnosis, treatment planning, or regulatory compliance. Outputs must be reviewed by a qualified Swahili speaker and clinician before any patient-facing or clinical use. The realization layer is deterministic and does not generate unconstrained prose.

---

## What Shipped in `v0.2.5`

1. **Full Verb Morphology ([src/synth/mod.rs](src/synth/mod.rs), [src/synth/sw.toml](src/synth/sw.toml)):**
   - **Relative clauses (Slot 2b):** Noun-class and person relative infixes (`-ye-`, `-o-`, `-yo-`, `-lo-`, `-cho-`, `-vyo-`, `-zo-`, `-po-`, `-ko-`, `-mo-`) plus future allomorphy `-ta- -> -taka-` (`waliowaua`, `watakaojibu`, `nitakapofika`).
   - **Conditionals & counterfactuals:** `COND` (`-ki-`), `COND_PRES` (`-nge-`), `COND_PAST` (`-ngali-`), and negative conditionals with positive subject prefixes (`nisiposoma`, `nisingesoma`, `nisingalisoma`).
   - **Subjectless tenses, short-stem rules, & imperatives:** Habitual `hu-` and infinitive `ku-` (`husoma`, `kumuua`, `kuyakabili`), monosyllabic object elision (`alinipa` vs `anakula`), `m + u -> mu` glide (`alimuuliza` vs `sijamwona`), singular/plural/object imperatives (`nenda`, `njoo`, `someni`, `mpe`), extended epenthesis (`ilifunguliwa`, `pewa`), C1/C2 negative fusion, and strict `Engine::validate(&Intent)`.
2. **18-Class Non-Verbal Concord Engine ([src/concord/mod.rs](src/concord/mod.rs)):**
   - Table-driven synthesis for plain/emphatic copulas (`ni`/`si`, `ndiyo`/`siyo`), locative copulas (`yuko`/`hayuko`/`kiko`), existential/possessive `-na` (`nina`/`hakuna`), associatives (`wa`/`ya`/`cha`/`za`), possessives (`wangu`/`vyetu`), demonstratives (`huyu`/`hicho`), adjectives (`mdogo`/`vingi`), numerals (`mmoja`/`viwili`), and locative `-ni` (`hospitalini`).
3. **Pure-Rust Morphological Decomposer ([src/morph/mod.rs](src/morph/mod.rs)):**
   - Peels affixes directly from `sw.toml` tables across finite verbs, infinitives, imperatives, copulas, locative copulas, and `-na` possessives, verifying every analysis via exact round-trip re-synthesis (`500/500` gold verbs exact).
4. **Deterministic Clause Composer ([src/compose/mod.rs](src/compose/mod.rs)):**
   - Compiles structured templates (`{t:child} {poss:@child|1SG} {loc_cop:@child|ko} {loc:@hospital}` -> `"mtoto wangu yuko hospitalini"`) with automatic `@term` noun-class propagation and verb round-trip verification.
5. **Hardened Fact Guardian ([src/guardian/mod.rs](src/guardian/mod.rs)):**
   - Zero-regex tokenizer shielding decimals (`2.5`, `2,5`), fractions (`1/2`, `nusu`), hyphen/word ranges (`500-1000`, `500 to 1000`), spaced digits (`5 0 0`), Swahili & English number words (`mia tano`, `five hundred`), brands (`Panadol`, `Septra`, `Augmentin`), and Damerau-Levenshtein typo-tolerant drug names (`paracetemol` -> `paracetamol`), cutting adversarial boundary leaks from **`54.2%` (`13/24`) in `v0.2.0` down to `0.0%` (`0/24`) in `v0.2.5`**.

---

## Grammatical Coverage Matrix (`v0.2.5`)

| Grammatical Feature | Status | Implementation Details |
|---|---|---|
| Verb subject concords | Supported | Personal concords (`1SG`..`3PL`) and noun classes `C1`–`C18` mapped in [src/synth/sw.toml](src/synth/sw.toml) + C1/C2 negative fusion. |
| Verb object concords (infixes) | Supported | Personal (`-ni-`, `-ku-`, `-m-/-mw-/-mu-`, `-tu-`, `-wa-`) and noun-class infixes (`-ki-`, `-vi-`, `-i-`, `-zi-`, `-li-`, `-ya-`), including short-stem elision. |
| Tense and aspect markers | Supported | Present (`-na-`), past (`-li-`), future (`-ta-`/`-taka-`), perfect (`-me-`), habitual (`hu-`), consecutive (`-ka-`), not-yet (`-ja-`), infinitive (`ku-`). |
| Negation morphology | Supported | Fused negative-subject prefixes (`si-`, `hu-`, `ha-`, `hatu-`, `ham-`, `hawa-`), negative past (`-ku-`), not-yet (`-ja-`), present negative `-a -> -i`, negative subjunctive (`-si-`). |
| Relative clauses (`-ye-`, `-o-`, `-cho-`, `-po-`) | Supported | All noun-class & person relative infixes inside Slot 2b plus future `-taka-` allomorphy (`watakaojibu`, `nitakapofika`). |
| Conditionals (`-ki-`, `-nge-`, `-ngali-`, `-sipo-`) | Supported | Positive and negative conditionals (`-sipo-`, `-singe-`, `-singali-`) with positive subject prefixes. |
| Verb extensions (all 5) | Supported | Applicative (`-ia/-ea/-lia/-lea`), causative (`-isha/-esha/-za`), passive (`-wa/-iwa/-ewa/-liwa`), reciprocal (`-an-`), stative (`-ika/-eka/-ka`), stacked extensions & borrowed roots. |
| Subjunctive and imperative | Supported | Subjunctive `-e`, negative `-si-...-e`, irregular imperatives (`nenda`, `njoo`, `kula`), plural `-eni/-ni`, and object-bearing imperatives (`mpe`). |
| Adjective & numeral concords | Supported | Table-driven dynamic inflection across noun classes (`mdogo`/`wadogo`/`vingi`, `mmoja`/`wawili`/`sita`) in [src/concord/mod.rs](src/concord/mod.rs). |
| Copulas & locatives (`ni/si`, `yuko`, `-ni`) | Supported | Emphatic copulas (`ndiyo`/`siyo`), locative copulas (`yuko`/`hayuko`), existential `-na` (`hakuna`), associatives, possessives, demonstratives, and `-ni` suffixation. |
| Compound tenses & `amba-` relatives | Planned (`v0.3`) | Two-verb auxiliary chains (`alikuwa akisoma`) and independent `amba-` pronouns (`ambaye`) remain outside the single-verb pipeline for `v0.3`. |

---

## Verified Benchmarks (`v0.2.5`)

Run locally with `cargo test` and `cargo run --release --example acceptance`:

| Check / Suite | Scope | Measured Result |
|---|---|---|
| Library & v2.5 Integration Suites | `cargo test` (`lib`, `adversarial`, `gold100`, `synth`, `v25_*`, doctest) | **96 passing, 0 failing** |
| Pure-Rust Decomposer Round-Trip | [tests/v25_morph.rs](tests/v25_morph.rs) (`Sema_Benchmark_1000` `EN->SW` gold verbs + 28 complex forms) | **500 / 500 (100.0%)** |
| Hardened FactGuardian Adversarial Gate | [tests/adversarial.rs](tests/adversarial.rs) & [tests/v25_guardian.rs](tests/v25_guardian.rs) (`24` cases across `8` categories) | **0 / 24 leaked (0.0% vs 54.2% in v0.2.0)** |
| Distilled Lexicon Sentence Coverage | `cargo run --release --example acceptance` (`19,717` lemmas) | **20 / 24 words (83%)** |
| Core Word Synthesis Latency | `100,000` runs in release mode (`Engine::synthesize` on pre-built `Intent`) | **~0.78 us (3.2 GHz x86_64) / 1.06 us (ARM)** |

*Methodology Note:* Synthesis, round-trip, and gold benchmark checks verify strict internal consistency against the `sw.toml` linguistic specification; clinical deployment still requires review by qualified Swahili clinicians.

---

## Quickstart

### 1. Build & Verify

```bash
git clone https://github.com/Nama-ResearchLab/Sema
cd Sema
cargo test
cargo run --release --example acceptance
```

### 2. Command-Line Interface (`sema` & `sema_cc`)

```bash
# Relative clause synthesis with future -taka- allomorph -> watakaojibu
cargo run --release --bin sema -- synth --subj 3PL --tense FUT --rel C2 --root jibu

# Non-verbal concord & deterministic clause composition -> mtoto wangu yuko hospitalini
cargo run --release --bin sema -- concord --kind adj --class KI --stem dogo
cargo run --release --bin sema -- compose "{t:child} {poss:@child|1SG} {loc_cop:@child|ko} {loc:@hospital}"

# Pure-Rust morphological decomposition + exact round-trip check
cargo run --release --bin sema -- roundtrip waliowaua

# Hardened FactGuardian clinical & Swahili number shielding
cargo run --release --bin sema -- terms "Meza paracetemol mia tano mg kila asubuhi"
```

### 3. Local Web Studio & MCP Server

```bash
# Interactive Studio UI (http://127.0.0.1:4242)
cargo run --release --bin sema-gui

# Stdio Model Context Protocol (MCP) tool server
cargo run --release --bin sema -- mcp
```

---

## Rust Library API (`v0.2.5`)

```rust
use sema::{
    Composer, ConcordEngine, Decomposer, FactGuardian, RestoreMode,
    SynthEngine, SynthIntent,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Verb synthesis with relative infix & future -taka- allomorphy
    let engine = SynthEngine::swahili();
    let mut intent = SynthIntent::simple("3PL", "FUT", "jibu");
    intent.relative = Some("C2".into());
    assert_eq!(engine.synthesize(&intent).surface, "watakaojibu");

    // 2. Non-verbal concord & deterministic clause composition
    let concord = ConcordEngine::swahili();
    assert_eq!(concord.adjective("KI", "dogo")?, "kidogo");
    let clause = Composer::swahili()
        .render("{t:child} {poss:@child|1SG} {loc_cop:@child|ko} {loc:@hospital}")?;
    assert_eq!(clause.surface, "mtoto wangu yuko hospitalini");

    // 3. Pure-Rust decomposition & exact round-trip check
    let rep = Decomposer::swahili().verify_roundtrip("waliowaua");
    assert!(rep.ok && rep.output.as_deref() == Some("waliowaua"));

    // 4. Zero-leak clinical & numeric shielding
    let guardian = FactGuardian::default_clinical();
    let prot = guardian.protect("Give paracetemol mia tano mg to the child");
    let res = guardian.restore(&prot.text, &prot, RestoreMode::Swahili);
    assert!(res.integrity_ok);
    Ok(())
}
```

---

## Repository Structure

```text
Sema/
├── Cargo.toml                  # v0.2.5 crate manifest (deps: serde, serde_json, toml)
├── index.html & docs/          # GitHub Pages Studio Pro, Academy & Benchmarks site
├── learn/                      # Self-contained offline Swahili grammar & systems course
├── src/
│   ├── lib.rs                  # Unified v0.2.5 API (v0.2.0 modules retained for compat)
│   ├── synth/                  # Data-driven verb engine & sw.toml rule tables
│   ├── concord/                # 18-class copula, locative, adj, num & demonstrative concord
│   ├── morph/                  # Pure-Rust morphological decomposer & round-trip verifier
│   ├── compose/                # Deterministic template clause composer
│   ├── guardian/               # Hardened numeric, Swahili number word & typo-drug guardian
│   ├── lexicon.rs, linearizer.rs, racer.rs, render.rs, rolodex.rs, terms.rs
│   ├── bin/{sema,sema_gui,sema_cc,sema-distill}.rs
│   └── gui/app.html            # Embedded Studio UI
├── data/
│   ├── swahili.distilled.jsonl # 19,717 Wiktionary-distilled lemmas (CC BY-SA 4.0)
│   ├── terms.clinical.tsv      # Starter clinical dictionary
│   ├── terms.core.tsv          # Core Swahili nominal/number lexicon
│   └── terms.general.tsv       # General noun extraction
└── tests/
    ├── gold100.rs, adversarial.rs, synth.rs
    └── v25_{concord,grammar,guardian,morph}.rs
```

---

## Attribution and Licenses

- **Code**: Massachusetts Institute of Technology (MIT) license (see [LICENSE](LICENSE)).
- **Lexicon Data**: Creative Commons Attribution-ShareAlike 4.0 International (CC BY-SA 4.0) (see [DATA-LICENSE](DATA-LICENSE) and [DATA-SOURCES](DATA-SOURCES)). Distilled from Wiktionary via Kaikki.
- Developed by **Nama Research Lab**.
