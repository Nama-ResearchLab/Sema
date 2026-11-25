//! Sema, pure data-driven Swahili verb synthesis.
//!
//! This is the back-pass of the Sema workflow: a structured [`Intent`]
//! (what the user means) is compiled into a grammatically correct Swahili
//! surface verb. The engine contains **zero hardcoded Swahili**, every
//! prefix, tense marker, mood rule, relative infix, imperative form,
//! short-stem, and phonological glide is loaded at runtime from `sw.toml`
//! (see the `MORPHOLOGY` embedded table and [`Engine::from_morphology`]).
//!
//! Slot template: `[NEG][SUBJ][TENSE][REL][OBJ][ROOT][DERIV][MOOD]`.

pub mod intent;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub use intent::Intent;

/// The Swahili morphology data, embedded at compile time so the binary runs
/// standalone. Swap in a different language by replacing this file / loader.
pub const MORPHOLOGY: &str = include_str!("sw.toml");

/// Fully-deserialized morphology tables from `sw.toml`.
#[derive(Debug, Clone, Deserialize)]
pub struct Morphology {
    #[serde(default)]
    pub subjects: HashMap<String, String>,
    #[serde(default)]
    pub subjects_neg: HashMap<String, String>,
    #[serde(default)]
    pub subjects_neg_by_tag: HashMap<String, String>,
    /// Standalone negation marker fallback.
    #[serde(default)]
    pub negation: Negation,
    #[serde(default)]
    pub tenses: HashMap<String, String>,
    #[serde(default)]
    pub tense_neg: HashMap<String, String>,
    #[serde(default)]
    pub tense_behavior: TenseBehavior,
    #[serde(default)]
    pub relatives: HashMap<String, String>,
    #[serde(default)]
    pub relative_rules: RelativeRules,
    #[serde(default)]
    pub objects: HashMap<String, String>,
    #[serde(default)]
    pub final_vowel: Vec<FinalVowelRule>,
    #[serde(default)]
    pub defaults: Defaults,
    #[serde(default)]
    pub negative_subjunctive: NegativeSubjunctive,
    #[serde(default)]
    pub imperatives: Imperatives,
    #[serde(default)]
    pub short_stems: ShortStems,
    #[serde(default)]
    pub vowel_glide: VowelGlide,
    #[serde(default)]
    pub borrowed: Borrowed,
    /// Valence-changing derivational extensions: tag -> surface suffix.
    #[serde(default)]
    pub derivations: HashMap<String, String>,
    /// Phonologically-conditioned allomorph variants (first match wins).
    #[serde(default)]
    pub derivation_variants: Vec<DerivationVariant>,
    /// Noun-class concord metadata (tag/infix -> class label).
    #[serde(default)]
    pub noun_classes: NounClasses,
    /// Non-verbal concord and copula tables (used by `ConcordEngine`).
    #[serde(default)]
    pub concord: ConcordTables,
    /// Disambiguation guards for affixes/viambishi and noun-verb homographs.
    #[serde(default)]
    pub morph_guards: MorphGuards,
}

/// Morphological disambiguation guards loaded from `[morph_guards]`.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct MorphGuards {
    #[serde(default)]
    pub blocked_verb_roots: Vec<String>,
    #[serde(default)]
    pub never_decompose_standalone: Vec<String>,
}

/// Tense behavior controls: subjectless tenses (e.g. HAB, INF) and
/// negated tenses that retain positive subject concords (e.g. -sipo-).
#[derive(Debug, Clone, Default, Deserialize)]
pub struct TenseBehavior {
    #[serde(default)]
    pub subjectless: Vec<String>,
    #[serde(default)]
    pub positive_subject_when_neg: Vec<String>,
}

/// Relative clause marker interaction rules (e.g. future -ta- -> -taka-).
#[derive(Debug, Clone, Default, Deserialize)]
pub struct RelativeRules {
    #[serde(default)]
    pub future_marker: String,
    #[serde(default)]
    pub future_rel_marker: String,
}

