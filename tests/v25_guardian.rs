//! Sema v2.5 Fact Guardian adversarial security & restoration tests.
//!
//! Runs all 24 adversarial boundary cases across 8 categories (originally
//! measuring a 54.2% leak rate in v0.2.0) and verifies 0% leak rate.

use sema::guardian::{FactGuardian, RestoreMode};

struct AdversarialCase {
    category: &'static str,
    input: &'static str,
    targets: &'static [&'static str],
}

fn strip_placeholders(s: &str) -> String {
    let mut out = String::new();
    let mut rest = s;
    while let Some(a) = rest.find("[#") {
        out.push_str(&rest[..a]);
        let after = &rest[a + 2..];
        if let Some(b) = after.find(']') {
            rest = &after[b + 1..];
        } else {
            out.push_str(&rest[a..]);
            rest = "";
        }
    }
    out.push_str(rest);
    out
}

#[test]
fn adversarial_24_cases_zero_leak_rate() {
    let guardian = FactGuardian::default_clinical();
    let cases = [
        // 1. Swahili number words
        AdversarialCase { category: "Swahili number words", input: "Meza mia tano mg", targets: &["mia tano"] },
        AdversarialCase { category: "Swahili number words", input: "Toa vidonge kumi", targets: &["kumi"] },
        AdversarialCase { category: "Swahili number words", input: "Sindano elfu moja", targets: &["elfu moja"] },
        // 2. Ranges
        AdversarialCase { category: "Ranges", input: "Take 500-1000 mg daily", targets: &["500-1000"] },
        AdversarialCase { category: "Ranges", input: "Take 500 to 1000 mg daily", targets: &["500 to 1000"] },
        AdversarialCase { category: "Ranges", input: "Take 500 - 1000 mg daily", targets: &["500 - 1000"] },
        // 3. Fractions
        AdversarialCase { category: "Fractions", input: "Take nusu kidonge asubuhi", targets: &["nusu"] },
        AdversarialCase { category: "Fractions", input: "Take half a tablet daily", targets: &["half a tablet"] },
        AdversarialCase { category: "Fractions", input: "Take 1/2 tablet daily", targets: &["1/2"] },
        // 4. Decimals
        AdversarialCase { category: "Decimals", input: "Administer 2.5 mg orally", targets: &["2.5"] },
        AdversarialCase { category: "Decimals", input: "Administer 0.5 ml injection", targets: &["0.5"] },
        AdversarialCase { category: "Decimals", input: "Administer 2,5 mg orally", targets: &["2,5"] },
        // 5. Brand names
        AdversarialCase { category: "Brand names", input: "Prescribe Panadol 500 mg", targets: &["Panadol"] },
        AdversarialCase { category: "Brand names", input: "Prescribe Septra suspension", targets: &["Septra"] },
        AdversarialCase { category: "Brand names", input: "Prescribe Augmentin 625 mg", targets: &["Augmentin"] },
        // 6. Misspelled drug names
        AdversarialCase { category: "Misspelled drug names", input: "Give paracetemol 500 mg", targets: &["paracetemol"] },
        AdversarialCase { category: "Misspelled drug names", input: "Give amoxillin 250 mg", targets: &["amoxillin"] },
        AdversarialCase { category: "Misspelled drug names", input: "Give ibuprufen 400 mg", targets: &["ibuprufen"] },
        // 7. Mixed English-Swahili units
        AdversarialCase { category: "Mixed English-Swahili units", input: "Take vidonge 2 daily", targets: &["vidonge", "2"] },
        AdversarialCase { category: "Mixed English-Swahili units", input: "Kunywa mg 500 kwa siku", targets: &["mg", "500"] },
        AdversarialCase { category: "Mixed English-Swahili units", input: "Take tablets mbili kwa siku", targets: &["tablets", "mbili"] },
        // 8. Numbers split across tokens
        AdversarialCase { category: "Numbers split across tokens", input: "Take 5 0 0 mg daily", targets: &["500", "5 0 0"] },
        AdversarialCase { category: "Numbers split across tokens", input: "Take five hundred mg daily", targets: &["five hundred"] },
        AdversarialCase { category: "Numbers split across tokens", input: "Give 1 0 ml syrup", targets: &["10", "1 0"] },
    ];

    let mut leaks = Vec::new();
    for tc in &cases {
        let prot = guardian.protect(tc.input);
        let unshielded = strip_placeholders(&prot.text).to_lowercase();
        for &target in tc.targets {
            if unshielded.contains(&target.to_lowercase()) {
                leaks.push(format!("[{}] input='{}' target='{}' protected='{}'", tc.category, tc.input, target, prot.text));
            }
        }
    }
    assert!(leaks.is_empty(), "Fact Guardian leaks detected ({}/24):\n{}", leaks.len(), leaks.join("\n"));
}

#[test]
fn fuzzy_drug_restores_canonical_and_avoids_false_positives() {
    let guardian = FactGuardian::default_clinical();

    let p = guardian.protect("Give paracetemol 500 mg to the child");
    assert_eq!(p.slots[0].sw, "paracetamol");
    assert_eq!(p.slots[0].fuzzy_from.as_deref(), Some("paracetemol"));
    let r = guardian.restore(&p.text, &p, RestoreMode::Swahili);
    assert!(r.integrity_ok);
    assert_eq!(r.text, "Give paracetamol 500 mg to the mtoto");

    // Ordinary prose words must NOT be hijacked by fuzzy drug matching.
    let prose = guardian.protect("General clinical parallel camera medical discussion");
    assert!(
        prose.slots.is_empty(),
        "unexpected slots on ordinary prose: {:?}",
        prose.slots
    );
}

#[test]
fn restore_integrity_detects_missing_and_unknown_ids() {
    let guardian = FactGuardian::default_clinical();
    let p = guardian.protect("Give 500 mg Panadol to the child");
    let bad = guardian.restore("[#N1] [#T1] [#T99]", &p, RestoreMode::Swahili);
    assert!(!bad.integrity_ok);
    assert_eq!(bad.unknown, vec!["T99"]);
    assert!(bad.missing.contains(&"N2".to_string()));
    assert!(bad.missing.contains(&"T2".to_string()));

    let bi = guardian.restore("[#T1] [#T2]", &p, RestoreMode::Bilingual);
    assert!(bi.text.contains("Panadol (Panadol)"));
    assert!(bi.text.contains("mtoto (child)"));
}
