//! Backpass: structured intent -> Swahili surface string.
//!
//! The Rolodex takes a lightweight structured payload (or telegraphic
//! English tokens) from the LLM and synthesizes natural, grammatically
//! sound Swahili using deterministic slot assembly.
//!
//! Slot template: [NEG][SUBJ][TENSE][OBJ][ROOT][DERIV][MOOD]
//!
//! The LLM never has to guess Swahili grammar. It outputs intent,
//! and the Rolodex compiles it.

use serde::Deserialize;

/// A structured intent payload for Swahili synthesis.
#[derive(Debug, Clone, Deserialize)]
pub struct Intent {
    pub subject: Option<String>,
    pub tense: Option<String>,
    pub negation: bool,
    pub object: Option<String>,
    pub root: String,
    pub derivations: Vec<String>,
    pub mood: String,
    /// Optional standalone words (e.g. "hapana" for "no").
    pub particles: Vec<String>,
}

impl Intent {
    pub fn simple(subj: &str, tense: &str, root: &str) -> Self {
        Self {
            subject: Some(subj.to_string()),
            tense: Some(tense.to_string()),
            negation: false,
            object: None,
            root: root.to_string(),
            derivations: Vec::new(),
            mood: "a".to_string(),
            particles: Vec::new(),
        }
    }

    pub fn negated(subj: &str, tense: &str, root: &str) -> Self {
        Self {
            subject: Some(subj.to_string()),
            tense: Some(tense.to_string()),
            negation: true,
            object: None,
            root: root.to_string(),
            derivations: Vec::new(),
            mood: "a".to_string(),
            particles: Vec::new(),
        }
    }
}

/// Result of Rolodex synthesis.
#[derive(Debug, Clone)]
pub struct Synthesized {
    /// The surface Swahili string.
    pub surface: String,
    /// Individual slot values used (for debugging/inspection).
    pub slots: Vec<(String, String)>,
    /// Whether phonological smoothing was applied.
    pub smoothed: bool,
}

/// The Rolodex assembler: compiles Intent -> surface Swahili.
pub struct Rolodex {
    /// Subject prefix lookup: tag -> prefix
    subjects: std::collections::HashMap<String, String>,
    /// Combined neg+subject lookup: tag -> prefix
    subjects_neg: std::collections::HashMap<String, String>,
    /// Tense marker lookup: tag -> marker
    tenses: std::collections::HashMap<String, String>,
    /// Object infix lookup: infix key -> description
    objects: std::collections::HashMap<String, String>,
    /// Negative tense overrides: neg_key -> marker
    tense_neg: std::collections::HashMap<String, String>,
    /// Negation prefix
    neg_prefix: String,
}

