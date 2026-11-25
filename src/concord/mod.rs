//! Non-verbal Swahili concord, copula, and locative engine.
//!
//! All concords, copulas, possessives, demonstratives, associatives,
//! adjectives, and numerals are driven strictly by `sw.toml` `[concord.*]`
//! tables -- zero hardcoded Swahili lives in Rust.

use crate::synth::Morphology;

/// Data-driven concord and copula synthesizer.
#[derive(Debug, Clone)]
pub struct ConcordEngine {
    m: Morphology,
}

impl ConcordEngine {
    /// Build from loaded morphology tables.
    pub fn from_morphology(m: Morphology) -> Self {
        Self { m }
    }

    /// Convenience: the built-in Swahili concord engine.
    pub fn swahili() -> Self {
        Self::from_morphology(Morphology::swahili())
    }

    /// Read-only access to the underlying morphology tables.
    pub fn morphology(&self) -> &Morphology {
        &self.m
    }

    /// Normalize a class/concord tag to canonical form (e.g. `KI -> C7`, `c8 -> C8`).
    pub fn normalize_class(&self, class: &str) -> String {
        let up = class.trim().to_uppercase();
        if self.m.concord.associative.contains_key(&up)
            || self.m.concord.locative_subjects.contains_key(&up)
        {
            return up;
        }
        match up.as_str() {
            "NI" => "1SG".into(),
            "U" => "2SG".into(),
            "A" => "3SG".into(),
            "TU" => "1PL".into(),
            "M" => "2PL".into(),
            "WA" => "3PL".into(),
            "LI" => "C5".into(),
            "YA" => "C6".into(),
            "PA" => "C16".into(),
            "KI" => "C7".into(),
            "VI" => "C8".into(),
            "ZI" => "C10".into(),
            "I" => "C9".into(),
            _ => up,
        }
    }

    fn normalize_owner(&self, owner: &str) -> String {
        match owner.trim().to_uppercase().as_str() {
            "NI" | "1SG" => "1SG".into(),
            "U" | "YOU" | "2SG" => "2SG".into(),
            "A" | "HE" | "SHE" | "3SG" => "3SG".into(),
            "TU" | "WE" | "1PL" => "1PL".into(),
            "M" | "YOU.PL" | "2PL" => "2PL".into(),
            "WA" | "THEY" | "3PL" => "3PL".into(),
            other => other.to_string(),
        }
    }

    /// Map a singular noun class to its plural class when `:pl` is requested.
    pub fn plural_class(&self, class: &str) -> String {
        let c = self.normalize_class(class);
        match c.as_str() {
            "C1" => "C2".into(),
            "C3" => "C4".into(),
            "C5" => "C6".into(),
            "C7" => "C8".into(),
            "C9" => "C10".into(),
            "C11" => "C10".into(),
            _ => c,
        }
    }

    /// Plain copula ("is/are" vs "is/are not"): `ni` / `si`.
    pub fn copula(&self, negation: bool) -> String {
        if negation {
            self.m.concord.copula.neg.clone()
        } else {
            self.m.concord.copula.pos.clone()
        }
    }

    /// Emphatic copula ("indeed is" vs "is not"): `ndiyo` / `siyo`.
    pub fn emphatic_copula(&self, negation: bool) -> String {
        if negation {
            self.m.concord.copula.emph_neg.clone()
        } else {
            self.m.concord.copula.emph_pos.clone()
        }
    }

    /// Locative copula: `SUBJ + -po/-ko/-mo` (e.g. `yuko`, `hayuko`, `kiko`, `viko`, `lipo`).
    pub fn locative_copula(
        &self,
        subject: &str,
        loc_kind: &str,
        negation: bool,
    ) -> Result<String, String> {
        let key = self.normalize_class(subject);
        let suffix = self
            .m
            .concord
            .locative_suffixes
            .get(&loc_kind.trim().to_lowercase())
            .ok_or_else(|| format!("unknown locative kind '{loc_kind}'"))?;
        let prefix = if negation {
            self.m
                .concord
                .locative_subjects_neg
                .get(&key)
                .ok_or_else(|| format!("unknown negative locative subject '{subject}'"))?
        } else {
            self.m
                .concord
                .locative_subjects
                .get(&key)
                .ok_or_else(|| format!("unknown locative subject '{subject}'"))?
        };
        Ok(format!("{prefix}{suffix}"))
    }

    /// Possessive/existential "have / there is" verb (`-na`): `nina`, `sina`, `hana`, `kuna`, `hakuna`, `hamna`, `hapana`.
    pub fn have(&self, subject: &str, negation: bool) -> Result<String, String> {
        let key = self.normalize_class(subject);
        let marker = &self.m.concord.possessive_verb.marker;
        if negation {
            if let Some(form) = self.m.concord.possessive_verb.neg_overrides.get(&key) {
                return Ok(form.clone());
            }
            if let Some((neg_pref, _)) = self.m.subjects_neg.iter().find(|(_, tag)| **tag == key) {
                return Ok(format!("{neg_pref}{marker}"));
            }
            let pos = self
                .m
                .subjects
                .get(&key)
                .ok_or_else(|| format!("unknown subject '{subject}'"))?;
            return Ok(format!("{}{}{}", self.m.negation.lead, pos, marker));
        }
        let pos = self
            .m
            .subjects
            .get(&key)
            .ok_or_else(|| format!("unknown subject '{subject}'"))?;
        Ok(format!("{pos}{marker}"))
    }

