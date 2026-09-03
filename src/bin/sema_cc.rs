//! Sema Command Centre helper.
//!
//! A thin CLI that exposes Sema's library API over stdout so the external
//! Tkinter command centre (tools/sema_cc.py) can drive it for testing,
//! review and health checks. Every subcommand prints a single JSON object.
//!
//! Usage:
//!   sema_cc lex --data data/swahili.distilled.jsonl health
//!   sema_cc lex --data data/swahili.distilled.jsonl anchor --word umefikia
//!   sema_cc lex --data data/swahili.distilled.jsonl morph --word hawatakujibu
//!   sema_cc lex --data data/swahili.distilled.jsonl linearize --sentence "umefika sokoni"
//!   sema_cc rolodex --subj u --tense me --root fika
//!   sema_cc rolodex --neg --subj wa --tense ta --obj ku --root jibu
//!   sema_cc synth --subj 1SG --tense PAST --root fanya --neg
//!   sema_cc synth --subj 1SG --tense PRES --root soma --deriv causative
//!   sema_cc race --lex <path> --sentence "habari umefika"
//!   sema_cc bench --lex <path> --csv bench/Sema_Benchmark_1000.csv --limit 200
//!
//! Every command ends with `--json` (default) and writes one JSON document.

use sema::lexicon::{Lexicon, Skeleton, segment_words};
use sema::linearizer;
use sema::racer::{RaceResult, Racer};
use sema::rolodex::{Intent, Rolodex};
use serde_json::{Map, Value, json};
use std::time::Instant;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    match run(&args[1..]) {
        Ok(v) => println!("{}", serde_json::to_string(&v).unwrap()),
        Err(e) => {
            eprintln!("error: {}", e);
            std::process::exit(2);
        }
    }
}

fn get(args: &[String], i: usize) -> Option<&str> {
    args.get(i).map(|s| s.as_str())
}

fn run(args: &[String]) -> Result<Value, String> {
    let sub = get(args, 0).ok_or("missing subcommand")?;
    match sub {
        "health" => health(args),
        "anchor" => anchor(args),
        "morph" => morph(args),
        "linearize" => linearize(args),
        "rolodex" => rolodex(args),
        "synth" => synth(args),
        "race" => race(args),
        "bench" => bench(args),
        _ => Err(format!("unknown subcommand: {sub}")),
    }
}

fn data_path(args: &[String]) -> Result<String, String> {
    // find --data <path>
    for (i, a) in args.iter().enumerate() {
        if a == "--data" {
            return get(args, i + 1)
                .map(String::from)
                .ok_or("missing --data value".to_string());
        }
    }
    Ok("data/swahili.distilled.jsonl".to_string())
}

fn load_lex(args: &[String]) -> Result<Lexicon, String> {
    let p = data_path(args)?;
    Lexicon::load(&p).map_err(|e| format!("lexicon load failed: {e}"))
}

fn health(args: &[String]) -> Result<Value, String> {
    let lex = load_lex(args)?;
    let mut report = Map::new();
    report.insert("status".into(), json!("ok"));
    report.insert("version".into(), json!(sema::VERSION));
    report.insert("lexicon_path".into(), json!(data_path(args)?));
    report.insert("lemmas".into(), json!(lex.len()));

    // quick anchor probe
    let probe = ["umefikia", "hawapo", "siku"];
    let mut resolved = 0usize;
    for w in &probe {
        if lex.skeleton_for(w).is_some() {
            resolved += 1;
        }
    }
    report.insert(
        "anchor_probe".into(),
        json!(format!("{}/{}", resolved, probe.len())),
    );

    // latency sample (100 lookups)
    let t0 = Instant::now();
    let mut hits = 0usize;
    for _ in 0..100 {
        if lex.skeleton_for("umefikia").is_some() {
            hits += 1;
        }
    }
    let dt_us = t0.elapsed().as_secs_f64() * 1e6 / 100.0;
    report.insert(
        "lookup_avg_us_per_word".into(),
        json!(format!("{:.1}", dt_us)),
    );
    report.insert("lookup_sample_hits".into(), json!(hits));

    // model files present?
    let mut models = Vec::new();
    for f in [
        "Qwen2.5-0.5B-Instruct.Q4_K_M.gguf",
        "qwen2.5-1.5b-instruct-q4_k_m.gguf",
    ] {
        let p = std::path::Path::new(f);
        let present = p.exists();
        let size = if present {
            p.metadata().map(|m| m.len()).unwrap_or(0)
        } else {
            0
        };
        models.push(json!({ "file": f, "present": present, "bytes": size }));
    }
    report.insert("models".into(), Value::Array(models));

    Ok(Value::Object(report))
}

