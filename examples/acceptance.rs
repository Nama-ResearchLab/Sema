// Acceptance harness: resolve real sentences through the shipped lexicon.
//
// Usage: cargo run --release --example acceptance

use sema::lexicon::segment_words;

const SENTENCES: &[&str] = &[
    "habari",
    "umefikia wapi?",
    "unasema nini?",
    "amina na juma hawapo",
    "saida ni mtoto mrefu ila ana mguu mdogo",
    "mtoto ana homa na kikohozi",
];

fn main() -> std::io::Result<()> {
    let data = concat!(env!("CARGO_MANIFEST_DIR"), "/data/swahili.distilled.jsonl");
    let lex = sema::Lexicon::load(data)?;
    println!("sema {} | lexicon: {} lemmas\n", sema::VERSION, lex.len());

    let mut resolved = 0usize;
    let mut total = 0usize;
    for s in SENTENCES {
        println!("SW: {s}");
        for word in segment_words(s) {
            total += 1;
            match lex.skeleton_for(&word) {
                Some(sk) => {
                    resolved += 1;
                    println!("   -> {sk}");
                }
                None => println!("   -> {word}[?? unresolved]"),
            }
        }
        println!();
    }
    let pct = resolved as f64 / total.max(1) as f64 * 100.0;
    println!("anchor coverage: {resolved}/{total} words ({pct:.0}%)");
    Ok(())
}
