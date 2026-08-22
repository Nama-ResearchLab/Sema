//! Per-language affix rule tables.
//!
//! Adding a language = adding a TOML file (see `sw.toml`), not touching the
//! parser. Tables are `include_str!`-embedded at compile time so binaries
//! work standalone; override via [`AffixTable::from_path`] if you like.

use serde::Deserialize;

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
    /// Minimum stem length after stripping — guards against nonsense.
    #[serde(default = "default_min_stem")]
    pub min_stem: usize,
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
}

/// Embedded Swahili table.
pub const SWAHILI: &str = include_str!("sw.toml");
