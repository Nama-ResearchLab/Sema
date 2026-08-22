//! Render layer: map English model output back to Swahili via Sema's gloss table.
//!
//! After the model reasons in English, this module finds Swahili equivalents
//! for key content words and produces a bilingual response. The model never
//! back-translates prose — Sema handles the rendering.

use sema::{Lexicon, Skeleton};
use std::collections::HashMap;
use std::sync::Arc;

/// Builds a reverse index: English gloss -> Swahili lemma(s).
pub struct RenderIndex {
    /// English lowercase gloss -> Vec of (swahili lemma, POS)
    eng_to_sw: HashMap<String, Vec<(String, String)>>,
}

impl RenderIndex {
    /// Build from a loaded lexicon.
    pub fn from_lexicon(lex: &Lexicon) -> Self {
        let mut eng_to_sw: HashMap<String, Vec<(String, String)>> = HashMap::new();
        // Lexicon doesn't expose entries() directly — we need to iterate
        // via a known approach. For now, use the public API.
        // We'll build this from skeleton_for calls during render instead.
        Self { eng_to_sw }
    }

    /// Build by scanning a word list against the lexicon.
    pub fn from_word_pairs(pairs: &[(String, String)], lex: &Lexicon) -> Self {
        let mut eng_to_sw: HashMap<String, Vec<(String, String)>> = HashMap::new();
        for (sw_word, eng_word) in pairs {
            if let Some(sk) = lex.skeleton_for(sw_word) {
                let gloss_lower = eng_word.to_lowercase();
                eng_to_sw
                    .entry(gloss_lower)
                    .or_default()
                    .push((sk.lemma, sk.pos));
            }
        }
        Self { eng_to_sw }
    }

    /// Find Swahili equivalents for English content words.
    pub fn find_swahili(&self, english_word: &str) -> Option<&Vec<(String, String)>> {
        self.eng_to_sw.get(&english_word.to_lowercase())
    }
}

/// Produces a bilingual output from model's English response.
pub fn render_bilingual(
    model_output: &str,
    anchor_summary: &str,
) -> String {
    // For now, render as: English response + anchor summary
    // Full render would map content words back to Swahili via RenderIndex
    format!("{}\n\n[Swahili context: {}]", model_output, anchor_summary)
}
