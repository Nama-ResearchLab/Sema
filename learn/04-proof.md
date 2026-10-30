# Lesson 4: Measured Proof

Goal: understand our verification standards and how each benchmark is measured. Every benchmark is reproducible offline on an ordinary central processing unit (CPU).

## 1. Gold Intent Benchmark (100 out of 100 Matching)

The gold benchmark suite validates 100 Swahili intent combinations against surface forms. It exercises agreement, irregular monosyllabic verbs, glide transformations, and multiple derivation stacks.

```bash
cargo test --test gold100
```

Result: 100 out of 100 matching character outputs. This verifies that slot compositions honor internal Swahili phonological rules.

**Provenance and Scope:** These 100 benchmark intents were authored by the engine developer to verify rule coverage. They have not been audited or verified by an independent native speaker or medical professional.

## 2. Derivational Stretch Suite (10 out of 10 Matching)

This suite evaluates verbal extensions:
- Causative: pigisha, somesha
- Passive: pigwa, somwa
- Applicative: pigia, somea (with automated vowel harmony)
- Reciprocal: pigana, somana
- Stative: pigika, someka

Result: 10 out of 10 derivations match expected surface forms with dynamic vowel harmony.

## 3. Realization Latency and Hardware Profile

In offline edge environments, speed directly translates to battery life and responsiveness. On an ARM Cortex-A76 processor (4 cores at 2.4 GHz, Linux 6.6, rustc 1.99.1), Sema achieves an average synthesis latency of 1.06 microseconds per verb in release mode (and approximately 0.78 microseconds on 3.2 GHz workstation cores).

**Measurement Scope:** This latency figure measures `synthesize()` only on pre-constructed intent structs. It does not measure free-text parsing or lexicon search. Realization dynamically allocates heap memory for the resulting surface string and slot vector records.

## 4. Negation Consistency Gating

Sema passes synthesized output through the forward analyzer to verify that polarity remains consistent between intent and surface text. Across the 100 gold intent test cases and synthetic round-trip sweeps, 0 negation inversions are observed.

**Methodology Scope:** This check is an internal round-trip self-consistency test comparing the engine's backward synthesizer against its own forward analyzer, rather than an independent clinical or native-speaker accuracy validation.

Back to the [course index](README.md). Next step: [Lesson 5 (Practical Usage)](05-use-it.md).
