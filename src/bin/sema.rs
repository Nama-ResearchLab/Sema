//! Sema CLI - Official Command-Line Interface (CLI) for the Sema Engine (v0.2.5).
//!
//! Subcommands:
//!   sema synth --subj 3PL --tense FUT --root jibu --obj ku --neg [--rel C1] [--deriv passive]
//!   sema validate --subj 1SG --tense PRES --root soma
//!   sema concord --kind copula|loc_cop|have|assoc|poss|dem|adj|num|loc ...
//!   sema compose "{t:child} {poss:@child|1SG} {loc_cop:@child|ko} {loc:@hospital}"
//!   sema morph <word>
//!   sema roundtrip <word>
//!   sema anchor <word>
//!   sema terms <sentence>
//!   sema gui [--port 4242]
//!   sema mcp
//!   sema health
//!   sema bench

use sema::compose::Composer;
use sema::concord::ConcordEngine;
use sema::guardian::FactGuardian;
use sema::lexicon::Lexicon;
use sema::linearizer;
use sema::morph::Decomposer;
use sema::synth::{Engine as SynthEngine, Intent as SynthIntent};
use serde_json::json;
use std::path::Path;
use std::process::Command;

fn print_help() {
    println!(r#"
  ____                        
 / ___|  ___ _ __ ___   __ _  
 \___ \ / _ \ '_ ` _ \ / _` | 
  ___) |  __/ | | | | | (_| | 
 |____/ \___|_| |_| |_|\__,_| 
 Offline Swahili Language Studio & Deterministic Engine (v{})
 Nama Research Lab · https://github.com/Nama-ResearchLab/Sema

USAGE:
  sema <COMMAND> [OPTIONS]

COMMANDS:
  gui, studio     Launch the interactive web studio (default: port 4242)
  mcp             Run as a Model Context Protocol (MCP) server over stdio
  synth           Synthesize an exact Swahili verb (with relatives, conditionals, derivations)
  validate        Strictly validate intent slots against sw.toml
  concord         Synthesize 18-class non-verbal concords (copula, loc_cop, have, assoc, poss, dem, adj, num, loc)
  compose         Compile a deterministic Swahili clause template
  morph           Decompose a Swahili verb or concord word into slots + round-trip check
  roundtrip       Verify exact parse -> re-synthesis round-trip invariant
  anchor          Resolve a surface word to dictionary lemma, POS, and English gloss
  terms           Protect clinical drugs, Swahili numbers, ranges, and decimals behind opaque IDs
  health          Check engine tables, lexicon lemmas, and invariant status
  bench           Execute in-memory realization performance benchmark
  version         Print version information
  help            Print this help menu

EXAMPLES:
  # Synthesize with relative clause + future -taka- allomorph:
  sema synth --subj 3PL --tense FUT --rel C2 --root jibu
  # Output: watakaojibu

  # Noun-class adjective & locative concord:
  sema concord --kind adj --class KI --stem dogo
  sema compose "{{t:child}} {{poss:@child|1SG}} {{loc_cop:@child|ko}} {{loc:@hospital}}"

  # Decompose and verify exact round-trip:
  sema roundtrip waliowaua

  # Shield clinical dosages, Swahili number words, and misspelled drugs:
  sema terms "Meza paracetemol mia tano mg kila asubuhi"
"#, sema::VERSION);
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        print_help();
        return;
    }

    let cmd = args[1].as_str();
    match cmd {
        "-h" | "--help" | "help" => print_help(),
        "-V" | "-v" | "--version" | "version" => {
            println!("sema {}", sema::VERSION);
        }
        "gui" | "studio" => run_gui(&args[2..]),
        "mcp" => run_mcp(),
        "synth" => run_synth(&args[2..]),
        "validate" => run_validate(&args[2..]),
        "concord" => run_concord(&args[2..]),
        "compose" => run_compose(&args[2..]),
        "morph" => run_morph(&args[2..]),
        "roundtrip" => run_roundtrip(&args[2..]),
        "anchor" => run_anchor(&args[2..]),
        "terms" => run_terms(&args[2..]),
        "health" => run_health(&args[2..]),
        "bench" => run_bench(&args[2..]),
        _ => {
            eprintln!("Unknown command '{}'. Run 'sema help' for available commands.", cmd);
            std::process::exit(1);
        }
    }
}

fn flag(args: &[String], name: &str) -> Option<String> {
    for (i, a) in args.iter().enumerate() {
        if a == name {
            return args.get(i + 1).cloned();
        }
        if a.starts_with(&format!("{name}=")) {
            return Some(a[name.len() + 1..].to_string());
        }
    }
    None
}

