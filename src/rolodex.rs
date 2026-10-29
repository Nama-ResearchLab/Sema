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
    #[serde(default)]
    pub subject: Option<String>,
    #[serde(default)]
    pub tense: Option<String>,
    #[serde(default)]
    pub negation: bool,
    #[serde(default)]
    pub object: Option<String>,
    pub root: String,
    #[serde(default)]
    pub derivations: Vec<String>,
    #[serde(default = "default_mood")]
    pub mood: String,
    /// Optional standalone words (e.g. "hapana" for "no").
    #[serde(default)]
    pub particles: Vec<String>,
}

fn default_mood() -> String {
    "a".to_string()
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
    /// Noun class the subject concord agrees with (None for persons).
    pub subject_class: Option<String>,
    /// Noun class of the object infix (None for persons).
    pub object_class: Option<String>,
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
    #[allow(dead_code)]
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
        // Explicit class tags. Use these (not bare "I", which is the English
        // pronoun and resolves to 1SG).
        for (tag, cl) in [
            ("C1", "a"), ("C2", "wa"), ("C3", "u"), ("C4", "i"), ("C5", "li"),
            ("C6", "ya"), ("C7", "ki"), ("C8", "vi"), ("C9", "i"), ("C10", "zi"),
            ("C11", "u"), ("C14", "u"), ("C15", "ku"), ("C16", "pa"), ("C17", "ku"),
            ("C18", "mu"), ("VI", "vi"),
        ] {
            subjects.insert(tag.into(), cl.into());
        }

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

        // Irregular bare imperatives (no subject, tense, object or derivation):
        // enda -> nenda, ja -> njoo, la -> kula.
        if intent.subject.is_none()
            && intent.tense.is_none()
            && !intent.negation
            && intent.object.is_none()
            && intent.derivations.is_empty()
            && intent.mood == "a"
        {
            let irregular = match intent.root.as_str() {
                "enda" => Some("nenda"),
                "ja" => Some("njoo"),
                "la" => Some("kula"),
                _ => None,
            };
            if let Some(form) = irregular {
                slots.push(("imperative_irregular".into(), form.into()));
                return Synthesized {
                    surface: form.to_string(),
                    slots,
                    smoothed: false,
                    subject_class: None,
                    object_class: None,
                };
            }
        }

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
                        // ha + a contracts to ha (class 1: hakula, not haakula)
                        if s != "a" {
                            out.push_str(s);
                        }
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
            let surface_obj = self.object_infix(obj);
            // Vowel-glide: object infix -m- before a vowel-initial root
            // surfaces as -mw- (e.g. m + ona -> mwona, as in "sijamwona").
            let root_starts_vowel = matches!(
                intent.root.chars().next(),
                Some('a' | 'e' | 'i' | 'o' | 'u')
            );
            let rendered = if surface_obj == "m" && root_starts_vowel {
                "mw".to_string()
            } else {
                surface_obj
            };
            out.push_str(&rendered);
            slots.push(("obj".into(), rendered));
        }

        // Slot 4/5: Root + derivational extensions. Derivations are applied
        // to the stem first (harmony, -w- passive, -ish- causative ...) so the
        // monosyllabic check sees the real stem (somesha is not monosyllabic).
        let mut stem = intent.root.clone();
        for d in &intent.derivations {
            match derive(&stem, d) {
                Some(next) => {
                    slots.push(("deriv".into(), d.clone()));
                    stem = next;
                }
                None => slots.push(("deriv_unknown".into(), d.clone())),
            }
        }
        let root_is_monosyllabic = self.is_monosyllabic(&stem);
        let ku_ok = intent.object.is_none() && (!intent.negation || self.neg_keeps_ku(intent.tense.as_deref()));
        if root_is_monosyllabic && ku_ok && self.mono_needs_ku(intent.tense.as_deref()) {
            out.push_str("ku");
            slots.push(("mono_ku".into(), "ku".into()));
        }
        let root_final_vowel = stem.chars().last();
        out.push_str(&stem);
        slots.push(("root".into(), stem.clone()));

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

        let subject_class = intent.subject.as_deref().and_then(|x| self.class_label(x));
        let object_class = intent.object.as_deref().and_then(|x| self.class_label(x));
        Synthesized {
            surface: out,
            slots,
            smoothed,
            subject_class,
            object_class,
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
        matches!(root, "la" | "ja" | "nywa" | "wa" | "pa" | "fa" | "cha")
    }

    /// Negative tenses that still carry the intrusive -ku- on mono stems.
    /// Future negative keeps it (hatutakula); past/present negative do not
    /// (sikula, sili).
    fn neg_keeps_ku(&self, tense: Option<&str>) -> bool {
        matches!(
            tense.map(|t| t.to_uppercase()).as_deref(),
            Some("FUT") | Some("TA")
        )
    }

    /// Resolve an object tag or a raw infix to its surface infix.
    fn object_infix(&self, obj: &str) -> String {
        match obj.to_uppercase().as_str() {
            "1SG" | "NI" => "ni".into(),
            "2SG" | "KU" => "ku".into(),
            "3SG" | "3SG_M" | "M" | "MU" => "m".into(),
            "3SG_F" | "YU" => "m".into(),
            "1PL" | "1PL_OBJ" | "TU" => "tu".into(),
            "2PL" | "2PL_OBJ" | "3PL" | "3PL_WA" | "WA" | "C2" => "wa".into(),
            "3SG_KI" | "KI" | "C7" => "ki".into(),
            "3PL_VI" | "VI" | "C8" => "vi".into(),
            "3PL_YA" | "YA" | "C6" => "ya".into(),
            "3SG_N" | "C9" | "C4" => "i".into(),
            "3PL_ZI" | "ZI" | "C10" => "zi".into(),
            "3SG_LI" | "LI" | "C5" => "li".into(),
            "3SG_U" | "C3" | "C11" => "u".into(),
            _ => obj.to_string(),
        }
    }

    /// Human-readable noun class for a subject/object tag (None = person).
    fn class_label(&self, tag: &str) -> Option<String> {
        match tag.to_uppercase().as_str() {
            "KI" | "C7" | "3SG_KI" => Some("7 (ki/vi)".into()),
            "VI" | "C8" | "3PL_VI" => Some("8 (vi)".into()),
            "LI" | "JI" | "C5" | "3SG_LI" => Some("5 (li/ji)".into()),
            "YA" | "C6" | "3PL_YA" => Some("6 (ya)".into()),
            "ZI" | "C10" | "3PL_ZI" => Some("10 (zi)".into()),
            "C9" | "C4" | "3SG_N" => Some("9/4 (i)".into()),
            "C3" | "C11" | "C14" | "3SG_U" => Some("3/11/14 (u)".into()),
            "PA" | "C16" => Some("16 (pa)".into()),
            "C15" | "C17" => Some("15/17 (ku)".into()),
            "C18" => Some("18 (mu)".into()),
            _ => None,
        }
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

fn last_vowel_is_mid(base: &str) -> bool {
    base.chars().rev().find(|c| "aeiou".contains(*c)).map_or(false, |c| c == 'e' || c == 'o')
}

/// Apply one derivational extension to a verb stem (stem includes final -a).
///
/// Rules: vowel harmony (-i- after a/i/u, -e- after e/o), passive -w-,
/// causative -ish-/-esh-, applicative -i-/-e-, stative -ik-/-ek-,
/// reciprocal -an-. A short table covers the irregular monosyllabic stems.
/// Returns None for an unknown derivation name.
pub fn derive(stem: &str, kind: &str) -> Option<String> {
    let kind = kind.to_lowercase();
    if !stem.ends_with('a') {
        // borrowed stems (jibu, subiri) take no productive derivation here
        return match kind.as_str() {
            "passive" | "applicative" | "causative" | "stative" | "reciprocal" => Some(stem.to_string()),
            _ => None,
        };
    }
    let base = &stem[..stem.len() - 1];
    let mid = last_vowel_is_mid(base);
    let (i, e) = ("i", "e");
    let h = if mid { e } else { i };
    let base_vowel_final = base.chars().last().map_or(false, |c| "aeiou".contains(c));
    let r = match kind.as_str() {
        "passive" => match stem {
            "la" => "liwa".to_string(),
            "pa" => "pewa".to_string(),
            "nywa" => "nywewa".to_string(),
            _ if base.ends_with('u') => format!("{base}liwa"),
            _ if base.ends_with('o') || base.ends_with('e') => format!("{base}lewa"),
            _ => format!("{base}wa"),
        },
        "applicative" => match stem {
            "la" => "lia".to_string(),
            "pa" => "pea".to_string(),
            _ if base_vowel_final => format!("{base}l{h}a"),
            _ => format!("{base}{h}a"),
        },
        "causative" => if base_vowel_final { format!("{base}z{}", "a") } else { format!("{base}{h}sha") },
        "stative" if base_vowel_final => format!("{base}ka"),
        "stative" => format!("{base}{h}ka"),
        "reciprocal" => format!("{base}ana"),
        _ => return None,
    };
    Some(r)
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
    fn negated_class_subjects() {
        let r = Rolodex::swahili();
        assert_eq!(r.build_verb(true, Some("C1"), Some("PAST"), None, "la", "a"), "hakula");
        assert_eq!(r.build_verb(true, Some("C7"), Some("PERF"), None, "vunja", "a"), "hakijavunja");
        assert_eq!(r.build_verb(true, Some("C9"), Some("PAST"), None, "fika", "a"), "haikufika");
        assert_eq!(r.build_verb(true, Some("C5"), Some("PRES"), None, "anguka", "a"), "halianguki");
    }

    #[test]
    fn mono_stems_and_class_i_tag() {
        let r = Rolodex::swahili();
        assert_eq!(r.build_verb(false, Some("ni"), Some("PAST"), None, "fa", "a"), "nilikufa");
        assert_eq!(r.build_verb(true, Some("tu"), Some("FUT"), None, "la", "a"), "hatutakula");
        assert_eq!(r.build_verb(true, Some("ni"), Some("PAST"), None, "la", "a"), "sikula");
        // bare "I" is the English pronoun; class 9 is C9
        assert_eq!(r.build_verb(false, Some("I"), Some("PAST"), None, "fika", "a"), "nilifika");
        assert_eq!(r.build_verb(false, Some("C9"), Some("PAST"), None, "fika", "a"), "ilifika");
    }

    #[test]
    fn derivations_with_harmony() {
        assert_eq!(derive("soma", "causative").as_deref(), Some("somesha"));
        assert_eq!(derive("fika", "applicative").as_deref(), Some("fikia"));
        assert_eq!(derive("penda", "passive").as_deref(), Some("pendwa"));
        assert_eq!(derive("ona", "passive").as_deref(), Some("onwa"));
        assert_eq!(derive("vunja", "stative").as_deref(), Some("vunjika"));
        assert_eq!(derive("piga", "reciprocal").as_deref(), Some("pigana"));
        assert_eq!(derive("soma", "bogus"), None);
    }

    #[test]
    fn irregular_imperatives() {
        let r = Rolodex::swahili();
        assert_eq!(r.build_verb(false, None, None, None, "enda", "a"), "nenda");
        assert_eq!(r.build_verb(false, None, None, None, "ja", "a"), "njoo");
        assert_eq!(r.build_verb(false, None, None, None, "la", "a"), "kula");
        assert_eq!(r.build_verb(false, None, None, None, "soma", "a"), "soma");
    }

    #[test]
    fn mono_stem_with_object_has_no_intrusive_ku() {
        let r = Rolodex::swahili();
        assert_eq!(r.build_verb(false, Some("3SG"), Some("PAST"), Some("ni"), "pa", "a"), "alinipa");
        assert_eq!(r.build_verb(false, Some("3SG"), Some("PAST"), Some("m"), "pa", "a"), "alimpa");
        assert_eq!(r.build_verb(false, Some("3SG"), Some("PRES"), None, "la", "a"), "anakula");
    }

    #[test]
    fn vowel_final_stem_derivations() {
        assert_eq!(derive("fungua", "passive").as_deref(), Some("funguliwa"));
        assert_eq!(derive("fungua", "stative").as_deref(), Some("funguka"));
        assert_eq!(derive("sikia", "passive").as_deref(), Some("sikiwa"));
        assert_eq!(derive("sikia", "stative").as_deref(), Some("sikika"));
    }

    #[test]
    fn object_tags_map_to_infixes() {
        let r = Rolodex::swahili();
        assert_eq!(r.build_verb(true, Some("3PL"), Some("FUT"), Some("2SG"), "jibu", "a"), "hawatakujibu");
        assert_eq!(r.build_verb(false, Some("2SG"), Some("PAST"), Some("3SG_KI"), "weka", "a"), "ulikiweka");
    }

    #[test]
    fn object_vowel_glide_mw() {
        let r = Rolodex::swahili();
        // si-ja-mw-ona -> sijamwona (m + vowel-initial ona -> mw)
        let result = r.build_verb(true, Some("ni"), Some("PERF"), Some("m"), "ona", "a");
        assert_eq!(result, "sijamwona");
    }
}
