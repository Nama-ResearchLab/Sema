//! Hardened clinical & domain Fact Guardian (term, drug, brand, and quantity shielding).
//!
//! Protects numbers, decimals, fractions, ranges, spaced digits, Swahili &
//! English number words, metric units, acronyms, clinical phrases, and
//! typo-tolerant drug/brand names behind opaque `[#T1]` / `[#N1]` tokens,
//! then deterministically restores them.
//!
//! Pure stdlib Rust (zero regex, zero network, ASCII-only).

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

pub const DEFAULT_CLINICAL_TSV: &str = include_str!("../../data/terms.clinical.tsv");
pub const DEFAULT_CORE_TSV: &str = include_str!("../../data/terms.core.tsv");

/// Origin of a dictionary entry (curated overrides auto-lexicon).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TermSource {
    Curated,
    Lexicon,
}

/// Stored dictionary entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TermEntry {
    pub en: String,
    pub sw: String,
    pub sw_plural: Option<String>,
    pub class: Option<String>,
    pub kind: String,
    pub source: TermSource,
}

/// Shielded placeholder metadata recorded during `protect()`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuardianSlot {
    pub id: String,
    pub original: String,
    pub sw: String,
    pub class: Option<String>,
    pub kept: bool,
    pub auto: bool,
    pub fuzzy_from: Option<String>,
}

/// Output of `FactGuardian::protect()`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtectedText {
    pub text: String,
    pub slots: Vec<GuardianSlot>,
}

/// Restoration display mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RestoreMode {
    Swahili,
    Bilingual,
}

/// Output of `FactGuardian::restore()`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestoredText {
    pub text: String,
    pub replaced: usize,
    pub missing: Vec<String>,
    pub unknown: Vec<String>,
    pub integrity_ok: bool,
}

/// Hardened term & numeric Fact Guardian.
#[derive(Debug, Clone, Default)]
pub struct FactGuardian {
    by_en: HashMap<String, TermEntry>,
    by_sw: HashMap<String, TermEntry>,
    fuzzy_candidates: Vec<(String, TermEntry)>,
    max_words: usize,
}

const UNITS: [&str; 16] = [
    "mg", "ml", "kg", "g", "mcg", "ug", "iu", "l", "dl", "cm", "mm", "mmhg", "bpm", "c", "f", "%",
];

const SW_NUM_WORDS: [&str; 22] = [
    "sifuri", "moja", "mosi", "mbili", "pili", "tatu", "nne", "tano", "sita", "saba", "nane",
    "tisa", "kumi", "ishirini", "thelathini", "arobaini", "hamsini", "sitini", "sabini",
    "themanini", "tisini", "mia",
];

const SW_SCALE_WORDS: [&str; 3] = ["elfu", "laki", "milioni"];

const EN_NUM_WORDS: [&str; 35] = [
    "zero", "one", "two", "three", "four", "five", "six", "seven", "eight", "nine", "ten",
    "eleven", "twelve", "thirteen", "fourteen", "fifteen", "sixteen", "seventeen", "eighteen",
    "nineteen", "twenty", "thirty", "forty", "fifty", "sixty", "seventy", "eighty", "ninety",
    "hundred", "thousand", "million", "billion", "once", "twice", "thrice",
];

const FRACTION_WORDS: [&str; 6] = ["half", "quarter", "third", "nusu", "robo", "thuluthi"];

const FUZZY_STOPWORDS: [&str; 22] = [
    "children", "child", "patient", "doctor", "hospital", "clinic", "tablet", "tablets",
    "medicine", "morning", "evening", "another", "between", "because", "without", "through",
    "already", "present", "general", "parallel", "clinical", "medical",
];

const NOUN_CONTEXT: [&str; 42] = [
    "the", "a", "an", "my", "your", "his", "her", "its", "our", "their", "this", "that",
    "these", "those", "some", "any", "no", "many", "much", "every", "each", "another",
    "of", "in", "on", "at", "with", "for", "from", "by", "near", "into", "about", "new",
    "two", "three", "four", "five", "six", "seven", "eight", "nine",
];

