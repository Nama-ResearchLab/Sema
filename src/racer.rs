//! Best-Racer cascade: multiple lightweight parsers race, best wins.
//!
//! The racer runs 2-3 deterministic decomposition strategies in a
//! waterfall short-circuit. The first strategy that meets a quality
//! threshold wins; the heavy fallback only runs when needed.
//!
//! Strategy 0: Hash table (idioms, greetings, fixed phrases) -- O(1) lookup
//! Strategy 1: Fast heuristic (strip + gloss, ~0.15ms target)
//! Strategy 2: Full morphological decomposition (decompose_verb + affix strip)
//!
//! No LLM calls. No autoregressive generation. Pure C/Rust string ops.

use crate::lexicon::Lexicon;
use crate::linearizer::{self, Token};

/// Result from a single racer strategy.
#[derive(Debug, Clone)]
pub struct RaceResult {
    /// The English tokens produced.
    pub tokens: Vec<Token>,
    /// Quality score 0.0 - 1.0.
    pub score: f64,
    /// Which racer produced this (0, 1, or 2).
    pub racer_id: u8,
    /// Microseconds taken (for benchmarking).
    pub latency_us: f64,
}

/// The racer engine.
pub struct Racer<'a> {
    lex: &'a Lexicon,
    /// Common Swahili phrases mapped to telegraphic English.
    phrase_table: std::collections::HashMap<String, Vec<Token>>,
}

impl<'a> Racer<'a> {
    pub fn new(lex: &'a Lexicon) -> Self {
        let mut phrase_table = std::collections::HashMap::new();
        init_phrase_table(&mut phrase_table);
        Self { lex, phrase_table }
    }

    /// Run the full cascade: try each racer, return the best result.
    pub fn race(&self, word: &str) -> RaceResult {
        // Racer 0: instant hash lookup for fixed phrases
        if let Some(result) = self.racer_hash(word) {
            return result;
        }

        // Negation gate: a negated verb must never be swallowed by a bare
        // noun hit (sili -> "seal"). Checked before the heuristic racer.
        if let Some(result) = self.racer_neg_gate(word) {
            return result;
        }

        // Racer 1: fast heuristic -- gloss extraction without deep morph
        if let Some(result) = self.racer_heuristic(word) {
            if result.score >= 0.80 {
                return result;
            }
        }

        // Racer 2: full morphological decomposition (the heavy lifter)
        self.racer_full_morph(word)
    }

    /// Race a full sentence, returning the best result.
    pub fn race_sentence(&self, sentence: &str) -> Vec<RaceResult> {
        crate::lexicon::segment_words(sentence)
            .into_iter()
            .filter(|t| t.chars().any(|c| c.is_alphabetic()))
            .map(|t| self.race(&t))
            .collect()
    }

    /// Anchor a whole sentence through the cascade. Every word is kept:
    /// unresolved roots come out as `[?root]` tokens, never silently dropped.
    pub fn anchor_sentence(&self, sentence: &str) -> linearizer::Linearized {
        let mut tokens = Vec::new();
        for w in crate::lexicon::segment_words(sentence)
            .into_iter()
            .filter(|t| t.chars().any(|c| c.is_alphabetic()))
        {
            let r = self.race(&w);
            if r.tokens.is_empty() {
                tokens.push(Token::new(format!("[?{w}]"), linearizer::Role::Unknown));
            } else {
                tokens.extend(r.tokens);
            }
        }
        linearizer::Linearized {
            tokens,
            source: sentence.to_string(),
        }
    }

    /// Words the cascade could not resolve (tokens marked `[?...]`).
    pub fn unresolved(lin: &linearizer::Linearized) -> Vec<String> {
        lin.tokens
            .iter()
            .filter(|t| t.word.starts_with("[?"))
            .map(|t| t.word.trim_matches(|c| c == '[' || c == ']' || c == '?').to_string())
            .collect()
    }

    /// First lexicon verb among the candidate roots.
    fn verb_root(&self, cands: &[String]) -> Option<(String, String)> {
        for c in cands {
            if let Some(sk) = self.lex.verb_for(c) {
                return Some((c.clone(), linearizer::clean_gloss(&sk.gloss)));
            }
            if let Some(sk) = self.lex.skeleton_for(c) {
                if sk.pos == "verb" {
                    return Some((c.clone(), linearizer::clean_gloss(&sk.gloss)));
                }
            }
        }
        None
    }

