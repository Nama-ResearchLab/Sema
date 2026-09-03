//! Core lexicon: distilled Wiktionary entries + resolution pipeline.

use crate::affix::AffixTable;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::BufRead;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LexEntry {
    /// lemma
    pub w: String,
    /// part of speech
    pub p: String,
    /// english glosses (form-of glosses ranked last)
    pub g: Vec<String>,
    /// derivational root, e.g. "-fika"
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub r: Option<String>,
    /// known inflected forms
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub f: Vec<String>,
}

#[derive(Debug, Default, Clone)]
pub struct Lexicon {
    entries: HashMap<String, LexEntry>,
    form_index: HashMap<String, Vec<String>>,
    affix: AffixTable,
    count: usize,
}

/// Pull the derivational root from a form-of gloss like
/// "Applicative form of -fika: to arrive at".
pub fn root_from_glosses(glosses: &[String]) -> Option<String> {
    for g in glosses {
        if let Some(pos) = g.find(" of -") {
            let rest = &g[pos + 5..];
            let end = rest
                .find(|c: char| c == ':' || c == ' ' || c == ',')
                .unwrap_or(rest.len());
            let cand = &rest[..end];
            if cand.starts_with('-') && cand.len() > 1 {
                return Some(cand.to_string());
            }
        }
        if g.starts_with('-') {
            if let Some(end) = g.find(':') {
                if end > 1
                    && g[..end]
                        .chars()
                        .all(|c| c.is_ascii_alphabetic() || c == '-')
                {
                    return Some(g[..end].to_string());
                }
            }
        }
    }
    None
}

fn is_form_of_gloss(s: &str) -> bool {
    s.contains("form of") || s.starts_with("Inflection of") || s.contains("augmentative of")
}

/// Strip Wiktionary markup: [[a|b]]->b, [[a]]->a, {{...}} dropped.
pub fn clean_wiki(s: &str) -> String {
    let ch: Vec<char> = s.chars().collect();
    let mut out = String::with_capacity(s.len());
    let mut i = 0usize;
    while i < ch.len() {
        let two = |k: usize| ch.get(k).copied().unwrap_or('\0');
        if ch[i] == '[' && two(i + 1) == '[' {
            let mut depth = 0usize;
            let mut j = i + 2;
            while j < ch.len() {
                if ch[j] == '[' && two(j + 1) == '[' {
                    depth += 1;
                    j += 2;
                } else if ch[j] == ']' && two(j + 1) == ']' {
                    if depth == 0 {
                        break;
                    }
                    depth -= 1;
                    j += 2;
                } else {
                    j += 1;
                }
            }
            if j < ch.len() {
                let inner: String = ch[i + 2..j].iter().collect();
                out.push_str(inner.rsplit('|').next().unwrap_or(""));
                i = j + 2;
                continue;
            }
        }
        if ch[i] == '{' && two(i + 1) == '{' {
            let mut depth = 0usize;
            let mut j = i + 2;
            while j < ch.len() {
                if ch[j] == '{' && two(j + 1) == '{' {
                    depth += 1;
                    j += 2;
                } else if ch[j] == '}' && two(j + 1) == '}' {
                    if depth == 0 {
                        break;
                    }
                    depth -= 1;
                    j += 2;
                } else {
                    j += 1;
                }
            }
            i = if j < ch.len() { j + 2 } else { ch.len() };
            continue;
        }
        out.push(ch[i]);
        i += 1;
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

impl Lexicon {
    /// Stream-load distilled JSONL (see `sema-distill` to produce one).
    /// Uses the embedded default language table (Swahili).
    pub fn load(path: impl AsRef<Path>) -> std::io::Result<Self> {
        let affix = AffixTable::from_str(crate::affix::SWAHILI)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        Self::load_with_affix(path, affix)
    }

    pub fn load_with_affix(path: impl AsRef<Path>, affix: AffixTable) -> std::io::Result<Self> {
        let f = std::fs::File::open(path)?;
        let r = std::io::BufReader::new(f);
        let mut lex = Self {
            affix,
            ..Default::default()
        };
        for line in r.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }
            let e: LexEntry = match serde_json::from_str(&line) {
                Ok(e) => e,
                Err(_) => continue,
            };
            let quality = e.g.iter().filter(|g| is_form_of_gloss(g)).count();
            let insert = match lex.entries.get(&e.w) {
                None => true,
                Some(old) => {
                    let old_q = old.g.iter().filter(|g| is_form_of_gloss(g)).count();
                    quality < old_q
                }
            };
            if insert {
                lex.entries.insert(e.w.clone(), e);
            }
        }
        lex.count = lex.entries.len();
        let mut form_index: HashMap<String, Vec<String>> = HashMap::new();
        for e in lex.entries.values() {
            for form in &e.f {
                let bucket = form_index.entry(form.clone()).or_default();
                if !bucket.contains(&e.w) {
                    bucket.push(e.w.clone());
                }
            }
        }
        lex.form_index = form_index;
        Ok(lex)
    }

    pub fn len(&self) -> usize {
        self.count
    }

    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// Resolve a surface word: exact lemma, then forms-index, then affix
    /// stripping. Returns (entry, resolved_via_lemma).
    pub fn lookup(&self, surface: &str) -> Option<(&LexEntry, Option<&str>)> {
        let lower = surface.to_lowercase();
        if let Some(e) = self.entries.get(&lower) {
            return Some((e, None));
        }
        if let Some(lemmas) = self.form_index.get(&lower) {
            if let Some(first) = lemmas.first() {
                return self.entries.get(first).map(|e| (e, Some(first.as_str())));
            }
        }
        let mut best: Option<(&LexEntry, usize)> = None;
        for (stem, stripped) in self.affix.candidates(&lower) {
            if let Some(e) = self.entries.get(stem) {
                if best.map(|(_, b)| stripped > b).unwrap_or(true) {
                    best = Some((e, stripped));
                }
            }
        }
        best.map(|(e, _)| (e, None))
    }

    /// English skeleton fragment for one surface word.
    pub fn skeleton_for(&self, surface: &str) -> Option<Skeleton> {
        let (e, via) = self.lookup(surface)?;
        let clean: Vec<&String> = e.g.iter().filter(|g| !is_form_of_gloss(g)).collect();
        let gloss = match clean.first() {
            Some(g) => (*g).clone(),
            None => e.g.first().cloned().unwrap_or_default(),
        };
        Some(Skeleton {
            surface: surface.to_string(),
            lemma: e.w.clone(),
            pos: e.p.clone(),
            gloss,
            root: e.r.clone(),
            resolved_via_form: via.map(|s| s.to_string()),
        })
    }
}

