//! Table-driven backpass synthesis engine.
//!
//! This module is the Option-B design: a *generic* interpreter with NO
//! per-language code. Every rule that governs how a structured `Intent` is
//! assembled into a surface verb form lives in the language TOML under the
//! `[synthesis]` section (see `affix/sw.toml`). Retarget a language simply
//! by supplying its own `[synthesis]` tables — nothing here needs to change.
//!
//! Slot template: [NEG][SUBJ][TENSE][OBJ][ROOT][DERIV][MOOD]

use crate::affix::AffixTable;
use crate::rolodex::Intent;
use serde::Deserialize;
use std::collections::HashMap;

/// The complete `[synthesis]` block of a language TOML.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct SynthesisSpec {
    /// Ordered final-vowel selection rules (first match wins).
    #[serde(default)]
    pub final_vowel: Vec<FinalVowelRule>,
    /// Defaults applied when no final-vowel rule matches.
    #[serde(default)]
    pub defaults: Defaults,
    /// Negative-subjunctive assembly (SUBJ + si + ROOT + mood_vowel).
    #[serde(default)]
    pub negative_subjunctive: Option<NegativeSubjunctive>,
    /// Intrusive support morpheme for short/monosyllabic stems.
    #[serde(default)]
    pub short_stems: Option<ShortStems>,
    /// Vowel-glide allomorph rules at morpheme boundaries.
    #[serde(default)]
    pub vowel_glide: VowelGlide,
    /// Borrowed/irregular stems whose final vowel never shifts.
    #[serde(default)]
    pub borrowed: Borrowed,
}

/// A single final-vowel selection rule.
#[derive(Debug, Clone, Deserialize)]
pub struct FinalVowelRule {
    pub step: String,
    /// For `step = "mood"`: the intent mood tag this rule handles.
    #[serde(default)]
    pub mood: String,
    /// For `step = "present_neg"`: tense tags treated as present.
    #[serde(default)]
    pub tense: Vec<String>,
    /// The vowel to use when this rule matches.
    #[serde(default = "default_a")]
    pub use_vowel: String,
}

fn default_a() -> String {
    "a".to_string()
}

/// Defaults block.
#[derive(Debug, Clone, Deserialize)]
pub struct Defaults {
    #[serde(default = "default_a")]
    pub final_vowel: String,
}

impl Default for Defaults {
    fn default() -> Self {
        Self { final_vowel: "a".to_string() }
    }
}

/// Negative-subjunctive config.
#[derive(Debug, Clone, Deserialize)]
pub struct NegativeSubjunctive {
    #[serde(default = "default_si")]
    pub infix: String,
    #[serde(default)]
    pub mood: String,
    #[serde(default)]
    pub tense: String,
}

fn default_si() -> String {
    "si".to_string()
}

/// Short-stem intrusive-morpheme config.
#[derive(Debug, Clone, Deserialize)]
pub struct ShortStems {
    #[serde(default)]
    pub stems: Vec<String>,
    #[serde(default)]
    pub morpheme: String,
    /// TAM tags after which the support morpheme is inserted.
    #[serde(default)]
    pub ku_after_tenses: Vec<String>,
    /// Skip insertion in negative constructions.
    #[serde(default)]
    pub skip_negative: bool,
}

/// Vowel-glide rules.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct VowelGlide {
    /// object infix -> surface allomorph (e.g. m_vowel = "mw").
    #[serde(default)]
    pub m_vowel: String,
}

/// Borrowed/irregular stem config.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct Borrowed {
    /// Stems whose citation final vowel lives in this set are fixed.
    #[serde(default)]
    pub fixed_final_vowels: Vec<String>,
}

/// Load the synthesis spec embedded for a language table string.
pub fn load(s: &str) -> Result<SynthesisSpec, toml::de::Error> {
    #[derive(Deserialize)]
    struct File {
        #[serde(default)]
        synthesis: Option<SynthesisSpec>,
    }
    let f: File = toml::from_str(s)?;
    Ok(f.synthesis.unwrap_or_default())
}

/// A single assembled slot for inspection/debugging.
#[derive(Debug, Clone)]
pub struct Slot {
    pub name: String,
    pub value: String,
}

/// Result of generic synthesis.
#[derive(Debug, Clone)]
pub struct Outcome {
    pub surface: String,
    pub slots: Vec<(String, String)>,
    pub smoothed: bool,
}

/// The generic, table-driven engine.
pub struct Engine<'a> {
    spec: &'a SynthesisSpec,
    affix: &'a AffixTable,
}

impl<'a> Engine<'a> {
    pub fn new(spec: &'a SynthesisSpec, affix: &'a AffixTable) -> Self {
        Self { spec, affix }
    }