fn norm(s: &str) -> String {
    s.trim()
        .to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Tokenize text into `(start_byte, end_byte, lowercase_token)`.
/// Keeps numeric commas (`2,5`), decimals (`2.5`), fractions (`1/2`), and
/// internal hyphens (`500-1000`, `x-ray`) intact inside a single token.
fn tokenize_words(text: &str) -> Vec<(usize, usize, String)> {
    let mut out = Vec::new();
    let mut start: Option<usize> = None;
    let chars: Vec<(usize, char)> = text.char_indices().collect();
    for (k, &(i, c)) in chars.iter().enumerate() {
        let prev = if k > 0 { Some(chars[k - 1].1) } else { None };
        let next = chars.get(k + 1).map(|&(_, n)| n);
        let numeric_sep = (c == ',' || c == '.' || c == '/')
            && start.is_some()
            && prev.map(|p| p.is_ascii_digit()).unwrap_or(false)
            && next.map(|n| n.is_ascii_digit()).unwrap_or(false);
        let word_inner = (c == '\'' || c == '-' || c == '#')
            && start.is_some()
            && next.map(|n| n.is_alphanumeric()).unwrap_or(false);
        let trailing_pct = c == '%' && start.is_some();
        if c.is_alphanumeric() || numeric_sep || word_inner || trailing_pct {
            if start.is_none() {
                start = Some(i);
            }
        } else if let Some(s) = start.take() {
            out.push((s, i, text[s..i].to_lowercase()));
        }
    }
    if let Some(s) = start {
        out.push((s, text.len(), text[s..].to_lowercase()));
    }
    out
}

fn is_sw_number_word(w: &str) -> bool {
    SW_NUM_WORDS.contains(&w) || SW_SCALE_WORDS.contains(&w)
}

fn is_en_number_word(w: &str) -> bool {
    EN_NUM_WORDS.contains(&w)
}

fn is_digit_token(w: &str) -> bool {
    w.chars().any(|c| c.is_ascii_digit())
}

fn is_acronym(raw: &str) -> bool {
    raw.len() >= 2
        && raw
            .chars()
            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
}

fn is_keep_single(raw: &str, low: &str) -> bool {
    is_digit_token(raw)
        || is_acronym(raw)
        || UNITS.contains(&low)
        || FRACTION_WORDS.contains(&low)
}

/// Optimal string alignment (restricted Damerau-Levenshtein) distance on ASCII/chars.
pub fn damerau_levenshtein(a: &str, b: &str) -> usize {
    let ac: Vec<char> = a.chars().collect();
    let bc: Vec<char> = b.chars().collect();
    let m = ac.len();
    let n = bc.len();
    if m == 0 {
        return n;
    }
    if n == 0 {
        return m;
    }
    let mut d = vec![vec![0usize; n + 1]; m + 1];
    for (i, row) in d.iter_mut().enumerate() {
        row[0] = i;
    }
    for (j, cell) in d[0].iter_mut().enumerate() {
        *cell = j;
    }
    for i in 1..=m {
        for j in 1..=n {
            let cost = usize::from(ac[i - 1] != bc[j - 1]);
            let mut best = (d[i - 1][j] + 1)
                .min(d[i][j - 1] + 1)
                .min(d[i - 1][j - 1] + cost);
            if i > 1 && j > 1 && ac[i - 1] == bc[j - 2] && ac[i - 2] == bc[j - 1] {
                best = best.min(d[i - 2][j - 2] + 1);
            }
            d[i][j] = best;
        }
    }
    d[m][n]
}

impl FactGuardian {
    /// Empty dictionary.
    pub fn new() -> Self {
        Self {
            by_en: HashMap::new(),
            by_sw: HashMap::new(),
            fuzzy_candidates: Vec::new(),
            max_words: 1,
        }
    }

    /// Build the default embedded clinical + core dictionary.
    pub fn default_clinical() -> Self {
        let mut g = Self::from_tsv_domain(DEFAULT_CLINICAL_TSV, TermSource::Curated, true);
        let core = Self::from_tsv_domain(DEFAULT_CORE_TSV, TermSource::Curated, false);
        g.merge_with_sw_flag(&core, false);
        g
    }

    /// Load a dictionary from TSV text (`en\tsw[\tplural[\tclass[\tkind]]]`).
    pub fn from_tsv(tsv: &str) -> Self {
        Self::from_tsv_domain(tsv, TermSource::Curated, true)
    }

    /// Load an auto-seeded dictionary (gated by noun context).
    pub fn from_tsv_auto(tsv: &str) -> Self {
        Self::from_tsv_domain(tsv, TermSource::Lexicon, false)
    }

    /// Load a TSV file from disk.
    pub fn from_tsv_path(path: impl AsRef<Path>) -> std::io::Result<Self> {
        Ok(Self::from_tsv(&std::fs::read_to_string(path)?))
    }

    fn from_tsv_domain(tsv: &str, source: TermSource, index_sw_reverse: bool) -> Self {
        let mut g = Self::new();
        for line in tsv.lines() {
            let line = line.trim_start_matches('\u{feff}');
            if line.trim().is_empty() || line.trim_start().starts_with('#') {
                continue;
            }
            let f: Vec<&str> = line.split('\t').collect();
            if f.len() < 2 {
                continue;
            }
            g.insert_internal(
                f[0],
                f[1],
                f.get(2).copied(),
                f.get(3).copied(),
                f.get(4).copied().unwrap_or("term"),
                source,
                index_sw_reverse,
            );
        }
        g
    }

    /// Layer multiple TSV strings; earlier layers take precedence over later ones.
    pub fn layered(tsvs: &[&str]) -> Self {
        let mut g = Self::new();
        for tsv in tsvs.iter().rev() {
            let layer = Self::from_tsv(tsv);
            g.merge(&layer);
        }
        g
    }

    pub fn len(&self) -> usize {
        self.by_en.len()
    }

    pub fn is_empty(&self) -> bool {
        self.by_en.is_empty()
    }

    pub fn insert(
        &mut self,
        en: &str,
        sw: &str,
        sw_plural: Option<&str>,
        class: Option<&str>,
        source: TermSource,
    ) {
        self.insert_with_kind(en, sw, sw_plural, class, "term", source);
    }

    pub fn insert_with_kind(
        &mut self,
        en: &str,
        sw: &str,
        sw_plural: Option<&str>,
        class: Option<&str>,
        kind: &str,
        source: TermSource,
    ) {
        self.insert_internal(en, sw, sw_plural, class, kind, source, true);
    }

    #[allow(clippy::too_many_arguments)]
    fn insert_internal(
        &mut self,
        en: &str,
        sw: &str,
        sw_plural: Option<&str>,
        class: Option<&str>,
        kind: &str,
        source: TermSource,
        index_sw_reverse: bool,
    ) {
        let key = norm(en);
        if key.is_empty() || sw.trim().is_empty() {
            return;
        }
        if let Some(old) = self.by_en.get(&key)
            && old.source == TermSource::Curated
            && source == TermSource::Lexicon
        {
            return;
        }
        let clean_kind = if kind.trim().is_empty() {
            "term"
        } else {
            kind.trim()
        };
        let entry = TermEntry {
            en: key.clone(),
            sw: sw.trim().to_string(),
            sw_plural: sw_plural
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty()),
            class: class
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty()),
            kind: clean_kind.to_string(),
            source,
        };
        self.max_words = self.max_words.max(key.split(' ').count()).min(10);
        self.by_en.insert(key.clone(), entry.clone());

        if index_sw_reverse {
            let sw_key = norm(&entry.sw);
            if !sw_key.is_empty() {
                self.max_words = self.max_words.max(sw_key.split(' ').count()).min(10);
                self.by_sw.entry(sw_key).or_insert_with(|| entry.clone());
            }
            if let Some(pl) = &entry.sw_plural {
                let pl_key = norm(pl);
                if !pl_key.is_empty() {
                    self.max_words = self.max_words.max(pl_key.split(' ').count()).min(10);
                    self.by_sw.entry(pl_key).or_insert_with(|| entry.clone());
                }
            }
        }

        // Index drug/brand/clinical long single words for typo-tolerant shielding.
        if source == TermSource::Curated
            && !key.contains(' ')
            && key.chars().count() >= 6
            && (clean_kind == "drug" || clean_kind == "brand" || key.chars().count() >= 9)
            && !self.fuzzy_candidates.iter().any(|(k, _)| k == &key)
        {
            self.fuzzy_candidates.push((key, entry));
        }
    }

    fn merge_with_sw_flag(&mut self, other: &FactGuardian, index_sw_reverse: bool) {
        for entry in other.by_en.values() {
            self.insert_internal(
                &entry.en,
                &entry.sw,
                entry.sw_plural.as_deref(),
                entry.class.as_deref(),
                &entry.kind,
                entry.source,
                index_sw_reverse,
            );
        }
    }

    /// Merge another dictionary into `self`; curated beats lexicon.
    pub fn merge(&mut self, other: &FactGuardian) {
        self.merge_with_sw_flag(other, true);
    }

    /// Look up by English or Swahili key.
    pub fn get(&self, key: &str) -> Option<&TermEntry> {
        let k = norm(key);
        self.by_en.get(&k).or_else(|| self.by_sw.get(&k))
    }

    /// Look up by exact English/Swahili key or plural-stripped fallback, returning `(entry, is_plural)`.
    pub fn lookup_with_plural(&self, key: &str) -> Option<(&TermEntry, bool)> {
        let k = norm(key);
        if let Some(e) = self.by_en.get(&k) {
            return Some((e, false));
        }
        if let Some(e) = self.by_sw.get(&k) {
            let is_pl = e
                .sw_plural
                .as_deref()
                .map(|p| norm(p) == k && norm(&e.sw) != k)
                .unwrap_or(false);
            return Some((e, is_pl));
        }
        if let Some(s) = k.strip_suffix("ies")
            && let Some(e) = self.by_en.get(&format!("{s}y"))
        {
            return Some((e, true));
        }
        if let Some(s) = k.strip_suffix("es")
            && let Some(e) = self.by_en.get(s)
        {
            return Some((e, true));
        }
        if let Some(s) = k.strip_suffix('s')
            && let Some(e) = self.by_en.get(s)
        {
            return Some((e, true));
        }
        None
    }

    /// Longest-match dictionary lookup at word index `i`.
    /// Returns `(entry, word_count, is_plural, matched_via_swahili)`.
    fn match_dict_at(
        &self,
        ws: &[(usize, usize, String)],
        text: &str,
        i: usize,
    ) -> Option<(&TermEntry, usize, bool, bool)> {
        let max = self.max_words.min(ws.len() - i);
        for len in (1..=max).rev() {
            let contiguous = (i..i + len - 1)
                .all(|k| text[ws[k].1..ws[k + 1].0].chars().all(char::is_whitespace));
            if !contiguous {
                continue;
            }
            let head: Vec<&str> = ws[i..i + len - 1].iter().map(|w| w.2.as_str()).collect();
            let last = ws[i + len - 1].2.as_str();
            let make_key = |l: &str| {
                if head.is_empty() {
                    l.to_string()
                } else {
                    format!("{} {}", head.join(" "), l)
                }
            };
            let full = make_key(last);
            if let Some(t) = self.by_en.get(&full) {
                return Some((t, len, false, false));
            }
            if let Some(t) = self.by_sw.get(&full) {
                let is_pl = t
                    .sw_plural
                    .as_deref()
                    .map(|p| norm(p) == full)
                    .unwrap_or(false);
                return Some((t, len, is_pl, true));
            }

            let mut alts = Vec::new();
            if let Some(s) = last.strip_suffix("ies") {
                alts.push(format!("{s}y"));
            }
            if let Some(s) = last.strip_suffix("es") {
                alts.push(s.to_string());
            }
            if let Some(s) = last.strip_suffix('s') {
                alts.push(s.to_string());
            }
            for alt in alts {
                if let Some(t) = self.by_en.get(&make_key(&alt)) {
                    return Some((t, len, true, false));
                }
            }
        }
        None
    }

    /// Detect multi-word numeric/quantity spans at word index `i`.
    /// Returns `Some(word_count)` if a numeric span (spaced digits, range,
    /// fraction phrase, Swahili/English number words) begins at `i`.
    fn match_numeric_span_at(&self, ws: &[(usize, usize, String)], text: &str, i: usize) -> Option<usize> {
        let w0 = ws[i].2.as_str();
        let raw0 = &text[ws[i].0..ws[i].1];

        // 1. Spaced single digits (e.g. "5 0 0", "1 0")
        if raw0.len() == 1 && raw0.chars().all(|c| c.is_ascii_digit()) {
            let mut j = i + 1;
            while j < ws.len() {
                let gap = &text[ws[j - 1].1..ws[j].0];
                let raw_j = &text[ws[j].0..ws[j].1];
                if gap.chars().all(char::is_whitespace)
                    && raw_j.len() == 1
                    && raw_j.chars().all(|c| c.is_ascii_digit())
                {
                    j += 1;
                } else {
                    break;
                }
            }
            if j - i >= 2 {
                return Some(j - i);
            }
        }

        // 2. Numeric range: "<num> - <num>" or "<num> to/hadi/mpaka <num>"
        if (is_digit_token(raw0) || is_sw_number_word(w0) || is_en_number_word(w0)) && i + 1 < ws.len() {
            let gap01 = text[ws[i].1..ws[i + 1].0].trim();
            if gap01 == "-" && is_digit_token(&text[ws[i + 1].0..ws[i + 1].1]) {
                return Some(2);
            }
            if i + 2 < ws.len()
                && text[ws[i].1..ws[i + 1].0].chars().all(char::is_whitespace)
                && text[ws[i + 1].1..ws[i + 2].0].chars().all(char::is_whitespace)
                && matches!(ws[i + 1].2.as_str(), "to" | "hadi" | "mpaka")
                && (is_digit_token(&text[ws[i + 2].0..ws[i + 2].1])
                    || is_sw_number_word(&ws[i + 2].2)
                    || is_en_number_word(&ws[i + 2].2))
            {
                return Some(3);
            }
        }

        // 3. Fraction phrases: "half a", "half an", "half of", "nusu ya", "robo ya"
        if matches!(w0, "half" | "quarter")
            && i + 1 < ws.len()
            && text[ws[i].1..ws[i + 1].0].chars().all(char::is_whitespace)
            && matches!(ws[i + 1].2.as_str(), "a" | "an" | "of")
        {
            return Some(2);
        }
        if matches!(w0, "nusu" | "robo" | "thuluthi")
            && i + 1 < ws.len()
            && text[ws[i].1..ws[i + 1].0].chars().all(char::is_whitespace)
            && ws[i + 1].2 == "ya"
        {
            return Some(2);
        }

        // 4. Swahili number-word sequence (e.g. "mia tano", "elfu moja", "ishirini na tano")
        if is_sw_number_word(w0) {
            let mut j = i + 1;
            while j < ws.len() {
                let gap = &text[ws[j - 1].1..ws[j].0];
                if !gap.chars().all(char::is_whitespace) {
                    break;
                }
                let wj = ws[j].2.as_str();
                if is_sw_number_word(wj) {
                    j += 1;
                } else if wj == "na"
                    && j + 1 < ws.len()
                    && text[ws[j].1..ws[j + 1].0].chars().all(char::is_whitespace)
                    && is_sw_number_word(&ws[j + 1].2)
                {
                    j += 2;
                } else {
                    break;
                }
            }
            return Some(j - i);
        }

        // 5. English number-word sequence (e.g. "five hundred", "two hundred and fifty")
        if is_en_number_word(w0) {
            let mut j = i + 1;
            while j < ws.len() {
                let gap = &text[ws[j - 1].1..ws[j].0];
                if !gap.chars().all(char::is_whitespace) {
                    break;
                }
                let wj = ws[j].2.as_str();
                if is_en_number_word(wj) {
                    j += 1;
                } else if wj == "and"
                    && j + 1 < ws.len()
                    && text[ws[j].1..ws[j + 1].0].chars().all(char::is_whitespace)
                    && is_en_number_word(&ws[j + 1].2)
                {
                    j += 2;
                } else {
                    break;
                }
            }
            return Some(j - i);
        }

        None
    }

    fn match_fuzzy_drug(&self, word_low: &str) -> Option<&TermEntry> {
        let wlen = word_low.chars().count();
        if wlen < 6 || FUZZY_STOPWORDS.contains(&word_low) {
            return None;
        }
        let w_first = word_low.chars().next()?;
        let w_last = word_low.chars().last()?;
        let mut best: Option<(usize, &TermEntry)> = None;
        for (key, entry) in &self.fuzzy_candidates {
            let klen = key.chars().count();
            if !key.starts_with(w_first) {
                continue;
            }
            let max_dist = if klen >= 9 && wlen >= 9 { 2 } else { 1 };
            if klen.abs_diff(wlen) > max_dist {
                continue;
            }
            // Guard: require last-char match OR second-char match to prevent unrelated hits
            let same_last = key.ends_with(w_last);
            let same_second = key.chars().nth(1) == word_low.chars().nth(1);
            if !same_last && !same_second {
                continue;
            }
            let dist = damerau_levenshtein(word_low, key);
            if dist > 0 && dist <= max_dist && dist * 4 <= klen.max(wlen) {
                match best {
                    None => best = Some((dist, entry)),
                    Some((prev_d, _)) if dist < prev_d => best = Some((dist, entry)),
                    _ => {}
                }
            }
        }
        best.map(|(_, e)| e)
    }

    /// Shield terms, drugs, brands, numbers, ranges, fractions, and units.
    pub fn protect_with(&self, text: &str, use_lexicon_entries: bool) -> ProtectedText {
        let ws = tokenize_words(text);
        let mut out = String::new();
        let mut slots: Vec<GuardianSlot> = Vec::new();
        let (mut tn, mut nn) = (0usize, 0usize);
        let mut cursor = 0usize;
        let mut i = 0usize;

        while i < ws.len() {
            let (s, e, ref wlow) = ws[i];
            let raw = &text[s..e];

            // Leave pre-existing `[#...]` tokens untouched.
            if s >= 2 && text[..s].ends_with("[#") {
                i += 1;
                continue;
            }

            // Prefer multi-word curated dictionary phrases (e.g. "three times a day", "one week")
            // before splitting numeric spans out of them.
            let dict_hit = self.match_dict_at(&ws, text, i);
            let multi_curated = matches!(dict_hit, Some((t, len, _, _)) if len >= 2 && t.source == TermSource::Curated);

            if !multi_curated && let Some(nlen) = self.match_numeric_span_at(&ws, text, i) {
                let end = ws[i + nlen - 1].1;
                let span_raw = &text[s..end];
                nn += 1;
                let id = format!("N{nn}");
                out.push_str(&text[cursor..s]);
                out.push_str(&format!("[#{id}]"));
                cursor = end;
                slots.push(GuardianSlot {
                    id,
                    original: span_raw.to_string(),
                    sw: span_raw.to_string(),
                    class: None,
                    kept: true,
                    auto: false,
                    fuzzy_from: None,
                });
                i += nlen;
                continue;
            }

            if let Some((t, len, plural, matched_sw)) = dict_hit {
                let prev_ok = i > 0
                    && text[ws[i - 1].1..s].chars().all(char::is_whitespace)
                    && NOUN_CONTEXT.contains(&ws[i - 1].2.as_str());
                if t.source == TermSource::Curated || (use_lexicon_entries && prev_ok) {
                    let end = ws[i + len - 1].1;
                    tn += 1;
                    let id = format!("T{tn}");
                    let mut sw = if matched_sw {
                        text[s..end].to_string()
                    } else if plural {
                        t.sw_plural.clone().unwrap_or_else(|| t.sw.clone())
                    } else {
                        t.sw.clone()
                    };
                    if !matched_sw && raw.chars().next().is_some_and(|c| c.is_uppercase()) {
                        let mut c = sw.chars();
                        if let Some(f) = c.next() {
                            sw = f.to_uppercase().collect::<String>() + c.as_str();
                        }
                    }
                    out.push_str(&text[cursor..s]);
                    out.push_str(&format!("[#{id}]"));
                    cursor = end;
                    slots.push(GuardianSlot {
                        id,
                        original: text[s..end].to_string(),
                        sw,
                        class: t.class.clone(),
                        kept: false,
                        auto: t.source == TermSource::Lexicon,
                        fuzzy_from: None,
                    });
                    i += len;
                    continue;
                }
            }

            // Single-token numeric literal / unit / acronym / fraction word
            if is_keep_single(raw, wlow) {
                nn += 1;
                let id = format!("N{nn}");
                out.push_str(&text[cursor..s]);
                out.push_str(&format!("[#{id}]"));
                cursor = e;
                slots.push(GuardianSlot {
                    id,
                    original: raw.to_string(),
                    sw: raw.to_string(),
                    class: None,
                    kept: true,
                    auto: false,
                    fuzzy_from: None,
                });
                i += 1;
                continue;
            }

            // Typo-tolerant clinical/drug/brand fallback
            if let Some(t) = self.match_fuzzy_drug(wlow) {
                tn += 1;
                let id = format!("T{tn}");
                let mut sw = t.sw.clone();
                if raw.chars().next().is_some_and(|c| c.is_uppercase()) {
                    let mut c = sw.chars();
                    if let Some(f) = c.next() {
                        sw = f.to_uppercase().collect::<String>() + c.as_str();
                    }
                }
                out.push_str(&text[cursor..s]);
                out.push_str(&format!("[#{id}]"));
                cursor = e;
                slots.push(GuardianSlot {
                    id,
                    original: raw.to_string(),
                    sw,
                    class: t.class.clone(),
                    kept: false,
                    auto: false,
                    fuzzy_from: Some(raw.to_string()),
                });
                i += 1;
                continue;
            }

            i += 1;
        }

        out.push_str(&text[cursor..]);
        ProtectedText { text: out, slots }
    }

    /// Shield terms using default settings (includes auto entries in noun context).
    pub fn protect(&self, text: &str) -> ProtectedText {
        self.protect_with(text, true)
    }

    /// Restore `[#T..]` and `[#N..]` placeholders (supporting `[#T1]` or `[#T1:hint]`).
    pub fn restore(
        &self,
        model_output: &str,
        p: &ProtectedText,
        mode: RestoreMode,
    ) -> RestoredText {
        let mut out = String::new();
        let mut seen: Vec<String> = Vec::new();
        let mut unknown: Vec<String> = Vec::new();
        let mut replaced = 0usize;
        let mut rest = model_output;

        while let Some(a) = rest.find("[#") {
            out.push_str(&rest[..a]);
            let after = &rest[a + 2..];
            if let Some(b) = after.find(']') {
                let raw_inside = &after[..b];
                let id = raw_inside
                    .split(':')
                    .next()
                    .unwrap_or(raw_inside)
                    .trim()
                    .to_uppercase();
                if let Some(slot) = p.slots.iter().find(|s| s.id == id) {
                    match mode {
                        RestoreMode::Swahili => out.push_str(&slot.sw),
                        RestoreMode::Bilingual if slot.kept => out.push_str(&slot.sw),
                        RestoreMode::Bilingual => {
                            out.push_str(&format!("{} ({})", slot.sw, slot.original))
                        }
                    }
                    replaced += 1;
                    if !seen.contains(&id) {
                        seen.push(id);
                    }
                } else {
                    out.push_str(&rest[a..a + 2 + b + 1]);
                    unknown.push(id);
                }
                rest = &after[b + 1..];
            } else {
                out.push_str(&rest[a..]);
                rest = "";
            }
        }
        out.push_str(rest);

        let missing: Vec<String> = p
            .slots
            .iter()
            .filter(|s| !seen.contains(&s.id))
            .map(|s| s.id.clone())
            .collect();
        let integrity_ok = missing.is_empty() && unknown.is_empty();
        RestoredText {
            text: out,
            replaced,
            missing,
            unknown,
            integrity_ok,
        }
    }

}
