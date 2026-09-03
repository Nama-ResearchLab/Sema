//! Back-pass: structured intent to exact Swahili surface verb.
//!
//! This module completes Sema in both directions. The forward stack in
//! this crate takes Swahili apart (lexicon, decomposer, linearizer);
//! the engine used here puts it back together from structured fields.
//! Morphology lives in data tables loaded at runtime; the assembler
//! holds zero hardcoded language rules.
//!
//! Re-exports the engine types so callers need only this module, plus
//! a parts-based constructor that accepts forward-analysis output
//! (subject and tense surfaces, polarity, object infix, root) exactly
//! as `linearizer::decompose_verb` reports it.

pub use sema_tena::synth::{Engine, Intent, Morphology, SynthesisResult};

/// Negated-TAM allomorph surfaces the forward pass emits, mapped to
/// canonical tense tags.
fn negated_tam(surface: &str) -> Option<&'static str> {
    match surface {
        "ku" => Some("PAST"),
        "ja" => Some("PERF"),
        _ => None,
    }
}

/// The shared back-pass engine (embedded Swahili morphology).
pub fn engine() -> Engine {
    Engine::swahili()
}

/// Build an intent from forward-analysis parts. Fused negative subject
/// surfaces resolve through the engine's own data table; negated-TAM
/// allomorphs map to canonical tags; everything else passes through to
/// the engine's normalizers. Unsegmented material (derivations,
/// subjunctive si-) rides inside the root and round-trips verbatim.
pub fn intent_from_parts(
    engine: &Engine,
    subject: Option<&str>,
    tense: Option<&str>,
    negated: bool,
    object: Option<&str>,
    root: &str,
) -> Intent {
    let subject = subject.map(|s| {
        engine
            .morphology()
            .subjects_neg
            .get(s)
            .cloned()
            .unwrap_or_else(|| s.to_string())
    });
    let tense = tense.map(|t| {
        if negated {
            negated_tam(t).unwrap_or(t).to_string()
        } else {
            t.to_string()
        }
    });
    Intent {
        subject,
        tense,
        negation: negated,
        object: object.map(str::to_string),
        root: root.to_string(),
        derivations: Vec::new(),
        mood: "a".to_string(),
        particles: Vec::new(),
    }
}

/// Decompose a verb with the forward analyzer, re-synthesize it here,
/// and report whether the surface round-trips exactly.
pub fn roundtrip(engine: &Engine, verb: &str) -> Roundtrip {
    let m = crate::linearizer::decompose_verb(verb);
    let intent = intent_from_parts(
        engine,
        m.subject.as_deref(),
        m.tense.as_deref(),
        m.negation.is_some(),
        m.object.as_deref(),
        &m.root,
    );
    let out = engine.synthesize(&intent);
    let produced = out.surface.clone();
    Roundtrip {
        verb: verb.to_string(),
        exact: produced.trim() == verb.trim(),
        produced,
        subject_class: out.subject_class,
        object_class: out.object_class,
    }
}

/// Outcome of one forward/backward round-trip check.
#[derive(Debug, Clone)]
pub struct Roundtrip {
    pub verb: String,
    pub produced: String,
    pub exact: bool,
    pub subject_class: Option<String>,
    pub object_class: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn synthesis_is_exact() {
        let e = engine();
        let get = |s: Option<&str>, t: Option<&str>, n: bool, r: &str| {
            e.synthesize(&intent_from_parts(&e, s, t, n, None, r))
                .surface
        };
        assert_eq!(get(Some("1SG"), Some("PAST"), true, "fanya"), "sikufanya");
        assert_eq!(get(Some("3SG"), Some("PRES"), false, "la"), "anakula");
        assert_eq!(get(Some("3PL"), Some("FUT"), true, "jibu"), "hawatajibu");
    }

    #[test]
    fn roundtrip_holds_on_known_verbs() {
        let e = engine();
        for verb in ["sikufanya", "anakula", "hawatakujibu", "hajibu", "usijali"] {
            let r = roundtrip(&e, verb);
            assert!(r.exact, "{verb} produced {:?}", r.produced);
        }
    }

    #[test]
    fn concord_classes_reported() {
        let e = engine();
        let r = roundtrip(&e, "kilivunja");
        assert!(r.exact);
        assert_eq!(r.subject_class.as_deref(), Some("7 (ki/vi)"));
    }
}
