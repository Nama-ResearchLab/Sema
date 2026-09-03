//! Forward pass: resolve Swahili surface text to telegraphic English.
//!
//! The linearizer takes a Swahili sentence and produces a lean,
//! unambiguous "telegram" of semantic content for the downstream LLM.
//!
//! It consumes [`Skeleton`] structs from the anchor layer and maps them
//! to English tokens using deterministic rules:
//!
//! - Subject markers -> pronouns (I, you, he/she, we, they)
//! - Tense markers -> auxiliaries (have, did, will, do)
//! - Root glosses -> English verbs/nouns/adjectives
//! - Object infixes -> pronouns (me, you, it, them)
//! - Negation -> "not"
//!
//! The output is a flat list of English tokens, orderable as:
//!   [SUBJ] [NEG] [TENSE] [ROOT_GLOSS] [OBJ] [EXTRAS...]

use crate::lexicon::Skeleton;

/// A single telegraphic English token with its source role.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Role {
    Subject,
    Negation,
    Tense,
    Root,
    Object,
    Modifier,
    Unknown,
}

/// A resolved English token with its grammatical role.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub word: String,
    pub role: Role,
}

impl Token {
    pub fn new(word: impl Into<String>, role: Role) -> Self {
        Self {
            word: word.into(),
            role,
        }
    }
}

/// The full linearized output for a sentence.
#[derive(Debug, Clone)]
pub struct Linearized {
    pub tokens: Vec<Token>,
    /// Raw surface text that was linearized.
    pub source: String,
}

impl Linearized {
    /// Join all tokens into a single telegraphic English string.
    pub fn to_telegram(&self) -> String {
        self.tokens
            .iter()
            .map(|t| t.word.as_str())
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Number of tokens produced.
    pub fn len(&self) -> usize {
        self.tokens.len()
    }

    pub fn is_empty(&self) -> bool {
        self.tokens.is_empty()
    }
}

/// Maps Swahili subject markers to English pronouns.
fn subject_to_english(subject: &str) -> &'static str {
    match subject {
        "ni" => "I",
        "u" => "you",
        "a" => "he",
        "tu" => "we",
        "m" => "you",
        "wa" => "they",
        "li" => "it",
        "ya" => "it",
        "pa" => "it",
        "ki" => "it",
        "ji" => "it",
        "zi" => "they",
        "i" => "it",
        _ => "it",
    }
}

/// Maps combined negation+subject prefixes to English negated pronouns.
fn neg_subject_to_english(neg_subj: &str) -> Option<&'static str> {
    match neg_subj {
        "si" => Some("I not"),
        "hu" => Some("you not"),
        "ha" => Some("he not"),
        "hatu" => Some("we not"),
        "ham" => Some("you not"),
        "hawa" => Some("they not"),
        _ => None,
    }
}

/// Maps tense markers to English tense auxiliaries or aspect words.
fn tense_to_english(tense: &str) -> Option<&'static str> {
    match tense {
        "li" => Some("did"),
        "na" => Some("is"),
        "ta" => Some("will"),
        "me" => Some("have"),
        "ka" => Some("then"),
        "hu" => Some("always"),
        "ja" => Some("not yet"),
        "ku" => None, // infinitive, often dropped in english
        "po" => Some("when"),
        "ko" => Some("at"),
        "mo" => Some("inside"),
        _ => None,
    }
}

/// Maps Swahili object infixes to English object pronouns.
fn object_to_english(obj: &str) -> Option<&'static str> {
    match obj {
        "m" => Some("me"),
        "ni" => Some("me"),
        "ku" => Some("you"),
        "wa" => Some("them"),
        "mi" => Some("me"),
        "ki" => Some("it"),
        "vi" => Some("them"),
        "yi" => Some("them"),
        "zi" => Some("them"),
        "li" => Some("it"),
        "yu" => Some("him"),
        _ => None,
    }
}

/// Clean a gloss for use as a telegraphic English token.
/// Strips "to " prefix from verb glosses, takes first sense.
pub fn clean_gloss(gloss: &str) -> String {
    let g = gloss.trim();
    // strip "to " prefix from verbs
    if let Some(rest) = g.strip_prefix("to ") {
        return first_sense(rest);
    }
    first_sense(g)
}

/// Take the first sense from a multi-sense gloss ("eat, consume" -> "eat").
fn first_sense(g: &str) -> String {
    if let Some(pos) = g.find(',') {
        g[..pos].trim().to_string()
    } else if let Some(pos) = g.find(';') {
        g[..pos].trim().to_string()
    } else {
        g.trim().to_string()
    }
}