fn anchor(args: &[String]) -> Result<Value, String> {
    let lex = load_lex(args)?;
    let word = flag(args, "--word").ok_or("missing --word")?;
    match lex.skeleton_for(&word) {
        Some(sk) => Ok(json!({
            "surface": sk.surface,
            "lemma": sk.lemma,
            "pos": sk.pos,
            "gloss": sk.gloss,
            "root": sk.root,
            "resolved_via_form": sk.resolved_via_form,
        })),
        None => Ok(json!({"surface": word, "resolved": false})),
    }
}

fn morph(args: &[String]) -> Result<Value, String> {
    let word = flag(args, "--word").ok_or("missing --word")?;
    let m = linearizer::decompose_verb(&word);
    Ok(json!({
        "raw": m.raw,
        "negation": m.negation,
        "subject": m.subject,
        "tense": m.tense,
        "object": m.object,
        "root": m.root,
    }))
}

fn linearize(args: &[String]) -> Result<Value, String> {
    let lex = load_lex(args)?;
    let sentence = flag(args, "--sentence").ok_or("missing --sentence")?;
    let mut skeletons = Vec::new();
    let mut unresolved = Vec::new();
    for tok in segment_words(&sentence) {
        if !tok.chars().any(|c| c.is_alphabetic()) {
            continue;
        }
        match lex.skeleton_for(&tok) {
            Some(sk) => skeletons.push(sk),
            None => unresolved.push(tok),
        }
    }
    let lin = linearizer::linearize_sentence(&skeletons, &sentence);
    let tokens: Vec<Value> = lin
        .tokens
        .iter()
        .map(|t| json!({"word": t.word, "role": format!("{:?}", t.role)}))
        .collect();
    Ok(json!({
        "source": sentence,
        "telegram": lin.to_telegram(),
        "tokens": tokens,
        "unresolved": unresolved,
    }))
}

fn rolodex(args: &[String]) -> Result<Value, String> {
    let r = Rolodex::swahili();
    let subj = flag(args, "--subj");
    let tense = flag(args, "--tense");
    let obj = flag(args, "--obj");
    let root = flag(args, "--root").ok_or("missing --root")?;
    let neg = has(args, "--neg");
    let mood = flag(args, "--mood").unwrap_or_else(|| "a".to_string());
    let intent = Intent {
        subject: subj.map(String::from),
        tense: tense.map(String::from),
        negation: neg,
        object: obj.map(String::from),
        root: root.to_string(),
        derivations: Vec::new(),
        mood: mood.to_string(),
        particles: Vec::new(),
    };
    let syn = r.synthesize(&intent);
    Ok(json!({
        "surface": syn.surface,
        "slots": syn.slots,
        "smoothed": syn.smoothed,
    }))
}

fn synth(args: &[String]) -> Result<Value, String> {
    let e = sema::backpass::engine();
    let root = flag(args, "--root").ok_or("missing --root")?;
    let mut intent = sema::backpass::intent_from_parts(
        &e,
        flag(args, "--subj").as_deref(),
        flag(args, "--tense").as_deref(),
        has(args, "--neg"),
        flag(args, "--obj").as_deref(),
        &root,
    );
    intent.derivations = flag(args, "--deriv")
        .map(|s| {
            s.split(',')
                .map(|t| t.trim().to_string())
                .filter(|t| !t.is_empty())
                .collect()
        })
        .unwrap_or_default();
    if let Some(mood) = flag(args, "--mood") {
        intent.mood = mood;
    }
    let out = e.synthesize(&intent);
    Ok(json!({
        "surface": out.surface,
        "slots": out.slots,
        "subject_class": out.subject_class,
        "object_class": out.object_class,
    }))
}

fn race(args: &[String]) -> Result<Value, String> {
    let lex = load_lex(args)?;
    let racer = Racer::new(&lex);
    let sentence = flag(args, "--sentence").ok_or("missing --sentence")?;
    let results: Vec<Value> = racer
        .race_sentence(&sentence)
        .iter()
        .map(result_json)
        .collect();
    Ok(json!({ "sentence": sentence, "results": results }))
}

fn result_json(r: &RaceResult) -> Value {
    let words: Vec<Value> = r
        .tokens
        .iter()
        .map(|t| json!({"word": t.word, "role": format!("{:?}", t.role)}))
        .collect();
    json!({
        "racer_id": r.racer_id,
        "score": r.score,
        "latency_us": r.latency_us,
        "tokens": words,
    })
}

/// Minimal RFC-4180 CSV line parser (handles quoted fields with "" escapes).
/// No regex, std-lib only, returns the raw fields with quotes stripped.
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