    /// Negation gate: negated verb forms (hatuwezi, sili, usinywe) win over
    /// a coincidental noun hit, and the noun reading is kept as `[alt: ...]`.
    fn racer_neg_gate(&self, word: &str) -> Option<RaceResult> {
        let lower = word.to_lowercase();
        let start = std::time::Instant::now();
        // negative subjunctive / negative imperative: usinywe, msile, asitumie
        if let Some((subj, cands)) = linearizer::decompose_neg_subjunctive(&lower) {
            if let Some((_, gloss)) = self.verb_root(&cands) {
                let eng = match subj.as_str() {
                    "ni" => "I", "u" => "you", "a" => "he", "tu" => "we", "m" => "you all", "wa" => "they",
                    _ => "it",
                };
                let tokens = vec![
                    Token::new(eng, linearizer::Role::Subject),
                    Token::new("should", linearizer::Role::Tense),
                    Token::new("not", linearizer::Role::Negation),
                    Token::new(gloss, linearizer::Role::Root),
                ];
                return Some(RaceResult { tokens, score: 0.9, racer_id: 2, latency_us: start.elapsed().as_secs_f64() * 1e6 });
            }
        }
        // positive subjunctive / hortative: nimpe, nimeze, atoke, apewe
        if lower.ends_with('e') && !self.lex.skeleton_for(&lower).map_or(false, |k| k.lemma == lower) {
            for subj in ["tu", "wa", "ni", "u", "m", "a"] {
                let Some(rest) = lower.strip_prefix(subj) else { continue };
                let mut splits: Vec<(Option<&str>, String)> = vec![(None, rest.to_string())];
                for obj in ["wa", "ki", "vi", "zi", "li", "ni", "ku", "tu", "m"] {
                    if let Some(r2) = rest.strip_prefix(obj) {
                        splits.push((Some(obj), r2.to_string()));
                    }
                }
                for (obj, stem) in splits {
                    if stem.len() < 2 {
                        continue;
                    }
                    let base = &stem[..stem.len() - 1];
                    let mut cands = vec![format!("{base}a")];
                    // passive: ...we -> ...wa stays; also strip -w- for lookup
                    if let Some(b) = base.strip_suffix('w') {
                        cands.push(format!("{b}a"));
                    }
                    if obj == Some("m") && stem.starts_with('w') {
                        cands.push(format!("{}a", &stem[1..stem.len() - 1]));
                    }
                    if let Some((_, gloss)) = self.verb_root(&cands) {
                        let who = match subj {
                            "ni" => "I", "u" => "you", "a" => "he", "tu" => "we", "m" => "you all", "wa" => "they",
                            _ => "it",
                        };
                        let mut tokens = vec![
                            Token::new(who, linearizer::Role::Subject),
                            Token::new("should", linearizer::Role::Tense),
                            Token::new(gloss, linearizer::Role::Root),
                        ];
                        if let Some(o) = obj {
                            let eng = match o {
                                "m" | "ni" => if o == "ni" { "me" } else { "him/her" },
                                "ku" => "you", "wa" | "vi" | "zi" => "them", "tu" => "us", _ => "it",
                            };
                            tokens.push(Token::new(eng, linearizer::Role::Object));
                        }
                        return Some(RaceResult { tokens, score: 0.85, racer_id: 2, latency_us: start.elapsed().as_secs_f64() * 1e6 });
                    }
                }
            }
        }
        let morph = linearizer::decompose_verb(&lower);
        if morph.negation.is_none() {
            // non-negated verbs with a glide object (walimwona): let full-morph
            // handle it only when the plain lookup would fail
            return None;
        }
        let cands = linearizer::root_candidates(&morph);
        self.verb_root(&cands)?;
        let mut res = self.racer_full_morph(word);
        if let Some(sk) = self.lex.skeleton_for(&lower) {
            if sk.lemma == lower {
                res.tokens.push(Token::new(
                    format!("[alt: {}]", linearizer::clean_gloss(&sk.gloss)),
                    linearizer::Role::Modifier,
                ));
            }
        }
        Some(res)
    }

    /// Strategy 0: O(1) hash table lookup for common phrases.
    fn racer_hash(&self, word: &str) -> Option<RaceResult> {
        let start = std::time::Instant::now();
        let lower = word.to_lowercase();

        // Direct match
        if let Some(tokens) = self.phrase_table.get(&lower) {
            let latency = start.elapsed().as_secs_f64() * 1_000_000.0;
            return Some(RaceResult {
                tokens: tokens.clone(),
                score: 1.0,
                racer_id: 0,
                latency_us: latency,
            });
        }

        None
    }