impl Rolodex {
    /// Build the default Swahili Rolodex.
    pub fn swahili() -> Self {
        let mut subjects = std::collections::HashMap::new();
        subjects.insert("1SG".into(), "ni".into());
        subjects.insert("2SG".into(), "u".into());
        subjects.insert("3SG".into(), "a".into());
        subjects.insert("1PL".into(), "tu".into());
        subjects.insert("2PL".into(), "m".into());
        subjects.insert("3PL".into(), "wa".into());
        // Class markers
        subjects.insert("LI".into(), "li".into());
        subjects.insert("YA".into(), "ya".into());
        subjects.insert("PA".into(), "pa".into());
        subjects.insert("KI".into(), "ki".into());
        subjects.insert("JI".into(), "ji".into());
        subjects.insert("ZI".into(), "zi".into());
        subjects.insert("I".into(), "i".into());

        let mut subjects_neg = std::collections::HashMap::new();
        subjects_neg.insert("1SG".into(), "si".into());
        subjects_neg.insert("2SG".into(), "hu".into());
        subjects_neg.insert("3SG".into(), "ha".into());
        subjects_neg.insert("1PL".into(), "hatu".into());
        subjects_neg.insert("2PL".into(), "ham".into());
        subjects_neg.insert("3PL".into(), "hawa".into());

        let mut tenses = std::collections::HashMap::new();
        tenses.insert("PRES".into(), "na".into());
        tenses.insert("PAST".into(), "li".into());
        tenses.insert("FUT".into(), "ta".into());
        tenses.insert("PERF".into(), "me".into());
        tenses.insert("CONSEC".into(), "ka".into());
        tenses.insert("HAB".into(), "hu".into());
        tenses.insert("NOTYET".into(), "ja".into());
        tenses.insert("INF".into(), "ku".into());
        tenses.insert("WHEN".into(), "po".into());
        tenses.insert("LOC".into(), "ko".into());
        tenses.insert("INT".into(), "mo".into());
        tenses.insert("BARE".into(), "".into());

        let mut objects = std::collections::HashMap::new();
        objects.insert("1SG".into(), "ni".into());
        objects.insert("2SG".into(), "ku".into());
        objects.insert("3SG_M".into(), "m".into());
        objects.insert("3SG_F".into(), "yu".into());
        objects.insert("3SG_KI".into(), "ki".into());
        objects.insert("3SG_N".into(), "i".into());
        objects.insert("1PL_OBJ".into(), "tu".into());
        objects.insert("2PL_OBJ".into(), "wa".into());
        objects.insert("3PL_WA".into(), "wa".into());
        objects.insert("3PL_VI".into(), "vi".into());
        objects.insert("3PL_YA".into(), "yi".into());
        objects.insert("3PL_ZI".into(), "zi".into());
        // Direct infix keys too (so we can pass "ki", "wa" etc. directly)
        objects.insert("m".into(), "me/you sg".into());
        objects.insert("ni".into(), "me".into());
        objects.insert("ku".into(), "you sg".into());
        objects.insert("wa".into(), "them".into());
        objects.insert("ki".into(), "it".into());
        objects.insert("vi".into(), "them".into());
        objects.insert("yi".into(), "them".into());
        objects.insert("zi".into(), "them".into());
        objects.insert("li".into(), "it".into());
        objects.insert("yu".into(), "him".into());

        let mut tense_neg = std::collections::HashMap::new();
        tense_neg.insert("li_neg".into(), "ku".into()); // past negative
        tense_neg.insert("pres_neg".into(), "".into()); // present negative drops marker
        tense_neg.insert("fut_neg".into(), "ta".into()); // future negative keeps -ta-
        tense_neg.insert("perf_neg".into(), "ja".into()); // perfect negative -> -ja-

        Self {
            subjects,
            subjects_neg,
            tenses,
            objects,
            tense_neg,
            neg_prefix: "ha".into(),
        }
    }

    /// Resolve a raw tense value (e.g. "me", "li", "PERF") to its marker.
    fn resolve_tense(&self, raw: &str) -> String {
        self.tenses
            .get(raw)
            .or_else(|| self.tenses.get(&raw.to_uppercase()))
            .cloned()
            .unwrap_or_else(|| raw.to_string())
    }