fn bench(args: &[String]) -> Result<Value, String> {
    let lex = load_lex(args)?;
    let racer = Racer::new(&lex);
    let rolodex = Rolodex::swahili();
    let csv = flag(args, "--csv").unwrap_or_else(|| "bench/Sema_Benchmark_1000.csv".to_string());
    let limit = flag(args, "--limit")
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(200);

    let content = std::fs::read_to_string(&csv).map_err(|e| format!("csv read failed: {e}"))?;
    let mut rows: Vec<Vec<String>> = Vec::new();
    for line in content.lines().skip(1) {
        if line.trim().is_empty() {
            continue;
        }
        let parts = parse_csv_line(line);
        if parts.len() >= 2 {
            rows.push(parts);
        }
    }
    if rows.len() > limit {
        rows.truncate(limit);
    }

    // ---- Forward (SW -> EN): coverage + fuzzy telegraphic match ----
    let mut f_total = 0usize;
    let mut f_words = 0usize;
    let mut f_resolved = 0usize;
    let mut f_telegram_exact = 0usize;
    let mut f_racer: [usize; 3] = [0, 0, 0];
    let mut f_latency_sum = 0.0;
    let mut f_mismatches: Vec<Value> = Vec::new();

    // ---- Back (EN -> SW): synthesize intent -> verb, compare to gold ----
    let mut b_total = 0usize;
    let mut b_hit = 0usize;
    let mut b_by_category: Map<String, Value> = Map::new();
    let mut b_mismatches: Vec<Value> = Vec::new();

    // per-category/tier aggregates for forward
    let mut f_by_category: Map<String, Value> = Map::new();
    let mut f_by_tier: Map<String, Value> = Map::new();

    for row in &rows {
        let direction = row.get(1).map(|s| s.as_str()).unwrap_or("");
        if direction == "SW_TO_EN" {
            let source = row.get(2).cloned().unwrap_or_default();
            let expected_tele =
                linearizer::clean_gloss(row.get(5).map(|s| s.as_str()).unwrap_or_default());
            let category = row.get(6).cloned().unwrap_or_default();
            let tier = row.get(7).cloned().unwrap_or_default();

            f_total += 1;

            // Coverage: resolve every content word via the racer cascade.
            let mut words = 0usize;
            let mut resolved = 0usize;
            let mut lat_acc = 0.0;
            let mut cat_bucket: [usize; 3] = [0, 0, 0];
            for tok in sema::lexicon::segment_words(&source) {
                if !tok.chars().any(|c| c.is_alphabetic()) {
                    continue;
                }
                words += 1;
                let r = racer.race(&tok);
                if !r.tokens.is_empty() {
                    resolved += 1;
                    lat_acc += r.latency_us;
                    cat_bucket[r.racer_id.min(2) as usize] += 1;
                    // match the word's telegram token against expected tokens
                }
            }
            f_words += words;
            f_resolved += resolved;
            f_latency_sum += lat_acc;
            for i in 0..3 {
                f_racer[i] += cat_bucket[i];
            }

            // Fuzzy telegraphic match: token-set similarity after cleaning
            // the brackets from expected_telegraphic. This is an honest
            // fuzzy gate (0..=1.0), not an exact-string overclaim.
            let skels: Vec<Skeleton> = sema::lexicon::segment_words(&source)
                .into_iter()
                .filter_map(|w| lex.skeleton_for(&w))
                .collect();
            let produced = linearizer::linearize_sentence(&skels, &source);
            let tele = produced.to_telegram();
            let similarity = telegrams_token_jaccard(&tele, &expected_tele);
            if similarity >= 0.99 {
                f_telegram_exact += 1;
            } else {
                if f_mismatches.len() < 8 {
                    f_mismatches.push(json!({
                        "source": source,
                        "expected": expected_tele,
                        "produced": tele,
                        "jaccard": similarity,
                    }));
                }
            }

            // category / tier aggregates
            let cat_ent = f_by_category
                .entry(category)
                .or_insert_with(|| json!({ "rows": 0, "resolved": 0, "words": 0 }))
                .as_object_mut()
                .unwrap();
            cat_ent["rows"] = json!(cat_ent["rows"].as_u64().unwrap_or(0) + 1);
            cat_ent["words"] = json!(cat_ent["words"].as_u64().unwrap_or(0) + words as u64);
            cat_ent["resolved"] =
                json!(cat_ent["resolved"].as_u64().unwrap_or(0) + resolved as u64);

            let tier_ent = f_by_tier
                .entry(tier)
                .or_insert_with(|| json!({ "rows": 0, "resolved": 0, "words": 0 }))
                .as_object_mut()
                .unwrap();
            tier_ent["rows"] = json!(tier_ent["rows"].as_u64().unwrap_or(0) + 1);
            tier_ent["words"] = json!(tier_ent["words"].as_u64().unwrap_or(0) + words as u64);
            tier_ent["resolved"] =
                json!(tier_ent["resolved"].as_u64().unwrap_or(0) + resolved as u64);
        } else if direction == "EN_TO_SW" {
            let expected_verb = row.get(13).cloned().unwrap_or_default();
            let intent_str = row.get(12).cloned().unwrap_or_default();
            let category = row.get(6).cloned().unwrap_or_default();
            b_total += 1;
            if intent_str.is_empty() {
                continue;
            }
            let intent: Intent =
                serde_json::from_str(&intent_str).map_err(|e| format!("bad intent JSON: {e}"))?;
            let surface = rolodex.synthesize(&intent).surface;
            // exact match against the gold finite verb
            let ok = surface.trim() == expected_verb.trim();
            if ok {
                b_hit += 1;
            } else if b_mismatches.len() < 8 {
                b_mismatches.push(json!({
                    "source_en": row.get(2).cloned().unwrap_or_default(),
                    "expected_verb": expected_verb,
                    "produced": surface,
                    "intent": intent_str,
                }));
            }
            let cat_ent = b_by_category
                .entry(category)
                .or_insert_with(|| json!({ "rows": 0, "hits": 0 }))
                .as_object_mut()
                .unwrap();
            cat_ent["rows"] = json!(cat_ent["rows"].as_u64().unwrap_or(0) + 1);
            if ok {
                cat_ent["hits"] = json!(cat_ent["hits"].as_u64().unwrap_or(0) + 1);
            }
        }
    }

    let f_coverage = if f_words == 0 {
        0.0
    } else {
        100.0 * f_resolved as f64 / f_words as f64
    };
    let f_tele_pct = if f_total == 0 {
        0.0
    } else {
        100.0 * f_telegram_exact as f64 / f_total as f64
    };
    let b_pass_pct = if b_total == 0 {
        0.0
    } else {
        100.0 * b_hit as f64 / b_total as f64
    };
    let avg_latency = if rows.is_empty() {
        0.0
    } else {
        f_latency_sum / rows.len() as f64
    };

    let mut forward = Map::new();
    forward.insert("rows".into(), json!(f_total));
    forward.insert("content_words".into(), json!(f_words));
    forward.insert("resolved_words".into(), json!(f_resolved));
    forward.insert("coverage_pct".into(), json!(format!("{:.2}", f_coverage)));
    forward.insert(
        "telegram_fuzzy_exact_pct".into(),
        json!(format!("{:.2}", f_tele_pct)),
    );
    forward.insert(
        "avg_word_latency_us".into(),
        json!(format!("{:.2}", avg_latency)),
    );
    forward.insert("by_category".into(), Value::Object(f_by_category));
    forward.insert("by_tier".into(), Value::Object(f_by_tier));
    forward.insert("sample_mismatches".into(), Value::Array(f_mismatches));

    let mut back = Map::new();
    back.insert("rows".into(), json!(b_total));
    back.insert("hits".into(), json!(b_hit));
    back.insert("exact_pass_pct".into(), json!(format!("{:.2}", b_pass_pct)));
    back.insert("by_category".into(), Value::Object(b_by_category));
    back.insert("sample_mismatches".into(), Value::Array(b_mismatches));

    Ok(json!({
        "csv": csv,
        "rows_loaded": rows.len(),
        "forward": Value::Object(forward),
        "back": Value::Object(back),
        "racer_winner_distribution": json!({"racer0": f_racer[0], "racer1": f_racer[1], "racer2": f_racer[2]}),
    }))
}