/// Bare singular, plural, object-bearing, and irregular imperative rules.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct Imperatives {
    #[serde(default)]
    pub plural_mood: String,
    #[serde(default)]
    pub plural_suffix: String,
    #[serde(default)]
    pub plural_vowel: String,
    #[serde(default)]
    pub object_vowel: String,
    #[serde(default)]
    pub irregular_sg: HashMap<String, String>,
    #[serde(default)]
    pub irregular_pl: HashMap<String, String>,
}

/// Non-verbal concord and copula tables loaded from `[concord.*]`.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct ConcordTables {
    #[serde(default)]
    pub copula: CopulaTable,
    #[serde(default)]
    pub locative_subjects: HashMap<String, String>,
    #[serde(default)]
    pub locative_subjects_neg: HashMap<String, String>,
    #[serde(default)]
    pub locative_suffixes: HashMap<String, String>,
    #[serde(default)]
    pub possessive_verb: PossessiveVerbTable,
    #[serde(default)]
    pub associative: HashMap<String, String>,
    #[serde(default)]
    pub possessive_prefix: HashMap<String, String>,
    #[serde(default)]
    pub possessive_stems: HashMap<String, String>,
    #[serde(default)]
    pub demonstrative_prox: HashMap<String, String>,
    #[serde(default)]
    pub demonstrative_dist: HashMap<String, String>,
    #[serde(default)]
    pub demonstrative_ref: HashMap<String, String>,
    #[serde(default)]
    pub adjective_prefix: HashMap<String, String>,
    #[serde(default)]
    pub adjective_prefix_vowel: HashMap<String, String>,
    #[serde(default)]
    pub adjective_overrides: HashMap<String, String>,
    #[serde(default)]
    pub numeral_stems: HashMap<String, String>,
    #[serde(default)]
    pub numeral_invariable: HashMap<String, String>,
    #[serde(default)]
    pub numeral_prefix: HashMap<String, String>,
    #[serde(default)]
    pub numeral_overrides: HashMap<String, String>,
    #[serde(default)]
    pub locative_noun: LocativeNounTable,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct CopulaTable {
    #[serde(default)]
    pub pos: String,
    #[serde(default)]
    pub neg: String,
    #[serde(default)]
    pub emph_pos: String,
    #[serde(default)]
    pub emph_neg: String,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct PossessiveVerbTable {
    #[serde(default)]
    pub marker: String,
    #[serde(default)]
    pub neg_overrides: HashMap<String, String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct LocativeNounTable {
    #[serde(default)]
    pub suffix: String,
}

/// Noun-class agreement labels, keyed by subject tag and object infix.
/// Person concords are absent by design (they agree with persons, not classes).
#[derive(Debug, Clone, Default, Deserialize)]
pub struct NounClasses {
    /// Subject tag -> class label (e.g. KI -> "7 (ki/vi)").
    #[serde(default)]
    pub subjects: HashMap<String, String>,
    /// Object infix -> class label (e.g. ki -> "7 (ki/vi)").
    #[serde(default)]
    pub objects: HashMap<String, String>,
}

/// A single derivational allomorph variant. The engine performs only generic
/// lookups (vowel-set membership, vowel-cluster shape, borrowed status,
/// short-stem list membership); every vowel set and suffix lives in data.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct DerivationVariant {
    /// Which derivation tag this variant belongs to (e.g. "causative").
    #[serde(default)]
    pub derivation: String,
    /// Match only this exact root string (e.g. "pa" -> "ewa").
    #[serde(default)]
    pub stem_exact: String,
    /// Use `suffix` when the root's last vowel is in this set (e.g. "eo").
    #[serde(default)]
    pub trigger_vowels: String,
    /// Use `suffix` only for listed short/monosyllabic stems.
    #[serde(default)]
    pub short_only: bool,
    /// Match only stems ending in two adjacent vowels (e.g. fungua, ondoa).
    #[serde(default)]
    pub vowel_cluster: bool,
    /// Match only borrowed/fixed-vowel stems.
    #[serde(default)]
    pub borrowed_only: bool,
    /// Match when the root ends with this string (e.g. "e", "i", "u").
    #[serde(default)]
    pub ends_with: String,
    /// When true, do not strip the root's final vowel before appending suffix.
    #[serde(default)]
    pub keep_final_vowel: bool,
    /// The variant surface suffix.
    #[serde(default)]
    pub suffix: String,
}

/// Standalone negation marker fallback.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct Negation {
    #[serde(default)]
    pub lead: String,
}

