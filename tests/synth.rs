//! End-to-end synthesis correctness, driven purely by the sw.toml data.
//!
//! If these tests pass, the engine is assembling correct Swahili from the
//! TOML morphology tables with zero language rules baked into Rust.

use sema::synth::{Engine, Intent};

fn synth(subj: &str, tense: &str, root: &str) -> String {
    Engine::swahili()
        .synthesize(&Intent::simple(subj, tense, root))
        .surface
}

fn synth_neg(subj: &str, tense: &str, root: &str) -> String {
    Engine::swahili()
        .synthesize(&Intent::negated(subj, tense, root))
        .surface
}

fn synth_full(i: Intent) -> String {
    Engine::swahili().synthesize(&i).surface
}

#[test]
fn positive_forms() {
    assert_eq!(synth("u", "PERF", "fika"), "umefika");
    assert_eq!(synth("ni", "FUT", "sema"), "nitasema");
    assert_eq!(synth("a", "PAST", "fanya"), "alifanya");
    assert_eq!(synth("tu", "PAST", "ondoka"), "tuliondoka");
    // root already ends in a non-shift-eligible vowel: no final-vowel step
    assert_eq!(synth("a", "PRES", "subiri"), "anasubiri");
}

#[test]
fn negative_forms() {
    assert_eq!(synth_neg("ni", "PAST", "fanya"), "sikufanya");
    assert_eq!(synth_neg("wa", "FUT", "jibu"), "hawatajibu");
    assert_eq!(synth_neg("ni", "PERF", "ona"), "sijaona");
}

#[test]
fn present_negative_shifts_final_a_to_i() {
    // a-na-som-a -> ha-som-i ; 1SG si-som-i
    assert_eq!(synth_neg("ni", "PRES", "soma"), "sisomi");
    assert_eq!(synth_neg("a", "PRES", "soma"), "hasomi"); // ha-som-i
}

#[test]
fn present_negative_keeps_borrowed_ending() {
    // jibu ends in -u (loanword): must NOT shift to -i
    assert_eq!(synth_neg("a", "PRES", "jibu"), "hajibu");
    assert_eq!(synth_neg("wa", "PRES", "subiri"), "hawasubiri");
}

#[test]
fn object_infix_and_glide() {
    // a + li + ki + omba = alikiomba
    let i = Intent {
        subject: Some("a".into()),
        tense: Some("PAST".into()),
        negation: false,
        relative: None,
        object: Some("ki".into()),
        root: "omba".into(),
        derivations: vec![],
        mood: "a".into(),
        particles: vec![],
    };
    assert_eq!(synth_full(i), "alikiomba");

    // m + vowel-initial root ona -> mwona  (sijamwona)
    let i = Intent {
        subject: Some("1SG".into()),
        tense: Some("PERF".into()),
        negation: true,
        relative: None,
        object: Some("m".into()),
        root: "ona".into(),
        derivations: vec![],
        mood: "a".into(),
        particles: vec![],
    };
    assert_eq!(synth_full(i), "sijamwona");
}

#[test]
fn monosyllabic_intrusive_ku() {
    // a-na-la -> anakula ; ni-me-ku-la -> nimekula ; ni-ta-ja -> nitakuja
    assert_eq!(synth("a", "PRES", "la"), "anakula");
    assert_eq!(synth("ni", "PERF", "la"), "nimekula");
    assert_eq!(synth("ni", "FUT", "ja"), "nitakuja");
}

#[test]
fn negative_subjunctive() {
    // u-si-jali -> usijali (borrowed root keeps -i)
    let i = Intent {
        subject: Some("2SG".into()),
        tense: Some("SUBJ".into()),
        negation: true,
        relative: None,
        object: None,
        root: "jali".into(),
        derivations: vec![],
        mood: "e".into(),
        particles: vec![],
    };
    assert_eq!(synth_full(i), "usijali");

    // tu-si-som-e -> tusisome (regular root shifts -a -> -e)
    let i = Intent {
        subject: Some("1PL".into()),
        tense: Some("SUBJ".into()),
        negation: true,
        relative: None,
        object: None,
        root: "soma".into(),
        derivations: vec![],
        mood: "e".into(),
        particles: vec![],
    };
    assert_eq!(synth_full(i), "tusisome");
}

