<div align="center">

# Sema

**Offline Swahili Language Studio and Deterministic Engine**

*Rule-based morphological synthesis, slotted entity preservation, and self-consistency verification for Swahili language processing.*

[![license](https://img.shields.io/badge/code-MIT-blue.svg)](LICENSE)
[![data license](https://img.shields.io/badge/data-CC%20BY--SA%204.0-orange.svg)](DATA-LICENSE)
[![language](https://img.shields.io/badge/rust-2024%20edition-DEA584.svg)](https://rustup.rs)
[![version](https://img.shields.io/badge/version-v0.2.0-0D9488.svg)](Cargo.toml)
[![tests](https://img.shields.io/badge/tests-61%2F61%20green-059669.svg)](tests/)
[![gold suite](https://img.shields.io/badge/gold%20bench-100%2F100%20self--consistency-059669.svg)](tests/gold100.rs)
[![gui](https://img.shields.io/badge/studio-local%20gui%20%2B%20mcp-7C3AED.svg)](src/bin/sema_gui.rs)

[Web Studio and Downloads](https://nama-researchlab.github.io/Sema/) | [Documentation](https://nama-researchlab.github.io/Sema/) | [Mini Course](learn/)

</div>

---

## Scope and Limitations

> [!IMPORTANT]
> **Status:** Research prototype, version 0.2.0 (v0.2.0). Sema is not validated for clinical use, medical diagnosis, treatment planning, or regulatory compliance. Outputs must be reviewed by a qualified Swahili speaker and clinician before any patient-facing or clinical use.

Sema is a specialized rule-based engine designed to compile structured grammatical intents into Swahili surface verbs and replace protected entity spans via dictionary lookup. It is not an unconstrained text generator. The realization step is rule-based and does not generate unconstrained text.

### Grammatical Coverage Matrix

The table below details the grammatical features evaluated in the engine rules as of version 0.2.0 (v0.2.0):

| Grammatical Feature | Status | Implementation Details |
|---|---|---|
| Verb subject concords | Supported | Personal concords (1SG, 2SG, 3SG, 1PL, 2PL, 3PL) and noun classes 1 through 18 (C1 to C18) mapped in [src/rolodex.rs](file:///home/ahadi/nama/Sema/src/rolodex.rs). |
| Verb object concords (infixes) | Supported | Personal object infixes (-ni-, -ku-, -m-/-mw-, -tu-, -wa-) and noun class infixes (-ki-, -vi-, -i-, -zi-, -li-, -ya-). |
| Tense and aspect markers | Supported | Present (-na-), past (-li-), future (-ta-), perfect (-me-), habitual (-hu-), consecutive (-ka-), not-yet (-ja-), infinitive (-ku-). |
| Negation morphology | Supported | Subject-negation portmanteaus (si-, hu-, ha-, hatu-, ham-, hawa-), negative past (-ku-), negative perfect (-ja-), present negative zero marker with stem shift (-a to -i), negative subjunctive (-si-). |
| Verb extensions (applicative) | Supported | Suffixes -i- and -e- with vowel harmony; -li- and -le- for vowel-final stems. |
| Verb extensions (causative) | Supported | Suffixes -ish- and -esh- with vowel harmony; -z- for vowel-final stems. |
| Verb extensions (passive) | Supported | Suffix -w-; -liwa and -lewa for vowel-final stems; irregular monosyllabic forms (liwa, pewa, nywewa). |
| Verb extensions (reciprocal) | Supported | Suffix -an-. |
| Verb extensions (stative) | Supported | Suffixes -ik- and -ek- with vowel harmony; -k- for vowel-final stems. |
| Subjunctive and imperative | Supported | Subjunctive final vowel -e, negative subjunctive -si-, bare imperatives and irregular imperatives (nenda, njoo, kula). |
| Conditionals (-ki-, -ngali-, -nge-) | Partial | Markers can be passed as raw tense tags and monosyllabic -ku- support recognizes them, but negative conditional forms (-sipo-) and counterfactual compound clauses are not modeled. |
| Relative clauses | Partial | Temporal and locative relative infixes (-po-, -ko-, -mo-) are recognized; general relative pronoun infixes (-ye-, -o-, -cho-) and amba- relative clauses are not supported. |
| Adjective and numeral concords | Not supported | The engine synthesizes verb forms only; noun class concord agreement for adjectives and numerals is not dynamically inflected. |
| Locative morphology | Partial | Subject prefixes for locative classes 16, 17, and 18 (pa-, ku-, mu-) are mapped; noun locative suffixation (-ni) is not generated. |
| Copula (ni, si, ndiyo, sio, locatives) | Not supported | Copular sentences and inflected locative copulas (yuko, kiko) are not handled by the verbal synthesizer. |
| Compound tenses | Not supported | Multi-verb constructions (e.g. alikuwa akisoma) are outside the single-verb slot pipeline. |

### Input Limitations

- **Structured Intent Required:** Sema takes structured intent objects as input (specifying subject, tense, negation, object, root, derivations, and mood). It does not parse unconstrained free-form Swahili text into intent.
- **Morphological Analyzer Limits:** The forward analyzer in [src/linearizer.rs](file:///home/ahadi/nama/Sema/src/linearizer.rs) decomposes single verb words by peeling standard affixes. It cannot parse multi-word sentence syntax, colloquial contractions, non-standard dialectal variants, or resolve homographs that depend on syntactic context.

### Fact Protection Boundaries and Adversarial Results

The term protection component in [src/terms.rs](file:///home/ahadi/nama/Sema/src/terms.rs) replaces dictionary matches and numeric literals with opaque tokens before external processing, restoring them afterwards. In an adversarial boundary evaluation of 24 test cases across 8 categories, 11 passed and 13 leaked (54.2% leak rate):

- **Protected Categories (Passing):**
  - Numeric ranges with digits (e.g. "500-1000 mg", "500 to 1000 mg"): 3/3 passed.
  - Decimal values (e.g. "2.5 mg", "0.5 ml"): 3/3 passed.
  - Standard metric abbreviations and digit pairs (e.g. "500 mg", "10 ml"): passed.
- **Unprotected Categories (Known Leaks):**
  - Swahili number words (e.g. "mia tano", "kumi", "elfu moja"): 0/3 passed (100% leak rate; number words without digits are not recognized as numbers).
  - Unlisted brand names (e.g. "Panadol", "Septra", "Augmentin"): 0/3 passed (100% leak rate; unrecognized terms pass through unshielded).
  - Misspelled medication names (e.g. "paracetemol", "amoxillin", "ibuprufen"): 0/3 passed (100% leak rate; exact dictionary matching fails on typos).
  - Natural language fractions (e.g. "nusu kidonge"): 1/3 leaked (Swahili fraction words are unshielded unless explicitly entered into the dictionary).
  - Word numbers (e.g. "five hundred"): 1/3 leaked (word-based quantities pass through unless containing digits).
  - Mixed Swahili-English units (e.g. "vidonge 2 daily"): 2/3 leaked (Swahili nouns in mixed text are unshielded without dictionary entries).

### Failure Behavior Outside the Grammar

When input falls outside the supported grammar, [src/rolodex.rs](file:///home/ahadi/nama/Sema/src/rolodex.rs) returns an assembled string using fallback rules rather than an explicit error type:
- An unknown subject tag results in an omitted subject prefix.
- An unknown tense tag is inserted verbatim into the surface string.
- An unrecognized derivation name is skipped, leaving the stem unmodified.
- Non-verbal roots are processed as regular verb stems.

Users must validate input intent structures prior to synthesis.

---

## Architectural Overview

Sema provides deterministic rule-based checking, slot compilation, and lexical term protection:

```
[Structured Input Intent]
         │
         ▼
[Sema Synthesizer]      ──>  Deterministic synthesis via state tables
         │                   Applies phonological epenthesis ("tutakula"), vowel harmony,
         │                   and dictionary term replacements.
         ▼
[Self-Consistency Gate] ──>  Forward-pass deconstruction check:
                             Decomposes its own output to verify that negation
                             and slot roles match the original intent.
         │
         ▼
[Consistent Swahili Output]
```

---

## What is in the Box

| Component | Description |
|---|---|
| **19,717 Distilled Lemmas** | Headwords distilled from Wiktionary data via Kaikki into JavaScript Object Notation Lines (JSONL) with International Phonetic Alphabet (IPA), part of speech (POS), and English glosses. |
| **slotted Synthesizer** | Rule-based verb synthesis in [src/rolodex.rs](file:///home/ahadi/nama/Sema/src/rolodex.rs) mapping intent slots into surface forms. |
| **Self-Consistency Check** | Forward deconstruction validates backward synthesis against internal rules. |
| **Monosyllabic Epenthesis** | Automated -ku- infix insertion for monosyllabic roots (-la -> tutakula, kula, hatutakula). |
| **Vowel Harmony Engine** | Stative and applicative suffixes (-ea versus -ia, -esha versus -isha) computed dynamically from stem vowels. |
| **Noun Class Concords** | Concord mapping covering noun classes 1 through 18 (C1 to C18) for subject and object markers. |
| **Term Protector** | [src/terms.rs](file:///home/ahadi/nama/Sema/src/terms.rs): replaces dictionary-listed vocabulary and numeric literals with [#T1] and [#N1] tokens. |
| **Local Graphical Interface** | sema-gui: local graphical user interface (GUI) with interactive morpheme graph and local testing views. |
| **Model Context Protocol** | Local Model Context Protocol (MCP) server (sema mcp) over standard input/output (stdio) for local tool clients. |
| **Pure Local Execution** | Pure Rust implementation with zero network calls at runtime, zero telemetry, and zero remote dependencies. |

---

## Correctness and Benchmarks

### Correctness and Coverage Suites

1. **Gold Benchmark Suite (tests/gold100.rs):**
   - 100/100 matching intent vectors: executes 100 benchmark intents with matching surface strings.
   - **Provenance note:** These 100 benchmark intents were authored by the engine developer to verify rule coverage. They have not been audited or validated by an independent native speaker or medical professional.
2. **Derivational Coverage Suite:**
   - 10/10 derivational extension patterns passing across applicative, causative, reciprocal, stative, and passive derivations.
3. **Negation Consistency Suite:**
   - 0 negation inversions across the 100 gold intent vectors and round-trip consistency tests.
   - **Methodology note:** This is an internal self-consistency check comparing the engine output against its own forward rules, not an independent validation of linguistic correctness.
4. **Automated Test Suite:**
   - 61 automated tests passing (58 library unit tests, 1 documentation test, 2 integration test suites).

### Latency and Hardware Profile

Realization latency measures the time required for `Rolodex::synthesize(&intent)` to assemble a pre-constructed intent struct into an allocated surface string and slot vector:

- **Hardware and Environment:**
  - Architecture: ARM Cortex-A76 (4 cores at up to 2.4 GHz).
  - Operating System: Linux 6.6.
  - Compiler: rustc 1.99.1 (release profile, opt-level 3, thin LTO).
- **Measured Latency:**
  - 100,000 iterations: Mean latency of **1.06 microseconds** per word (1,062 nanoseconds; 941,505 operations per second).
  - Workstation reference (3.2 GHz core): Mean latency of ~0.78 microseconds per word.
- **Measurement Scope:**
  - This latency figure measures `synthesize()` only on pre-constructed intent structs. It does not include text parsing, lexical search, dictionary loading, or disk input/output.
- **Memory Allocation:**
  - Realization allocates dynamic memory on the heap for the returned surface `String` and `Vec<(String, String)>` slot records. It is not zero-heap.

---

## Quickstart

### 1. Build and Test

```bash
git clone https://github.com/Nama-ResearchLab/Sema
cd Sema
cargo test
```

### 2. Launch Sema Studio (Local GUI)

Sema includes a standalone graphical user interface (GUI) with an interactive morpheme relation graph and live testbed:

```bash
# Launch interactive Studio (binds http://127.0.0.1:4242 and opens local browser)
cargo run --release --bin sema-gui
```

### 3. Agent Tool Server (Model Context Protocol)

Run Sema as a local Model Context Protocol (MCP) server over standard input/output (stdio):

```bash
cargo run --release --bin sema -- mcp
```

Example tool client configuration:

```json
{
  "mcpServers": {
    "sema": {
      "command": "sema",
      "args": ["mcp"]
    }
  }
}
```

### 4. Command-Line Interface (CLI)

The command-line interface (CLI) provides utilities for morphology, synthesis, and term protection:

Decompose a Swahili verb into morpheme slots:

```bash
cargo run --release --bin sema -- morph --word hawatakujibu
# {"negation":"ha","object":"ku","root":"jibu","subject":"wa","tense":"ta",...}
```

Synthesize a Swahili verb from slot intent:

```bash
cargo run --release --bin sema -- synth --subj 3PL --tense FUT --root jibu --obj ku --neg
# {"surface":"hawatakujibu",...}
```

Protect clinical terms and quantities:

```bash
cargo run --release --bin sema -- terms \
  --text "Give two tablets of paracetamol 500 mg twice daily"
```

---

## Rust Library API

Add Sema to your `Cargo.toml`:

```toml
[dependencies]
sema = { git = "https://github.com/Nama-ResearchLab/Sema.git" }
```

```rust
use sema::{backpass, linearizer, Lexicon};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Load distilled lexicon
    let lex = Lexicon::load("data/swahili.distilled.jsonl")?;
    println!("Lemma: {:?}", lex.skeleton_for("unasema"));

    // 2. Decompose verb to morpheme parts
    let parts = linearizer::decompose_verb("sikufanya");
    println!("Subject: {:?}, Tense: {:?}, Neg: {:?}", parts.subject, parts.tense, parts.negation);

    // 3. Synthesize deterministically back to surface form
    let engine = backpass::engine();
    let intent = backpass::intent_from_parts(&engine, Some("ni"), Some("ku"), true, None, "fanya");
    let result = engine.synthesize(&intent);
    assert_eq!(result.surface, "sikufanya");

    println!("Synthesized surface: {}", result.surface);
    Ok(())
}
```

---

## Term and Quantity Shielding Workflow

To protect quantities and known terminology from unconstrained alteration, Sema provides dictionary-based token shielding in [src/terms.rs](file:///home/ahadi/nama/Sema/src/terms.rs):

```
Input Prompt: "Mpe mtoto vidonge viwili vya paracetamol 500 mg mara mbili kwa siku. Usimpe aspirin."
  │
  ▼ [protect] (matches data/terms.clinical.tsv)
"Mpe mtoto [#T1] vya [#T2] [#N1] [#N2] mara mbili kwa siku. Usimpe [#T3]."
  │
  ▼ [External reasoning over shielded tokens]
  │
  ▼ [Deterministic restore with slot replacement]
"Mpe mtoto vidonge viwili vya paracetamol 500 mg mara mbili kwa siku. Usimpe aspirin."
  │
  ▼ [Self-Consistency Check]
Status: Tokens restored: match (All issued IDs matched and accounted for)
```

---

## Repository Structure

```
Sema/
├── Cargo.toml                  # Standalone v0.2.0 crate definition
├── install.sh                  # Shell installer for local system
├── index.html                  # Project site and interactive client testbed
├── docs/                       # Site mirror for GitHub Pages
│   ├── index.html
│   └── assets/
├── assets/                     # Media and vector assets
├── DATA-SOURCES                # Provenance and licensing documentation
├── DATA-LICENSE                # Creative Commons Attribution-ShareAlike 4.0 license
├── LICENSE                     # MIT license for code
├── src/
│   ├── lib.rs                  # Public crate exports
│   ├── affix/                  # TOML grammar definitions (sw.toml)
│   ├── backpass/               # Deterministic synthesis engine and vowel harmony
│   ├── linearizer.rs           # Surface deconstruction and slot extraction
│   ├── terms.rs                # Term and numeric ID protector
│   ├── racer.rs                # Cascade resolution engine
│   ├── rolodex.rs              # Intent serialization and morphological parsing
│   ├── bin/
│   │   ├── sema.rs             # Unified command-line interface (CLI)
│   │   ├── sema_gui.rs         # Local Studio GUI and MCP server
│   │   ├── sema_cc.rs          # Legacy CLI utility
│   │   └── sema-distill.rs     # Lexicon distillation tool
│   └── gui/
│       └── app.html            # Embedded offline Studio application
├── data/
│   ├── swahili.distilled.jsonl # 19,717 Wiktionary-distilled lemmas
│   ├── terms.clinical.tsv      # Starter clinical dictionary (unreviewed)
│   ├── terms.core.tsv          # Core Swahili vocabulary (unreviewed)
│   └── terms.general.tsv       # Automated vocabulary extraction
└── tests/
    ├── gold100.rs              # 100 gold intent benchmark suite
    └── adversarial.rs          # Adversarial boundary evaluation suite
```

---

## Attribution and Licenses

- **Code**: Massachusetts Institute of Technology (MIT) license (see [LICENSE](LICENSE)).
- **Lexicon Data**: Creative Commons Attribution-ShareAlike 4.0 International (CC BY-SA 4.0) (see [DATA-LICENSE](DATA-LICENSE) and [DATA-SOURCES](DATA-SOURCES)). Distilled from Wiktionary via Kaikki.
- Developed by **Nama Research Lab**.