/// Decompose an agglutinated Swahili word into its morphological parts.
///
/// Returns (negation, subject, tense, object, root_stem, raw_remainder).
/// Uses the 7-slot template: [NEG][SUBJ][TENSE][OBJ][ROOT]
pub fn decompose_verb(word: &str) -> MorphDecomp {
    let lower = word.to_lowercase();
    let w = lower.as_str();

    // Try combined neg+subject first (si-, hu-, ha-, hatu-, ham-, hawa-)
    // We split into clear negation + subject:
    //   si-   -> neg "ha"  + subj "ni"  (negation fuses)
    //   hawa- -> neg "ha"  + subj "wa"
    //   hatu- -> neg "ha"  + subj "tu"
    //   ham-  -> neg "ha"  + subj "m"
    //   hu-   -> neg "ha"  + subj "u"  (habitual neg)
    //   ha-   -> neg "ha"  + subj "a"
    let combined_neg = [
        ("hawa", "wa"),
        ("hatu", "tu"),
        ("ham", "m"),
        ("si", "ni"),
        ("hu", "u"),
        ("ha", "a"),
    ];
    for (prefix, subj) in &combined_neg {
        if w.starts_with(prefix) && w.len() > prefix.len() {
            let after = &w[prefix.len()..];
            // after subject, try tense markers, then object infix, then root
            for tense in &[
                "me", "li", "na", "ta", "ka", "hu", "ja", "ku", "po", "ko", "mo",
            ] {
                if after.starts_with(tense) && after.len() > tense.len() {
                    let stem = &after[tense.len()..];
                    if stem.len() >= 2 {
                        // try object infix after tense
                        if let Some((obj, root)) = strip_object(stem) {
                            return MorphDecomp {
                                negation: Some("ha".to_string()),
                                subject: Some((*subj).to_string()),
                                tense: Some(tense.to_string()),
                                object: Some(obj),
                                root,
                                raw: word.to_string(),
                            };
                        }
                        return MorphDecomp {
                            negation: Some("ha".to_string()),
                            subject: Some((*subj).to_string()),
                            tense: Some(tense.to_string()),
                            object: None,
                            root: stem.to_string(),
                            raw: word.to_string(),
                        };
                    }
                }
            }
            // no tense found, try object marker
            if after.len() >= 3 {
                if let Some((obj, root)) = strip_object(after) {
                    return MorphDecomp {
                        negation: Some("ha".to_string()),
                        subject: Some((*subj).to_string()),
                        tense: None,
                        object: Some(obj),
                        root,
                        raw: word.to_string(),
                    };
                }
            }
            // bare stem after neg+subj
            if after.len() >= 3 {
                return MorphDecomp {
                    negation: Some("ha".to_string()),
                    subject: Some((*subj).to_string()),
                    tense: None,
                    object: None,
                    root: after.to_string(),
                    raw: word.to_string(),
                };
            }
        }
    }

    // No combined negation: try standalone subject markers
    let subjects = [
        "ni", "tu", "wa", "zi", "ki", "ji", "li", "ya", "pa", "i", "u", "m", "a",
    ];
    for subj in &subjects {
        if w.starts_with(subj) && w.len() > subj.len() {
            let after = &w[subj.len()..];
            // try tense
            for tense in &[
                "me", "li", "na", "ta", "ka", "hu", "ja", "ku", "po", "ko", "mo",
            ] {
                if after.starts_with(tense) && after.len() > tense.len() {
                    let stem = &after[tense.len()..];
                    if stem.len() >= 2 {
                        // try object after tense
                        if let Some((obj, root)) = strip_object(stem) {
                            return MorphDecomp {
                                negation: None,
                                subject: Some(subj.to_string()),
                                tense: Some(tense.to_string()),
                                object: Some(obj),
                                root,
                                raw: word.to_string(),
                            };
                        }
                        return MorphDecomp {
                            negation: None,
                            subject: Some(subj.to_string()),
                            tense: Some(tense.to_string()),
                            object: None,
                            root: stem.to_string(),
                            raw: word.to_string(),
                        };
                    }
                }
            }
            // no tense, bare stem
            if after.len() >= 3 {
                return MorphDecomp {
                    negation: None,
                    subject: Some(subj.to_string()),
                    tense: None,
                    object: None,
                    root: after.to_string(),
                    raw: word.to_string(),
                };
            }
        }
    }

    // No decomposition possible
    MorphDecomp {
        negation: None,
        subject: None,
        tense: None,
        object: None,
        root: lower,
        raw: word.to_string(),
    }
}