    /// Assemble a finite verb from an intent.
    pub fn synthesize(&self, intent: &Intent) -> Outcome {
        let mut slots: Vec<(String, String)> = Vec::new();
        let mut out = String::new();

        let neg_subj = self.spec.negative_subjunctive.as_ref();

        // Determine negative-subjunctive mode from the table, generically.
        // Mode = negation AND (mood matches neg-subj mood OR tense matches).
        let is_neg_subj = intent.negation
            && neg_subj.is_some()
            && (intent.mood == neg_subj.as_ref().unwrap().mood
                || neg_subj
                    .as_ref()
                    .unwrap()
                    .tense
                    .eq_ignore_ascii_case(intent.tense.as_deref().unwrap_or("")));

        // Slot 1: Negation + Subject (or negative-subjunctive).
        if is_neg_subj {
            let infix = &neg_subj.as_ref().unwrap().infix;
            if let Some(subj) = &intent.subject {
                if let Some(prefix) = self.positive_subject(subj) {
                    out.push_str(&prefix);
                    slots.push(("subj".into(), prefix));
                }
            }
            out.push_str(infix);
            slots.push(("neg_infix".into(), infix.clone()));
        } else if intent.negation {
            if let Some(subj) = &intent.subject {
                if let Some(combined) = self.negative_subject(subj) {
                    out.push_str(&combined);
                    slots.push(("neg+subj".into(), combined));
                } else if let Some(neg) = self.affix.negation.first() {
                    out.push_str(neg);
                    slots.push(("neg".into(), neg.clone()));
                    if let Some(pos) = self.positive_subject(subj) {
                        out.push_str(&pos);
                        slots.push(("subj".into(), pos));
                    }
                }
            } else if let Some(neg) = self.affix.negation.first() {
                out.push_str(neg);
                slots.push(("neg".into(), neg.clone()));
            }
        } else if let Some(subj) = &intent.subject {
            if let Some(prefix) = self.positive_subject(subj) {
                out.push_str(&prefix);
                slots.push(("subj".into(), prefix));
            }
        }

        // Slot 2: Tense.
        if !is_neg_subj {
            if let Some(tense) = &intent.tense {
                let raw = tense.as_str();
                if intent.negation {
                    let m = self.negative_tense(raw);
                    if !m.is_empty() {
                        out.push_str(&m);
                        slots.push(("tense".into(), m));
                    }
                } else {
                    let m = self.resolve_tense(raw);
                    if !m.is_empty() {
                        out.push_str(&m);
                        slots.push(("tense".into(), m));
                    }
                }
            }
        }

        // Intrusive support morpheme for short stems (after TAM, before obj).
        let short = self.spec.short_stems.as_ref();
        let is_short = short
            .map(|s| s.stems.iter().any(|st| st == &intent.root))
            .unwrap_or(false);
        let insert_short = is_short
            && short.map(|s| !(s.skip_negative && intent.negation)).unwrap_or(false)
            && self.short_needs_morpheme(intent.tense.as_deref());

        // Slot 3: Object infix (with glide).
        if let Some(obj) = &intent.object {
            let root_vowel = matches!(intent.root.chars().next(), Some('a' | 'e' | 'i' | 'o' | 'u'));
            let rendered = self.affix.objects.get_key_value(obj.as_str()).map(|(k, _)| k.clone())
                .or_else(|| {
                    let up = obj.to_uppercase();
                    self.affix.objects.get(&up).cloned().or_else(|| Some(obj.clone()))
                })
                .unwrap_or_else(|| obj.clone());
            // generic glide: object "m" before vowel root -> spec.m_vowel
            let surface = if rendered == "m" && root_vowel && !self.spec.vowel_glide.m_vowel.is_empty() {
                self.spec.vowel_glide.m_vowel.clone()
            } else {
                rendered
            };
            if insert_short {
                // support morpheme goes between TAM and object
                let mv = short.as_ref().unwrap().morpheme.clone();
                out.push_str(&mv);
                slots.push(("short_morph".into(), mv));
            }
            out.push_str(&surface);
            slots.push(("obj".into(), surface));
        } else if insert_short {
            let mv = short.as_ref().unwrap().morpheme.clone();
            out.push_str(&mv);
            slots.push(("short_morph".into(), mv));
        }

        // Slot 4: Root.
        let root_final = intent.root.chars().last();
        out.push_str(&intent.root);
        slots.push(("root".into(), intent.root.clone()));

        // Slot 5: Derivational extensions.
        for d in &intent.derivations {
            out.push_str(d);
            slots.push(("deriv".into(), d.clone()));
        }

        // Slot 6: Final vowel / mood (table-driven).
        let borrowed = self.spec
            .borrowed
            .fixed_final_vowels
            .iter()
            .any(|v| root_final.map(|c| c.to_string() == *v).unwrap_or(false));
        let wanted = self.select_final_vowel(intent, is_neg_subj);

        let last = out.chars().last();
        match last {
            // Replace a trailing regular -a when the wanted vowel differs.
            Some('a') if wanted != 'a' && !borrowed => {
                out.pop();
                out.push(wanted);
            }
            // Ends in some vowel other than -a (a borrowed stem): keep it.
            Some(c) if "aeiou".contains(c) => {
                // No change: borrowed or already-vowel-final stem.
            }
            // Ends in a consonant: append the wanted vowel.
            _ => {
                out.push(wanted);
            }
        }
        slots.push(("mood".into(), intent.mood.clone()));

        Outcome {
            surface: out,
            slots,
            smoothed: false,
        }
    }

