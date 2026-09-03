# Lesson 4: proof, not promises

Goal: know exactly what has been measured, what each number means,
and how to reproduce it. Every claim here reruns with the command
shown, on an ordinary CPU, offline.

## 1. Back-pass benchmark: 500/500 exact

A 1,000-row gold corpus pairs English prompts with intents and gold
verbs. The engine must rebuild each verb character-for-character.

    sema_cc bench --csv bench/Sema_Benchmark_1000.csv --limit 1000

Result: 500 of 500 back-pass rows exact, about 2.4 microseconds per
synthesis (release build). What it proves: the tables cover the
template corpus completely. What it does not prove: words outside
the corpus. That is the next harness's job.

## 2. Torture sweep: 45,970,215 cases at 100 percent

The torture command crosses every subject, tense, polarity, object,
mood, and derivation over 6,555 real dictionary verb roots (actual
headwords from merged dictionaries, not generated templates) and
checks four things per case: deterministic output, non-empty, no
whitespace, correct agreement marker from the data tables.

    sema_cc torture --verbs data/dictionary/verbs.en-sw.jsonl

Result: all cases pass, about 1.3 microseconds each, zero failures.
What it proves: no panics, no malformed output, no agreement slip at
scale. This harness once caught a real bug (a hardcoded alias
shadowing a data tag) at 90 percent before the fix and 100 after,
which is the point of running it.

## 3. Forward/backward round-trip: 500/500 plus 14/14

Decompose each gold verb with the forward analyzer, map the parts to
an intent, re-synthesize, compare exactly. Covers subjunctives,
glides, derivations, and noun-class subjects.

    python tools/roundtrip.py --limit 500

Two small mapping rules bridge analyzer output to intent input
(negated past "ku" to PAST, "ja" to PERF); everything else passes
through. Unsegmented material (derivations, subjunctive si-) rides
inside the root and round-trips verbatim, recorded as known behavior.

## 4. Neural comparison: 0/15 against 15/15

Two small offline models (0.5 and 1.5 billion parameters) translate
the same verb phrases through the local Ollama server. Scoring uses
a fixed error taxonomy: exact, backend failure, empty,
no-verbal-morphology (output has no subject concord at all),
polarity-flip (negation disagrees with gold), form-wrong (concord
present, form still wrong).

    python tools/neural_eval.py --models qwen2.5:0.5b,qwen2.5:1.5b --limit 30

Result: both models 0 exact out of 15 unique prompts; the symbolic
engine 15 of 15. The 0.5B model mostly stutters syllables and echoes
English; the 1.5B model guesses frequent Swahili words and flips
polarity. Repeats return byte-identical output (temperature 0.0), so
the failure patterns are stable, not sampling noise. Needs `ollama
serve` running; fully offline once models are present.

## 5. Stated limitation

The 100 percent figures are self-consistency results: the engine
against its own tables, forward against backward over the same slot
theory. Real dictionary roots (not templates) limit circularity, but
no independent native-speaker audit of several hundred outputs has
been performed. Treat the numbers as strong evidence with one known
gap, not as a final certificate.

Next: [Lesson 5: use it](05-use-it.md).
