use sema::terms::TermDict;

struct TestCase {
    category: &'static str,
    input: &'static str,
    target_terms: &'static [&'static str],
}

#[test]
fn test_adversarial_fact_guardian() {
    let clinical_tsv = include_str!("../data/terms.clinical.tsv");
    let core_tsv = include_str!("../data/terms.core.tsv");
    let general_tsv = include_str!("../data/terms.general.tsv");
    let dict = TermDict::layered_curated(&[clinical_tsv, core_tsv, general_tsv]);

    let cases = vec![
        // 1. Swahili number words
        TestCase { category: "Swahili number words", input: "Meza mia tano mg", target_terms: &["mia tano"] },
        TestCase { category: "Swahili number words", input: "Toa vidonge kumi", target_terms: &["kumi"] },
        TestCase { category: "Swahili number words", input: "Sindano elfu moja", target_terms: &["elfu moja"] },

        // 2. Ranges
        TestCase { category: "Ranges", input: "Take 500-1000 mg daily", target_terms: &["500-1000"] },
        TestCase { category: "Ranges", input: "Take 500 to 1000 mg daily", target_terms: &["500 to 1000"] },
        TestCase { category: "Ranges", input: "Take 500 - 1000 mg daily", target_terms: &["500 - 1000"] },

        // 3. Fractions
        TestCase { category: "Fractions", input: "Take nusu kidonge asubuhi", target_terms: &["nusu"] },
        TestCase { category: "Fractions", input: "Take half a tablet daily", target_terms: &["half a tablet"] },
        TestCase { category: "Fractions", input: "Take 1/2 tablet daily", target_terms: &["1/2"] },

        // 4. Decimals
        TestCase { category: "Decimals", input: "Administer 2.5 mg orally", target_terms: &["2.5"] },
        TestCase { category: "Decimals", input: "Administer 0.5 ml injection", target_terms: &["0.5"] },
        TestCase { category: "Decimals", input: "Administer 2,5 mg orally", target_terms: &["2,5"] },

        // 5. Brand names
        TestCase { category: "Brand names", input: "Prescribe Panadol 500 mg", target_terms: &["Panadol"] },
        TestCase { category: "Brand names", input: "Prescribe Septra suspension", target_terms: &["Septra"] },
        TestCase { category: "Brand names", input: "Prescribe Augmentin 625 mg", target_terms: &["Augmentin"] },

        // 6. Misspelled drug names
        TestCase { category: "Misspelled drug names", input: "Give paracetemol 500 mg", target_terms: &["paracetemol"] },
        TestCase { category: "Misspelled drug names", input: "Give amoxillin 250 mg", target_terms: &["amoxillin"] },
        TestCase { category: "Misspelled drug names", input: "Give ibuprufen 400 mg", target_terms: &["ibuprufen"] },

        // 7. Mixed English-Swahili units
        TestCase { category: "Mixed English-Swahili units", input: "Take vidonge 2 daily", target_terms: &["vidonge", "2"] },
        TestCase { category: "Mixed English-Swahili units", input: "Kunywa mg 500 kwa siku", target_terms: &["mg", "500"] },
        TestCase { category: "Mixed English-Swahili units", input: "Take tablets mbili kwa siku", target_terms: &["tablets", "mbili"] },

        // 8. Numbers split across tokens
        TestCase { category: "Numbers split across tokens", input: "Take 5 0 0 mg daily", target_terms: &["500", "5 0 0"] },
        TestCase { category: "Numbers split across tokens", input: "Take five hundred mg daily", target_terms: &["five hundred"] },
        TestCase { category: "Numbers split across tokens", input: "Give 1 0 ml syrup", target_terms: &["10", "1 0"] },
    ];

    println!("\n=== ADVERSARIAL FACT GUARDIAN EVALUATION ===");
    let mut total_cases = 0;
    let mut leaked_cases = 0;
    let mut cat_stats: std::collections::BTreeMap<&str, (usize, usize)> = std::collections::BTreeMap::new();

    for tc in &cases {
        total_cases += 1;
        let entry = cat_stats.entry(tc.category).or_insert((0, 0));
        entry.0 += 1; // total in cat

        let p = dict.protect(tc.input);
        // Check if any of the target clinical terms leaked unshielded into p.text
        let mut leaked = false;
        for &t in tc.target_terms {
            // If the target term still appears verbatim in p.text (and is not an ID like [#T..] or [#N..])
            if p.text.to_lowercase().contains(&t.to_lowercase()) {
                leaked = true;
                break;
            }
        }

        if leaked {
            leaked_cases += 1;
            entry.1 += 1; // leaked in cat
            println!("  [LEAK] Category: {:<28} Input: '{}' -> Shielded: '{}'", tc.category, tc.input, p.text);
        } else {
            println!("  [PASS] Category: {:<28} Input: '{}' -> Shielded: '{}'", tc.category, tc.input, p.text);
        }
    }

    println!("\n--- Category Breakdown ---");
    for (cat, (tot, leak)) in &cat_stats {
        let pass = tot - leak;
        let leak_pct = (*leak as f64 / *tot as f64) * 100.0;
        println!("  {:<30} Passed: {}/{} | Leaked: {}/{} ({:.1}%)", cat, pass, tot, leak, tot, leak_pct);
    }
    println!("Overall Leak Rate: {}/{} ({:.1}%)", leaked_cases, total_cases, (leaked_cases as f64 / total_cases as f64) * 100.0);
}