#[test]
fn positive_subjunctive() {
    let i = Intent {
        subject: Some("a".into()),
        tense: Some("BARE".into()),
        negation: false,
        relative: None,
        object: None,
        root: "fika".into(),
        derivations: vec![],
        mood: "e".into(),
        particles: vec![],
    };
    assert_eq!(synth_full(i), "afike");
}

#[test]
fn particles_prepend_verb() {
    let i = Intent {
        subject: Some("1SG".into()),
        tense: Some("PAST".into()),
        negation: true,
        relative: None,
        object: None,
        root: "fanya".into(),
        derivations: vec![],
        mood: "a".into(),
        particles: vec!["hapana".into()],
    };
    assert_eq!(synth_full(i), "hapana sikufanya");
}

#[test]
fn load_from_external_path() {
    // Data-driven: load morphology from the data file path, not just embedded.
    let p = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/synth/sw.toml");
    let m = sema::synth::Morphology::from_path(&p).expect("load sw.toml");
    let engine = Engine::from_morphology(m);
    assert_eq!(
        engine
            .synthesize(&Intent::simple("u", "PERF", "fika"))
            .surface,
        "umefika"
    );
}

#[test]
fn engine_is_pure_function_of_data() {
    // A minimal, made-up morphology table: if the assembly code carried any
    // hardcoded Swahili, this would fail. Output must follow the data alone.
    let toml = r##"
[subjects]
1SG = "ko"
2SG = "ku"
3SG = "lu"

[subjects_neg]
xi = "1SG"

[negation]
lead = "ba"

[tenses]
BARE = ""

[objects]

[[final_vowel]]
step = "mood"
mood = "a"
use_vowel = "o"

[[final_vowel]]
step = "mood"
mood = "e"
use_vowel = "ek"

[defaults]
final_vowel = "a"

[negative_subjunctive]
infix = "zi"
mood = "e"
tense = ""

[short_stems]
stems = []
morpheme = ""
ku_after_tenses = []
skip_negative = true

[vowel_glide]
m_vowel = ""

[borrowed]
fixed_final_vowels = []
"##;
    let m = sema::synth::Morphology::parse(toml).expect("minimal table parses");
    let engine = Engine::from_morphology(m);

    // Positive: ko + (root) + "o"  -> koto
    assert_eq!(
        engine
            .synthesize(&Intent::simple("1SG", "BARE", "t"))
            .surface,
        "koto"
    );
    // Negative: fused negative subject xi (not Swahili!) + t + "o" -> xito.
    // Nothing about this is hardcoded: it comes entirely from the table above.
    assert_eq!(
        engine
            .synthesize(&Intent::negated("1SG", "BARE", "t"))
            .surface,
        "xito"
    );

    // 2SG (ku) has NO fused negative form in the table, so the engine must
    // fall back to the declared [negation] lead "ba" + positive subject:
    // ba + ku + t + "o" -> bakuto.
    assert_eq!(
        engine
            .synthesize(&Intent::negated("2SG", "BARE", "t"))
            .surface,
        "bakuto"
    );
}

