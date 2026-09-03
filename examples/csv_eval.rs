// CSV evaluation: run Sema anchoring over a labeled test set.
//
// Expected columns: ID,Swahili Sentence[,English Translation[,Focus]]
// Reports overall + per-focus anchor coverage, unresolved word frequency.
//
// Usage: cargo run --release --example csv_eval -- data/swahili.distilled.jsonl <file.csv>

use std::collections::BTreeMap;

/// Minimal quote-aware CSV line splitter.
fn split_csv(line: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut in_q = false;
    let mut chars = line.chars().peekable();
    while let Some(c) = chars.next() {
        if in_q {
            if c == '"' {
                if chars.peek() == Some(&'"') {
                    cur.push('"');
                    chars.next();
                } else {
                    in_q = false;
                }
            } else {
                cur.push(c);
            }
        } else if c == '"' {
            in_q = true;
        } else if c == ',' {
            out.push(std::mem::take(&mut cur));
        } else {
            cur.push(c);
        }
    }
    out.push(cur);
    out
}

fn main() -> std::io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        eprintln!("usage: csv_eval <distilled.jsonl> <test.csv>");
        std::process::exit(2);
    }
    let lex = sema::Lexicon::load(&args[1])?;
    let raw = std::fs::read_to_string(&args[2])?;

    let (mut tot_w, mut res_w) = (0usize, 0usize);
    let mut per_focus: BTreeMap<String, (usize, usize)> = BTreeMap::new();
    let mut misses: BTreeMap<String, usize> = BTreeMap::new();
    let mut sent_stats: Vec<(usize, usize, String)> = Vec::new();

    for (i, line) in raw.lines().enumerate() {
        if i == 0 || line.trim().is_empty() {
            continue;
        }
        let f = split_csv(line);
        if f.len() < 2 {
            continue;
        }
        let focus = if f.len() >= 4 {
            f[3].trim().to_string()
        } else {
            "?".into()
        };
        let words = sema::lexicon::segment_words(&f[1]);
        let e = per_focus.entry(focus).or_default();
        for w in &words {
            tot_w += 1;
            e.0 += 1;
            match lex.skeleton_for(w) {
                Some(_) => {
                    res_w += 1;
                    e.1 += 1;
                }
                None => {
                    *misses.entry(w.to_lowercase()).or_default() += 1;
                }
            }
        }
        sent_stats.push((res_w, tot_w, f[1].trim().to_string()));
    }

    println!(
        "sema {} :: csv eval on {} sentences\n",
        sema::VERSION,
        sent_stats.len()
    );
    println!(
        "OVERALL: {}/{} words anchored ({:.1}%)\n",
        res_w,
        tot_w,
        res_w as f64 / tot_w.max(1) as f64 * 100.0
    );

    println!("per grammatical focus:");
    for (k, (t, r)) in &per_focus {
        let (t, r) = (*t, *r);
        println!(
            "  {:<42} {r:>3}/{t:<3} {:.0}%",
            k,
            r as f64 / t.max(1) as f64 * 100.0
        );
    }

    println!("\ntop unresolved words:");
    let mut v: Vec<_> = misses.iter().collect();
    v.sort_by_key(|(_, c)| std::cmp::Reverse(**c));
    for (w, c) in v.iter().take(20) {
        println!("  {w:<20} x{c}");
    }
    Ok(())
}
