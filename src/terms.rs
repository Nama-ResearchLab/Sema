//! Term / phrase ID replacer: the non-verb half of the back pass.
//!
//! Verbs are synthesized by the back-pass engine from slots. Everything
//! else (nouns, names, fixed clinical phrases, numbers, doses, acronyms)
//! must not pass through an LLM free-hand. This module swaps those spans
//! for opaque IDs before the model sees the text, and restores them from a
//! dictionary afterwards.
//!
//!   English text --protect--> "Give [#T1] to the [#T2]"  (LLM reasons here)
//!   model output --restore--> Swahili terms dropped in verbatim
//!
//! The dictionary is a drop-in file (TSV, see `TermDict::from_tsv`), so a
//! new corpus is a data change, not a code change. Longest match wins, so a
//! whole phrase ("three times a day") is just a longer key than a noun.
//! No regex; ASCII IDs of the form `[#T12]` (terms) and `[#N3]` (kept
//! literally: numbers, units, acronyms).

use crate::backpass::{Engine, Intent};
use std::collections::HashMap;

/// Where a dictionary entry came from. Curated always beats auto.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source {
    Curated,
    Lexicon,
}

#[derive(Debug, Clone)]
pub struct Term {
    pub sw: String,
    pub sw_plural: Option<String>,
    /// Noun class tag ("C7", "C9", "C1"...), used for verb agreement.
    pub class: Option<String>,
    pub source: Source,
}

#[derive(Debug, Clone)]
pub struct Slot {
    pub id: String,
    pub en: String,
    pub sw: String,
    pub class: Option<String>,
    pub kept: bool,
    /// True when the entry was auto-seeded from the lexicon (unreviewed).
    pub auto: bool,
}

#[derive(Debug, Clone)]
pub struct Protected {
    pub text: String,
    pub slots: Vec<Slot>,
}

#[derive(Debug, Clone)]
pub struct Restored {
    pub text: String,
    pub replaced: usize,
    /// IDs that were protected but the model dropped.
    pub missing: Vec<String>,
    /// IDs in the model output that were never issued.
    pub unknown: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Swahili,
    /// "swahili (english)", for review screens.
    Bilingual,
}

#[derive(Debug, Default, Clone)]
pub struct TermDict {
    map: HashMap<String, Term>,
    max_words: usize,
}

/// Words after which an English noun is expected. Auto-built (unreviewed)
/// entries only match in this context, which blocks verb/noun homographs
/// (start, stand, return, breathes) that English POS-blind lookup would hit.
const NOUN_CONTEXT: [&str; 42] = [
    "the", "a", "an", "my", "your", "his", "her", "its", "our", "their", "this", "that",
    "these", "those", "some", "any", "no", "many", "much", "every", "each", "another",
    "of", "in", "on", "at", "with", "for", "from", "by", "near", "into", "about", "new",
    "two", "three", "four", "five", "six", "seven", "eight", "nine",
];

const UNITS: [&str; 14] = [
    "mg", "ml", "kg", "g", "mcg", "iu", "l", "cm", "mm", "mmhg", "bpm", "c", "f", "%",
];

fn norm(s: &str) -> String {
    s.trim().to_lowercase().split_whitespace().collect::<Vec<_>>().join(" ")
}