    /// Choose the final vowel using the declared rules.
    fn select_final_vowel(&self, intent: &Intent, is_neg_subj: bool) -> char {
        if is_neg_subj {
            // Negative subjunctive uses the neg-subj mood vowel (e.g. e),
            // NOT the present-negative shift.
            if let Some(rule) = self.spec.final_vowel.iter().find(|r| r.mood == intent.mood) {
                return rule.use_vowel.chars().next().unwrap_or('e');
            }
            return 'e';
        }
        for rule in &self.spec.final_vowel {
            match rule.step.as_str() {
                "mood" if rule.mood == intent.mood => {
                    return rule.use_vowel.chars().next().unwrap_or('a');
                }
                "present_neg" if intent.negation && self.is_present(intent.tense.as_deref(), rule) => {
                    return rule.use_vowel.chars().next().unwrap_or('i');
                }
                _ => {}
            }
        }
        self.spec.defaults.final_vowel.chars().next().unwrap_or('a')
    }

    fn is_present(&self, tense: Option<&str>, rule: &FinalVowelRule) -> bool {
        let t = tense.unwrap_or("");
        let up = t.to_uppercase();
        rule.tense.iter().any(|rt| {
            rt.to_uppercase() == up || (rt.is_empty() && t.is_empty())
        }) || t.is_empty()
    }

    /// Does a short stem require its intrusive support morpheme?
    fn short_needs_morpheme(&self, tense: Option<&str>) -> bool {
        let Some(short) = &self.spec.short_stems else {
            return false;
        };
        let t = tense.unwrap_or("").to_uppercase();
        short.ku_after_tenses.iter().any(|tt| tt.to_uppercase() == t)
    }

    /// Resolve a positive subject tag to a prefix.
    fn positive_subject(&self, subj: &str) -> Option<String> {
        let tag = self.subject_tag(subj);
        self.affix.subjects.iter().enumerate()
            .find(|(i, _)| self.subject_tag_at(*i) == tag)
            .map(|(_, s)| s.clone())
    }

    /// Resolve a negative subject: prefer combined neg+subject forms.
    fn negative_subject(&self, subj: &str) -> Option<String> {
        let tag = self.subject_tag(subj);
        self.affix.subjects_neg.iter()
            .find(|(_, v)| self.subject_tag(v) == tag)
            .map(|(k, _)| k.clone())
    }

    fn subject_tag_at(&self, index: usize) -> String {
        // TOML subject order in sw.toml maps positionally to canonical tags:
        // ni, u, a, tu, m, wa, ... We map by known order for the engine.
        let order = ["1SG", "2SG", "3SG", "1PL", "2PL", "3PL",
                     "LI", "YA", "PA", "KI", "JI", "ZI", "I"];
        order.get(index).map(|s| s.to_string()).unwrap_or_else(|| "?".into())
    }

    fn subject_tag(&self, subj: &str) -> String {
        match subj.to_uppercase().as_str() {
            "I" | "NI" | "1SG" => "1SG".into(),
            "YOU" | "U" | "2SG" => "2SG".into(),
            "HE" | "SHE" | "A" | "3SG" => "3SG".into(),
            "WE" | "TU" | "1PL" => "1PL".into(),
            "YOU.PL" | "M" | "2PL" => "2PL".into(),
            "THEY" | "WA" | "3PL" => "3PL".into(),
            other => other.to_string(),
        }
    }

    fn resolve_tense(&self, raw: &str) -> String {
        self.affix.tenses.iter().find(|t| t.eq_ignore_ascii_case(raw) || **t == raw)
            .cloned().unwrap_or_else(|| raw.to_string())
    }

    fn negative_tense(&self, raw: &str) -> String {
        let key = match raw.to_uppercase().as_str() {
            "PAST" | "LI" => "li_neg",
            "PRES" | "NA" => "pres_neg",
            "FUT" | "TA" => "fut_neg",
            "PERF" | "ME" => "perf_neg",
            _ => return self.resolve_tense(raw),
        };
        self.affix.tense_neg.get(key).cloned().unwrap_or_default()
    }
}