    /// Strategy 1: Fast heuristic -- lexicon lookup without morph decomposition.
    fn racer_heuristic(&self, word: &str) -> Option<RaceResult> {
        let start = std::time::Instant::now();
        let lower = word.to_lowercase();

        // Direct lexicon lookup (exact match only -- no affix stripping).
        // If the resolved lemma differs from the surface, the word is an
        // inflected/agglutinated form and should go to racer_2 (full morph)
        // so we don't lose subject/tense information.
        if let Some(sk) = self.lex.skeleton_for(&lower) {
            if sk.lemma == lower {
                let gloss = linearizer::clean_gloss(&sk.gloss);
                let tokens = if gloss.is_empty() {
                    vec![Token::new(&sk.surface, linearizer::Role::Unknown)]
                } else {
                    vec![Token::new(gloss, role_from_pos(&sk.pos))]
                };
                let latency = start.elapsed().as_secs_f64() * 1_000_000.0;
                return Some(RaceResult {
                    tokens,
                    score: 0.85,
                    racer_id: 1,
                    latency_us: latency,
                });
            }
            // Inflected form -> defer to racer_2
            return None;
        }

        None
    }

    /// Strategy 2: Full morphological decomposition.
    fn racer_full_morph(&self, word: &str) -> RaceResult {
        let start = std::time::Instant::now();
        let lower = word.to_lowercase();

        // Use the linearizer's decompose_verb for structured parsing
        let morph = linearizer::decompose_verb(&lower);
        let mut tokens = Vec::new();

        // Subject pronoun
        if let Some(ref subj) = morph.subject {
            let eng = match subj.as_str() {
                "ni" => "I",
                "u" => "you",
                "a" => "he",
                "tu" => "we",
                "m" => "you",
                "wa" => "they",
                _ => "it",
            };
            tokens.push(Token::new(eng, linearizer::Role::Subject));
        }

        // Negation
        if morph.negation.is_some() {
            tokens.push(Token::new("not", linearizer::Role::Negation));
        }

        // Tense
        if let Some(ref tense) = morph.tense {
            let eng = match tense.as_str() {
                "li" => "did",
                "me" => "have",
                "ta" => "will",
                "na" => "is",
                "ka" => "then",
                "hu" => "always",
                "ja" => "not yet",
                _ => "",
            };
            if !eng.is_empty() {
                tokens.push(Token::new(eng, linearizer::Role::Tense));
            }
        }

        // Root: prefer a lexicon verb among the candidate roots (present
        // negatives end in -i, so try the -a form first), then any entry,
        // else mark it unresolved instead of passing the raw string through.
        let cands = linearizer::root_candidates(&morph);
        let mut root_resolved = true;
        let root_gloss = if let Some((_, g)) = self.verb_root(&cands) {
            g
        } else if let Some(sk) = cands.iter().find_map(|c| self.lex.skeleton_for(c)) {
            linearizer::clean_gloss(&sk.gloss)
        } else if let Some(sk) = self.lex.skeleton_for(&lower) {
            linearizer::clean_gloss(&sk.gloss)
        } else {
            root_resolved = false;
            format!("[?{}]", morph.root)
        };

        if !root_gloss.is_empty() {
            tokens.push(Token::new(&root_gloss, linearizer::Role::Root));
        }

        // Object
        if let Some(ref obj) = morph.object {
            let eng = match obj.as_str() {
                "m" | "ni" => "me",
                "ku" => "you",
                "wa" => "them",
                "ki" | "li" | "i" => "it",
                "vi" | "yi" | "zi" => "them",
                "yu" => "him",
                _ => "it",
            };
            tokens.push(Token::new(eng, linearizer::Role::Object));
        }

        let latency = start.elapsed().as_secs_f64() * 1_000_000.0;

        // Score: based on how many slots were successfully resolved
        let slots_filled = [
            morph.subject.is_some(),
            morph.tense.is_some() || morph.negation.is_some(),
            root_resolved,
        ];
        let score = slots_filled.iter().filter(|&&b| b).count() as f64 / 3.0;

        RaceResult {
            tokens,
            score,
            racer_id: 2,
            latency_us: latency,
        }
    }
}

/// Map POS tag to token role.
fn role_from_pos(pos: &str) -> linearizer::Role {
    match pos {
        "noun" | "pron" | "name" => linearizer::Role::Modifier,
        "verb" => linearizer::Role::Root,
        "adj" | "adv" | "num" => linearizer::Role::Modifier,
        "intj" => linearizer::Role::Modifier,
        _ => linearizer::Role::Unknown,
    }
}

