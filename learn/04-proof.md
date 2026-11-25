# Lesson 4: Measured Proof (v0.2.5)

Goal: understand our verification standards and how each benchmark is measured. Every benchmark is reproducible offline on an ordinary central processing unit (CPU).

## 1. Automated Suite Summary (`cargo test`)

Run all unit, integration, and documentation tests across the crate:

```bash
cargo test
```

Measured totals as of `v0.2.5`: **96 tests passing, 0 failing** across library unit tests (58), `gold100` (2), `adversarial` (1), `synth` (16 passing, 1 optional preview), `v25_grammar` (8), `v25_concord` (5), `v25_guardian` (3), `v25_morph` (3), and crate doctest (1).

## 2. Pure-Rust Morphological Round-Trip (500 / 500)

The `Decomposer` peels prefixes, relative infixes (`-ye-`, `-cho-`, `-taka-`), conditionals (`-sipo-`, `-nge-`), object infixes, and derivational suffixes using only `src/synth/sw.toml` tables, then re-synthesizes every intent to prove exact inversion:

```bash
cargo test --test v25_morph
```

Result: **500 / 500 (100.0%)** across all `EN_TO_SW` gold verbs in `bench/Sema_Benchmark_1000.csv`, plus 28 complex relative, short-stem, conditional, and stacked-derivation forms.

## 3. Hardened Clinical Fact Guardian (0 / 24 Leaks)

In `v0.2.0`, basic exact matching leaked 13 out of 24 adversarial boundary cases (54.2%), missing Swahili number words (`mia tano`), misspelled generics (`paracetemol`), and unwritten brands (`Panadol`). In `v0.2.5`:

```bash
cargo test --test adversarial
```

Result: **0 / 24 unshielded leaks (0.0% leak rate)** across all 8 boundary categories (Swahili number words, hyphen/word ranges, decimals with `.` and `,`, fractions, brand names, Damerau-Levenshtein drug typos, mixed units, and spaced digits).

## 4. Non-Verbal Concord & Clause Composition Proof

```bash
cargo test --test v25_concord
cargo test --test v25_grammar
```

Verifies all 9 non-verbal concord families (copulas, locative copulas `yuko`/`hayuko`, existential `-na`, associatives, possessives, demonstratives, adjectives, numerals, `-ni` locatives), strict `Intent` validation, relative future `-taka-`, and deterministic clause rendering (`mtoto wangu yuko hospitalini`).

## 5. Realization Latency & Honest Scope

On an ARM Cortex-A76 processor (4 cores at 2.4 GHz, Linux 6.6) and x86_64 workstation cores (3.2 GHz), pure intent synthesis (`Engine::synthesize`) runs in approximately **0.78 to 1.06 microseconds per verb**.

**Methodology & Limitations Scope:** Gold benchmarks and round-trip invariants prove deterministic agreement between Sema's tables and rules. Two-verb compound auxiliaries (e.g., `alikuwa akisoma`) and `amba-` independent relatives are deferred to `v0.3`, and clinical deployments always require sign-off by qualified Swahili speakers and clinicians.

Back to the [course index](README.md). Next step: [Lesson 5 (Practical Usage)](05-use-it.md) and [Lesson 6 (Concords, Relatives & Clauses)](06-concord-and-composition.md).
