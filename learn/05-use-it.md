# Lesson 5: Practical Usage

Goal: explore hands-on workflows with Sema. We cover three paths: command line utilities, Rust library integration, and the tactile Studio interface.

## 1. Command Line Tools

Deconstruct a verb to inspect its morpheme slots:

```bash
cargo run --bin sema_cc -- morph --word hawatakujibu
# {"negation":"ha","object":"ku","root":"jibu","subject":"wa","tense":"ta",...}
```

Synthesize an exact Swahili verb from intent slots:

```bash
cargo run --bin sema_cc -- synth --subj 3PL --tense FUT --root jibu --obj ku --neg
# hawatakujibu
```

Protect clinical facts, dosages, and drug names:

```bash
cargo run --bin sema_cc -- terms \
  --dict data/terms.clinical.tsv,data/terms.core.tsv \
  --text "Mpe mtoto vidonge viwili vya paracetamol 500 mg mara mbili kwa siku"
```

Run the complete verification test suite (61 tests plus gold 100 benchmarks):

```bash
cargo test
```

## 2. Interactive Studio and Model Context Protocol (MCP) Server

Launch the native claymorphic interface directly on your local system:

```bash
cargo run --release --bin sema-gui
```

To connect Sema directly to local desktop agent environments and tool hosts, start the built-in Model Context Protocol (MCP) server:

```bash
cargo run --release --bin sema-gui -- --mcp
```

## 3. Library Recipes

Resolve words in a sentence to dictionary lemmas and parts of speech:

```rust
use sema::Lexicon;

let lex = Lexicon::load("data/swahili.distilled.jsonl")?;
for tok in sema::lexicon::segment_words("umefikia wapi?") {
    if let Some(sk) = lex.skeleton_for(&tok) {
        println!("{} -> {} ({})", sk.surface, sk.lemma, sk.pos);
    }
}
```

Decompose a surface verb into structured morphological parts:

```rust
use sema::linearizer::decompose_verb;

let m = decompose_verb("sikufanya");
// subject "ni", tense "ku", polarity negative, root "fanya"
```

Shield clinical terms before sending prompts to edge models:

```rust
use sema::terms::TermReplacer;

let replacer = TermReplacer::from_files(&["data/terms.clinical.tsv"])?;
let protected = replacer.protect("Mpe mgonjwa paracetamol 500 mg");
println!("Shielded: {}", protected.text);
```

## 4. Attribution and Data Stewardship

Code is released under the MIT License (see LICENSE). The distilled lexicon originates from Wiktionary through kaikki.org and is shared under Creative Commons Attribution-ShareAlike 4.0 (CC BY-SA 4.0). You are warmly invited to extend the dictionary or build new affix tables for sister Bantu languages.

Back to the [course index](README.md). Deeper background: [references](refs.md).
