//! Deterministic Swahili clause and template composer.
//!
//! Combines [`Engine`], [`ConcordEngine`], [`FactGuardian`], and [`Decomposer`]
//! to render complete, noun-class-agreed Swahili clauses from templates like:
//!
//! - `{t:child} {poss:@child|1SG} {loc_cop:@child|ko} {loc:@hospital}`
//!   -> `mtoto wangu yuko hospitalini`
//! - `Mpe {t:child} {t:tablet:pl} {num:@tablet:pl|2} {assoc:@tablet:pl} paracetamol 500 mg`
//!   -> `Mpe mtoto vidonge viwili vya paracetamol 500 mg`
//! - `{t:chair} {dem:@chair|prox} {v:@chair|PERF|-|-|vunja|stative}`
//!   -> `kiti hiki kimevunjika`

use crate::concord::ConcordEngine;
use crate::guardian::FactGuardian;
use crate::morph::Decomposer;
use crate::synth::{Engine, Intent};
use serde::Serialize;

/// Output of rendering a deterministic Swahili template.
#[derive(Debug, Clone, Serialize)]
pub struct ComposedOutput {
    pub surface: String,
    pub verbs_verified: usize,
    pub slots_rendered: Vec<(String, String)>,
}

/// Multi-layer deterministic Swahili clause composer.
#[derive(Debug, Clone)]
pub struct Composer {
    pub engine: Engine,
    pub concord: ConcordEngine,
    pub guardian: FactGuardian,
    decomposer: Decomposer,
}

impl Composer {
    /// Build a composer from explicit engine, concord, and term guardian components.
    pub fn new(engine: Engine, concord: ConcordEngine, guardian: FactGuardian) -> Self {
        let decomposer = Decomposer::from_engine(engine.clone());
        Self {
            engine,
            concord,
            guardian,
            decomposer,
        }
    }

    /// Default Swahili composer with embedded morphology and clinical/core dictionaries.
    pub fn swahili() -> Self {
        let engine = Engine::swahili();
        let concord = ConcordEngine::from_morphology(engine.morphology().clone());
        let guardian = FactGuardian::default_clinical();
        Self::new(engine, concord, guardian)
    }

    fn resolve_class_ref(&self, spec: &str) -> Result<String, String> {
        let s = spec.trim();
        if let Some(rest) = s.strip_prefix('@') {
            let (term, plural) = match rest.rsplit_once(':') {
                Some((t, "pl")) => (t.trim(), true),
                Some((t, "sg")) => (t.trim(), false),
                _ => (rest.trim(), false),
            };
            let entry = self
                .guardian
                .get(term)
                .ok_or_else(|| format!("no dictionary entry for '@{term}'"))?;
            let cls = entry
                .class
                .as_deref()
                .ok_or_else(|| format!("term '{term}' has no noun class"))?;
            Ok(if plural {
                self.concord.plural_class(cls)
            } else {
                self.concord.normalize_class(cls)
            })
        } else {
            Ok(self.concord.normalize_class(s))
        }
    }

    fn resolve_term_surface(&self, spec: &str) -> Result<String, String> {
        let s = spec.trim();
        let (term, force_pl) = match s.rsplit_once(':') {
            Some((t, "pl")) => (t.trim(), true),
            Some((t, "sg")) => (t.trim(), false),
            _ => (s, false),
        };
        let (entry, matched_pl) = self
            .guardian
            .lookup_with_plural(term)
            .ok_or_else(|| format!("no dictionary entry for '{term}'"))?;
        if force_pl || matched_pl {
            Ok(entry
                .sw_plural
                .clone()
                .unwrap_or_else(|| entry.sw.clone()))
        } else {
            Ok(entry.sw.clone())
        }
    }