/// A single final-vowel selection rule.
#[derive(Debug, Clone, Deserialize)]
pub struct FinalVowelRule {
    pub step: String,
    #[serde(default)]
    pub mood: String,
    #[serde(default)]
    pub tense: Vec<String>,
    #[serde(default = "default_a")]
    pub use_vowel: String,
}

fn default_a() -> String {
    "a".to_string()
}

/// Defaults applied when no final-vowel rule matches.
#[derive(Debug, Clone, Deserialize)]
pub struct Defaults {
    #[serde(default = "default_a")]
    pub final_vowel: String,
}

impl Default for Defaults {
    fn default() -> Self {
        Self {
            final_vowel: "a".to_string(),
        }
    }
}

/// Negative-subjunctive config: `SUBJ + infix + ROOT + mood_vowel`.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct NegativeSubjunctive {
    #[serde(default)]
    pub infix: String,
    #[serde(default)]
    pub mood: String,
    #[serde(default)]
    pub tense: String,
}

/// Intrusive support morpheme for short/monosyllabic stems.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct ShortStems {
    #[serde(default)]
    pub stems: Vec<String>,
    #[serde(default)]
    pub morpheme: String,
    #[serde(default)]
    pub ku_after_tenses: Vec<String>,
    /// Tenses after which the intrusive morpheme is kept under negation
    /// (e.g. FUT: hatutakula). Other negated tenses skip it.
    #[serde(default)]
    pub ku_after_tenses_neg: Vec<String>,
    #[serde(default)]
    pub skip_negative: bool,
    /// Omit intrusive support morpheme when an object infix precedes the root.
    #[serde(default)]
    pub skip_when_object: bool,
}

/// Vowel-glide allomorphs.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct VowelGlide {
    #[serde(default)]
    pub m_vowel: String,
    #[serde(default)]
    pub m_before_u: String,
}

/// Borrowed/irregular stem config.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct Borrowed {
    #[serde(default)]
    pub fixed_final_vowels: Vec<String>,
}

impl Morphology {
    /// Parse morphology from a TOML string.
    pub fn parse(s: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(s)
    }

    /// Parse morphology from a TOML file path.
    pub fn from_path(p: &std::path::Path) -> std::io::Result<Self> {
        let s = std::fs::read_to_string(p)?;
        Self::parse(&s).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }

    /// The built-in Swahili tables.
    pub fn swahili() -> Self {
        Self::parse(MORPHOLOGY).expect("sw.toml must parse")
    }
}

/// A single assembled slot (for inspection/debugging).
#[derive(Debug, Clone)]
pub struct Slot {
    pub name: String,
    pub value: String,
}

/// Validation failure on a slot in an [`Intent`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ValidationError {
    pub slot: String,
    pub value: String,
    pub message: String,
}

/// The result of synthesis.
#[derive(Debug, Clone)]
pub struct SynthesisResult {
    /// The assembled Swahili surface verb.
    pub surface: String,
    /// Individual slots used, in assembly order.
    pub slots: Vec<(String, String)>,
    /// Noun class the subject concord agrees with (None for persons/unmapped).
    pub subject_class: Option<String>,
    /// Noun class the object infix agrees with (None for persons/unmapped).
    pub object_class: Option<String>,
}