    /// Associative "-a" ("of") concord: `C7 -> cha`, `C8 -> vya`, `C1 -> wa`, `C9 -> ya`, `C10 -> za`.
    pub fn associative(&self, class: &str) -> Result<String, String> {
        let key = self.normalize_class(class);
        self.m
            .concord
            .associative
            .get(&key)
            .cloned()
            .ok_or_else(|| format!("unknown noun class '{class}'"))
    }

    /// Possessive pronoun with noun-class agreement: `C7 + 1SG -> changu`, `C1 + 1SG -> wangu`, `C8 + 1PL -> vyetu`.
    pub fn possessive(&self, class: &str, owner: &str) -> Result<String, String> {
        let ckey = self.normalize_class(class);
        let okey = self.normalize_owner(owner);
        let pref = self
            .m
            .concord
            .possessive_prefix
            .get(&ckey)
            .ok_or_else(|| format!("unknown noun class '{class}'"))?;
        let stem = self
            .m
            .concord
            .possessive_stems
            .get(&okey)
            .ok_or_else(|| format!("unknown possessive owner '{owner}'"))?;
        Ok(format!("{pref}{stem}"))
    }

    /// Demonstrative concord (`prox` = this/these, `dist` = that/those, `ref` = aforementioned).
    pub fn demonstrative(&self, class: &str, distance: &str) -> Result<String, String> {
        let key = self.normalize_class(class);
        let table = match distance.trim().to_lowercase().as_str() {
            "prox" | "near" | "this" | "these" => &self.m.concord.demonstrative_prox,
            "dist" | "far" | "that" | "those" => &self.m.concord.demonstrative_dist,
            "ref" | "emph" | "aforementioned" => &self.m.concord.demonstrative_ref,
            other => return Err(format!("unknown demonstrative distance '{other}'")),
        };
        table
            .get(&key)
            .cloned()
            .ok_or_else(|| format!("unknown noun class '{class}'"))
    }

    /// Adjective concord agreement: `C1 + dogo -> mdogo`, `C8 + zuri -> vizuri`, `C7 + ema -> chema`.
    pub fn adjective(&self, class: &str, stem: &str) -> Result<String, String> {
        let key = self.normalize_class(class);
        let clean = stem.trim().to_lowercase();
        if clean.is_empty() {
            return Err("empty adjective stem".into());
        }
        let override_key = format!("{key}:{clean}");
        if let Some(form) = self.m.concord.adjective_overrides.get(&override_key) {
            return Ok(form.clone());
        }
        let vowel_initial = matches!(clean.chars().next(), Some('a' | 'e' | 'i' | 'o' | 'u'));
        let pref = if vowel_initial {
            self.m
                .concord
                .adjective_prefix_vowel
                .get(&key)
                .ok_or_else(|| format!("unknown noun class '{class}'"))?
        } else {
            self.m
                .concord
                .adjective_prefix
                .get(&key)
                .ok_or_else(|| format!("unknown noun class '{class}'"))?
        };
        Ok(format!("{pref}{clean}"))
    }

    /// Numeral concord agreement: `C8 + 2 -> viwili`, `C1 + 1 -> mmoja`, `C10 + 2 -> mbili`, `C8 + 6 -> sita`.
    pub fn numeral(&self, class: &str, n: u32) -> Result<String, String> {
        let key = self.normalize_class(class);
        let n_str = n.to_string();
        let override_key = format!("{key}:{n_str}");
        if let Some(form) = self.m.concord.numeral_overrides.get(&override_key) {
            return Ok(form.clone());
        }
        if let Some(inv) = self.m.concord.numeral_invariable.get(&n_str) {
            return Ok(inv.clone());
        }
        let stem = self
            .m
            .concord
            .numeral_stems
            .get(&n_str)
            .ok_or_else(|| format!("unsupported numeral concord '{n}'"))?;
        let pref = self
            .m
            .concord
            .numeral_prefix
            .get(&key)
            .ok_or_else(|| format!("unknown noun class '{class}'"))?;
        Ok(format!("{pref}{stem}"))
    }

    /// Locative noun suffixation (`hospitali -> hospitalini`, `nyumba -> nyumbani`).
    pub fn locative_noun(&self, noun: &str) -> String {
        let clean = noun.trim();
        let suf = &self.m.concord.locative_noun.suffix;
        if clean.is_empty() || suf.is_empty() {
            return clean.to_string();
        }
        if clean.to_lowercase().ends_with(suf) {
            clean.to_string()
        } else {
            format!("{clean}{suf}")
        }
    }
}