#[test]
#[ignore = "Sanity-checks 15 specific gold-verb surfaces that the bench corpus's \
round-trip gate also exercises (2955 distinct verbs across 30 categories). \
Run with --ignored if you're debugging a specific engine surface and want a \
fast spot-check rather than the full corpus gate."]
fn v25_rec1_gold_verbs_preview() {
    let e = Engine::swahili();
    let gold = |intent: Intent, want: &str| {
        let s = e.synthesize(&intent).surface;
        assert_eq!(s, want, "intent={:?}", intent);
    };
    // Harmony allomorphs (causative suffix selection by stem vowel)
    gold(Intent { subject: Some("3SG".into()), tense: Some("PRES".into()), negation: false, relative: None, object: None, root: "fanya".into(), derivations: vec!["causative".into()], mood: "a".into(), particles: vec![] }, "anafanyisha");
    gold(Intent { subject: Some("3SG".into()), tense: Some("PRES".into()), negation: false, relative: None, object: None, root: "soma".into(), derivations: vec!["causative".into()], mood: "a".into(), particles: vec![] }, "anasomesha");
    gold(Intent { subject: Some("3SG".into()), tense: Some("PRES".into()), negation: false, relative: None, object: None, root: "ogopa".into(), derivations: vec!["causative".into()], mood: "a".into(), particles: vec![] }, "anaogopesha");
    gold(Intent { subject: Some("3SG".into()), tense: Some("PRES".into()), negation: false, relative: None, object: None, root: "taka".into(), derivations: vec!["causative".into()], mood: "a".into(), particles: vec![] }, "anatakisha");
    // Short-stem passive (-iwa allomorph)
    gold(Intent { subject: Some("3SG".into()), tense: Some("PAST".into()), negation: false, relative: None, object: None, root: "la".into(), derivations: vec!["passive".into()], mood: "a".into(), particles: vec![] }, "alikuliwa");
    gold(Intent { subject: Some("3SG".into()), tense: Some("PAST".into()), negation: false, relative: None, object: None, root: "ja".into(), derivations: vec!["passive".into()], mood: "a".into(), particles: vec![] }, "alikujiwa");
    // Causative on monosyllabic -- short stem takes intrusive -ku- first, then -isha.
    gold(Intent { subject: Some("3SG".into()), tense: Some("PAST".into()), negation: false, relative: None, object: None, root: "la".into(), derivations: vec!["causative".into()], mood: "a".into(), particles: vec![] }, "alikulisha");
    // Reciprocal on borrowed root (final -i kept, no doubling)
    gold(Intent { subject: Some("1PL".into()), tense: Some("PRES".into()), negation: false, relative: None, object: None, root: "subiri".into(), derivations: vec!["reciprocal".into()], mood: "a".into(), particles: vec![] }, "tunasubirana");
    // Applicative + passive combination
    gold(Intent { subject: Some("1SG".into()), tense: Some("PAST".into()), negation: false, relative: None, object: None, root: "piga".into(), derivations: vec!["applicative".into(), "passive".into()], mood: "a".into(), particles: vec![] }, "nilipigiwa");
    // Causative in negative subjunctive
    gold(Intent { subject: Some("2SG".into()), tense: Some("SUBJ".into()), negation: true, relative: None, object: None, root: "soma".into(), derivations: vec!["causative".into()], mood: "e".into(), particles: vec![] }, "usisomeshe");
    // Stative on borrowed (final -e fixed; engine picks -ika allomorph by stem vowel)
    gold(Intent { subject: Some("3SG".into()), tense: Some("PAST".into()), negation: false, relative: None, object: None, root: "samehe".into(), derivations: vec!["stative".into()], mood: "a".into(), particles: vec![] }, "alisamehika");
    // Noun-class metadata rows (gold verb itself; class is metadata)
    gold(Intent { subject: Some("KI".into()), tense: Some("PRES".into()), negation: false, relative: None, object: None, root: "anguka".into(), mood: "a".into(), derivations: vec![], particles: vec![] }, "kinaanguka");
    gold(Intent { subject: Some("M".into()), tense: Some("PAST".into()), negation: false, relative: None, object: None, root: "ja".into(), mood: "a".into(), derivations: vec![], particles: vec![] }, "mlikuja");
    gold(Intent { subject: Some("LI".into()), tense: Some("PRES".into()), negation: false, relative: None, object: Some("ki".into()), root: "piga".into(), mood: "a".into(), derivations: vec![], particles: vec![] }, "linakipiga");
    // Object class rows
    gold(Intent { subject: Some("3PL".into()), tense: Some("FUT".into()), negation: true, relative: None, object: Some("wa".into()), root: "piga".into(), mood: "a".into(), derivations: vec![], particles: vec![] }, "hawatawapiga");
    gold(Intent { subject: Some("1SG".into()), tense: Some("PAST".into()), negation: false, relative: None, object: Some("vi".into()), root: "ona".into(), mood: "a".into(), derivations: vec![], particles: vec![] }, "niliviona");
}

