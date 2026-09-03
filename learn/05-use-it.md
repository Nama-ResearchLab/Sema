# Lesson 5: use it

Goal: run everything yourself. Three levels: commands, library code,
and the desktop app.

## 1. Command line

Resolve one word (forward):

```bash
cargo run --bin sema_cc -- morph --word hawatakujibu
# {"negation":"ha","object":"ku","root":"jibu","subject":"wa","tense":"ta",...}
```

Benchmark forward coverage over the corpus:

```bash
cargo run --release --bin sema_cc -- bench --lex data/swahili.distilled.jsonl \
  --csv bench/Sema_Benchmark_1000.csv --limit 200
```

Run the bundled examples (role tagger, render layer):

```bash
cargo run --example role_tagger
cargo run --example render
```

Run the test suite (37 tests plus doctest):

```bash
cargo test
```

## 2. Library recipes

Anchor a sentence (resolve each word):

```rust
use sema::Lexicon;

let lex = Lexicon::load("data/swahili.distilled.jsonl")?;
for tok in sema::lexicon::segment_words("umefikia wapi?") {
    if let Some(sk) = lex.skeleton_for(&tok) {
        println!("{} -> {} ({})", sk.surface, sk.lemma, sk.pos);
    }
}
```

Decompose one verb into parts:

```rust
use sema::linearizer::decompose_verb;

let m = decompose_verb("sikufanya");
// subject "ni", tense "ku", negation present, root "fanya"
```

Synthesize from parts (companion engine):

```rust
// subject 1SG, tense PAST, negated, root "fanya" -> "sikufanya"
```

Round-trip check in Python (uses both binaries, read-only):

```bash
python tools/roundtrip.py --limit 500
```

## 3. Inside the desktop app (Jericho)

Jericho embeds the lexicon for two jobs: anchoring chat input and
lemmatizing the RAG index (so "umefikia" and "wamefika" count as the
same root when retrieving). The local model reasons in English over
the anchors. Structured answers render back to Swahili through the
back-pass engine, which is being wired in as the render layer. You
can choose any local Ollama model; rolodex features toggle in config.
Interface polish is still to come.

## 4. Data, licensing, and rebuilding

Code is MIT (LICENSE). The lexicon data is CC BY-SA 4.0, distilled
from Wiktionary via kaikki.org (see DATA-LICENSE and data/NOTICE.md):
keep the credit and share-alike terms if you ship the data file.
Rebuild it from raw dumps with `sema-distill`. The affix table is
plain TOML: new languages mean one data file plus one table.

Back to the [course index](README.md). Deeper background: [references](refs.md).