/// The engine: generic assembler over a loaded [`Morphology`].
#[derive(Debug, Clone)]
pub struct Engine {
    m: Morphology,
}

impl Engine {
    /// Build an engine from loaded morphology tables.
    pub fn from_morphology(m: Morphology) -> Self {
        Self { m }
    }

    /// Convenience: the built-in Swahili engine.
    pub fn swahili() -> Self {
        Self::from_morphology(Morphology::swahili())
    }

    /// Read-only access to the loaded morphology tables (used by test harnesses
    /// to drive combinatorial grids straight from the data).
    pub fn morphology(&self) -> &Morphology {
        &self.m
    }

    /// Resolve a relative tag/infix against `[relatives]`.
    pub fn resolve_relative(&self, rel: &str) -> Option<String> {
        self.m
            .relatives
            .get(rel)
            .or_else(|| self.m.relatives.get(&rel.to_uppercase()))
            .or_else(|| self.m.relatives.get(&rel.to_lowercase()))
            .cloned()
    }

    /// Validate all tags and stem invariants on `intent` against the loaded tables.
    pub fn validate(&self, intent: &Intent) -> Result<(), Vec<ValidationError>> {
        let mut errs = Vec::new();
        let root_trim = intent.root.trim();
        if root_trim.is_empty() || root_trim.chars().any(char::is_whitespace) {
            errs.push(ValidationError {
                slot: "root".into(),
                value: intent.root.clone(),
                message: "root must be a non-empty single word".into(),
            });
        }
        if let Some(subj) = &intent.subject {
            let tag = self.tag(subj);
            if !self.m.subjects.contains_key(&tag) {
                errs.push(ValidationError {
                    slot: "subject".into(),
                    value: subj.clone(),
                    message: format!("unknown subject concord '{subj}'"),
                });
            }
        }
        if let Some(tense) = &intent.tense {
            let ttag = self.tense_tag(tense);
            let is_neg_subj_tense = !self.m.negative_subjunctive.tense.is_empty()
                && tense.eq_ignore_ascii_case(&self.m.negative_subjunctive.tense);
            if !is_neg_subj_tense
                && !self.m.tenses.contains_key(&ttag)
                && !self.m.tense_neg.contains_key(&ttag)
            {
                errs.push(ValidationError {
                    slot: "tense".into(),
                    value: tense.clone(),
                    message: format!("unknown tense tag '{tense}'"),
                });
            }
        }
        if let Some(rel) = &intent.relative
            && self.resolve_relative(rel).is_none()
        {
            errs.push(ValidationError {
                slot: "relative".into(),
                value: rel.clone(),
                message: format!("unknown relative marker/class '{rel}'"),
            });
        }
        if let Some(obj) = &intent.object
            && !self.m.objects.contains_key(obj)
        {
            errs.push(ValidationError {
                slot: "object".into(),
                value: obj.clone(),
                message: format!("unknown object infix '{obj}'"),
            });
        }
        for d in &intent.derivations {
            if !self.m.derivations.contains_key(d) {
                errs.push(ValidationError {
                    slot: "derivations".into(),
                    value: d.clone(),
                    message: format!("unknown derivation '{d}'"),
                });
            }
        }
        let valid_mood = intent.mood == self.m.defaults.final_vowel
            || (!self.m.imperatives.plural_mood.is_empty()
                && intent.mood == self.m.imperatives.plural_mood)
            || self
                .m
                .final_vowel
                .iter()
                .any(|r| r.step == "mood" && r.mood == intent.mood);
        if !valid_mood {
            errs.push(ValidationError {
                slot: "mood".into(),
                value: intent.mood.clone(),
                message: format!("unknown mood '{}'", intent.mood),
            });
        }
        if errs.is_empty() {
            Ok(())
        } else {
            Err(errs)
        }
    }