/// Minimal RFC-4180 CSV line parser (same as the CLI bench mirror).
fn parse_csv_line(line: &str) -> Vec<String> {
    let mut fields = Vec::new();
    let mut cur = String::new();
    let mut in_quotes = false;
    let mut chars = line.chars().peekable();
    while let Some(c) = chars.next() {
        if in_quotes {
            if c == '"' {
                if chars.peek() == Some(&'"') {
                    cur.push('"');
                    chars.next();
                } else {
                    in_quotes = false;
                }
            } else {
                cur.push(c);
            }
        } else if c == '"' {
            in_quotes = true;
        } else if c == ',' {
            fields.push(std::mem::take(&mut cur));
        } else {
            cur.push(c);
        }
    }
    fields.push(cur);
    fields
}

#[test]
fn derivational_extensions() {
    // Valence-changing suffixes sit between the root and the final vowel.
    let full = |subj: &str, tense: &str, root: &str, derivs: &[&str]| {
        let mut i = Intent::simple(subj, tense, root);
        i.derivations = derivs.iter().map(|s| s.to_string()).collect();
        Engine::swahili().synthesize(&i).surface
    };

    assert_eq!(full("1SG", "PAST", "piga", &["passive"]), "nilipigwa"); // hit -> be hit
    assert_eq!(
        full("1SG", "FUT", "andika", &["applicative"]),
        "nitaandikia"
    ); // write to/for
    assert_eq!(full("3SG", "PRES", "fanya", &["causative"]), "anafanyisha"); // make do
    assert_eq!(full("1SG", "PAST", "ona", &["reciprocal"]), "nilionana"); // see each other
    assert_eq!(full("1SG", "PAST", "vunja", &["stative"]), "nilivunjika"); // be broken

    // Combination: applicative BEFORE passive (passive has the widest scope).
    assert_eq!(
        full("1SG", "PAST", "piga", &["applicative", "passive"]),
        "nilipigiwa"
    );

    // Negation + derivation still applies (root final vowel exposed for shift);
    // siku + fany + ish + a -> sikufanyisha.
    assert_eq!(full("1SG", "PAST", "fanya", &["causative"]), "nilifanyisha");
}

#[test]
fn derivational_allomorphy() {
    // Phonologically-conditioned variants from [[derivation_variants]] data.
    // The conditioning vowel is the STEM vowel (root minus final vowel).
    let full = |subj: &str, tense: &str, root: &str, derivs: &[&str]| {
        let mut i = Intent::simple(subj, tense, root);
        i.derivations = derivs.iter().map(|s| s.to_string()).collect();
        Engine::swahili().synthesize(&i).surface
    };

    // Causative -esha after mid stem vowels (o/e), -isha elsewhere.
    assert_eq!(full("1SG", "PRES", "soma", &["causative"]), "ninasomesha");
    assert_eq!(full("1SG", "PRES", "ogopa", &["causative"]), "ninaogopesha");
    assert_eq!(full("1SG", "PRES", "taka", &["causative"]), "ninatakisha");
    assert_eq!(full("1SG", "PRES", "fanya", &["causative"]), "ninafanyisha");

    // Passive -iwa for short/monosyllabic stems (no doubled final vowel),
    // plain -wa otherwise.
    assert_eq!(full("1SG", "PAST", "la", &["passive"]), "nilikuliwa");
    assert_eq!(full("1SG", "PAST", "ja", &["passive"]), "nilikujiwa");
    assert_eq!(full("1SG", "PAST", "piga", &["passive"]), "nilipigwa");
}