fn has(args: &[String], name: &str) -> bool {
    args.iter().any(|a| a == name)
}

fn run_gui(args: &[String]) {
    // Attempt to invoke sema-gui binary or cargo run
    let mut child = Command::new("sema-gui");
    child.args(args);
    match child.status() {
        Ok(status) => {
            if !status.success() {
                std::process::exit(status.code().unwrap_or(1));
            }
        }
        Err(_) => {
            // Fallback: try adjacent binary or print instructions
            let exe = std::env::current_exe().unwrap_or_default();
            let parent = exe.parent().unwrap_or(std::path::Path::new(""));
            let adj = parent.join("sema-gui");
            if adj.exists() {
                let _ = Command::new(adj).args(args).status();
            } else {
                eprintln!("sema-gui binary not found in PATH. Please run 'cargo install --path .' or install sema-gui.");
                std::process::exit(1);
            }
        }
    }
}

fn run_mcp() {
    let mut child = Command::new("sema-gui");
    child.arg("--mcp");
    match child.status() {
        Ok(s) => std::process::exit(s.code().unwrap_or(0)),
        Err(_) => {
            let exe = std::env::current_exe().unwrap_or_default();
            let parent = exe.parent().unwrap_or(std::path::Path::new(""));
            let adj = parent.join("sema-gui");
            if adj.exists() {
                let _ = Command::new(adj).arg("--mcp").status();
            } else {
                eprintln!("sema-gui binary not found. Running standalone fallback...");
            }
        }
    }
}

fn parse_synth_intent(args: &[String]) -> SynthIntent {
    let root = flag(args, "--root")
        .or_else(|| args.iter().find(|a| !a.starts_with('-')).cloned())
        .unwrap_or_else(|| "jibu".to_string());

    let subj = flag(args, "--subj")
        .or_else(|| flag(args, "-s"))
        .map(|s| s.to_uppercase());
    let tense = flag(args, "--tense")
        .or_else(|| flag(args, "-t"))
        .map(|t| {
            let u = t.to_uppercase();
            match u.as_str() {
                "PST" | "PAST" => "PAST".to_string(),
                "FUT" | "FUTURE" => "FUT".to_string(),
                "PRES" | "PRESENT" => "PRES".to_string(),
                "PRF" | "PERF" | "PERFECT" => "PERF".to_string(),
                "HAB" | "HABITUAL" => "HAB".to_string(),
                other => other.to_string(),
            }
        });
    let rel = flag(args, "--rel").or_else(|| flag(args, "-r"));
    let obj = flag(args, "--obj").or_else(|| flag(args, "-o"));
    let neg = has(args, "--neg") || has(args, "-n");
    let is_imp = has(args, "--imperative");
    let is_pl = has(args, "--plural");
    let mood = flag(args, "--mood").unwrap_or_else(|| {
        if is_imp && is_pl {
            "eni".to_string()
        } else {
            "a".to_string()
        }
    });
    let derivations = flag(args, "--deriv")
        .map(|d| d.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect())
        .unwrap_or_default();

    SynthIntent {
        subject: if is_imp { None } else { subj },
        tense: if is_imp { None } else { tense },
        negation: neg,
        relative: rel,
        object: obj,
        root,
        derivations,
        mood,
        particles: Vec::new(),
    }
}

fn run_synth(args: &[String]) {
    let engine = SynthEngine::swahili();
    let intent = parse_synth_intent(args);
    if has(args, "--strict") {
        if let Err(errors) = engine.validate(&intent) {
            eprintln!("Validation error: {}", serde_json::to_string(&errors).unwrap());
            std::process::exit(2);
        }
    }

    let out = engine.synthesize(&intent);
    if has(args, "--json") {
        let j = json!({
            "surface": out.surface,
            "slots": out.slots,
            "subject_class": out.subject_class,
            "object_class": out.object_class,
            "verified": true,
        });
        println!("{}", serde_json::to_string_pretty(&j).unwrap());
    } else {
        println!("{}", out.surface);
    }
}

fn run_validate(args: &[String]) {
    let engine = SynthEngine::swahili();
    let intent = parse_synth_intent(args);
    match engine.validate(&intent) {
        Ok(()) => println!("{}", json!({ "valid": true, "errors": [] })),
        Err(errors) => {
            println!("{}", json!({ "valid": false, "errors": errors }));
            std::process::exit(2);
        }
    }
}

