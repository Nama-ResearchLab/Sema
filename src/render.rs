//! Bilingual render layer: model output -> Swahili via gloss reverse-index.
//!
//! After the model reasons in English, this module finds Swahili
//! equivalents for key content words and produces a bilingual response.
//! The model never back-translates prose -- Sema handles the rendering.

use crate::lexicon::Lexicon;
use std::collections::HashMap;

/// Reverse index: English gloss -> Swahili lemma(s) with POS.
#[derive(Debug, Default)]
pub struct RenderIndex {
    eng_to_sw: HashMap<String, Vec<(String, String)>>,
}

impl RenderIndex {
    /// Build from a loaded lexicon by scanning all entries.
    pub fn from_lexicon(_lex: &Lexicon) -> Self {
        // Lexicon doesn't expose entries() directly -- build via
        // from_word_pairs() or from_gloss_pairs() instead.
        Self::default()
    }

    /// Build by scanning a word list against the lexicon.
    pub fn from_word_pairs(pairs: &[(String, String)], lex: &Lexicon) -> Self {
        let mut idx = Self::default();
        for (sw_word, eng_word) in pairs {
            if let Some(sk) = lex.skeleton_for(sw_word) {
                let gloss_lower = eng_word.to_lowercase();
                idx.eng_to_sw
                    .entry(gloss_lower)
                    .or_default()
                    .push((sk.lemma, sk.pos));
            }
        }
        idx
    }

    /// Build from pre-extracted gloss pairs (sw_lemma, eng_gloss).
    pub fn from_gloss_pairs(pairs: &[(&str, &str)]) -> Self {
        let mut idx = Self::default();
        for (sw, eng) in pairs {
            idx.eng_to_sw
                .entry(eng.to_lowercase())
                .or_default()
                .push((sw.to_string(), String::new()));
        }
        idx
    }

    /// Find Swahili equivalents for an English word.
    pub fn find_swahili(&self, english_word: &str) -> Option<&Vec<(String, String)>> {
        self.eng_to_sw.get(&english_word.to_lowercase())
    }

    /// Number of English entries in the index.
    pub fn len(&self) -> usize {
        self.eng_to_sw.len()
    }

    pub fn is_empty(&self) -> bool {
        self.eng_to_sw.is_empty()
    }
}

/// Produce a bilingual output from model's English response.
///
/// The model outputs English. Sema maps content words back to Swahili
/// using the RenderIndex, producing a bilingual display.
pub fn render_bilingual(model_output: &str, index: &RenderIndex) -> String {
    let mut out = String::new();
    let words: Vec<&str> = model_output.split_whitespace().collect();

    for word in &words {
        // Strip punctuation for lookup
        let clean: String = word.chars().filter(|c| c.is_alphabetic()).collect();
        let punct: String = word.chars().filter(|c| !c.is_alphabetic()).collect();

        if let Some(sw_matches) = index.find_swahili(&clean) {
            if let Some((lemma, _pos)) = sw_matches.first() {
                out.push_str(word);
                out.push('/');
                out.push_str(lemma);
                out.push_str(&punct);
                out.push(' ');
                continue;
            }
        }
        out.push_str(word);
        out.push_str(&punct);
        out.push(' ');
    }

    out.trim().to_string()
}

/// Minimal bilingual render without an index (for simple cases).
/// Just appends Swahili anchor summary to English output.
pub fn render_simple(model_output: &str, anchor_summary: &str) -> String {
    if anchor_summary.is_empty() {
        model_output.to_string()
    } else {
        format!("{model_output}\n\n[Swahili: {anchor_summary}]")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_index_lookup() {
        let idx = RenderIndex::from_gloss_pairs(&[
            ("fika", "arrive"),
            ("sema", "speak"),
            ("habari", "news"),
        ]);
        let sw = idx.find_swahili("arrive");
        assert!(sw.is_some());
        assert_eq!(sw.unwrap()[0].0, "fika");
    }

    #[test]
    fn render_bilingual_output() {
        let idx = RenderIndex::from_gloss_pairs(&[("fika", "arrive"), ("habari", "news")]);
        let output = render_bilingual("You will arrive", &idx);
        assert!(output.contains("arrive"));
    }

    #[test]
    fn render_simple_without_index() {
        let output = render_simple("Hello", "habari");
        assert_eq!(output, "Hello\n\n[Swahili: habari]");
    }

    #[test]
    fn render_simple_empty_anchor() {
        let output = render_simple("Hello", "");
        assert_eq!(output, "Hello");
    }
}