#[derive(Debug, Clone)]
pub struct Skeleton {
    pub surface: String,
    pub lemma: String,
    pub pos: String,
    pub gloss: String,
    pub root: Option<String>,
    pub resolved_via_form: Option<String>,
}

impl std::fmt::Display for Skeleton {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}[{}:{} '{}'",
            self.surface, self.lemma, self.pos, self.gloss
        )?;
        if let Some(r) = &self.root {
            write!(f, " root={r}")?;
        }
        if let Some(v) = &self.resolved_via_form {
            write!(f, " (via '{v}')")?;
        }
        write!(f, "]")
    }
}

/// Split a sentence into alphabetic runs / standalone non-alpha chars.
pub fn segment_words(s: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    for ch in s.chars() {
        if ch.is_alphabetic() {
            cur.push(ch);
        } else {
            if !cur.is_empty() {
                out.push(std::mem::take(&mut cur));
            }
            if !ch.is_whitespace() {
                out.push(ch.to_string());
            }
        }
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mini() -> Lexicon {
        let tmp = std::env::temp_dir().join("sema_mini_test.jsonl");
        let entries = [
            r#"{"w":"fikia","p":"verb","g":["Applicative form of -fika: to arrive at"],"r":"-fika"}"#,
            r#"{"w":"wapi","p":"adv","g":["where"]}"#,
            r#"{"w":"mtoto","p":"noun","g":["child"],"f":["watoto","mtoto mdogo"]}"#,
        ];
        std::fs::write(&tmp, entries.join("\n")).unwrap();
        let affix = AffixTable::from_str(crate::affix::SWAHILI).unwrap();
        Lexicon::load_with_affix(&tmp, affix).unwrap()
    }

    #[test]
    fn exact_and_form_and_affix_resolution() {
        let l = mini();
        assert_eq!(l.lookup("wapi").unwrap().0.w, "wapi");
        // via forms index
        assert_eq!(l.lookup("watoto").unwrap().0.w, "mtoto");
        // via affix strip: u-me-fikia -> fikia
        let (e, _) = l.lookup("umefikia").unwrap();
        assert_eq!(e.w, "fikia");
        assert_eq!(e.r.as_deref(), Some("-fika"));
    }

    #[test]
    fn skeleton_prefers_clean_gloss() {
        let l = mini();
        let sk = l.skeleton_for("umefikia").unwrap();
        assert!(sk.gloss.contains("arrive"), "got: {}", sk.gloss);
    }

    #[test]
    fn wiki_markup_cleaned() {
        assert_eq!(
            clean_wiki("[[Appendix:X|m class]] inflected form of -refu"),
            "m class inflected form of -refu"
        );
        assert_eq!(clean_wiki("a {{tpl|b}} c"), "a c");
    }

    #[test]
    fn segmenter() {
        assert_eq!(
            segment_words("habari, habari?"),
            vec!["habari", ",", "habari", "?"]
        );
    }
}