/// Initialize the phrase table with common Swahili expressions.
fn init_phrase_table(table: &mut std::collections::HashMap<String, Vec<Token>>) {
    // Greetings
    table.insert(
        "habari".into(),
        vec![
            Token::new("news", linearizer::Role::Modifier),
            Token::new("how", linearizer::Role::Modifier),
        ],
    );
    table.insert(
        "mambo".into(),
        vec![
            Token::new("how", linearizer::Role::Modifier),
            Token::new("are", linearizer::Role::Modifier),
            Token::new("things", linearizer::Role::Modifier),
        ],
    );
    table.insert(
        "jambo".into(),
        vec![Token::new("hello", linearizer::Role::Modifier)],
    );
    table.insert(
        "sawa".into(),
        vec![Token::new("okay", linearizer::Role::Modifier)],
    );
    table.insert(
        "ndiyo".into(),
        vec![Token::new("yes", linearizer::Role::Modifier)],
    );
    table.insert(
        "hapana".into(),
        vec![Token::new("no", linearizer::Role::Modifier)],
    );
    table.insert(
        "asante".into(),
        vec![
            Token::new("thank", linearizer::Role::Modifier),
            Token::new("you", linearizer::Role::Modifier),
        ],
    );
    table.insert(
        "pole".into(),
        vec![Token::new("sorry", linearizer::Role::Modifier)],
    );
    table.insert(
        "karibu".into(),
        vec![Token::new("welcome", linearizer::Role::Modifier)],
    );
    table.insert(
        "tafadhali".into(),
        vec![Token::new("please", linearizer::Role::Modifier)],
    );
    table.insert(
        "samahani".into(),
        vec![
            Token::new("excuse", linearizer::Role::Modifier),
            Token::new("me", linearizer::Role::Modifier),
        ],
    );

    // Fused negative "have" forms (-na): sina, huna, hana, hatuna, hamna, hawana
    for (w, who) in [("sina", "I"), ("huna", "you"), ("hana", "he"), ("hatuna", "we"), ("hamna", "you all"), ("hawana", "they")] {
        table.insert(
            w.into(),
            vec![
                Token::new(who, linearizer::Role::Subject),
                Token::new("not", linearizer::Role::Negation),
                Token::new("have", linearizer::Role::Root),
            ],
        );
    }
    table.insert(
        "hakuna".into(),
        vec![
            Token::new("there is", linearizer::Role::Root),
            Token::new("not", linearizer::Role::Negation),
            Token::new("any", linearizer::Role::Modifier),
        ],
    );
    table.insert(
        "hamna".into(),
        vec![
            Token::new("you all", linearizer::Role::Subject),
            Token::new("not", linearizer::Role::Negation),
            Token::new("have", linearizer::Role::Root),
        ],
    );

    // Common fixed expressions
    table.insert(
        "nzuri".into(),
        vec![Token::new("good", linearizer::Role::Modifier)],
    );
    table.insert(
        "mbaya".into(),
        vec![Token::new("bad", linearizer::Role::Modifier)],
    );
    table.insert(
        "sawa".into(),
        vec![Token::new("fine", linearizer::Role::Modifier)],
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_lex() -> crate::Lexicon {
        let tmp = std::env::temp_dir().join(format!("sema_racer_test_{}_{:?}.jsonl", std::process::id(), std::thread::current().id()));
        let entries = [
            r#"{"w":"fika","p":"verb","g":["to arrive"]}"#,
            r#"{"w":"sema","p":"verb","g":["to speak, to say"]}"#,
            r#"{"w":"habari","p":"noun","g":["news"]}"#,
            r#"{"w":"nzuri","p":"adj","g":["good"]}"#,
        ];
        std::fs::write(&tmp, entries.join("\n")).unwrap();
        crate::Lexicon::load(&tmp).unwrap()
    }

    #[test]
    fn racer_0_hashes_greetings() {
        let lex = test_lex();
        let racer = Racer::new(&lex);
        let result = racer.race("habari");
        assert_eq!(result.racer_id, 0);
        assert!(result.score >= 0.9);
    }

    #[test]
    fn racer_1_heuristic_exact_match() {
        let lex = test_lex();
        let racer = Racer::new(&lex);
        let result = racer.race("fika");
        // Should find "fika" directly in lexicon
        assert!(result.score > 0.0);
    }

    #[test]
    fn racer_2_full_morph_complex() {
        let lex = test_lex();
        let racer = Racer::new(&lex);
        let result = racer.race("umefika");
        // Should decompose u+me+fika and get "you have arrive"
        assert!(result.tokens.iter().any(|t| t.word == "you"));
        assert!(result.tokens.iter().any(|t| t.word == "have"));
    }

    #[test]
    fn racer_cascade_early_exit() {
        let lex = test_lex();
        let racer = Racer::new(&lex);
        // "habari" should be caught by racer 0 (hash), never reach racer 2
        let result = racer.race("habari");
        assert_eq!(result.racer_id, 0);
    }

    #[test]
    fn sentence_racing() {
        let lex = test_lex();
        let racer = Racer::new(&lex);
        let results = racer.race_sentence("habari umefika");
        assert_eq!(results.len(), 2);
    }
}
