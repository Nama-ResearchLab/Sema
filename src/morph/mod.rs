//! Pure-Rust data-driven Swahili morphological decomposer & round-trip verifier.
//!
//! Uses only `sw.toml` tables loaded into [`Engine`] and [`ConcordEngine`]
//! (plus an optional root inventory) -- zero hardcoded Swahili rules in Rust.

use crate::concord::ConcordEngine;
use crate::synth::{Engine, Intent};
use serde::Serialize;
use std::collections::HashSet;
use std::path::Path;

/// Grammatical category recognized by the decomposer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum MorphKind {
    FiniteVerb,
    Infinitive,
    Imperative,
    Copula,
    LocativeCopula,
    PossessiveVerb,
}

/// Complete morphological analysis of a Swahili surface word.
#[derive(Debug, Clone, Serialize)]
pub struct Decomposition {
    pub word: String,
    pub kind: MorphKind,
    pub intent: Option<Intent>,
    pub surface_roundtrip: Option<String>,
    pub roundtrip_exact: bool,
    pub known_root: bool,
    pub slots: Vec<(String, String)>,
    pub subject_class: Option<String>,
    pub object_class: Option<String>,
    pub gloss_summary: String,
    pub ambiguous_nonverb: bool,
}

/// Round-trip verification result for a single surface word.
#[derive(Debug, Clone, Serialize)]
pub struct RoundtripReport {
    pub input: String,
    pub ok: bool,
    pub output: Option<String>,
    pub decomposition: Option<Decomposition>,
}

/// Data-driven forward decomposer backed by [`Engine`].
#[derive(Debug, Clone)]
pub struct Decomposer {
    engine: Engine,
    concord: ConcordEngine,
    roots: HashSet<String>,
    non_verbs: HashSet<String>,
}

#[derive(Debug, Clone)]
struct Candidate {
    intent: Intent,
    kind: MorphKind,
    score: i32,
}

impl Decomposer {
    /// Build a decomposer from an [`Engine`].
    pub fn from_engine(engine: Engine) -> Self {
        let concord = ConcordEngine::from_morphology(engine.morphology().clone());
        Self {
            engine,
            concord,
            roots: HashSet::new(),
            non_verbs: HashSet::new(),
        }
    }

    /// Convenience: the built-in Swahili decomposer.
    pub fn swahili() -> Self {
        Self::from_engine(Engine::swahili())
    }

    /// Seed known verb roots for ranking ambiguous morphological splits.
    pub fn with_roots<I, S>(mut self, roots: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        for r in roots {
            let clean = r.into().trim().to_lowercase();
            if !clean.is_empty() && !clean.chars().any(char::is_whitespace) {
                self.roots.insert(clean);
            }
        }
        self
    }