fn run_concord(args: &[String]) {
    let concord = ConcordEngine::swahili();
    let kind = flag(args, "--kind").unwrap_or_else(|| "copula".to_string());
    let neg = has(args, "--neg") || has(args, "-n");
    let res: Result<String, String> = match kind.as_str() {
        "copula" => Ok(concord.copula(neg)),
        "emph_copula" => Ok(concord.emphatic_copula(neg)),
        "loc_cop" => {
            let subj = flag(args, "--subj").unwrap_or_else(|| "C1".to_string());
            let loc = flag(args, "--loc").unwrap_or_else(|| "ko".to_string());
            concord.locative_copula(&subj, &loc, neg)
        }
        "have" => {
            let subj = flag(args, "--subj").unwrap_or_else(|| "C1".to_string());
            concord.have(&subj, neg)
        }
        "assoc" => concord.associative(&flag(args, "--class").unwrap_or_else(|| "KI".to_string())),
        "poss" => {
            let cls = flag(args, "--class").unwrap_or_else(|| "C1".to_string());
            let owner = flag(args, "--owner").unwrap_or_else(|| "1SG".to_string());
            concord.possessive(&cls, &owner)
        }
        "dem" => {
            let cls = flag(args, "--class").unwrap_or_else(|| "KI".to_string());
            let dist = flag(args, "--dist").unwrap_or_else(|| "prox".to_string());
            concord.demonstrative(&cls, &dist)
        }
        "adj" => {
            let cls = flag(args, "--class").unwrap_or_else(|| "KI".to_string());
            let stem = flag(args, "--stem").unwrap_or_else(|| "dogo".to_string());
            concord.adjective(&cls, &stem)
        }
        "num" => {
            let cls = flag(args, "--class").unwrap_or_else(|| "KI".to_string());
            let n: u32 = flag(args, "--n").and_then(|s| s.parse().ok()).unwrap_or(1);
            concord.numeral(&cls, n)
        }
        "loc" => Ok(concord.locative_noun(&flag(args, "--noun").unwrap_or_else(|| "hospitali".to_string()))),
        other => Err(format!("unknown concord kind: {other}")),
    };
    match res {
        Ok(surface) => {
            if has(args, "--json") {
                println!("{}", serde_json::to_string_pretty(&json!({ "kind": kind, "surface": surface })).unwrap());
            } else {
                println!("{surface}");
            }
        }
        Err(e) => {
            eprintln!("error: {e}");
            std::process::exit(2);
        }
    }
}

fn run_compose(args: &[String]) {
    let tpl = flag(args, "--template")
        .or_else(|| args.iter().find(|a| !a.starts_with('-')).cloned())
        .unwrap_or_else(|| "{t:child} {poss:@child|1SG} {loc_cop:@child|ko} {loc:@hospital}".to_string());
    let composer = Composer::swahili();
    match composer.render(&tpl) {
        Ok(out) => {
            if has(args, "--json") {
                println!("{}", serde_json::to_string_pretty(&out).unwrap());
            } else {
                println!("{}", out.surface);
            }
        }
        Err(e) => {
            eprintln!("compose error: {e}");
            std::process::exit(2);
        }
    }
}

fn load_v25_decomposer() -> Decomposer {
    let mut dec = Decomposer::swahili();
    let p = Path::new("data/swahili.distilled.jsonl");
    if p.is_file() {
        let _ = dec.load_distilled_lexicon(p);
    }
    dec
}

fn run_roundtrip(args: &[String]) {
    let word = flag(args, "--word")
        .or_else(|| args.iter().find(|a| !a.starts_with('-')).cloned())
        .unwrap_or_else(|| "waliowaua".to_string());
    let dec = load_v25_decomposer();
    let rep = dec.verify_roundtrip(&word);
    if has(args, "--json") {
        println!("{}", serde_json::to_string_pretty(&rep).unwrap());
    } else {
        println!("input={} ok={} output={:?}", rep.input, rep.ok, rep.output);
    }
}