    /// Validate `intent` strictly before synthesis.
    pub fn synthesize_checked(&self, intent: &Intent) -> Result<SynthesisResult, Vec<ValidationError>> {
        self.validate(intent)?;
        Ok(self.synthesize(intent))
    }

    /// Compile an intent into a Swahili surface verb.
    pub fn synthesize(&self, intent: &Intent) -> SynthesisResult {
        let mut slots = Vec::new();
        let mut out = String::new();

        // Lead particles (e.g. "hapana", "ndiyo").
        for p in &intent.particles {
            out.push_str(p);
            out.push(' ');
            slots.push(("particle".into(), p.clone()));
        }

        let imp = &self.m.imperatives;
        let is_bare_imp = intent.subject.is_none()
            && intent.tense.is_none()
            && !intent.negation
            && intent.relative.is_none();
        let is_plural_imp =
            is_bare_imp && !imp.plural_mood.is_empty() && intent.mood == imp.plural_mood;

        // Irregular bare singular/plural imperatives (e.g. enda->nenda, ja->njoo).
        if is_bare_imp && intent.object.is_none() && intent.derivations.is_empty() {
            if is_plural_imp {
                if let Some(form) = imp.irregular_pl.get(&intent.root) {
                    out.push_str(form);
                    slots.push(("imperative_pl".into(), form.clone()));
                    return SynthesisResult {
                        surface: out,
                        slots,
                        subject_class: None,
                        object_class: None,
                    };
                }
            } else if intent.mood == "a"
                && let Some(form) = imp.irregular_sg.get(&intent.root)
            {
                out.push_str(form);
                slots.push(("imperative".into(), form.clone()));
                return SynthesisResult {
                    surface: out,
                    slots,
                    subject_class: None,
                    object_class: None,
                };
            }
        }

        let neg_subj = &self.m.negative_subjunctive;
        let is_neg_subj = intent.negation
            && (!neg_subj.mood.is_empty() && intent.mood == neg_subj.mood
                || !neg_subj.tense.is_empty()
                    && intent
                        .tense
                        .as_deref()
                        .map(|t| t.eq_ignore_ascii_case(&neg_subj.tense))
                        .unwrap_or(false));

        let is_subjectless = !intent.negation
            && intent
                .tense
                .as_deref()
                .map(|t| self.tense_in_list(t, &self.m.tense_behavior.subjectless))
                .unwrap_or(false);

        let pos_subj_when_neg = intent.negation
            && intent
                .tense
                .as_deref()
                .map(|t| self.tense_in_list(t, &self.m.tense_behavior.positive_subject_when_neg))
                .unwrap_or(false);

        // Slot 1: negation + subject (or negative-subjunctive, or subjectless omission).
        if is_subjectless {
            // Subject prefix omitted for habitual/infinitive.
        } else if is_neg_subj || pos_subj_when_neg {
            if let Some(prefix) = intent
                .subject
                .as_ref()
                .and_then(|s| self.m.subjects.get(&self.tag(s)))
            {
                out.push_str(prefix);
                slots.push(("subj".into(), prefix.clone()));
            }
            if is_neg_subj {
                out.push_str(&neg_subj.infix);
                slots.push(("neg_infix".into(), neg_subj.infix.clone()));
            }
        } else if intent.negation {
            if let Some(subj) = &intent.subject {
                let tag = self.tag(subj);
                // Prefer the fused negative subject concord.
                let fused = self
                    .m
                    .subjects_neg_by_tag
                    .get(&tag)
                    .cloned()
                    .or_else(|| {
                        self.m
                            .subjects_neg
                            .iter()
                            .find(|(_, t)| **t == tag)
                            .map(|(k, _)| k.clone())
                    });
                if let Some(f) = fused {
                    out.push_str(&f);
                    slots.push(("neg+subj".into(), f));
                } else if let Some(pos) = self.m.subjects.get(&tag) {
                    // Fall back to a leading negation marker if no fused form.
                    let lead = self.m.negation.lead.clone();
                    out.push_str(&lead);
                    out.push_str(pos);
                    slots.push(("neg".into(), lead));
                    slots.push(("subj".into(), pos.clone()));
                }
            }
        } else if let Some(prefix) = intent
            .subject
            .as_ref()
            .and_then(|s| self.m.subjects.get(&self.tag(s)))
        {
            out.push_str(prefix);
            slots.push(("subj".into(), prefix.clone()));
        }

        let rel_surface = intent
            .relative
            .as_deref()
            .and_then(|r| self.resolve_relative(r));

        // Slot 2: TAM marker (with relative future override when Slot 2b is filled).
        if !is_neg_subj && let Some(tense) = &intent.tense {
            let mut marker = if intent.negation {
                self.negative_tense(tense)
            } else {
                self.m
                    .tenses
                    .get(&self.tense_tag(tense))
                    .cloned()
                    .unwrap_or_else(|| tense.clone())
            };
            if rel_surface.is_some()
                && !self.m.relative_rules.future_marker.is_empty()
                && marker == self.m.relative_rules.future_marker
            {
                marker = self.m.relative_rules.future_rel_marker.clone();
            }
            if !marker.is_empty() {
                out.push_str(&marker);
                slots.push(("tense".into(), marker));
            }
        }

        // Slot 2b: relative concord infix.
        if let Some(rel) = &rel_surface {
            out.push_str(rel);
            slots.push(("rel".into(), rel.clone()));
        }

        // Short-stem intrusive support morpheme (after TAM/REL, skipped if object present).
        let skip_obj = self.m.short_stems.skip_when_object && intent.object.is_some();
        let needs_short = !skip_obj
            && self.m.short_stems.stems.contains(&intent.root)
            && intent
                .tense
                .as_deref()
                .map(|t| self.short_tense(t, intent.negation))
                .unwrap_or(false);
        if needs_short {
            out.push_str(&self.m.short_stems.morpheme);
            slots.push(("short_morph".into(), self.m.short_stems.morpheme.clone()));
        }

        // Slot 3: object infix (with vowel glides before vowel-initial roots).
        if let Some(obj) = &intent.object {
            let first_ch = intent.root.chars().next();
            let root_vowel = matches!(first_ch, Some('a' | 'e' | 'i' | 'o' | 'u'));
            let rendered = if obj == "m" && first_ch == Some('u') && !self.m.vowel_glide.m_before_u.is_empty() {
                self.m.vowel_glide.m_before_u.clone()
            } else if obj == "m" && root_vowel {
                self.m.vowel_glide.m_vowel.clone()
            } else {
                obj.clone()
            };
            out.push_str(&rendered);
            slots.push(("obj".into(), rendered));
        }

        // Slot 4: root.
        let root_final = intent.root.chars().last();
        out.push_str(&intent.root);
        slots.push(("root".into(), intent.root.clone()));

        // Slot 5: derivational extensions (valence-changing suffixes).
        let is_borrowed = matches!(
            root_final,
            Some(c) if self.m.borrowed.fixed_final_vowels.iter().any(|v| v == &c.to_string())
        );
        let root_chars: Vec<char> = intent.root.chars().collect();
        let is_vowel_cluster = root_chars.len() >= 2
            && root_chars
                .iter()
                .rev()
                .take(2)
                .all(|c| "aeiou".contains(*c));
        let mut stem_chars = root_chars;
        if stem_chars
            .last()
            .map(|c| "aeiou".contains(*c))
            .unwrap_or(false)
        {
            stem_chars.pop();
        }
        let root_last_vowel = stem_chars
            .iter()
            .rev()
            .find(|c| "aeiou".contains(**c))
            .copied();

        let mut keep_root_final_vowel = false;
        let derived: Vec<String> = intent
            .derivations
            .iter()
            .enumerate()
            .filter_map(|(idx, tag)| {
                let variant = self.m.derivation_variants.iter().find(|v| {
                    if v.derivation != *tag || v.suffix.is_empty() {
                        return false;
                    }
                    // Root-shape conditions apply only to the first derivation on the stem.
                    if idx > 0
                        && (!v.stem_exact.is_empty()
                            || v.short_only
                            || v.borrowed_only
                            || !v.ends_with.is_empty()
                            || v.vowel_cluster)
                    {
                        return false;
                    }
                    if !v.stem_exact.is_empty() && v.stem_exact != intent.root {
                        return false;
                    }
                    if v.short_only && !self.m.short_stems.stems.iter().any(|s| s == &intent.root) {
                        return false;
                    }
                    if v.borrowed_only && !is_borrowed {
                        return false;
                    }
                    if !v.ends_with.is_empty() && !intent.root.ends_with(&v.ends_with) {
                        return false;
                    }
                    if v.vowel_cluster && !is_vowel_cluster {
                        return false;
                    }
                    if !v.trigger_vowels.is_empty()
                        && !root_last_vowel
                            .map(|c| v.trigger_vowels.contains(c))
                            .unwrap_or(false)
                    {
                        return false;
                    }
                    true
                });
                if let Some(v) = variant {
                    if idx == 0 && v.keep_final_vowel {
                        keep_root_final_vowel = true;
                    }
                    Some(v.suffix.clone())
                } else {
                    self.m.derivations.get(tag).cloned()
                }
            })
            .collect();

        if !derived.is_empty() {
            if !keep_root_final_vowel
                && out
                    .chars()
                    .last()
                    .map(|c| "aeiou".contains(c))
                    .unwrap_or(false)
            {
                out.pop();
            }
            for d in &derived {
                out.push_str(d);
                slots.push(("deriv".into(), d.clone()));
            }
        }

        // Slot 6: mood / final vowel / imperative plural suffix.
        let mut wanted = self.select_final_vowel(intent, is_neg_subj);
        if is_plural_imp {
            let pv = if intent.object.is_some() && !imp.object_vowel.is_empty() {
                imp.object_vowel.chars().next().unwrap_or('e')
            } else {
                imp.plural_vowel.chars().next().unwrap_or('e')
            };
            wanted = pv;
        } else if is_bare_imp && intent.object.is_some() && intent.mood == "a" && !imp.object_vowel.is_empty() {
            wanted = imp.object_vowel.chars().next().unwrap_or('e');
        }

        let last = out.chars().last();
        if !derived.is_empty() {
            if last != Some(wanted) {
                out.push(wanted);
            }
        } else {
            match last {
                Some('a') if wanted != 'a' && !is_borrowed => {
                    out.pop();
                    out.push(wanted);
                }
                Some(c) if "aeiou".contains(c) => {}
                _ => out.push(wanted),
            }
        }
        slots.push(("mood".into(), intent.mood.clone()));

        if is_plural_imp && !imp.plural_suffix.is_empty() {
            out.push_str(&imp.plural_suffix);
            slots.push(("imp_pl_suffix".into(), imp.plural_suffix.clone()));
        }

        // Noun-class concord metadata: resolve the canonical subject tag and
        // the object infix through the data tables. Persons and unmapped tags
        // yield None by design.
        let subject_class = intent
            .subject
            .as_deref()
            .and_then(|s| self.m.noun_classes.subjects.get(&self.tag(s)).cloned());
        let object_class = intent
            .object
            .as_deref()
            .and_then(|o| self.m.noun_classes.objects.get(o).cloned());

        SynthesisResult {
            surface: out,
            slots,
            subject_class,
            object_class,
        }
    }