/// (start, end, lowercase) for each word; words are runs of alphanumerics
/// with inner apostrophes/hyphens/percent signs.
fn words(text: &str) -> Vec<(usize, usize, String)> {
    let mut out = Vec::new();
    let mut start: Option<usize> = None;
    let chars: Vec<(usize, char)> = text.char_indices().collect();
    for (k, &(i, c)) in chars.iter().enumerate() {
        let inner = (c == '\'' || c == '-' || c == '.' || c == '%' || c == '#')
            && start.is_some()
            && chars.get(k + 1).map_or(false, |&(_, n)| n.is_alphanumeric());
        if c.is_alphanumeric() || inner || (c == '%' && start.is_some()) {
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

fn is_keep(raw: &str) -> bool {
    let has_digit = raw.chars().any(|c| c.is_ascii_digit());
    let acronym = raw.len() >= 2 && raw.chars().all(|c| c.is_ascii_uppercase() || c.is_ascii_digit());
    has_digit || acronym || UNITS.contains(&raw.to_lowercase().as_str())
}

impl TermDict {
    pub fn new() -> Self {
        Self { map: HashMap::new(), max_words: 1 }
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    pub fn insert(&mut self, en: &str, sw: &str, sw_plural: Option<&str>, class: Option<&str>, source: Source) {
        let key = norm(en);
        if key.is_empty() || sw.trim().is_empty() {
            return;
        }
        if let Some(old) = self.map.get(&key) {
            if old.source == Source::Curated && source == Source::Lexicon {
                return;
            }
        }
        self.max_words = self.max_words.max(key.split(' ').count()).min(10);
        self.map.insert(
            key,
            Term {
                sw: sw.trim().to_string(),
                sw_plural: sw_plural.map(|s| s.trim().to_string()).filter(|s| !s.is_empty()),
                class: class.map(|s| s.trim().to_string()).filter(|s| !s.is_empty()),
                source,
            },
        );
    }

    /// Load a drop-in dictionary. Tab-separated, one entry per line:
    /// `english <TAB> swahili [<TAB> swahili plural [<TAB> class]]`.
    /// Lines starting with `#` and blank lines are ignored. The English key
    /// may be a whole phrase.
    pub fn from_tsv(tsv: &str) -> Self {
        Self::from_tsv_with(tsv, Source::Curated)
    }

    /// Same format, but entries are marked as auto-built (unreviewed): they
    /// never override curated entries and are flagged in `Slot::auto`.
    pub fn from_tsv_auto(tsv: &str) -> Self {
        Self::from_tsv_with(tsv, Source::Lexicon)
    }

    fn from_tsv_with(tsv: &str, source: Source) -> Self {
        let mut d = Self::new();
        for line in tsv.lines() {
            let line = line.trim_start_matches('\u{feff}');
            if line.trim().is_empty() || line.trim_start().starts_with('#') {
                continue;
            }
            let f: Vec<&str> = line.split('\t').collect();
            if f.len() < 2 {
                continue;
            }
            d.insert(f[0], f[1], f.get(2).copied(), f.get(3).copied(), source);
        }
        d
    }

    pub fn from_tsv_path(p: impl AsRef<std::path::Path>) -> std::io::Result<Self> {
        Ok(Self::from_tsv(&std::fs::read_to_string(p)?))
    }

    /// Auto-seed from the distilled lexicon: nouns with one short clean
    /// gloss, kept only when the English gloss maps to exactly one noun.
    /// Lower quality than a curated file (Wiktionary glosses are uneven), so
    /// curated entries always override these.
    pub fn from_distilled_nouns(path: impl AsRef<std::path::Path>) -> std::io::Result<Self> {
        let raw = std::fs::read_to_string(path)?;
        let mut count: HashMap<String, u32> = HashMap::new();
        let mut cand: Vec<(String, String)> = Vec::new();
        for line in raw.lines() {
            let Ok(v) = serde_json::from_str::<serde_json::Value>(line) else { continue };
            if v["p"] != "noun" {
                continue;
            }
            let Some(w) = v["w"].as_str() else { continue };
            let Some(g) = v["g"].get(0).and_then(|x| x.as_str()) else { continue };
            let bad = g.contains('(') || g.contains(':') || g.contains(" of ") || g.contains(',') || g.contains(';');
            let n = g.split_whitespace().count();
            if bad || n == 0 || n > 3 || g.len() > 30 {
                continue;
            }
            let k = norm(g);
            *count.entry(k.clone()).or_default() += 1;
            cand.push((k, w.to_string()));
        }
        let mut d = Self::new();
        for (en, sw) in cand {
            if count[&en] == 1 {
                d.insert(&en, &sw, None, None, Source::Lexicon);
            }
        }
        Ok(d)
    }

    /// Layer several curated files: earlier files win over later ones.
    pub fn layered_curated(files: &[&str]) -> Self {
        let mut d = Self::new();
        for f in files.iter().rev() {
            let layer = Self::from_tsv(f);
            for (k, t) in &layer.map {
                d.insert(k, &t.sw, t.sw_plural.as_deref(), t.class.as_deref(), Source::Curated);
            }
        }
        d
    }

    /// Merge `other` in; curated entries beat lexicon entries.
    pub fn merge(&mut self, other: &TermDict) {
        for (k, t) in &other.map {
            self.insert(k, &t.sw, t.sw_plural.as_deref(), t.class.as_deref(), t.source);
        }
    }

    pub fn get(&self, en: &str) -> Option<&Term> {
        self.map.get(&norm(en))
    }

    /// Longest-match lookup at word position `i`. Returns (term, words used, plural).
    fn match_at(&self, ws: &[(usize, usize, String)], text: &str, i: usize) -> Option<(&Term, usize, bool)> {
        let max = self.max_words.min(ws.len() - i);
        for len in (1..=max).rev() {
            // words must be separated by whitespace only (no punctuation)
            let contiguous = (i..i + len - 1).all(|k| text[ws[k].1..ws[k + 1].0].chars().all(char::is_whitespace));
            if !contiguous {
                continue;
            }
            let head: Vec<&str> = ws[i..i + len - 1].iter().map(|w| w.2.as_str()).collect();
            let last = ws[i + len - 1].2.as_str();
            let key = |l: &str| if head.is_empty() { l.to_string() } else { format!("{} {}", head.join(" "), l) };
            if let Some(t) = self.map.get(&key(last)) {
                return Some((t, len, false));
            }
            // English plural fallback on the last word only
            let mut alts: Vec<String> = Vec::new();
            if let Some(s) = last.strip_suffix("ies") {
                alts.push(format!("{s}y"));
            }
            if let Some(s) = last.strip_suffix("es") {
                alts.push(s.to_string());
            }
            if let Some(s) = last.strip_suffix('s') {
                alts.push(s.to_string());
            }
            for a in alts {
                if let Some(t) = self.map.get(&key(&a)) {
                    return Some((t, len, true));
                }
            }
        }
        None
    }

    /// Replace dictionary terms, phrases and literals with IDs.
    /// `use_lexicon_entries = false` ignores auto-seeded entries.
    pub fn protect_with(&self, text: &str, use_lexicon_entries: bool) -> Protected {
        let ws = words(text);
        let mut out = String::new();
        let mut slots: Vec<Slot> = Vec::new();
        let (mut tn, mut nn) = (0usize, 0usize);
        let mut cursor = 0usize;
        let mut i = 0usize;
        while i < ws.len() {
            let (s, e, _) = &ws[i];
            let raw = &text[*s..*e];
            // never touch an ID that is already there
            if *s >= 2 && text[..*s].ends_with("[#") {
                i += 1;
                continue;
            }
            if let Some((t, len, plural)) = self.match_at(&ws, text, i) {
                let prev_ok = i > 0
                    && text[ws[i - 1].1..*s].chars().all(char::is_whitespace)
                    && NOUN_CONTEXT.contains(&ws[i - 1].2.as_str());
                if t.source == Source::Curated || (use_lexicon_entries && prev_ok) {
                    let end = ws[i + len - 1].1;
                    tn += 1;
                    let id = format!("T{tn}");
                    let mut sw = if plural { t.sw_plural.clone().unwrap_or_else(|| t.sw.clone()) } else { t.sw.clone() };
                    if raw.chars().next().map_or(false, |c| c.is_uppercase()) {
                        let mut c = sw.chars();
                        if let Some(f) = c.next() {
                            sw = f.to_uppercase().collect::<String>() + c.as_str();
                        }
                    }
                    out.push_str(&text[cursor..*s]);
                    out.push_str(&format!("[#{id}]"));
                    cursor = end;
                    slots.push(Slot { id, en: text[*s..end].to_string(), sw, class: t.class.clone(), kept: false, auto: t.source == Source::Lexicon });
                    i += len;
                    continue;
                }
            }
            if is_keep(raw) {
                nn += 1;
                let id = format!("N{nn}");
                out.push_str(&text[cursor..*s]);
                out.push_str(&format!("[#{id}]"));
                cursor = *e;
                slots.push(Slot { id, en: raw.to_string(), sw: raw.to_string(), class: None, kept: true, auto: false });
            }
            i += 1;
        }
        out.push_str(&text[cursor..]);
        Protected { text: out, slots }
    }

    pub fn protect(&self, text: &str) -> Protected {
        self.protect_with(text, true)
    }

    /// Put the dictionary's Swahili back in place of the IDs.
    pub fn restore(&self, model_output: &str, p: &Protected, mode: Mode) -> Restored {
        let mut out = String::new();
        let mut seen: Vec<String> = Vec::new();
        let mut unknown: Vec<String> = Vec::new();
        let mut replaced = 0usize;
        let mut rest = model_output;
        while let Some(a) = rest.find("[#") {
            out.push_str(&rest[..a]);
            let after = &rest[a + 2..];
            if let Some(b) = after.find(']') {
                let id = after[..b].trim().to_uppercase();
                if let Some(slot) = p.slots.iter().find(|s| s.id == id) {
                    match mode {
                        Mode::Swahili => out.push_str(&slot.sw),
                        Mode::Bilingual if slot.kept => out.push_str(&slot.sw),
                        Mode::Bilingual => out.push_str(&format!("{} ({})", slot.sw, slot.en)),
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
        let missing = p.slots.iter().filter(|s| !seen.contains(&s.id)).map(|s| s.id.clone()).collect();
        Restored { text: out, replaced, missing, unknown }
    }
}

/// Render a sentence template mixing dictionary terms and synthesized verbs.
///
///   `{t:malaria}`                       dictionary term
///   `{t:tablets}`                       plural form (English plural fallback)
///   `{v:SUBJ|TENSE|NEG|OBJ|ROOT[|DERIVS[|MOOD]]}`  verb from the back-pass engine
///
/// In `SUBJ`, `@english term` takes the noun class from the dictionary, so
/// the verb agrees with the noun it renders (`@chair` -> class 7 -> ki-).
/// `-` means "none". NEG is `neg` or `-`. DERIVS is comma separated.
/// Returns Err naming the first term or tag that could not be resolved.
pub fn render_template(tpl: &str, dict: &TermDict, engine: &Engine) -> Result<String, String> {
    let mut out = String::new();
    let mut rest = tpl;
    while let Some(a) = rest.find('{') {
        out.push_str(&rest[..a]);
        let after = &rest[a + 1..];
        let b = after.find('}').ok_or("unclosed {")?;
        let body = &after[..b];
        let (kind, arg) = body.split_once(':').ok_or_else(|| format!("bad slot '{body}'"))?;
        match kind.trim() {
            "t" => {
                let p = dict.protect_with(arg.trim(), false);
                let slot = p.slots.first().ok_or_else(|| format!("no dictionary entry for '{}'", arg.trim()))?;
                out.push_str(&slot.sw);
            }
            "v" => {
                let f: Vec<&str> = arg.split('|').map(str::trim).collect();
                if f.len() < 5 {
                    return Err(format!("verb slot needs SUBJ|TENSE|NEG|OBJ|ROOT: '{arg}'"));
                }
                let none = |s: &str| if s == "-" || s.is_empty() { None } else { Some(s.to_string()) };
                let subject = match f[0].strip_prefix('@') {
                    Some(term) => {
                        let p = dict.protect_with(term, false);
                        let slot = p.slots.first().ok_or_else(|| format!("no dictionary entry for '{term}'"))?;
                        Some(slot.class.clone().ok_or_else(|| format!("'{term}' has no noun class"))?)
                    }
                    None => none(f[0]),
                };
                let intent = Intent {
                    subject,
                    tense: none(f[1]),
                    negation: f[2] == "neg",
                    object: none(f[3]),
                    root: f[4].to_string(),
                    derivations: f.get(5).and_then(|d| none(d)).map(|d| d.split(',').map(|x| x.trim().to_string()).collect()).unwrap_or_default(),
                    mood: f.get(6).and_then(|m| none(m)).unwrap_or_else(|| "a".to_string()),
                    particles: Vec::new(),
                };
                out.push_str(&engine.synthesize(&intent).surface);
            }
            other => return Err(format!("unknown slot kind '{other}'")),
        }
        rest = &after[b + 1..];
    }
    out.push_str(rest);
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dict() -> TermDict {
        TermDict::from_tsv(
            "# en\tsw\tpl\tclass\n\
             malaria\tmalaria\t\tC9\n\
             tablet\tkidonge\tvidonge\tC7\n\
             chair\tkiti\tviti\tC7\n\
             child\tmtoto\twatoto\tC1\n\
             three times a day\tmara tatu kwa siku\n\
             times\tmara\n\
             day\tsiku\n\
             blood pressure\tshinikizo la damu\t\tC5\n\
             blood\tdamu\t\tC9\n",
        )
    }

    #[test]
    fn longest_phrase_wins() {
        let p = dict().protect("Take it three times a day");
        assert_eq!(p.text, "Take it [#T1]");
        assert_eq!(p.slots[0].sw, "mara tatu kwa siku");
    }

    #[test]
    fn longer_noun_phrase_beats_shorter() {
        let p = dict().protect("check blood pressure and blood");
        assert_eq!(p.text, "check [#T1] and [#T2]");
        assert_eq!(p.slots[0].sw, "shinikizo la damu");
        assert_eq!(p.slots[1].sw, "damu");
    }

    #[test]
    fn plural_uses_plural_form() {
        let p = dict().protect("two tablets and the child");
        // "two" is not a term; "2" would be kept literally
        assert!(p.text.contains("[#T1]"));
        assert_eq!(p.slots[0].sw, "vidonge");
        assert_eq!(p.slots[1].sw, "mtoto");
    }

    #[test]
    fn numbers_units_acronyms_are_kept_literally() {
        let p = dict().protect("Give 500 mg for TB");
        assert_eq!(p.text, "Give [#N1] [#N2] for [#N3]");
        let r = dict().restore(&p.text, &p, Mode::Swahili);
        assert_eq!(r.text, "Give 500 mg for TB");
    }

    #[test]
    fn unknown_words_pass_through_and_ids_untouched() {
        let d = dict();
        let p = d.protect("hello [#T9] world malaria");
        assert!(p.text.starts_with("hello [#T9] world [#T"));
    }

    #[test]
    fn restore_reports_dropped_and_invented_ids() {
        let d = dict();
        let p = d.protect("malaria and child");
        let r = d.restore("[#T1] only [#T7]", &p, Mode::Swahili);
        assert_eq!(r.text, "malaria only [#T7]");
        assert_eq!(r.missing, vec!["T2".to_string()]);
        assert_eq!(r.unknown, vec!["T7".to_string()]);
    }

    #[test]
    fn layering_first_curated_wins_and_auto_never_overrides() {
        let mut d = TermDict::layered_curated(&["house\tnyumba\t\tC9\n", "house\tmakazi\n", "boat\tmashua\n"]);
        d.merge(&TermDict::from_tsv_auto("house\tmakao\nbicycle\tbaiskeli\n"));
        assert_eq!(d.get("house").unwrap().sw, "nyumba");
        assert_eq!(d.get("boat").unwrap().sw, "mashua");
        let p = d.protect("house near a bicycle");
        assert!(!p.slots[0].auto && p.slots[1].auto);
        // ignoring auto entries leaves the unreviewed word alone
        assert_eq!(d.protect_with("house near a bicycle", false).text, "[#T1] near a bicycle");
    }

    #[test]
    fn auto_entries_need_noun_context() {
        let mut d = TermDict::new();
        d.merge(&TermDict::from_tsv_auto("start\tmwanzo\nboat\tboti\n"));
        // verb use: no match. noun use after a determiner: match.
        assert_eq!(d.protect("Please start now").text, "Please start now");
        assert_eq!(d.protect("a boat near the start").text, "a [#T1] near the [#T2]");
        // curated entries are not gated
        let mut c = TermDict::from_tsv("start\tanza\n");
        c.merge(&TermDict::from_tsv_auto("boat\tboti\n"));
        assert_eq!(c.protect("Please start now").text, "Please [#T1] now");
    }

    #[test]
    fn bilingual_mode() {
        let d = dict();
        let p = d.protect("child");
        assert_eq!(d.restore("[#T1]", &p, Mode::Bilingual).text, "mtoto (child)");
    }

    #[test]
    fn capitalisation_follows_source() {
        let d = dict();
        let p = d.protect("Malaria is bad");
        assert_eq!(p.slots[0].sw, "Malaria");
    }

    #[test]
    fn template_agreement_from_dictionary_class() {
        let d = dict();
        let e = crate::backpass::engine();
        let s = render_template("{t:chair} {v:@chair|PERF|-|-|vunja|stative}", &d, &e).unwrap();
        assert_eq!(s, "kiti kimevunjika");
        let s = render_template("{t:child} {v:@child|PAST|neg|-|la}", &d, &e).unwrap();
        assert_eq!(s, "mtoto hakula");
        assert!(render_template("{t:nonsense}", &d, &e).is_err());
    }
}
