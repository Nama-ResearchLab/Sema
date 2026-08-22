//! CSV benchmark: anchor every sentence of a test set and report coverage,
//! resolution-path breakdown and timing.
//!
//! Usage:
//!   cargo run --release --example bench_csv -- <sentences.csv> [--json out.jsonl]
//!
//! Expected CSV layout: ID,Swahili Sentence,English Translation,Focus
//! (only column 2 is read; quoted later columns are tolerated).

use std::time::Instant;

fn main() -> std::io::Result<()> {
    let mut args = std::env::args().skip(1);
    let csv_path = match args.next() {
        Some(p) => p,
        None => {
            eprintln!("usage: bench_csv <sentences.csv> [--json out.jsonl]");
            std::process::exit(2);
        }
    };
    let mut json_out: Option<String> = None;
    while let Some(a) = args.next() {
        if a == "--json" {
            json_out = args.next();
        }
    }

    let lex = sema::Lexicon::load("data/swahili.distilled.jsonl")?;
    println!("lexicon: {} lemmas", lex.len());
    println!("csv: {}", csv_path);

    let content = std::fs::read_to_string(&csv_path)?;
    let mut rows: Vec<(String, String)> = Vec::new();
    for line in content.lines().skip(1) {
        if line.trim().is_empty() {
            continue;
        }
        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() >= 2 {
            rows.push((parts[0].to_string(), parts[1].to_string()));
        }
    }
    println!("sentences: {}\n", rows.len());

    let mut total_words = 0usize;
    let mut exact = 0usize;
    let mut via_forms = 0usize;
    let mut via_affix = 0usize;
    let mut unresolved_total = 0usize;
    let mut full_cover = 0usize;
    let mut zero_cover = 0usize;
    let mut times_us: Vec<f64> = Vec::new();
    let mut per_sentence: Vec<(f64, &str, usize, usize)> = Vec::new(); // (cov, id, hit, words)
    let mut json_lines: Vec<String> = Vec::new();

    for (id, sentence) in &rows {
        let t0 = Instant::now();
        let mut hit = 0usize;
        let mut words = 0usize;
        for tok in sema::lexicon::segment_words(sentence) {
            if !tok.chars().any(|c| c.is_alphabetic()) {
                continue;
            }
            words += 1;
            if let Some(sk) = lex.skeleton_for(&tok) {
                hit += 1;
                if sk.resolved_via_form.is_some() {
                    via_forms += 1;
                } else if sk.lemma == tok.to_lowercase() {
                    exact += 1;
                } else {
                    via_affix += 1;
                }
            }
        }
        let dt = t0.elapsed().as_secs_f64() * 1e6;
        times_us.push(dt);
        total_words += words;
        unresolved_total += words - hit;
        let cov = if words == 0 { 1.0 } else { hit as f64 / words as f64 };
        full_cover += (cov == 1.0) as usize;
        zero_cover += (hit == 0) as usize;
        per_sentence.push((cov, id.as_str(), hit, words));
        json_lines.push(json_row(id, sentence, &lex));
    }

    times_us.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let pct = |p: f64| times_us[((times_us.len() - 1) as f64 * p) as usize];
    let word_cov = 100.0 * (total_words - unresolved_total) as f64 / total_words.max(1) as f64;

    println!("=== anchor coverage ===");
    println!(
        "word-level: {:.1}%  ({}/{} content words)",
        word_cov,
        total_words - unresolved_total,
        total_words
    );
    println!(
        "sentences fully covered: {}  partially: {}  zero-hit: {}",
        full_cover,
        rows.len() - full_cover - zero_cover,
        zero_cover
    );
    println!("=== resolution paths ===");
    println!(
        "exact lemma: {}  forms index: {}  affix strip: {}  unresolved: {}",
        exact, via_forms, via_affix, unresolved_total
    );
    println!("=== latency (per sentence) ===");
    println!(
        "avg: {:.0}us   p50: {:.0}us   p95: {:.0}us   max: {:.0}us",
        times_us.iter().sum::<f64>() / times_us.len() as f64,
        pct(0.50),
        pct(0.95),
        times_us[times_us.len() - 1]
    );

    per_sentence.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    println!("\n=== weakest sentences (lowest coverage) ===");
    for (cov, id, hit, words) in per_sentence.iter().take(10) {
        println!("  #{id}: {hit}/{words} words ({:.0}%)", cov * 100.0);
    }

    if let Some(path) = json_out {
        std::fs::write(&path, json_lines.join("\n"))?;
        println!("\njson anchors written to {path}");
    }
    Ok(())
}

// ---- minimal JSON string escaping (no serde_json dep in examples) ----
fn jesc(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => {}
            c => out.push(c),
        }
    }
    out
}

/// One JSON row per sentence: id, sentence and the exact prompt block Jericho
/// would prepend (same tidy-gloss + format as src/sema_anchor.rs).
fn json_row(id: &str, sentence: &str, lex: &sema::Lexicon) -> String {
    let mut block = String::from(
        "[SEMANTIC ANCHORS - the user's Swahili words resolved offline to English]\n",
    );
    for tok in sema::lexicon::segment_words(sentence) {
        if !tok.chars().any(|c| c.is_alphabetic()) {
            continue;
        }
        if let Some(sk) = lex.skeleton_for(&tok) {
            let gloss = tidy(&sk.gloss);
            block.push_str(&format!(
                "{} -> {} ({}): '{}'",
                sk.surface, sk.lemma, sk.pos, gloss
            ));
            if let Some(r) = &sk.root {
                block.push_str(&format!(" root={r}"));
            }
            block.push('\n');
        }
    }
    format!(
        "{{\"id\":\"{}\",\"sentence\":\"{}\",\"block\":\"{}\"}}",
        jesc(id),
        jesc(sentence),
        jesc(block.trim_end())
    )
}

fn tidy(gloss: &str) -> String {
    let t = gloss.trim();
    if t.contains("form of") || t.starts_with("Inflection of") {
        if let Some(pos) = t.find(':') {
            let rest = t[pos + 1..].trim();
            if !rest.is_empty() {
                return rest.to_string();
            }
        }
    }
    t.to_string()
}