fn strip_object(stem: &str) -> Option<(String, String)> {
    // Object infixes ordered by length (longest first)
    const OBJS: [&str; 11] = [
        "wa", "ki", "vi", "yi", "zi", "li", "yu", "ni", "ku", "m", "tu",
    ];
    for obj in &OBJS {
        if stem.starts_with(obj) && stem.len() > obj.len() {
            let root = &stem[obj.len()..];
            if root.len() >= 2 {
                return Some((obj.to_string(), root.to_string()));
            }
        }
    }
    None
}

/// Morphological decomposition of a Swahili verb.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MorphDecomp {
    pub negation: Option<String>,
    pub subject: Option<String>,
    pub tense: Option<String>,
    pub object: Option<String>,
    pub root: String,
    pub raw: String,
}

/// Linearize a single resolved word (Skeleton) into English tokens.
pub fn linearize_word(sk: &Skeleton) -> Vec<Token> {
    let mut tokens = Vec::new();

    // If the POS is a verb, decompose the agglutinated form
    if sk.pos == "verb" {
        let morph = decompose_verb(&sk.surface);

        // Negation
        if let Some(ref neg) = morph.negation {
            tokens.push(Token::new("not", Role::Negation));
            // Check combined neg+subject
            if let Some(ref subj) = morph.subject {
                if let Some(eng) = neg_subject_to_english(&format!("{}{}", neg, subj)) {
                    // Extract just the pronoun part
                    let parts: Vec<&str> = eng.split_whitespace().collect();
                    if let Some(pronoun) = parts.first() {
                        tokens.push(Token::new(*pronoun, Role::Subject));
                    }
                }
            }
        } else if let Some(ref subj) = morph.subject {
            tokens.push(Token::new(subject_to_english(subj), Role::Subject));
        }

        // Tense
        if let Some(ref tense) = morph.tense {
            if let Some(eng) = tense_to_english(tense) {
                tokens.push(Token::new(eng, Role::Tense));
            }
        }

        // Root gloss (from the lexicon)
        let gloss = clean_gloss(&sk.gloss);
        if !gloss.is_empty() {
            tokens.push(Token::new(gloss, Role::Root));
        }

        // Object
        if let Some(ref obj) = morph.object {
            if let Some(eng) = object_to_english(obj) {
                tokens.push(Token::new(eng, Role::Object));
            }
        }
    } else {
        // Non-verb: just emit the gloss
        let gloss = clean_gloss(&sk.gloss);
        if !gloss.is_empty() {
            let role = match sk.pos.as_str() {
                "noun" | "pron" | "name" => Role::Modifier,
                "adj" | "adv" | "num" => Role::Modifier,
                "intj" => Role::Modifier,
                _ => Role::Unknown,
            };
            tokens.push(Token::new(gloss, role));
        }
    }

    tokens
}

/// Linearize a full sentence from pre-resolved skeletons.
pub fn linearize_sentence(skeletons: &[Skeleton], source: &str) -> Linearized {
    let mut tokens = Vec::new();
    for sk in skeletons {
        tokens.extend(linearize_word(sk));
    }
    Linearized {
        tokens,
        source: source.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decompose_umefika() {
        let m = decompose_verb("umefika");
        assert_eq!(m.subject.as_deref(), Some("u"));
        assert_eq!(m.tense.as_deref(), Some("me"));
        assert_eq!(m.root, "fika");
    }

    #[test]
    fn decompose_hawatakujibu() {
        let m = decompose_verb("hawatakujibu");
        assert_eq!(m.negation.as_deref(), Some("ha"));
        assert_eq!(m.subject.as_deref(), Some("wa"));
        assert_eq!(m.tense.as_deref(), Some("ta"));
        assert_eq!(m.object.as_deref(), Some("ku"));
        assert_eq!(m.root, "jibu");
    }

    #[test]
    fn decompose_nitasema() {
        let m = decompose_verb("nitasema");
        assert_eq!(m.subject.as_deref(), Some("ni"));
        assert_eq!(m.tense.as_deref(), Some("ta"));
        assert_eq!(m.root, "sema");
    }

    #[test]
    fn subject_pronouns() {
        assert_eq!(subject_to_english("ni"), "I");
        assert_eq!(subject_to_english("u"), "you");
        assert_eq!(subject_to_english("a"), "he");
        assert_eq!(subject_to_english("wa"), "they");
    }

    #[test]
    fn tense_mapping() {
        assert_eq!(tense_to_english("li"), Some("did"));
        assert_eq!(tense_to_english("me"), Some("have"));
        assert_eq!(tense_to_english("ta"), Some("will"));
        assert_eq!(tense_to_english("na"), Some("is"));
    }

    #[test]
    fn gloss_cleaning() {
        assert_eq!(clean_gloss("to arrive at"), "arrive at");
        assert_eq!(clean_gloss("eat, consume"), "eat");
        assert_eq!(clean_gloss("run; move quickly"), "run");
    }
}
