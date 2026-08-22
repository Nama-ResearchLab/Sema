// Distill a kaikki.org Wiktionary JSONL dump into Sema's compact format.
//
// Usage: sema-distill <raw.jsonl> <out.jsonl>

use std::io::{BufRead, BufWriter, Write};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        eprintln!("usage: sema-distill <kaikki_raw.jsonl> <distilled.jsonl>");
        std::process::exit(2);
    }
    let raw = std::fs::File::open(&args[1]).expect("open raw");
    let out = std::fs::File::create(&args[2]).expect("create out");
    let mut w = BufWriter::new(out);

    let reader = std::io::BufReader::new(raw);
    let mut kept = 0usize;
    let mut seen = 0usize;
    for (i, line) in reader.lines().enumerate() {
        let line = line.expect("read");
        seen += 1;
        let v: serde_json::Value = match serde_json::from_str(&line) {
            Ok(v) => v,
            Err(_) => continue,
        };
        let word = match v["word"].as_str() {
            Some(s) => s.to_string(),
            None => continue,
        };
        let pos = v["pos"].as_str().unwrap_or("unk").to_string();

        let mut glosses: Vec<String> = Vec::new();
        if let Some(senses) = v["senses"].as_array() {
            'senses: for s in senses {
                if let Some(gs) = s["glosses"].as_array() {
                    for g in gs {
                        if let Some(gstr) = g.as_str() {
                            let t = sema::lexicon::clean_wiki(gstr.trim());
                            if !t.is_empty() && !glosses.iter().any(|x| *x == t) {
                                glosses.push(t);
                            }
                        }
                        if glosses.len() >= 3 {
                            break 'senses;
                        }
                    }
                }
            }
        }
        if glosses.is_empty() {
            continue;
        }

        let root = sema::lexicon::root_from_glosses(&glosses);

        let mut forms: Vec<String> = Vec::new();
        if let Some(fs) = v["forms"].as_array() {
            for f in fs {
                if let Some(fstr) = f["form"].as_str() {
                    if fstr.chars().all(|c| c.is_alphabetic() || c == '-') && fstr.len() > 1 {
                        forms.push(fstr.to_string());
                    }
                }
                if forms.len() >= 8 {
                    break;
                }
            }
        }

        let e = sema::LexEntry {
            w: word,
            p: pos,
            g: glosses,
            r: root,
            f: forms,
        };
        writeln!(w, "{}", serde_json::to_string(&e).unwrap()).expect("write");
        kept += 1;
        if i % 5000 == 0 {
            eprint!("\r{i} scanned, {kept} kept...");
        }
    }
    eprintln!("\rdistilled {kept}/{seen} entries");
}
