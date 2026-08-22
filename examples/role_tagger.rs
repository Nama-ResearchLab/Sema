//! Role tagger: parse Swahili into structured roles using Sema's POS data.
//!
//! Given a Swahili sentence, segment it and tag each word with its POS
//! and English gloss. This produces structured data the model can reason
//! over — not just a word list, but a grammatical parse.

use sema::lexicon::segment_words;
use sema::{Lexicon, Skeleton};
use std::sync::Arc;

/// A single word tagged with its grammatical role.
#[derive(Debug, Clone)]
pub struct TaggedWord {
    pub surface: String,
    pub lemma: String,
    pub pos: String,
    pub gloss: String,
    pub root: Option<String>,
}

/// The full parse of a Swahili sentence.
#[derive(Debug, Clone)]
pub struct RoleTag {
    pub words: Vec<TaggedWord>,
    pub unresolved: Vec<String>,
    pub subjects: Vec<usize>,
    pub verbs: Vec<usize>,
    pub objects: Vec<usize>,
    pub modifiers: Vec<usize>,
}

impl RoleTag {
    /// Human-readable summary for the model.
    pub fn to_summary(&self) -> String {
        let mut parts = Vec::new();
        for &i in &self.subjects {
            parts.push(format!("subject: {} ({})", self.words[i].surface, self.words[i].gloss));
        }
        for &i in &self.verbs {
            parts.push(format!("verb: {} ({})", self.words[i].surface, self.words[i].gloss));
        }
        for &i in &self.objects {
            parts.push(format!("object: {} ({})", self.words[i].surface, self.words[i].gloss));
        }
        for &i in &self.modifiers {
            parts.push(format!("{}: {} ({})", self.words[i].pos, self.words[i].surface, self.words[i].gloss));
        }
        if !self.unresolved.is_empty() {
            parts.push(format!("unknown: {}", self.unresolved.join(", ")));
        }
        parts.join(" | ")
    }
}

/// Tags Swahili sentences using Sema's POS data.
pub struct RoleTagger {
    lex: Arc<Lexicon>,
}

impl RoleTagger {
    pub fn new(lex: Arc<Lexicon>) -> Self {
        Self { lex }
    }

    pub fn from_lexicon(lex: &Lexicon) -> Self {
        Self { lex: Arc::new(lex.clone()) }
    }

    /// Tag a sentence: segment, resolve each word, classify roles by POS.
    pub fn tag(&self, sentence: &str) -> RoleTag {
        let mut words = Vec::new();
        let mut unresolved = Vec::new();
        let mut subjects = Vec::new();
        let mut verbs = Vec::new();
        let mut objects = Vec::new();
        let mut modifiers = Vec::new();

        for tok in segment_words(sentence) {
            if !tok.chars().any(|c| c.is_alphabetic()) {
                continue;
            }
            match self.lex.skeleton_for(&tok) {
                Some(sk) => {
                    let idx = words.len();
                    let pos = sk.pos.clone();
                    let tag = TaggedWord {
                        surface: sk.surface,
                        lemma: sk.lemma,
                        pos: pos.clone(),
                        gloss: tidy_gloss(&sk.gloss),
                        root: sk.root,
                    };
                    words.push(tag);
                    match pos.as_str() {
                        "noun" | "pron" | "name" => {
                            if subjects.is_empty() && verbs.is_empty() {
                                subjects.push(idx);
                            } else if !verbs.is_empty() {
                                objects.push(idx);
                            } else {
                                modifiers.push(idx);
                            }
                        }
                        "verb" => verbs.push(idx),
                        "adj" | "adv" | "num" => modifiers.push(idx),
                        _ => modifiers.push(idx),
                    }
                }
                None => unresolved.push(tok),
            }
        }

        RoleTag { words, unresolved, subjects, verbs, objects, modifiers }
    }
}

fn tidy_gloss(gloss: &str) -> String {
    let t = gloss.trim();
    if let Some(pos) = t.find(':') {
        let rest = t[pos + 1..].trim();
        if !rest.is_empty() {
            return rest.to_string();
        }
    }
    let mut clean = t.to_string();
    while let Some(start) = clean.find("class_((") {
        if let Some(end) = clean[start..].find("))") {
            clean = format!("{}{}", &clean[..start], &clean[start+end+2..]);
        } else { break; }
    }
    let clean = clean.trim().to_string();
    let garbage = ["inflected form of", "infinitive of", "alternative form of",
        "first-person", "second-person", "present affirmative of",
        "positive degree present of", "plural of", "inflection of"];
    for g in &garbage {
        if clean.to_lowercase().contains(g) { return String::new(); }
    }
    if clean.starts_with('-') || clean.len() <= 1 { String::new() } else { clean }
}