    /// Load verb roots from a JSONL file containing `{"word": "..."}` or `{"w": "..."}` lines.
    pub fn load_verbs_jsonl(&mut self, path: &Path) -> std::io::Result<usize> {
        let content = std::fs::read_to_string(path)?;
        let m = self.engine.morphology();
        let mut added = 0usize;
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(trimmed)
                && let Some(w) = v["word"].as_str().or_else(|| v["w"].as_str())
            {
                let clean = w.trim().to_lowercase();
                if clean.is_empty()
                    || !clean.chars().all(|c| c.is_alphabetic())
                    || m.morph_guards.blocked_verb_roots.iter().any(|b| b == &clean)
                {
                    continue;
                }
                if clean.chars().count() <= 2
                    && !m.short_stems.stems.iter().any(|s| s == &clean)
                {
                    continue;
                }
                if self.roots.insert(clean) {
                    added += 1;
                }
            }
        }
        Ok(added)
    }

    /// Load non-verb headwords from the full bilingual dictionary JSONL so nouns,
    /// particles, and affixes are guarded against false verbal decomposition.
    pub fn load_dictionary_jsonl(&mut self, path: &Path) -> std::io::Result<usize> {
        let content = std::fs::read_to_string(path)?;
        let mut added = 0usize;
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(trimmed)
                && let Some(w) = v["word"].as_str().or_else(|| v["w"].as_str())
            {
                let pos = v["pos"]
                    .as_str()
                    .or_else(|| v["p"].as_str())
                    .unwrap_or("")
                    .trim()
                    .to_lowercase();
                if matches!(pos.as_str(), "verb" | "v" | "prefix" | "suffix" | "infix") {
                    continue;
                }
                let clean = w.trim().to_lowercase();
                if !clean.is_empty()
                    && clean.chars().all(|c| c.is_alphabetic())
                    && self.non_verbs.insert(clean)
                {
                    added += 1;
                }
            }
        }
        Ok(added)
    }

    /// Load both verb roots and non-verb headwords from a distilled Wiktionary JSONL file.
    pub fn load_distilled_lexicon(&mut self, path: &Path) -> std::io::Result<(usize, usize)> {
        let content = std::fs::read_to_string(path)?;
        let m = self.engine.morphology();
        let mut verbs_added = 0usize;
        let mut nonverbs_added = 0usize;
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(trimmed)
                && let Some(w) = v["w"].as_str().or_else(|| v["word"].as_str())
            {
                let pos = v["p"].as_str().or_else(|| v["pos"].as_str()).unwrap_or("").trim().to_lowercase();
                let clean = w.trim().to_lowercase();
                if clean.is_empty() || !clean.chars().all(|c| c.is_alphabetic()) {
                    continue;
                }
                if matches!(pos.as_str(), "verb" | "v") {
                    if !m.morph_guards.blocked_verb_roots.iter().any(|b| b == &clean)
                        && (clean.chars().count() > 2 || m.short_stems.stems.iter().any(|s| s == &clean))
                        && self.roots.insert(clean)
                    {
                        verbs_added += 1;
                    }
                } else if !matches!(pos.as_str(), "prefix" | "suffix" | "infix")
                    && self.non_verbs.insert(clean)
                {
                    nonverbs_added += 1;
                }
            }
        }
        Ok((verbs_added, nonverbs_added))
    }

    pub fn non_verb_count(&self) -> usize {
        self.non_verbs.len()
    }

    pub fn root_count(&self) -> usize {
        self.roots.len()
    }

    pub fn engine(&self) -> &Engine {
        &self.engine
    }

    /// Decompose a single surface word into its highest-scoring grammatical structure.
    pub fn decompose(&self, raw: &str) -> Option<Decomposition> {
        let clean = raw
            .trim()
            .trim_matches(|c: char| !c.is_alphabetic())
            .to_lowercase();
        if clean.is_empty() || !clean.chars().all(|c| c.is_alphabetic()) {
            return None;
        }

        let m = self.engine.morphology();
        if m.morph_guards
            .never_decompose_standalone
            .iter()
            .any(|w| w == &clean)
        {
            return None;
        }

        // 1. Check non-verbal copula / locative copula / possessive have-verb first.
        if let Some(non_verbal) = self.try_non_verbal(&clean) {
            return Some(non_verbal);
        }

        let mut raw_cands: Vec<(Intent, MorphKind)> = Vec::new();

        // 2. Irregular imperatives (singular and plural), known bare imperatives, and plural imperatives.
        for (root, surface) in &m.imperatives.irregular_sg {
            if surface == &clean {
                raw_cands.push((Intent::imperative(root), MorphKind::Imperative));
            }
        }
        if self.roots.contains(&clean) {
            raw_cands.push((Intent::imperative(&clean), MorphKind::Imperative));
        }
        for (root, surface) in &m.imperatives.irregular_pl {
            if surface == &clean {
                raw_cands.push((Intent::imperative_plural(root), MorphKind::Imperative));
            }
        }
        if !m.imperatives.plural_suffix.is_empty()
            && clean.ends_with(&m.imperatives.plural_suffix)
            && clean.len() >= m.imperatives.plural_suffix.len() + 2
        {
            let stem_part = &clean[..clean.len() - m.imperatives.plural_suffix.len()];
            self.collect_imperative_plural_roots(&mut raw_cands, stem_part);
        }

        // 3. Subjectless tenses (e.g. HAB hu-, INF ku-).
        let mut subjectless_pairs: Vec<(&str, &str)> = Vec::new();
        for ttag in &m.tense_behavior.subjectless {
            if let Some(marker) = m.tenses.get(ttag)
                && !marker.is_empty()
            {
                subjectless_pairs.push((ttag.as_str(), marker.as_str()));
            }
        }
        subjectless_pairs.sort_by_key(|a| std::cmp::Reverse(a.1.len()));
        for (ttag, marker) in subjectless_pairs {
            if let Some(after) = clean.strip_prefix(marker)
                && after.len() >= 2
            {
                let kind = if ttag == "INF" {
                    MorphKind::Infinitive
                } else {
                    MorphKind::FiniteVerb
                };
                self.collect_obj_and_root(&mut raw_cands, None, Some(ttag), false, None, after, None, kind);
            }
        }

        // 4. Negative fused subjects (si-, hu-, ha-, hatu-, ham-, hawa-).
        let mut neg_subjs: Vec<(&String, &String)> = m.subjects_neg.iter().collect();
        neg_subjs.sort_by_key(|a| std::cmp::Reverse(a.0.len()));
        for (pref, stag) in neg_subjs {
            if let Some(rest) = clean.strip_prefix(pref.as_str())
                && rest.len() >= 2
            {
                self.collect_tense_rest(
                    &mut raw_cands,
                    stag,
                    true,
                    rest,
                    &self.neg_tense_markers(),
                );
            }
        }

        // Ordered positive subject prefixes (preferring canonical 1SG..3PL and C* tags over aliases).
        let mut subj_pairs = self.ordered_subject_prefixes();

        // 5. Standalone negation lead + positive subject (e.g. ha + ki -> haki-).
        if !m.negation.lead.is_empty()
            && let Some(after_lead) = clean.strip_prefix(&m.negation.lead)
        {
            for (pref, stag) in &subj_pairs {
                if let Some(rest) = after_lead.strip_prefix(pref.as_str())
                    && rest.len() >= 2
                {
                    self.collect_tense_rest(
                        &mut raw_cands,
                        stag,
                        true,
                        rest,
                        &self.neg_tense_markers(),
                    );
                }
            }
        }

        // 6. Negative subjunctive (SUBJ + -si- + ...).
        if !m.negative_subjunctive.infix.is_empty() {
            for (pref, stag) in &subj_pairs {
                let pat = format!("{pref}{}", m.negative_subjunctive.infix);
                if let Some(rest) = clean.strip_prefix(&pat)
                    && rest.len() >= 2
                {
                    self.collect_obj_and_root(
                        &mut raw_cands,
                        Some(stag),
                        Some(&m.negative_subjunctive.tense),
                        true,
                        None,
                        rest,
                        Some(&m.negative_subjunctive.mood),
                        MorphKind::FiniteVerb,
                    );
                }
            }
        }

        // 7. Negative conditionals (positive subject + sipo/singe/singali).
        let mut neg_cond_markers: Vec<(String, String)> = Vec::new();
        for ttag in &m.tense_behavior.positive_subject_when_neg {
            if let Some(marker) = m.tense_neg.get(ttag)
                && !marker.is_empty()
            {
                neg_cond_markers.push((ttag.clone(), marker.clone()));
            }
        }
        neg_cond_markers.sort_by_key(|a| std::cmp::Reverse(a.1.len()));

        // 8. Positive subjects + positive TAMs (and negative conditionals).
        subj_pairs.sort_by_key(|a| std::cmp::Reverse(a.0.len()));
        let pos_tenses = self.pos_tense_markers();
        for (pref, stag) in &subj_pairs {
            if let Some(rest) = clean.strip_prefix(pref.as_str())
                && rest.len() >= 2
            {
                // Negative conditionals
                for (ttag, marker) in &neg_cond_markers {
                    if let Some(after) = rest.strip_prefix(marker.as_str())
                        && after.len() >= 2
                    {
                        self.collect_short_and_obj(
                            &mut raw_cands,
                            Some(stag),
                            Some(ttag),
                            true,
                            None,
                            after,
                            None,
                        );
                    }
                }
                // Positive tenses
                self.collect_tense_rest(&mut raw_cands, stag, false, rest, &pos_tenses);
            }
        }

        // 9. Score and verify round-trip identity via Engine::synthesize.
        let is_known_nonverb = self.non_verbs.contains(&clean);
        let mut best: Option<(Candidate, crate::synth::SynthesisResult, bool)> = None;
        for (intent, kind) in raw_cands {
            let syn = self.engine.synthesize(&intent);
            let exact = syn.surface == clean;
            if !exact {
                continue;
            }
            let known_root =
                self.roots.contains(&intent.root) || m.short_stems.stems.contains(&intent.root);
            if is_known_nonverb && (!known_root || intent.tense.as_deref() == Some("BARE")) {
                continue;
            }
            let score = self.score_candidate(&intent, kind, exact, known_root);
            let replace = match &best {
                None => true,
                Some((prev, _, _)) => score > prev.score,
            };
            if replace {
                best = Some((
                    Candidate {
                        intent,
                        kind,
                        score,
                    },
                    syn,
                    known_root,
                ));
            }
        }

        best.map(|(cand, syn, known_root)| {
            let summary = self.format_summary(&cand.intent, cand.kind);
            Decomposition {
                word: raw.trim().to_string(),
                kind: cand.kind,
                intent: Some(cand.intent),
                surface_roundtrip: Some(syn.surface),
                roundtrip_exact: true,
                known_root,
                slots: syn.slots,
                subject_class: syn.subject_class,
                object_class: syn.object_class,
                gloss_summary: summary,
                ambiguous_nonverb: is_known_nonverb,
            }
        })
    }

    /// Decompose `word` and verify round-trip identity back through the synthesizer.
    pub fn verify_roundtrip(&self, word: &str) -> RoundtripReport {
        let dec = self.decompose(word);
        match dec {
            Some(d) => RoundtripReport {
                input: word.to_string(),
                ok: d.roundtrip_exact,
                output: d.surface_roundtrip.clone(),
                decomposition: Some(d),
            },
            None => RoundtripReport {
                input: word.to_string(),
                ok: false,
                output: None,
                decomposition: None,
            },
        }
    }

    fn try_non_verbal(&self, clean: &str) -> Option<Decomposition> {
        let c = &self.engine.morphology().concord;
        if clean == c.copula.pos {
            return Some(self.non_verbal_decomp(clean, MorphKind::Copula, "copula", "is/are"));
        }
        if clean == c.copula.neg {
            return Some(self.non_verbal_decomp(clean, MorphKind::Copula, "copula_neg", "is/are not"));
        }
        if clean == c.copula.emph_pos {
            return Some(self.non_verbal_decomp(clean, MorphKind::Copula, "emph_copula", "indeed is"));
        }
        if clean == c.copula.emph_neg {
            return Some(self.non_verbal_decomp(clean, MorphKind::Copula, "emph_copula_neg", "is not"));
        }
        // Possessive / existential -na verbs (e.g. nina, sina, hakuna, hamna, hapana, yuko, hayuko)
        for (subj, neg_form) in &c.possessive_verb.neg_overrides {
            if neg_form == clean {
                return Some(self.non_verbal_decomp(
                    clean,
                    MorphKind::PossessiveVerb,
                    "have_neg",
                    &format!("{subj} NEG have/exist"),
                ));
            }
        }
        for subj in ["1SG", "2SG", "3SG", "1PL", "2PL", "3PL", "C16", "C17", "C18", "C7", "C8"] {
            if let Ok(pos) = self.concord.have(subj, false)
                && pos == clean
            {
                return Some(self.non_verbal_decomp(
                    clean,
                    MorphKind::PossessiveVerb,
                    "have",
                    &format!("{subj} have/exist"),
                ));
            }
        }
        for subj in [
            "1SG", "2SG", "3SG", "1PL", "2PL", "3PL", "C1", "C2", "C3", "C4", "C5", "C6", "C7",
            "C8", "C9", "C10", "C15", "C16", "C17", "C18",
        ] {
            for loc_kind in ["ko", "po", "mo"] {
                for neg in [false, true] {
                    if let Ok(form) = self.concord.locative_copula(subj, loc_kind, neg)
                        && form == clean
                    {
                        return Some(self.non_verbal_decomp(
                            clean,
                            MorphKind::LocativeCopula,
                            if neg { "loc_cop_neg" } else { "loc_cop" },
                            &format!("{subj} {}loc-{loc_kind}", if neg { "NOT " } else { "" }),
                        ));
                    }
                }
            }
        }
        None
    }

    fn non_verbal_decomp(
        &self,
        clean: &str,
        kind: MorphKind,
        slot_name: &str,
        summary: &str,
    ) -> Decomposition {
        Decomposition {
            word: clean.to_string(),
            kind,
            intent: None,
            surface_roundtrip: Some(clean.to_string()),
            roundtrip_exact: true,
            known_root: true,
            slots: vec![(slot_name.to_string(), clean.to_string())],
            subject_class: None,
            object_class: None,
            gloss_summary: summary.to_string(),
            ambiguous_nonverb: self.non_verbs.contains(clean),
        }
    }

    fn ordered_subject_prefixes(&self) -> Vec<(String, String)> {
        let priority = [
            "1SG", "2SG", "3SG", "1PL", "2PL", "3PL", "C3", "C4", "C5", "C6", "C7", "C8", "C9",
            "C10", "C11", "C14", "C15", "C16", "C17", "C18", "KI", "LI", "YA", "PA", "ZI",
        ];
        let subjects = &self.engine.morphology().subjects;
        let mut by_pref: std::collections::HashMap<String, String> =
            std::collections::HashMap::new();
        for tag in priority {
            if let Some(pref) = subjects.get(tag) {
                by_pref.entry(pref.clone()).or_insert_with(|| tag.to_string());
            }
        }
        for (tag, pref) in subjects {
            by_pref.entry(pref.clone()).or_insert_with(|| tag.clone());
        }
        by_pref.into_iter().collect()
    }

    fn pos_tense_markers(&self) -> Vec<(String, String)> {
        let m = self.engine.morphology();
        let mut list: Vec<(String, String)> = m
            .tenses
            .iter()
            .filter(|(k, v)| {
                !v.is_empty()
                    && !m
                        .tense_behavior
                        .subjectless
                        .iter()
                        .any(|s| s.eq_ignore_ascii_case(k))
                    && !m
                        .tense_behavior
                        .positive_subject_when_neg
                        .iter()
                        .any(|s| s.eq_ignore_ascii_case(k) && k.ends_with("_NEG"))
            })
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        // Relative future override marker (e.g. "taka" -> "FUT").
        if !m.relative_rules.future_rel_marker.is_empty() {
            list.push((
                "FUT".to_string(),
                m.relative_rules.future_rel_marker.clone(),
            ));
        }
        list.sort_by_key(|a| std::cmp::Reverse(a.1.len()));
        list
    }

    fn neg_tense_markers(&self) -> Vec<(String, String)> {
        let m = self.engine.morphology();
        let mut list: Vec<(String, String)> = m
            .tense_neg
            .iter()
            .filter(|(k, v)| {
                !v.is_empty()
                    && !m
                        .tense_behavior
                        .positive_subject_when_neg
                        .iter()
                        .any(|s| s.eq_ignore_ascii_case(k))
            })
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        if !m.relative_rules.future_rel_marker.is_empty() {
            list.push((
                "FUT".to_string(),
                m.relative_rules.future_rel_marker.clone(),
            ));
        }
        list.sort_by_key(|a| std::cmp::Reverse(a.1.len()));
        list
    }

    fn collect_tense_rest(
        &self,
        out: &mut Vec<(Intent, MorphKind)>,
        stag: &str,
        neg: bool,
        rest: &str,
        tense_list: &[(String, String)],
    ) {
        let m = self.engine.morphology();
        let rel_markers = self.ordered_relative_markers();

        for (ttag, marker) in tense_list {
            if let Some(after) = rest.strip_prefix(marker.as_str())
                && after.len() >= 2
            {
                let is_rel_fut = !m.relative_rules.future_rel_marker.is_empty()
                    && marker == &m.relative_rules.future_rel_marker;
                if !is_rel_fut {
                    self.collect_short_and_obj(
                        out,
                        Some(stag),
                        Some(ttag),
                        neg,
                        None,
                        after,
                        None,
                    );
                }
                // Optional Slot 2b relative marker
                for (rtag, rmark) in &rel_markers {
                    if let Some(after_rel) = after.strip_prefix(rmark.as_str())
                        && after_rel.len() >= 2
                    {
                        self.collect_short_and_obj(
                            out,
                            Some(stag),
                            Some(ttag),
                            neg,
                            Some(rtag),
                            after_rel,
                            None,
                        );
                    }
                }
            }
        }

        // Present negative (zero tense marker + final -i or borrowed ending)
        if neg {
            self.collect_obj_and_root(
                out,
                Some(stag),
                Some("PRES"),
                true,
                None,
                rest,
                None,
                MorphKind::FiniteVerb,
            );
        } else if m.tenses.contains_key("BARE") {
            // Bare positive subjunctive (a-som-e -> afike, u-subiri -> usubiri)
            if let Some(rule) = m.final_vowel.iter().find(|r| r.step == "mood" && r.mood == "e")
                && (rest.ends_with(&rule.use_vowel)
                    || m.borrowed
                        .fixed_final_vowels
                        .iter()
                        .any(|fv| rest.ends_with(fv.as_str())))
            {
                self.collect_obj_and_root(
                    out,
                    Some(stag),
                    Some("BARE"),
                    false,
                    None,
                    rest,
                    Some("e"),
                    MorphKind::FiniteVerb,
                );
            }
        }
    }

    fn ordered_relative_markers(&self) -> Vec<(String, String)> {
        let m = self.engine.morphology();
        let mut by_mark: std::collections::HashMap<String, String> =
            std::collections::HashMap::new();
        let pref_order = [
            "C1", "C2", "C3", "C4", "C5", "C6", "C7", "C8", "C9", "C10", "C11", "C14", "C15",
            "C16", "C17", "C18", "1SG", "2SG", "3SG", "1PL", "2PL", "3PL",
        ];
        for k in pref_order {
            if let Some(v) = m.relatives.get(k) {
                by_mark.entry(v.clone()).or_insert_with(|| k.to_string());
            }
        }
        for (k, v) in &m.relatives {
            by_mark.entry(v.clone()).or_insert_with(|| k.clone());
        }
        let mut list: Vec<(String, String)> =
            by_mark.into_iter().map(|(v, k)| (k, v)).collect();
        list.sort_by_key(|a| std::cmp::Reverse(a.1.len()));
        list
    }

    #[allow(clippy::too_many_arguments)]
    fn collect_short_and_obj(
        &self,
        out: &mut Vec<(Intent, MorphKind)>,
        stag: Option<&str>,
        ttag: Option<&str>,
        neg: bool,
        rel: Option<&str>,
        after: &str,
        forced_mood: Option<&str>,
    ) {
        let m = self.engine.morphology();
        let ku = &m.short_stems.morpheme;
        if !ku.is_empty()
            && let Some(after_ku) = after.strip_prefix(ku.as_str())
            && m.short_stems.stems.contains(&after_ku.to_string())
        {
            self.collect_obj_and_root(
                out,
                stag,
                ttag,
                neg,
                rel,
                after_ku,
                forced_mood,
                MorphKind::FiniteVerb,
            );
        }
        // Also strip intrusive -ku- when derived (e.g. alikulisha, alikuliwa)
        if !ku.is_empty()
            && let Some(after_ku) = after.strip_prefix(ku.as_str())
            && !after_ku.is_empty()
        {
            self.collect_roots_and_derivs(
                out,
                stag,
                ttag,
                neg,
                rel,
                None,
                after_ku,
                forced_mood,
                MorphKind::FiniteVerb,
                true,
            );
        }
        self.collect_obj_and_root(
            out,
            stag,
            ttag,
            neg,
            rel,
            after,
            forced_mood,
            MorphKind::FiniteVerb,
        );
    }

    #[allow(clippy::too_many_arguments)]
    fn collect_obj_and_root(
        &self,
        out: &mut Vec<(Intent, MorphKind)>,
        stag: Option<&str>,
        ttag: Option<&str>,
        neg: bool,
        rel: Option<&str>,
        rest: &str,
        forced_mood: Option<&str>,
        kind: MorphKind,
    ) {
        // Without object
        self.collect_roots_and_derivs(
            out,
            stag,
            ttag,
            neg,
            rel,
            None,
            rest,
            forced_mood,
            kind,
            false,
        );

        let m = self.engine.morphology();
        // Vowel glides: mu -> m before u, mw -> m before other vowel
        if !m.vowel_glide.m_before_u.is_empty()
            && let Some(after_mu) = rest.strip_prefix(&m.vowel_glide.m_before_u)
            && after_mu.starts_with('u')
            && after_mu.len() >= 2
        {
            self.collect_roots_and_derivs(
                out,
                stag,
                ttag,
                neg,
                rel,
                Some("m"),
                after_mu,
                forced_mood,
                kind,
                false,
            );
        }
        if !m.vowel_glide.m_vowel.is_empty()
            && let Some(after_mw) = rest.strip_prefix(&m.vowel_glide.m_vowel)
            && matches!(after_mw.chars().next(), Some('a' | 'e' | 'i' | 'o' | 'u'))
            && after_mw.len() >= 2
        {
            self.collect_roots_and_derivs(
                out,
                stag,
                ttag,
                neg,
                rel,
                Some("m"),
                after_mw,
                forced_mood,
                kind,
                false,
            );
        }

        let mut objs: Vec<&String> = m.objects.keys().collect();
        objs.sort_by_key(|a| std::cmp::Reverse(a.len()));
        for obj in objs {
            if let Some(after_obj) = rest.strip_prefix(obj.as_str())
                && self.plausible_stem_len(after_obj)
            {
                self.collect_roots_and_derivs(
                    out,
                    stag,
                    ttag,
                    neg,
                    rel,
                    Some(obj.as_str()),
                    after_obj,
                    forced_mood,
                    kind,
                    false,
                );
            }
        }
    }

    fn plausible_stem_len(&self, stem: &str) -> bool {
        let len = stem.chars().count();
        if len >= 3 {
            return true;
        }
        let m = self.engine.morphology();
        if m.short_stems.stems.iter().any(|s| s == stem) || self.roots.contains(stem) {
            return true;
        }
        // Two-vowel roots like oa, ua, or monosyllabic root with shifted vowel (li, pi, fi, pe, etc.)
        len == 2
            && (stem.chars().all(|c| "aeiou".contains(c))
                || m.short_stems.stems.iter().any(|s| {
                    s.len() >= 2 && s[..s.len() - 1] == stem[..stem.len() - 1]
                }))
    }

    fn collect_imperative_plural_roots(
        &self,
        out: &mut Vec<(Intent, MorphKind)>,
        stem_part: &str,
    ) {
        let mut cands = Vec::new();
        let m = self.engine.morphology();
        if stem_part.ends_with(&m.imperatives.plural_vowel) && stem_part.len() >= 2 {
            let as_a = format!("{}{}", &stem_part[..stem_part.len() - 1], m.defaults.final_vowel);
            cands.push(as_a);
        }
        cands.push(stem_part.to_string());
        for r in cands {
            out.push((Intent::imperative_plural(&r), MorphKind::Imperative));
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn collect_roots_and_derivs(
        &self,
        out: &mut Vec<(Intent, MorphKind)>,
        stag: Option<&str>,
        ttag: Option<&str>,
        neg: bool,
        rel: Option<&str>,
        obj: Option<&str>,
        stem: &str,
        forced_mood: Option<&str>,
        kind: MorphKind,
        short_only_guard: bool,
    ) {
        if stem.chars().count() < 2 {
            return;
        }
        let m = self.engine.morphology();
        let last_ch = stem.chars().last().unwrap_or('a');
        let is_borrowed_fv = m
            .borrowed
            .fixed_final_vowels
            .iter()
            .any(|v| v == &last_ch.to_string());
        let mood = if let Some(fm) = forced_mood {
            fm.to_string()
        } else if (neg && ttag == Some("PRES") && last_ch == 'i') || is_borrowed_fv {
            m.defaults.final_vowel.clone()
        } else if "aeio".contains(last_ch) {
            last_ch.to_string()
        } else {
            m.defaults.final_vowel.clone()
        };

        // Bare stem candidates
        let mut bare_roots = Vec::new();
        if last_ch == 'i' || last_ch == 'e' || last_ch == 'o' {
            let as_a = format!("{}{}", &stem[..stem.len() - last_ch.len_utf8()], "a");
            bare_roots.push(as_a);
        }
        bare_roots.push(stem.to_string());

        for r in bare_roots {
            if short_only_guard && !m.short_stems.stems.contains(&r) {
                continue;
            }
            out.push((
                Intent {
                    subject: stag.map(String::from),
                    tense: ttag.map(String::from),
                    negation: neg,
                    relative: rel.map(String::from),
                    object: obj.map(String::from),
                    root: r,
                    derivations: Vec::new(),
                    mood: mood.clone(),
                    particles: Vec::new(),
                },
                kind,
            ));
        }

        // Single-derivation stripping driven by [derivations] + [[derivation_variants]]
        let deriv_candidates = self.strip_single_derivation(stem);
        for (root_cand, dtag) in &deriv_candidates {
            if short_only_guard && !m.short_stems.stems.contains(root_cand) {
                continue;
            }
            out.push((
                Intent {
                    subject: stag.map(String::from),
                    tense: ttag.map(String::from),
                    negation: neg,
                    relative: rel.map(String::from),
                    object: obj.map(String::from),
                    root: root_cand.clone(),
                    derivations: vec![dtag.clone()],
                    mood: mood.clone(),
                    particles: Vec::new(),
                },
                kind,
            ));
        }

        // Stacked two-derivation stripping (e.g. applicative+passive: yalifikiwa, nilipigiwa)
        if !short_only_guard && let Some(base_no_fv) = self.strip_final_vowel(stem) {
            let mut tags: Vec<&String> = m.derivations.keys().collect();
            tags.sort();
            for outer_tag in &tags {
                let Some(outer_suf) = m.derivations.get(*outer_tag) else {
                    continue;
                };
                if let Some(after_outer) = base_no_fv.strip_suffix(outer_suf.as_str()) {
                    let mid_as_a = format!("{after_outer}a");
                    for (root_cand, inner_tag) in self.strip_single_derivation(&mid_as_a) {
                        out.push((
                            Intent {
                                subject: stag.map(String::from),
                                tense: ttag.map(String::from),
                                negation: neg,
                                relative: rel.map(String::from),
                                object: obj.map(String::from),
                                root: root_cand,
                                derivations: vec![inner_tag, (*outer_tag).clone()],
                                mood: mood.clone(),
                                particles: Vec::new(),
                            },
                            kind,
                        ));
                    }
                }
            }
        }
    }

    fn strip_final_vowel<'a>(&self, stem: &'a str) -> Option<&'a str> {
        let last = stem.chars().last()?;
        if "aeiou".contains(last) && stem.len() > last.len_utf8() {
            Some(&stem[..stem.len() - last.len_utf8()])
        } else {
            None
        }
    }

    fn strip_single_derivation(&self, stem: &str) -> Vec<(String, String)> {
        let m = self.engine.morphology();
        let mut results = Vec::new();
        let Some(body) = self.strip_final_vowel(stem) else {
            return results;
        };

        // Collect all (derivation_tag, suffix_without_final_vowel, keep_final_vowel, is_short)
        let mut patterns: Vec<(String, String, bool)> = Vec::new();
        for v in &m.derivation_variants {
            let clean_suf = if (v.short_only || !v.stem_exact.is_empty()) && v.suffix.ends_with('a')
            {
                v.suffix[..v.suffix.len() - 1].to_string()
            } else {
                v.suffix.clone()
            };
            if !clean_suf.is_empty() {
                patterns.push((v.derivation.clone(), clean_suf, v.keep_final_vowel));
            }
        }
        for (dtag, suf) in &m.derivations {
            if !suf.is_empty() {
                patterns.push((dtag.clone(), suf.clone(), false));
            }
        }
        patterns.sort_by_key(|a| std::cmp::Reverse(a.1.len()));

        for (dtag, suf, keep_fv) in patterns {
            if let Some(base) = body.strip_suffix(suf.as_str()) {
                if base.is_empty() {
                    continue;
                }
                if keep_fv {
                    results.push((base.to_string(), dtag.clone()));
                    continue;
                }
                if m.short_stems.stems.iter().any(|s| s == &format!("{base}a")) {
                    results.push((format!("{base}a"), dtag.clone()));
                }
                // Try regular -a ending and borrowed endings (-u, -i, -e)
                for tail in ["a", "u", "i", "e"] {
                    let cand = format!("{base}{tail}");
                    if cand.chars().count() >= 2 {
                        results.push((cand, dtag.clone()));
                    }
                }
            }
        }
        results
    }

    fn score_candidate(
        &self,
        intent: &Intent,
        kind: MorphKind,
        exact: bool,
        known_root: bool,
    ) -> i32 {
        let mut s = 0i32;
        if exact {
            s += 1000;
        }
        if known_root {
            s += 400;
        }
        match kind {
            MorphKind::FiniteVerb | MorphKind::Infinitive => s += 60,
            MorphKind::Imperative => s += 50,
            _ => {}
        }
        if intent.subject.is_some() {
            s += 30;
        }
        if intent.tense.is_some() {
            s += 30;
        }
        if intent.relative.is_some() {
            s += 50;
        }
        if intent.object.is_some() {
            s += 45;
        }
        if !intent.derivations.is_empty() {
            s += 35 * (intent.derivations.len() as i32);
        }
        // Prefer standard citation endings (-a or borrowed -i/-u/-e) over consonant-only stems
        if intent
            .root
            .chars()
            .last()
            .map(|c| "aeiou".contains(c))
            .unwrap_or(false)
        {
            s += 20;
        }
        s -= (intent.root.chars().count() as i32) * 3;
        s
    }

    fn format_summary(&self, intent: &Intent, kind: MorphKind) -> String {
        let subj = intent.subject.as_deref().unwrap_or("-");
        let neg = if intent.negation { "NEG " } else { "" };
        let tense = intent.tense.as_deref().unwrap_or("IMP");
        let rel = intent
            .relative
            .as_deref()
            .map(|r| format!(" REL:{r}"))
            .unwrap_or_default();
        let obj = intent
            .object
            .as_deref()
            .map(|o| format!(" OBJ:{o}"))
            .unwrap_or_default();
        let derivs = if intent.derivations.is_empty() {
            String::new()
        } else {
            format!(" +[{}]", intent.derivations.join(","))
        };
        format!(
            "{:?}: {} {}{}{}{} root:{}{}",
            kind, subj, neg, tense, rel, obj, intent.root, derivs
        )
    }
}
