//! Sema v2.5 pure-Rust morphological decomposition & exact round-trip gate.

use sema::morph::{Decomposer, MorphKind};
use std::path::Path;

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
fn decompose_and_roundtrip_real_unresolved_verbs() {
    let mut dec = Decomposer::swahili();
    let verbs_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("data/dictionary/verbs.en-sw.jsonl");
    if verbs_path.is_file() {
        dec.load_verbs_jsonl(&verbs_path).expect("load verbs jsonl");
    }

    let cases = [
        "waliowaua",
        "zilizojaa",
        "inayotoa",
        "inayopendeza",
        "wanaopandwa",
        "wanaoweza",
        "akimhusisha",
        "kumuua",
        "kuyakabili",
        "kuivaa",
        "kuiishi",
        "alimwandikia",
        "yalifikiwa",
        "vimeripotiwa",
        "ninasomesha",
        "hawatakujibu",
        "sikufanya",
        "sijamwona",
        "usijali",
        "nisiposoma",
        "ningesoma",
        "husoma",
        "nenda",
        "someni",
        "yuko",
        "hakuna",
        "watakaojibu",
        "nitakapofika",
    ];

    for word in cases {
        let rep = dec.verify_roundtrip(word);
        assert!(
            rep.ok && rep.output.as_deref() == Some(word),
            "roundtrip failed for '{word}': {rep:?}"
        );
    }

    // Spot-check structural accuracy on hard cases
    let w1 = dec.decompose("waliowaua").unwrap();
    let i1 = w1.intent.unwrap();
    assert_eq!(i1.subject.as_deref(), Some("3PL"));
    assert_eq!(i1.tense.as_deref(), Some("PAST"));
    assert_eq!(i1.object.as_deref(), Some("wa"));
    assert_eq!(i1.root, "ua");

    let w2 = dec.decompose("kumuua").unwrap();
    assert_eq!(w2.kind, MorphKind::Infinitive);
    let i2 = w2.intent.unwrap();
    assert_eq!(i2.tense.as_deref(), Some("INF"));
    assert_eq!(i2.object.as_deref(), Some("m"));
    assert_eq!(i2.root, "ua");

    let w3 = dec.decompose("yuko").unwrap();
    assert_eq!(w3.kind, MorphKind::LocativeCopula);

    let w4 = dec.decompose("hakuna").unwrap();
    assert_eq!(w4.kind, MorphKind::PossessiveVerb);
}

#[test]
fn pure_rust_roundtrip_over_all_500_gold_verbs() {
    let mut dec = Decomposer::swahili();
    let verbs_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("data/dictionary/verbs.en-sw.jsonl");
    if verbs_path.is_file() {
        dec.load_verbs_jsonl(&verbs_path).expect("load verbs jsonl");
    }

    let csv_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("bench/Sema_Benchmark_1000.csv");
    let content = std::fs::read_to_string(csv_path).expect("read gold csv");

    let mut total = 0usize;
    let mut hits = 0usize;
    for line in content.lines().skip(1) {
        if line.trim().is_empty() {
            continue;
        }
        let cols = parse_csv_line(line);
        if cols.len() < 14 || cols[1] != "EN_TO_SW" {
            continue;
        }
        let verb = cols[13].trim();
        total += 1;
        let rep = dec.verify_roundtrip(verb);
        if rep.ok && rep.output.as_deref() == Some(verb) {
            hits += 1;
        } else {
            panic!("pure-Rust roundtrip mismatch on '{verb}': {rep:?}");
        }
    }
    assert_eq!(total, 500);
    assert_eq!(hits, 500);
}

#[test]
fn viambishi_ngeli_and_nouns_not_confused_with_verbs() {
    let mut dec = Decomposer::swahili();
    let root_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let distilled = root_dir.join("data/swahili.distilled.jsonl");
    dec.load_distilled_lexicon(&distilled)
        .expect("load distilled lexicon");

    assert!(
        dec.non_verb_count() > 10_000,
        "expected >10000 non-verb headwords, got {}",
        dec.non_verb_count()
    );

    let blocked = [
        "me", "ke", "mume", "kiume", "mwanaume", "mwanamke", "habari", "haki", "huyu", "kitu",
        "maji", "meli", "na", "ki", "li", "ngali", "wa", "ya", "za", "la", "kwa",
    ];
    for w in blocked {
        let got = dec.decompose(w);
        assert!(
            got.is_none(),
            "viambishi/non-verb '{w}' falsely decomposed as: {got:?}"
        );
    }

    for homograph in ["meza", "hali"] {
        let dec_h = dec
            .decompose(homograph)
            .unwrap_or_else(|| panic!("homograph '{homograph}' should decompose"));
        assert!(
            dec_h.ambiguous_nonverb,
            "homograph '{homograph}' must set ambiguous_nonverb=true"
        );
    }

    let real_verbs = [
        "amesoma",
        "ninasoma",
        "tutakwenda",
        "finyeni",
        "usubiri",
        "anakula",
        "alikuwa",
        "amempa",
    ];
    for v in real_verbs {
        let d = dec
            .decompose(v)
            .unwrap_or_else(|| panic!("real verb '{v}' failed to decompose"));
        assert!(d.roundtrip_exact, "roundtrip failed for '{v}'");
    }
}