    /// Render a clause template into surface Swahili while verifying synthesized verbs.
    pub fn render(&self, template: &str) -> Result<ComposedOutput, String> {
        let mut out = String::new();
        let mut verbs_verified = 0usize;
        let mut slots_rendered = Vec::new();
        let mut rest = template;

        while let Some(a) = rest.find('{') {
            out.push_str(&rest[..a]);
            let after = &rest[a + 1..];
            let b = after.find('}').ok_or_else(|| "unclosed '{'".to_string())?;
            let body = &after[..b];
            let (kind, arg) = body
                .split_once(':')
                .ok_or_else(|| format!("malformed template slot '{body}'"))?;
            let rendered = match kind.trim() {
                "t" => self.resolve_term_surface(arg)?,
                "v" => {
                    let f: Vec<&str> = arg.split('|').map(str::trim).collect();
                    if f.len() < 5 {
                        return Err(format!(
                            "verb slot requires SUBJ|TENSE|NEG|OBJ|ROOT: '{arg}'"
                        ));
                    }
                    let none = |s: &str| {
                        if s == "-" || s.is_empty() {
                            None
                        } else {
                            Some(s.to_string())
                        }
                    };
                    let subj = match none(f[0]) {
                        Some(s) if s.starts_with('@') => Some(self.resolve_class_ref(&s)?),
                        other => other,
                    };
                    let obj = match f.get(3).and_then(|x| none(x)) {
                        Some(s) if s.starts_with('@') => {
                            let cls = self.resolve_class_ref(&s)?;
                            self.engine
                                .morphology()
                                .subjects
                                .get(&cls)
                                .cloned()
                                .or(Some(cls))
                        }
                        other => other,
                    };
                    let rel = match f.get(7).and_then(|x| none(x)) {
                        Some(s) if s.starts_with('@') => Some(self.resolve_class_ref(&s)?),
                        other => other,
                    };
                    let derivs = f
                        .get(5)
                        .and_then(|d| none(d))
                        .map(|d| {
                            d.split(',')
                                .map(|x| x.trim().to_string())
                                .filter(|x| !x.is_empty())
                                .collect()
                        })
                        .unwrap_or_default();
                    let intent = Intent {
                        subject: subj,
                        tense: none(f[1]),
                        negation: f[2] == "neg",
                        relative: rel,
                        object: obj,
                        root: f[4].to_string(),
                        derivations: derivs,
                        mood: f
                            .get(6)
                            .and_then(|m| none(m))
                            .unwrap_or_else(|| "a".to_string()),
                        particles: Vec::new(),
                    };
                    let syn = self.engine.synthesize_checked(&intent).map_err(|errs| {
                        errs.into_iter()
                            .map(|e| e.message)
                            .collect::<Vec<_>>()
                            .join("; ")
                    })?;
                    if self.decomposer.verify_roundtrip(&syn.surface).ok {
                        verbs_verified += 1;
                    }
                    syn.surface
                }
                "cop" => match arg.trim().to_lowercase().as_str() {
                    "pos" | "" => self.concord.copula(false),
                    "neg" => self.concord.copula(true),
                    "emph" | "emph_pos" => self.concord.emphatic_copula(false),
                    "emph_neg" => self.concord.emphatic_copula(true),
                    other => return Err(format!("unknown copula flag '{other}'")),
                },
                "loc_cop" => {
                    let f: Vec<&str> = arg.split('|').map(str::trim).collect();
                    if f.len() < 2 {
                        return Err(format!("loc_cop slot requires SUBJ|LOC_KIND: '{arg}'"));
                    }
                    let subj = self.resolve_class_ref(f[0])?;
                    let neg = f.get(2).is_some_and(|x| *x == "neg");
                    self.concord.locative_copula(&subj, f[1], neg)?
                }
                "have" => {
                    let f: Vec<&str> = arg.split('|').map(str::trim).collect();
                    let subj = self.resolve_class_ref(f[0])?;
                    let neg = f.get(1).is_some_and(|x| *x == "neg");
                    self.concord.have(&subj, neg)?
                }
                "assoc" => {
                    let cls = self.resolve_class_ref(arg)?;
                    self.concord.associative(&cls)?
                }
                "poss" => {
                    let f: Vec<&str> = arg.split('|').map(str::trim).collect();
                    if f.len() < 2 {
                        return Err(format!("poss slot requires CLASS|OWNER: '{arg}'"));
                    }
                    let cls = self.resolve_class_ref(f[0])?;
                    self.concord.possessive(&cls, f[1])?
                }
                "dem" => {
                    let f: Vec<&str> = arg.split('|').map(str::trim).collect();
                    if f.len() < 2 {
                        return Err(format!("dem slot requires CLASS|DISTANCE: '{arg}'"));
                    }
                    let cls = self.resolve_class_ref(f[0])?;
                    self.concord.demonstrative(&cls, f[1])?
                }
                "adj" => {
                    let f: Vec<&str> = arg.split('|').map(str::trim).collect();
                    if f.len() < 2 {
                        return Err(format!("adj slot requires CLASS|STEM: '{arg}'"));
                    }
                    let cls = self.resolve_class_ref(f[0])?;
                    self.concord.adjective(&cls, f[1])?
                }
                "num" => {
                    let f: Vec<&str> = arg.split('|').map(str::trim).collect();
                    if f.len() < 2 {
                        return Err(format!("num slot requires CLASS|N: '{arg}'"));
                    }
                    let cls = self.resolve_class_ref(f[0])?;
                    let n: u32 = f[1]
                        .parse()
                        .map_err(|_| format!("invalid numeral '{}'", f[1]))?;
                    self.concord.numeral(&cls, n)?
                }
                "loc" => {
                    let noun = if arg.trim().starts_with('@') {
                        let clean = arg.trim().trim_start_matches('@');
                        self.resolve_term_surface(clean)?
                    } else {
                        arg.trim().to_string()
                    };
                    self.concord.locative_noun(&noun)
                }
                other => return Err(format!("unknown template slot kind '{other}'")),
            };
            slots_rendered.push((kind.trim().to_string(), rendered.clone()));
            out.push_str(&rendered);
            rest = &after[b + 1..];
        }
        out.push_str(rest);

        Ok(ComposedOutput {
            surface: out,
            verbs_verified,
            slots_rendered,
        })
    }
}
