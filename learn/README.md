# Learn Sema: a mini-course from zero (`v0.2.5`)

No background needed. Each lesson defines every new word when it first appears, shows small examples, then lets you run the real deterministic Rust engine (`v0.2.5`). Interactive HTML pages need only a browser, zero network.

![Nama Research Lab](assets/nama-logo.png)

## How to use this course

Read a lesson, then run its commands from the repository root (`cargo run --bin sema -- ...` or `cargo test`). Interactive pages work completely offline.

## The course

1. [First principles](01-first-principles.md): the problem, the core idea, and key terms.
2. [Forward: anchoring](02-forward-anchor.md): turning Swahili words into English meaning skeletons.
3. [Back-pass: rendering](03-backpass-render.md): turning structured meaning back into exact Swahili verbs.
4. [Proof, not promises](04-proof.md): `96` tests, `500/500` pure-Rust round-trips, `0/24` FactGuardian leaks, and sub-microsecond benchmarks.
5. [Use it](05-use-it.md): CLI, Rust library recipes, local Studio GUI, and MCP server.
6. [Relatives, Concords & Clauses](06-concord-and-composition.md): `v0.2.5` relative `-taka-` allomorphy, negative conditionals `-sipo-`, 18-class `ConcordEngine`, `Composer`, and typo-safe `FactGuardian`.
7. [References](refs.md): data provenance and further reading.

Interactive pages: [course hub](index.html) (guided build + composer + 6-question quiz), [verb & concord explorer](verbs.html), and [glossary](glossary.html).
