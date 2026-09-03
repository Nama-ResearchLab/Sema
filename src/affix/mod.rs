//! Per-language affix rule tables.
//!
//! Adding a language = adding a TOML file (see `sw.toml`), not touching the
//! parser. Tables are `include_str!`-embedded at compile time so binaries
//! work standalone; override via [`AffixTable::from_path`] if you like.

use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Default, Deserialize)]
pub struct AffixTable {
    /// Language code, e.g. "sw".
    #[serde(default)]
    pub lang: String,
    /// Subject/agreement prefixes that may open a verb.
    #[serde(default)]
    pub subjects: Vec<String>,
    /// Tense/aspect markers that may follow a subject prefix. Empty string
    /// allowed (bare-stem forms).
    #[serde(default)]
    pub tenses: Vec<String>,
    /// Minimum stem length after stripping -- guards against nonsense.
    #[serde(default = "default_min_stem")]
    pub min_stem: usize,

    // --- v1 extended morphology (7-slot model) ---
    /// Negation prefixes (slot 1).
    #[serde(default)]
    pub negation: Vec<String>,
    /// Combined negation+subject forms: surface prefix -> subject tag.
    #[serde(default)]
    pub subjects_neg: HashMap<String, String>,
    /// Tense negation overrides.
    #[serde(default)]
    pub tense_neg: HashMap<String, String>,
    /// Object infix markers (slot 4).
    #[serde(default)]
    pub objects: HashMap<String, String>,
    /// Derivational extension flags (slot 5).
    #[serde(default)]
    pub derivations: HashMap<String, String>,
    /// Derivational extension ordering.
    #[serde(default)]
    pub deriv_order: Vec<String>,
    /// Final vowel / mood tags (slot 7).
    #[serde(default)]
    pub mood: HashMap<String, String>,
    /// Phonological smoothing flags.
    #[serde(default)]
    pub phonology: HashMap<String, bool>,
}

fn default_min_stem() -> usize {
    3
}

impl AffixTable {
    pub fn from_str(s: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(s)
    }

    pub fn from_path(p: &std::path::Path) -> std::io::Result<Self> {
        let s = std::fs::read_to_string(p)?;
        Self::from_str(&s).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }

    /// Generate candidate stems for `word`, longest strips first.
    /// This is the v0 two-slot resolver: [SUBJ][TENSE]stem
    pub fn candidates<'a>(&self, word: &'a str) -> Vec<(&'a str, usize)> {
        let mut out: Vec<(&str, usize)> = Vec::new();
        for s1 in &self.subjects {
            if !word.starts_with(s1.as_str()) || word.len() <= s1.len() {
                continue;
            }
            let after_s1 = &word[s1.len()..];
            for t in &self.tenses {
                if !after_s1.starts_with(t.as_str()) {
                    continue;
                }
                let stem = &after_s1[t.len()..];
                if stem.len() < self.min_stem {
                    continue;
                }
                out.push((stem, s1.len() + t.len()));
            }
        }
        // longest strip first
        out.sort_by(|a, b| b.1.cmp(&a.1));
        out
    }

    /// Generate candidate stems using the v1 7-slot model.
    /// Tries: [NEG][SUBJ][TENSE][OBJ]stem
    /// Returns candidates sorted by strip length (longest first).
    pub fn candidates_v1<'a>(&'a self, word: &'a str) -> Vec<MorphCandidate<'a>> {
        let mut out: Vec<MorphCandidate> = Vec::new();
        let lower = word;

        // Try combined neg+subject prefixes first
        for (neg_subj_prefix, _subj_tag) in &self.subjects_neg {
            if !lower.starts_with(neg_subj_prefix.as_str()) || lower.len() <= neg_subj_prefix.len()
            {
                continue;
            }
            let after = &lower[neg_subj_prefix.len()..];
            self.expand_after_neg_subj(word, neg_subj_prefix, after, &mut out);
        }

        // Try standalone negation + subject
        for neg in &self.negation {
            if neg.is_empty() {
                continue;
            }
            if !lower.starts_with(neg.as_str()) || lower.len() <= neg.len() {
                continue;
            }
            let after_neg = &lower[neg.len()..];
            for subj in &self.subjects {
                if subj.is_empty() || !after_neg.starts_with(subj.as_str()) {
                    continue;
                }
                let after_subj = &after_neg[subj.len()..];
                self.expand_after_subj(word, neg, subj, after_subj, &mut out);
            }
        }

        // Try no negation + subject (positive polarity)
        for subj in &self.subjects {
            if subj.is_empty() || !lower.starts_with(subj.as_str()) || lower.len() <= subj.len() {
                continue;
            }
            let after = &lower[subj.len()..];
            self.expand_after_subj(word, "", subj, after, &mut out);
        }

        // Sort by total strip length (longest first)
        out.sort_by(|a, b| b.strip_len.cmp(&a.strip_len));
        out
    }

    fn expand_after_neg_subj<'a>(
        &'a self,
        word: &'a str,
        neg_subj: &'a str,
        after: &'a str,
        out: &mut Vec<MorphCandidate<'a>>,
    ) {
        // Try tense markers
        for tense in &self.tenses {
            if !after.starts_with(tense.as_str()) {
                continue;
            }
            let after_tense = &after[tense.len()..];
            // Try object markers
            for (obj_tag, _obj_infix) in &self.objects {
                if obj_tag.is_empty() || !after_tense.starts_with(obj_tag.as_str()) {
                    continue;
                }
                let stem = &after_tense[obj_tag.len()..];
                if stem.len() >= self.min_stem {
                    out.push(MorphCandidate {
                        surface: word,
                        negation: Some(neg_subj),
                        subject: None,
                        tense: Some(tense),
                        object: Some(obj_tag),
                        stem,
                        strip_len: word.len() - stem.len(),
                    });
                }
            }
            // No object
            if after_tense.len() >= self.min_stem {
                out.push(MorphCandidate {
                    surface: word,
                    negation: Some(neg_subj),
                    subject: None,
                    tense: Some(tense),
                    object: None,
                    stem: after_tense,
                    strip_len: word.len() - after_tense.len(),
                });
            }
        }
        // No tense, just stem
        if after.len() >= self.min_stem {
            out.push(MorphCandidate {
                surface: word,
                negation: Some(neg_subj),
                subject: None,
                tense: None,
                object: None,
                stem: after,
                strip_len: word.len() - after.len(),
            });
        }
    }

    fn expand_after_subj<'a>(
        &'a self,
        word: &'a str,
        neg: &'a str,
        subj: &'a str,
        after: &'a str,
        out: &mut Vec<MorphCandidate<'a>>,
    ) {
        for tense in &self.tenses {
            if !after.starts_with(tense.as_str()) {
                continue;
            }
            let after_tense = &after[tense.len()..];
            // Try object markers
            for (obj_tag, _obj_infix) in &self.objects {
                if obj_tag.is_empty() || !after_tense.starts_with(obj_tag.as_str()) {
                    continue;
                }
                let stem = &after_tense[obj_tag.len()..];
                if stem.len() >= self.min_stem {
                    out.push(MorphCandidate {
                        surface: word,
                        negation: if neg.is_empty() { None } else { Some(neg) },
                        subject: Some(subj),
                        tense: Some(tense),
                        object: Some(obj_tag),
                        stem,
                        strip_len: word.len() - stem.len(),
                    });
                }
            }
            if after_tense.len() >= self.min_stem {
                out.push(MorphCandidate {
                    surface: word,
                    negation: if neg.is_empty() { None } else { Some(neg) },
                    subject: Some(subj),
                    tense: Some(tense),
                    object: None,
                    stem: after_tense,
                    strip_len: word.len() - after_tense.len(),
                });
            }
        }
        // No tense
        if after.len() >= self.min_stem {
            out.push(MorphCandidate {
                surface: word,
                negation: if neg.is_empty() { None } else { Some(neg) },
                subject: Some(subj),
                tense: None,
                object: None,
                stem: after,
                strip_len: word.len() - after.len(),
            });
        }
    }
}

/// A morphological decomposition candidate from the v1 affix resolver.
#[derive(Debug, Clone)]
pub struct MorphCandidate<'a> {
    pub surface: &'a str,
    pub negation: Option<&'a str>,
    pub subject: Option<&'a str>,
    pub tense: Option<&'a str>,
    pub object: Option<&'a str>,
    pub stem: &'a str,
    pub strip_len: usize,
}

/// Embedded Swahili table.
pub const SWAHILI: &str = include_str!("sw.toml");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn v0_candidates_basic() {
        let t = AffixTable::from_str(SWAHILI).unwrap();
        let c = t.candidates("umefika");
        assert!(!c.is_empty());
        assert_eq!(c[0].0, "fika");
    }

    #[test]
    fn v1_candidates_structured() {
        let t = AffixTable::from_str(SWAHILI).unwrap();
        let c = t.candidates_v1("umefika");
        assert!(!c.is_empty());
        // Best candidate should have subject=u, tense=me, stem=fika
        let best = &c[0];
        assert_eq!(best.stem, "fika");
        assert_eq!(best.subject, Some("u"));
        assert_eq!(best.tense, Some("me"));
    }

    #[test]
    fn v1_combined_neg_subject() {
        let t = AffixTable::from_str(SWAHILI).unwrap();
        let c = t.candidates_v1("hatujafika");
        assert!(!c.is_empty());
        let best = &c[0];
        assert_eq!(best.stem, "fika");
        assert!(best.negation.is_some());
        assert_eq!(best.tense, Some("ja"));
    }

    #[test]
    fn v1_with_object_infix() {
        let t = AffixTable::from_str(SWAHILI).unwrap();
        let c = t.candidates_v1("alikiomba");
        // Should find: a(li)ki(omba) -> subject=a, tense=li, object=ki, stem=omba
        let found = c.iter().find(|c| c.object == Some("ki"));
        assert!(found.is_some(), "expected object=ki candidate in {:?}", c);
        assert_eq!(found.unwrap().stem, "omba");
    }
}
