//! Structured synthesis intent for the back-pass (English meaning -> Swahili).

use serde::{Deserialize, Serialize};

/// A structured payload describing the Swahili verb to build, in the
/// language-agnostic slot vocabulary understood by the engine.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct Intent {
    /// Canonical subject tag, e.g. "1SG", "3PL", or a noun-class tag.
    pub subject: Option<String>,
    /// Canonical tense tag, e.g. "PRES", "PAST", "FUT", "PERF", "SUBJ", "COND".
    pub tense: Option<String>,
    /// Polarity.
    pub negation: bool,
    /// Optional relative concord or marker in Slot 2b, e.g. "C1", "C7", "ye", "cho", "po".
    #[serde(default)]
    pub relative: Option<String>,
    /// Object infix surface, e.g. "ki", "m", "ku", "wa".
    pub object: Option<String>,
    /// Verb root / stem, e.g. "soma", "la", "jibu".
    pub root: String,
    /// Ordered derivational extension TAGS (e.g. "passive", "applicative",
    /// "causative", "reciprocal", "stative"), resolved to surface suffixes
    /// via the `[derivations]` data table.
    #[serde(default)]
    pub derivations: Vec<String>,
    /// Mood tag: "a" (indicative), "e" (subjunctive), "o" (hortative),
    /// "i" (negative shift), "eni" (plural imperative).
    #[serde(default = "default_mood")]
    pub mood: String,
    /// Optional leading standalone words (e.g. "hapana").
    #[serde(default)]
    pub particles: Vec<String>,
}

fn default_mood() -> String {
    "a".to_string()
}

impl Intent {
    /// Build a simple positive verb intent.
    pub fn simple(subj: &str, tense: &str, root: &str) -> Self {
        Self {
            subject: Some(subj.to_string()),
            tense: Some(tense.to_string()),
            negation: false,
            relative: None,
            object: None,
            root: root.to_string(),
            derivations: Vec::new(),
            mood: "a".to_string(),
            particles: Vec::new(),
        }
    }

    /// Build a negated verb intent.
    pub fn negated(subj: &str, tense: &str, root: &str) -> Self {
        Self {
            subject: Some(subj.to_string()),
            tense: Some(tense.to_string()),
            negation: true,
            relative: None,
            object: None,
            root: root.to_string(),
            derivations: Vec::new(),
            mood: "a".to_string(),
            particles: Vec::new(),
        }
    }

    /// Build a bare singular imperative intent (e.g. "soma", "nenda").
    pub fn imperative(root: &str) -> Self {
        Self {
            subject: None,
            tense: None,
            negation: false,
            relative: None,
            object: None,
            root: root.to_string(),
            derivations: Vec::new(),
            mood: "a".to_string(),
            particles: Vec::new(),
        }
    }

    /// Build a plural imperative intent (e.g. "someni", "nendeni").
    pub fn imperative_plural(root: &str) -> Self {
        Self {
            subject: None,
            tense: None,
            negation: false,
            relative: None,
            object: None,
            root: root.to_string(),
            derivations: Vec::new(),
            mood: "eni".to_string(),
            particles: Vec::new(),
        }
    }

    /// Build an infinitive intent (e.g. "kusoma", "kumuua").
    pub fn infinitive(root: &str) -> Self {
        Self {
            subject: None,
            tense: Some("INF".to_string()),
            negation: false,
            relative: None,
            object: None,
            root: root.to_string(),
            derivations: Vec::new(),
            mood: "a".to_string(),
            particles: Vec::new(),
        }
    }

    pub fn with_object(mut self, obj: &str) -> Self {
        self.object = Some(obj.to_string());
        self
    }

    pub fn with_relative(mut self, rel: &str) -> Self {
        self.relative = Some(rel.to_string());
        self
    }

    pub fn with_derivations<I, S>(mut self, derivs: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.derivations = derivs.into_iter().map(Into::into).collect();
        self
    }

    pub fn with_mood(mut self, mood: &str) -> Self {
        self.mood = mood.to_string();
        self
    }

    pub fn with_negation(mut self, neg: bool) -> Self {
        self.negation = neg;
        self
    }
}