/// Jaccard similarity over whitespace-delimited tokens after stripping
/// "[...]" markers from the expected telegraphic annotation.
fn telegrams_token_jaccard(produced: &str, expected: &str) -> f64 {
    use std::collections::HashSet;
    let norm = |s: &str| -> String {
        s.chars()
            .filter(|c| !matches!(c, '[' | ']' | '?' | ',' | '(' | ')' | '"'))
            .collect::<String>()
            .to_lowercase()
    };
    let mk = |s: &str| -> HashSet<String> {
        norm(s).split_whitespace().map(|t| t.to_string()).collect()
    };
    let a = mk(produced);
    let b = mk(expected);
    if a.is_empty() && b.is_empty() {
        return 1.0;
    }
    if a.is_empty() || b.is_empty() {
        return 0.0;
    }
    let inter = a.intersection(&b).count() as f64;
    let union = (a.len() + b.len()) as f64 - inter;
    if union <= 0.0 { 0.0 } else { inter / union }
}

fn flag(args: &[String], name: &str) -> Option<String> {
    for (i, a) in args.iter().enumerate() {
        if a == name {
            return get(args, i + 1).map(String::from);
        }
    }
    None
}

fn has(args: &[String], name: &str) -> bool {
    args.iter().any(|a| a == name)
}