    /// Choose the trailing vowel using the declared rules.
    fn select_final_vowel(&self, intent: &Intent, is_neg_subj: bool) -> char {
        if is_neg_subj {
            return self
                .m
                .final_vowel
                .iter()
                .find(|r| r.step == "mood" && r.mood == intent.mood)
                .map(|r| r.use_vowel.chars().next().unwrap_or('e'))
                .unwrap_or('e');
        }
        for rule in &self.m.final_vowel {
            match rule.step.as_str() {
                "mood" if rule.mood == intent.mood => {
                    return rule.use_vowel.chars().next().unwrap_or('a');
                }
                "present_neg"
                    if intent.negation && self.is_present(intent.tense.as_deref(), rule) =>
                {
                    return rule.use_vowel.chars().next().unwrap_or('i');
                }
                _ => {}
            }
        }
        self.m.defaults.final_vowel.chars().next().unwrap_or('a')
    }

    fn is_present(&self, tense: Option<&str>, rule: &FinalVowelRule) -> bool {
        let t = tense.unwrap_or("");
        rule.tense
            .iter()
            .any(|rt| rt.eq_ignore_ascii_case(t) || (rt.is_empty() && t.is_empty()))
    }

    fn tense_in_list(&self, tense: &str, list: &[String]) -> bool {
        let ttag = self.tense_tag(tense);
        list.iter()
            .any(|x| x.eq_ignore_ascii_case(tense) || x.eq_ignore_ascii_case(&ttag))
    }