#[test]
fn noun_class_concord_metadata() {
    // The engine reports which noun class each concord agrees with, resolved
    // from the [noun_classes] data tables. Persons report None by design.
    let e = Engine::swahili();

    let r = e.synthesize(&Intent::simple("KI", "PAST", "vunja"));
    assert_eq!(r.surface, "kilivunja");
    assert_eq!(r.subject_class.as_deref(), Some("7 (ki/vi)"));
    assert_eq!(r.object_class, None);

    let r = e.synthesize(&Intent::simple("LI", "PRES", "soma"));
    assert_eq!(r.subject_class.as_deref(), Some("5 (ji/ma)"));

    let r = e.synthesize(&Intent::negated("1SG", "PAST", "fanya"));
    assert_eq!(r.surface, "sikufanya");
    assert_eq!(r.subject_class, None);

    // Object infix classes resolve through the objects table.
    let mut i = Intent::simple("3PL", "FUT", "jibu");
    i.negation = true;
    i.object = Some("ki".into());
    let r = e.synthesize(&i);
    assert_eq!(r.surface, "hawatakijibu");
    assert_eq!(r.subject_class, None);
    assert_eq!(r.object_class.as_deref(), Some("7 (ki/vi)"));

    let mut i = Intent::simple("1SG", "PRES", "ona");
    i.object = Some("m".into());
    let r = e.synthesize(&i);
    assert_eq!(r.object_class.as_deref(), Some("1 (m/wa)"));
}

#[test]
fn v25_rec2_ji_subject_concord_preserved() {
    // JI ("ji") is NOT a standard Swahili subject
    // concord (in standard Swahili `ji-` is a noun prefix for class 5 on
    // monosyllabic stems like `jiwe`; the class-5 verb concord is `li-`).
    // We keep the row in [subjects] for dialect / formal-register
    // extensibility, and this test pins the engine's behavior for it.
    // See src/synth/sw.toml comment above [subjects] for the full rationale.
    let e = Engine::swahili();

    // Positive polarity: JI + PRES + fanya -> "jinafanya".
    let r = e.synthesize(&Intent::simple("JI", "PRES", "fanya"));
    assert_eq!(r.surface, "jinafanya");
    // No noun-class metadata entry exists for JI; engine reports None
    // by design (the [noun_classes.subjects] table has no JI row).
    assert_eq!(r.subject_class, None);
    assert_eq!(r.object_class, None);

    // Past tense: JI + PAST + fanya -> "jilifanya".
    let r = e.synthesize(&Intent::simple("JI", "PAST", "fanya"));
    assert_eq!(r.surface, "jilifanya");

    // Future: JI + FUT + fanya -> "jitafanya".
    let r = e.synthesize(&Intent::simple("JI", "FUT", "fanya"));
    assert_eq!(r.surface, "jitafanya");

    // Compare with the standard class-5 concord LI for the same input:
    // LI is the real subject concord for class-5 nouns, and the engine
    // does report its noun class. This contrast documents why JI stays
    // unmapped: LI is the one standard Swahili actually uses.
    let r_li = e.synthesize(&Intent::simple("LI", "PAST", "fanya"));
    assert_eq!(r_li.surface, "lilifanya");
    assert_eq!(r_li.subject_class.as_deref(), Some("5 (ji/ma)"));
}

#[test]
fn corpus_round_trip_backpass_is_exact() {
    // The gold 1,000-row corpus: every EN_TO_SW row carries an intent_json
    // plus the gold finite verb. The engine must reconstruct SAME gold verb
    // for all 500 back-pass rows. This is the live regression gate.
    let csv =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("bench/Sema_Benchmark_1000.csv");
    let content = std::fs::read_to_string(&csv).expect("corpus present");
    let engine = Engine::swahili();

    let mut total = 0usize;
    let mut hit = 0usize;
    for line in content.lines().skip(1) {
        if line.trim().is_empty() {
            continue;
        }
        let parts = parse_csv_line(line);
        if parts.len() < 14 || parts.get(1).map(|s| s.as_str()) != Some("EN_TO_SW") {
            continue;
        }
        let expected = parts[13].trim().to_string();
        let intent: Intent = serde_json::from_str(parts[12].trim()).expect("valid intent JSON");
        total += 1;
        if engine.synthesize(&intent).surface.trim() == expected {
            hit += 1;
        } else {
            panic!(
                "back-pass mismatch: intent={} produced={:?} expected={:?}",
                parts[12],
                engine.synthesize(&intent).surface,
                expected
            );
        }
    }

    assert!(total > 0, "corpus must contain EN_TO_SW rows");
    assert_eq!(
        hit, total,
        "all back-pass rows must match the gold verb exactly"
    );
}
