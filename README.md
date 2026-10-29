<div align="center">

# Sema

**Offline Swahili Language Studio & Deterministic Engine**

*Sub-microsecond morphological synthesis, zero-hallucination fact slots, and forward-pass verification for sovereign Swahili AI.*

[![license](https://img.shields.io/badge/code-MIT-blue.svg)](LICENSE)
[![data license](https://img.shields.io/badge/data-CC%20BY--SA%204.0-orange.svg)](DATA-LICENSE)
[![language](https://img.shields.io/badge/rust-2024%20edition-DEA584.svg)](https://rustup.rs)
[![version](https://img.shields.io/badge/version-v0.2.0-0D9488.svg)](Cargo.toml)
[![tests](https://img.shields.io/badge/tests-61%2F61%20green-059669.svg)](tests/)
[![gold suite](https://img.shields.io/badge/gold%20bench-100%2F100%20passed-059669.svg)](tests/gold100.rs)
[![gui](https://img.shields.io/badge/studio-claymorphic%20gui%20%2B%20mcp-7C3AED.svg)](src/bin/sema_gui.rs)

[Web Studio & Downloads](https://nama-researchlab.github.io/Sema/) | [Documentation](https://nama-researchlab.github.io/Sema/) | [Mini Course](learn/)

</div>

---

## The Swahili-First Paradigm (v0.2.0)

When models are trained on African language corpora or prompted in Swahili, they do not need an English translation bridge. Translating to English loses nuance, introduces cultural distortion, and requires round-trip cloud latency.

Instead of acting as a translation bridge, **Sema** acts as the **deterministic checker, compiler, and fixer**:

```
[User writes Swahili]
         │
         ▼
[Swahili-Native Model]  ──>  Reasons directly in Swahili (zero English step)
         │                   Emits structured slot tuples and exact fact tokens:
         │                     - Morpheme slots: (SUBJ: 1PL, TNS: FUT, ROOT: la)
         │                     - Fact protection IDs: [#T1] (e.g. "paracetamol 500 mg")
         ▼
[Sema Linearizer]       ──>  Deterministic synthesis in < 1 µs (0.8 µs benchmark)
         │                   Applies phonological epenthesis ("tutakula"), vowel harmony,
         │                   and exact dictionary fact replacements.
         ▼
[Sema Verification]     ──>  Forward-pass readback check:
                             Decomposes its own output. If negation flipped or dosage
                             drifted, flags the response before the user ever sees it.
         │
         ▼
[Verified Swahili Output]
```

---

## What's in the Box

| Feature | Description |
|---|---|
| **19,717 Distilled Lemmas** | High-coverage Swahili dictionary distilled from Wiktionary with IPA, POS, and glosses |
| **Sub-microsecond Engine** | 0.8 µs per derivation; 187,000x faster than LLM token generation |
| **Double-Pass Verification** | Forward deconstruction validates backward synthesis; zero negation inversion |
| **Monosyllabic Epenthesis** | Automated `-ku-` infix handling for monosyllabic roots (`-la` -> `tutakula`, `kula`, `hatutakula`) |
| **Vowel Harmony Engine** | Stative and applicative suffixes (`-ea` vs `-ia`, `-esha` vs `-isha`) computed dynamically |
| **Noun Class Agreement** | Full agreement engine mapping concords `C1` through `C18` across prefixes and subject markers |
| **Clinical Fact Guardian** | `src/terms.rs`: locks drug names, dosages, and numbers behind opaque `[#T1]`, `[#N1]` IDs |
| **Single-Executable GUI** | `sema-gui`: claymorphic Apple HIG interface, dotted matrix graph DAG, and model tester |
| **Model Context Protocol** | Native MCP server (`sema-gui --mcp`) for Claude Desktop, Cursor, and Antigravity |
| **Zero Private Dependencies** | Self-contained, pure Rust, 100% offline, zero cloud calls, zero telemetry |

---

## Measured Performance & Proofs

All benchmarks are verified in automated test suites:

- **100/100 Gold Intents Verified**: `cargo test --test gold100` executes 100 benchmark intents with 100% exact surface matches.
- **10/10 Derivational Stretches**: Complete coverage across applicative, causative, reciprocal, stative, and passive derivations.
- **26/26 Negation Gate**: Strict invariant testing ensuring negated inputs never lose the `NOT` semantic token.
- **Sub-Microsecond Latency**: Average linearization time of 0.8 microseconds per word.
- **61/61 Total Tests Passing**: 58 library unit tests, 1 doc-test, and 2 gold integration test suites.

---

## Quickstart

### 1. Build and Test

```bash
git clone https://github.com/Nama-ResearchLab/Sema
cd Sema
cargo test
```

### 2. Launch Sema Studio (Claymorphic GUI)

Sema includes a standalone single-executable GUI with an Apple HIG claymorphic design, interactive morpheme relation graph DAG, and live pipeline testbed:

```bash
# Launch interactive Studio (binds http://127.0.0.1:4242 and auto-opens browser)
cargo run --release --bin sema-gui
```

### 3. Agent Tool Server (MCP)

Run Sema as a standard Model Context Protocol (MCP) server for Claude Desktop, Cursor, or Antigravity:

```bash
cargo run --release --bin sema-gui -- --mcp
```

Add to your `claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "sema": {
      "command": "sema-gui",
      "args": ["--mcp"]
    }
  }
}
```

### 4. CLI Utilities (`sema_cc`)

Decompose a Swahili verb into morpheme slots:

```bash
cargo run --bin sema_cc -- morph --word hawatakujibu
# {"negation":"ha","object":"ku","root":"jibu","subject":"wa","tense":"ta",...}
```

Synthesize an exact Swahili verb from slot intent:

```bash
cargo run --bin sema_cc -- synth --subj 3PL --tense FUT --root jibu --obj ku --neg
# {"surface":"hawatakujibu",...}
```

Protect clinical terms and dosages:

```bash
cargo run --bin sema_cc -- terms \
  --dict data/terms.clinical.tsv,data/terms.core.tsv \
  --text "Mpe mgonjwa paracetamol 500 mg mara mbili kwa siku"
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
    // 1. Load distilled 19k lexicon
    let lex = Lexicon::load("data/swahili.distilled.jsonl")?;
    println!("Lemma: {:?}", lex.skeleton_for("unasema"));

    // 2. Decompose verb to morpheme parts
    let parts = linearizer::decompose_verb("sikufanya");
    println!("Subject: {:?}, Tense: {:?}, Neg: {}", parts.subject, parts.tense, parts.negated);

    // 3. Synthesize deterministically back to surface form
    let engine = backpass::engine();
    let intent = backpass::intent_from_parts(&engine, Some("ni"), Some("ku"), true, None, "fanya");
    let result = engine.synthesize(&intent);
    assert_eq!(result.surface, "sikufanya");

    println!("Verified surface: {}", result.surface);
    Ok(())
}
```

---

## Clinical Safety Invariant: Term & ID Protection

In healthcare, language model hallucinations can be fatal:
- Dropping a negation converts "do not give" (`usinywe`) to "take" (`kunywa`).
- Number or unit drift converts `500 mg` to `50 mg`.
- Drug transliteration errors corrupt compound names.

Sema prevents this via deterministic tokenization (`src/terms.rs`):

```
Clinical Prompt: "Mpe mtoto vidonge viwili vya paracetamol 500 mg mara mbili kwa siku. Usimpe aspirin."
  │
  ▼ [protect] (matches data/terms.clinical.tsv)
"Mpe mtoto [#T1] vya [#T2] [#N1] [#N2] mara mbili kwa siku. Usimpe [#T3]."
  │
  ▼ [Model reasons in Swahili and generates structured recommendation]
  │
  ▼ [Deterministic restore with noun-class verification]
"Mpe mtoto vidonge viwili vya paracetamol 500 mg mara mbili kwa siku. Usimpe aspirin."
  │
  ▼ [Forward-Pass Verification Check]
Status: PASSED (Exact fact preservation, zero negation drift)
```

---

## Repository Structure

```
Sema/
├── Cargo.toml                  # Standalone v0.2.0 crate definition
├── install.sh                  # One-line universal installer
├── index.html                  # GitHub Pages web portal & interactive studio
├── docs/                       # Mirror documentation for GitHub Pages deployment
│   ├── index.html
│   └── assets/
├── assets/                     # Media & vector assets
├── src/
│   ├── lib.rs                  # Public crate exports
│   ├── affix/                  # TOML grammar definitions (sw.toml)
│   ├── backpass/               # Deterministic synthesis engine & vowel harmony
│   ├── linearizer.rs           # Surface deconstruction & slot extraction
│   ├── terms.rs                # Clinical term & numeric ID guardian
│   ├── racer.rs                # Cascade resolution engine
│   ├── rolodex.rs              # Intent serialization & morph parsing
│   ├── bin/
│   │   ├── sema_cc.rs          # Unified CLI utility
│   │   └── sema_gui.rs         # Claymorphic Studio GUI & MCP server
│   └── gui/
│       └── app.html            # Embedded offline Studio application
├── data/
│   ├── swahili.distilled.jsonl # 19,717 Wiktionary-distilled lemmas
│   ├── terms.clinical.tsv      # Standardized clinical dictionary
│   └── terms.core.tsv          # Core Swahili medical vocabulary
└── tests/
    ├── gold100.rs              # 100/100 gold intent benchmark suite
    └── gold_stretch.rs         # 10/10 derivational stretch suite
```

---

## Companion Repositories

- **[sema-kesho](https://github.com/Nama-ResearchLab/sema-kesho)**: Internal production integration and experimental laboratory. Hosts neural-vs-symbolic eval harnesses, dictionary ingest pipelines, and edge model adapters.

---

## Attribution & Licenses

- **Code**: MIT License (see [LICENSE](LICENSE)).
- **Lexicon Data**: CC BY-SA 4.0 (see [DATA-LICENSE](DATA-LICENSE)). Distilled from [Wiktionary](https://en.wiktionary.org) via [kaikki.org](https://kaikki.org).
- Developed by **Nama Research Lab** as part of sovereign edge AI infrastructure.