    /// Synthesize a Swahili verb from an Intent.
    pub fn synthesize(&self, intent: &Intent) -> Synthesized {
        let mut slots = Vec::new();
        let mut out = String::new();

        // Slot 0: Standalone particles (hapana, ndiyo, etc.)
        for p in &intent.particles {
            out.push_str(p);
            out.push(' ');
            slots.push(("particle".into(), p.clone()));
        }

        // Slot 1: Negation + Subject
        let is_neg_subjunctive = intent.negation
            && (intent.mood == "e"
                || intent
                    .tense
                    .as_deref()
                    .map(|t| t.eq_ignore_ascii_case("SUBJ"))
                    .unwrap_or(false));
        if is_neg_subjunctive {
            // Negative subjunctive: SUBJECT + -si- + ROOT (+e for regular stems).
            // e.g. u-si-jali -> usijali ; ha-tu-si-som-e -> tusisome
            if let Some(ref subj) = intent.subject {
                let key = self.subject_to_tag(subj);
                if let Some(s) = self.subjects.get(&key) {
                    out.push_str(s);
                    slots.push(("subj".into(), s.clone()));
                }
            }
            out.push_str("si");
            slots.push(("neg_si".into(), "si".into()));
        } else if intent.negation {
            if let Some(ref subj) = intent.subject {
                // Try combined neg+subject form first
                let key = self.subject_to_tag(subj);
                if let Some(combined) = self.subjects_neg.get(&key) {
                    out.push_str(combined);
                    slots.push(("neg+subj".into(), combined.clone()));
                } else {
                    out.push_str(&self.neg_prefix);
                    if let Some(s) = self.subjects.get(&key) {
                        out.push_str(s);
                        slots.push(("neg".into(), self.neg_prefix.clone()));
                        slots.push(("subj".into(), s.clone()));
                    }
                }
            } else {
                out.push_str(&self.neg_prefix);
                slots.push(("neg".into(), self.neg_prefix.clone()));
            }
        } else if let Some(ref subj) = intent.subject {
            let key = self.subject_to_tag(subj);
            if let Some(s) = self.subjects.get(&key) {
                out.push_str(s);
                slots.push(("subj".into(), s.clone()));
            }
        }

        // Slot 2: Tense
        if !is_neg_subjunctive {
            if let Some(ref tense) = intent.tense {
                let raw = tense.as_str();
                if intent.negation {
                    // Negative: check tense_neg overrides first
                    let neg_key = self.neg_tense_key(raw);
                    if let Some(m) = self.tense_neg.get(&neg_key) {
                        if !m.is_empty() {
                            out.push_str(m);
                            slots.push(("tense".into(), m.clone()));
                        }
                        // else: present negative drops tense marker entirely
                    } else {
                        // No override: use raw tense marker as-is
                        let m = self.resolve_tense(raw);
                        if !m.is_empty() {
                            out.push_str(&m);
                            slots.push(("tense".into(), m));
                        }
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

        // Slot 3: Object infix
        if let Some(ref obj) = intent.object {
            // The objects HashMap maps infix -> description.
            // We use the key (infix) directly as the surface form.
            let key_match = obj.to_uppercase();
            let surface_obj: &str = if self.objects.contains_key(obj.as_str()) {
                obj.as_str()
            } else if self.objects.contains_key(&key_match) {
                &key_match
            } else {
                obj.as_str()
            };
            // Vowel-glide: object infix -m- before a vowel-initial root
            // surfaces as -mw- (e.g. m + ona -> mwona, as in "sijamwona").
            let root_starts_vowel = matches!(
                intent.root.chars().next(),
                Some('a' | 'e' | 'i' | 'o' | 'u')
            );
            let rendered = if surface_obj == "m" && root_starts_vowel {
                "mw".to_string()
            } else {
                surface_obj.to_string()
            };
            out.push_str(&rendered);
            slots.push(("obj".into(), rendered));
        }

        // Slot 4: Root
        // Monosyllabic verb stems (la, ja, nywa, wa, pa) take an intrusive
        // -ku- right after the TAM marker in affirmative forms whose TAM is
        // in the stress-requiring group (-na-, -me-, -li-, -ta-, ...).
        // This keeps penultimate stress off the weak TAM prefix.
        let root_is_monosyllabic = self.is_monosyllabic(&intent.root);
        if root_is_monosyllabic && !intent.negation && self.mono_needs_ku(intent.tense.as_deref()) {
            out.push_str("ku");
            slots.push(("mono_ku".into(), "ku".into()));
        }
        let root_final_vowel = intent.root.chars().last();
        out.push_str(&intent.root);
        slots.push(("root".into(), intent.root.clone()));

        // Slot 5: Derivational extensions (appended after root, before mood vowel)
        for d in &intent.derivations {
            out.push_str(d);
            slots.push(("deriv".into(), d.clone()));
        }

        // Slot 6: Final vowel / mood
        // The surface ending is governed by the mood PLUS (for the present
        // negative) the productive -a -> -i shift on regular stems.
        //
        //   * Borrowed/irregular stems that already end in -e/-i/-u
        //     (jibu, subiri, samehe, sema is NOT borrowed; it ends in -a)
        //     never undergo a final-vowel change.
        //   * The present negative turns a regular final -a into -i (no -na-):
        //       a-na-som-a  ->  ha-som-i   (sisomi for 1SG)
        //   * Subjunctive/hortative replace a trailing -a with -e/-o.
        let present_negative = !is_neg_subjunctive
            && intent.negation
            && self.is_present_negative(intent.tense.as_deref());
        let borrowed = matches!(root_final_vowel, Some('e') | Some('i') | Some('u'));

        let mood_vowel = match intent.mood.as_str() {
            "e" => Some('e'),
            "o" => Some('o'),
            "i" => Some('i'),
            _ => Some('a'),
        };
        let wanted = if present_negative && !borrowed {
            'i' // productive present-negative shift on regular -a stems
        } else {
            mood_vowel.unwrap_or('a')
        };

        let last = out.chars().last();
        match last {
            // Regular -a stem with a mood shift OR present negative: replace -a
            Some('a') if wanted != 'a' && !borrowed => {
                out.pop();
                out.push(wanted);
            }
            // Stem ends in a non-a vowel (e.g. jibu, subiri + borrowed) -> leave
            Some(c) if "aeiou".contains(c) => {
                // present-negative on a regular stem only ever sees -a above;
                // borrowed stems here are left untouched as required.
            }
            _ => {
                // Consonant-final stem: append the wanted vowel.
                out.push(wanted);
            }
        }
        slots.push(("mood".into(), intent.mood.clone()));

        let smoothed = false; // TODO: phonological smoothing pass

        Synthesized {
            surface: out,
            slots,
            smoothed,
        }
    }

    /// Build a verb from explicit slot values (low-level API).
    pub fn build_verb(
        &self,
        neg: bool,
        subj: Option<&str>,
        tense: Option<&str>,
        obj: Option<&str>,
        root: &str,
        mood: &str,
    ) -> String {
        let intent = Intent {
            subject: subj.map(|s| s.to_string()),
            tense: tense.map(|t| t.to_string()),
            negation: neg,
            object: obj.map(|o| o.to_string()),
            root: root.to_string(),
            derivations: Vec::new(),
            mood: mood.to_string(),
            particles: Vec::new(),
        };
        self.synthesize(&intent).surface
    }

    /// Map a subject pronoun/tag to the lookup key format.
    fn subject_to_tag(&self, subj: &str) -> String {
        match subj.to_uppercase().as_str() {
            "I" | "NI" | "1SG" => "1SG".into(),
            "YOU" | "U" | "2SG" => "2SG".into(),
            "HE" | "SHE" | "A" | "3SG" => "3SG".into(),
            "WE" | "TU" | "1PL" => "1PL".into(),
            "YOU.PL" | "M" | "2PL" => "2PL".into(),
            "THEY" | "WA" | "3PL" => "3PL".into(),
            _ => subj.to_uppercase(),
        }
    }

    /// Map a tense tag to the negative tense override key.
    fn neg_tense_key(&self, tense: &str) -> String {
        match tense.to_uppercase().as_str() {
            "PAST" | "LI" => "li_neg".into(),
            "PRES" | "NA" => "pres_neg".into(),
            "FUT" | "TA" => "fut_neg".into(),
            "PERF" | "ME" => "perf_neg".into(),
            _ => tense.to_string(),
        }
    }

    /// Is this a present-tense construction? (present negative shifts -a to -i)
    fn is_present_negative(&self, tense: Option<&str>) -> bool {
        match tense.map(|t| t.to_uppercase()).as_deref() {
            Some("PRES") | Some("NA") | None | Some("") | Some("BARE") => true,
            _ => false,
        }
    }

    /// Is the root a monosyllabic verb stem that takes intrusive -ku-?
    fn is_monosyllabic(&self, root: &str) -> bool {
        matches!(root, "la" | "ja" | "nywa" | "wa" | "pa")
    }

    /// Monosyllabic -ku- appears in affirmative finite forms whose TAM
    /// cannot take stress: -na-, -me-, -li-, -ta-, -sha-, -nge-, -ngali-,
    /// -ngeli-. It is NOT inserted with -a- (gnomic), -hu-, -ki-, -ka-, -ku-,
    /// or present-negative (which has no TAM marker).
    fn mono_needs_ku(&self, tense: Option<&str>) -> bool {
        match tense.map(|t| t.to_uppercase()).as_deref() {
            Some("NA") | Some("ME") | Some("LI") | Some("TA") | Some("SHA") | Some("NGE")
            | Some("NGALI") | Some("NGELI") | Some("PRES") | Some("PAST") | Some("FUT")
            | Some("PERF") => true,
            _ => false,
        }
    }
}

/// Quick convenience: build a common verb form.
pub fn quick_verb(subj: &str, tense: &str, root: &str) -> String {
    Rolodex::swahili().build_verb(false, Some(subj), Some(tense), None, root, "a")
}

/// Quick convenience: build a negated verb form.
pub fn quick_verb_neg(subj: &str, tense: &str, root: &str) -> String {
    Rolodex::swahili().build_verb(true, Some(subj), Some(tense), None, root, "a")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_positive() {
        let r = Rolodex::swahili();
        let result = r.build_verb(false, Some("u"), Some("me"), None, "fika", "a");
        assert_eq!(result, "umefika");
    }

    #[test]
    fn simple_future() {
        let r = Rolodex::swahili();
        let result = r.build_verb(false, Some("ni"), Some("ta"), None, "sema", "a");
        assert_eq!(result, "nitasema");
    }

    #[test]
    fn negated_past() {
        let r = Rolodex::swahili();
        let result = r.build_verb(true, Some("ni"), Some("li"), None, "fanya", "a");
        // si + ni(not used in combined) -> si + li -> silifanya?
        // Actually: neg 1SG + PAST -> si- + -li- + root
        assert!(result.starts_with("si"));
        assert!(result.ends_with("fanya"));
    }

    #[test]
    fn negated_3pl() {
        let r = Rolodex::swahili();
        let result = r.build_verb(true, Some("wa"), Some("ta"), None, "jibu", "a");
        assert!(result.starts_with("hawa"));
        assert!(result.ends_with("jibu"));
    }

    #[test]
    fn with_object_infix() {
        let r = Rolodex::swahili();
        let result = r.build_verb(false, Some("a"), Some("li"), Some("ki"), "omba", "a");
        // a + li + ki + omba + a = alikiomba
        assert_eq!(result, "alikiomba");
    }

    #[test]
    fn subjunctive_mood() {
        let r = Rolodex::swahili();
        let result = r.build_verb(false, Some("a"), Some("BARE"), None, "fika", "e");
        assert!(result.ends_with('e'));
    }

    #[test]
    fn quick_verb_helper() {
        assert_eq!(quick_verb("ni", "ta", "enda"), "nitaenda");
    }

    #[test]
    fn quick_verb_neg_helper() {
        let r = quick_verb_neg("wa", "PAST", "fika");
        assert!(r.starts_with("ha"));
        assert!(r.ends_with("fika"));
    }

    #[test]
    fn present_negative_shifts_final_a_to_i() {
        let r = Rolodex::swahili();
        // a-na-som-a -> ha-som-i ; for 1SG: si-som-i
        let result = r.build_verb(true, Some("ni"), Some("PRES"), None, "soma", "a");
        assert_eq!(result, "sisomi");
    }

    #[test]
    fn present_negative_keeps_borrowed_ending() {
        let r = Rolodex::swahili();
        // Borrowed jibu (final -u) does NOT shift to -i under present negative.
        let result = r.build_verb(true, Some("a"), Some("PRES"), None, "jibu", "a");
        assert_eq!(result, "hajibu");
    }

    #[test]
    fn monosyllabic_inserts_ku() {
        let r = Rolodex::swahili();
        // a-na-la -> anakula
        let result = r.build_verb(false, Some("a"), Some("na"), None, "la", "a");
        assert_eq!(result, "anakula");
        // ni-ta-ja -> nitakuja
        let result = r.build_verb(false, Some("ni"), Some("ta"), None, "ja", "a");
        assert_eq!(result, "nitakuja");
    }

    #[test]
    fn negative_subjunctive_borrowed() {
        let r = Rolodex::swahili();
        // u-si-jali -> usijali (borrowed root jali keeps -i)
        let result = r.build_verb(true, Some("u"), Some("SUBJ"), None, "jali", "e");
        assert_eq!(result, "usijali");
    }

    #[test]
    fn negative_subjunctive_regular() {
        let r = Rolodex::swahili();
        // tu-si-som-e -> tusisome
        let result = r.build_verb(true, Some("tu"), Some("BARE"), None, "soma", "e");
        assert_eq!(result, "tusisome");
    }

    #[test]
    fn object_vowel_glide_mw() {
        let r = Rolodex::swahili();
        // si-ja-mw-ona -> sijamwona (m + vowel-initial ona -> mw)
        let result = r.build_verb(true, Some("ni"), Some("PERF"), Some("m"), "ona", "a");
        assert_eq!(result, "sijamwona");
    }
}