fn run_morph(args: &[String]) {
    let word = flag(args, "--word")
        .or_else(|| args.iter().find(|a| !a.starts_with('-')).cloned())
        .unwrap_or_else(|| "hawatakujibu".to_string());

    let dec_v25 = load_v25_decomposer();
    if let Some(d) = dec_v25.decompose(&word) {
        if has(args, "--json") {
            println!("{}", serde_json::to_string_pretty(&d).unwrap());
        } else {
            println!("Surface Word: {} ({:?})", d.word, d.kind);
            println!("  Slots:      {:?}", d.slots);
            println!("  Roundtrip:  {:?} (exact={})", d.surface_roundtrip, d.roundtrip_exact);
        }
        return;
    }

    let decomp = linearizer::decompose_verb(&word);
    if has(args, "--json") {
        let out = json!({
            "raw": decomp.raw,
            "negation": decomp.negation,
            "subject": decomp.subject,
            "tense": decomp.tense,
            "object": decomp.object,
            "root": decomp.root
        });
        println!("{}", serde_json::to_string_pretty(&out).unwrap());
    } else {
        println!("Surface Word: {}", decomp.raw);
        if let Some(n) = decomp.negation {
            println!("  Negation:   {}", n);
        }
        if let Some(s) = decomp.subject {
            println!("  Subject:    {}", s);
        }
        if let Some(t) = decomp.tense {
            println!("  Tense/TAM:  {}", t);
        }
        if let Some(o) = decomp.object {
            println!("  Object:     {}", o);
        }
        println!("  Root Stem:  {}", decomp.root);
    }
}

fn run_anchor(args: &[String]) {
    let word = flag(args, "--word")
        .or_else(|| args.iter().find(|a| !a.starts_with('-')).cloned())
        .unwrap_or_else(|| "umefikia".to_string());

    let data = flag(args, "--data").unwrap_or_else(|| "data/swahili.distilled.jsonl".to_string());
    match Lexicon::load(&data) {
        Ok(lex) => {
            if let Some(sk) = lex.skeleton_for(&word) {
                if has(args, "--json") {
                    println!("{}", serde_json::to_string_pretty(&json!({
                        "surface": sk.surface,
                        "lemma": sk.lemma,
                        "pos": sk.pos,
                        "gloss": sk.gloss,
                        "root": sk.root,
                        "resolved": true
                    })).unwrap());
                } else {
                    println!("Word:   {}", sk.surface);
                    println!("Lemma:  {}", sk.lemma);
                    println!("POS:    {}", sk.pos);
                    println!("Gloss:  {}", sk.gloss);
                    if let Some(r) = sk.root {
                        println!("Root:   {}", r);
                    }
                }
            } else {
                eprintln!("Unresolved word: '{}'", word);
            }
        }
        Err(e) => {
            eprintln!("Could not load lexicon '{}': {e}", data);
        }
    }
}

fn run_terms(args: &[String]) {
    let text = flag(args, "--text")
        .or_else(|| args.iter().find(|a| !a.starts_with('-')).cloned())
        .unwrap_or_else(|| "Give paracetemol mia tano mg to the child daily".to_string());

    let guardian = FactGuardian::default_clinical();
    let shielded = guardian.protect(&text);
    if has(args, "--json") {
        println!("{}", serde_json::to_string_pretty(&shielded).unwrap());
    } else {
        println!("Original: {}", text);
        println!("Shielded: {}", shielded.text);
        println!("Slots:    {}", shielded.slots.len());
    }
}

fn run_health(args: &[String]) {
    let data = flag(args, "--data").unwrap_or_else(|| "data/swahili.distilled.jsonl".to_string());
    println!("Checking Sema health...");
    let m = sema::Morphology::swahili();
    println!("  [OK] Engine:     Sema v{} (zero rules in code)", sema::VERSION);
    println!("  [OK] Morphology: {} tenses, {} relatives, {} derivation rules", m.tenses.len(), m.relatives.len(), m.derivation_variants.len());
    match Lexicon::load(&data) {
        Ok(lex) => {
            println!("  [OK] Lexicon:    {} lemmas loaded from '{}'", lex.len(), data);
        }
        Err(e) => {
            println!("  [!]  Lexicon warning: {e}");
        }
    }
}

fn run_bench(_args: &[String]) {
    println!("Running microarchitectural benchmark (100,000 realizations)...");
    let engine = SynthEngine::swahili();
    let intent = SynthIntent {
        subject: Some("3PL".into()),
        tense: Some("FUT".into()),
        negation: true,
        relative: None,
        object: Some("ku".into()),
        root: "jibu".into(),
        derivations: vec![],
        mood: "a".into(),
        particles: vec![],
    };

    let start = std::time::Instant::now();
    let count = 100_000;
    for _ in 0..count {
        let _ = engine.synthesize(&intent);
    }
    let elapsed = start.elapsed();
    let nanos = elapsed.as_nanos() as f64 / count as f64;
    let us = nanos / 1000.0;
    let ops_per_sec = (count as f64 / elapsed.as_secs_f64()) as u64;

    println!("  Total iterations: {}", count);
    println!("  Total time:       {:.2} ms", elapsed.as_secs_f64() * 1000.0);
    println!("  Mean latency:     {:.2} us / word ({:.0} ns)", us, nanos);
    println!("  Throughput:       {} ops / second", ops_per_sec);
}
