# Lesson 5: Practical Usage (`v0.2.5`)

Goal: explore hands-on workflows with `v0.2.5`: command line utilities, the Rust library API, and the tactile Studio interface.

## 1. Command Line Tools

Synthesize a Swahili verb with relative clauses, conditionals, or strict slot validation:

```bash
# Relative clause with future -taka- allomorph -> watakaojibu
cargo run --bin sema -- synth --subj 3PL --tense FUT --rel C2 --root jibu

# Non-verbal concord & deterministic clause composition
cargo run --bin sema -- concord --kind adj --class KI --stem dogo
cargo run --bin sema -- compose "{t:child} {poss:@child|1SG} {loc_cop:@child|ko} {loc:@hospital}"

# Pure-Rust morphological decomposition + round-trip check
cargo run --bin sema -- roundtrip waliowaua

# Zero-leak clinical & Swahili number shielding
cargo run --bin sema -- terms "Meza paracetemol mia tano mg kila asubuhi"
```

Run all verification suites:

```bash
cargo test
```

## 2. Interactive Studio and Model Context Protocol (MCP) Server

Launch the native local interface (`http://127.0.0.1:4242`) or run MCP over stdio:

```bash
cargo run --release --bin sema-gui
cargo run --release --bin sema -- mcp
```

## 3. Rust Library Recipes (`v0.2.5`)

```rust
use sema::{Composer, ConcordEngine, Decomposer, FactGuardian, RestoreMode, SynthEngine, SynthIntent};

// 1. Verb synthesis with relative infix
let engine = SynthEngine::swahili();
let mut intent = SynthIntent::simple("3PL", "FUT", "jibu");
intent.relative = Some("C2".into());
assert_eq!(engine.synthesize(&intent).surface, "watakaojibu");

// 2. Concords + clause composer
assert_eq!(ConcordEngine::swahili().adjective("KI", "dogo")?, "kidogo");
let clause = Composer::swahili()
    .render("{t:child} {poss:@child|1SG} {loc_cop:@child|ko} {loc:@hospital}")?;
assert_eq!(clause.surface, "mtoto wangu yuko hospitalini");

// 3. Round-trip verification & FactGuardian
assert!(Decomposer::swahili().verify_roundtrip("waliowaua").ok);
let g = FactGuardian::default_clinical();
let prot = g.protect("Give paracetemol mia tano mg to the child");
let res = g.restore(&prot.text, &prot, RestoreMode::Swahili);
assert!(res.integrity_ok);
```

## 4. Attribution and Data Stewardship

Code is released under the MIT License (`LICENSE`). The distilled lexicon originates from Wiktionary via `kaikki.org` under Creative Commons Attribution-ShareAlike 4.0 (`CC BY-SA 4.0`).

Back to the [course index](README.md). Continue to [Lesson 6 (Concords, Relatives & Clauses)](06-concord-and-composition.md).