    fn short_tense(&self, tense: &str, negated: bool) -> bool {
        let up = tense.to_uppercase();
        let ttag = self.tense_tag(tense);
        let list = if negated {
            &self.m.short_stems.ku_after_tenses_neg
        } else {
            &self.m.short_stems.ku_after_tenses
        };
        list.iter()
            .any(|t| t.eq_ignore_ascii_case(&up) || t.eq_ignore_ascii_case(&ttag))
    }

    fn negative_tense(&self, tense: &str) -> String {
        let tag = self.tense_tag(tense);
        self.m
            .tense_neg
            .get(&tag)
            .cloned()
            .unwrap_or_else(|| self.m.tenses.get(&tag).cloned().unwrap_or_default())
    }

    /// Map a raw subject token to a canonical tag.
    pub fn canonical_subject(&self, subj: &str) -> String {
        self.tag(subj)
    }

    /// Map a raw tense token to a canonical tag.
    pub fn canonical_tense(&self, tense: &str) -> String {
        self.tense_tag(tense)
    }

    /// Map a raw subject token to a canonical tag.
    fn tag(&self, subj: &str) -> String {
        match subj.to_uppercase().as_str() {
            // Note: "I" is NOT aliased to 1SG here because the morphology data
            // defines a genuine noun-class tag literally named `I`. Aliasing it
            // would shadow real data with a hardcoded rule. Use "NI" or "1SG".
            "NI" | "1SG" => "1SG",
            "YOU" | "U" | "2SG" => "2SG",
            "HE" | "SHE" | "A" | "3SG" => "3SG",
            "WE" | "TU" | "1PL" => "1PL",
            "YOU.PL" | "M" | "2PL" => "2PL",
            "THEY" | "WA" | "3PL" => "3PL",
            other => other,
        }
        .to_string()
    }

    /// Map a raw tense token to a canonical tag from the loaded `[tenses]` table.
    fn tense_tag(&self, tense: &str) -> String {
        let up = tense.to_uppercase();
        if self.m.tenses.contains_key(&up) || self.m.tense_neg.contains_key(&up) {
            return up;
        }
        if let Some((tag, _)) = self
            .m
            .tenses
            .iter()
            .find(|(_, marker)| !marker.is_empty() && marker.eq_ignore_ascii_case(tense))
        {
            return tag.clone();
        }
        up
    }
}

/// Convenience wrapper matching the old public surface.
pub mod prelude {
    pub use super::intent::Intent;
    pub use super::{Engine, Morphology, SynthesisResult, ValidationError};
}
