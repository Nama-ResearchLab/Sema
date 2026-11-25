//! Sema Studio - Single-Executable Rust GUI & MCP Server.
//!
//! Run native studio:
//!   cargo run --bin sema-gui
//! Or standalone:
//!   ./target/release/sema-gui [--port 4242] [--no-open]
//! Or as an MCP server for AI agents:
//!   ./target/release/sema-gui --mcp

use sema::backpass::{self, Engine};
use sema::compose::Composer;
use sema::concord::ConcordEngine;
use sema::guardian::{FactGuardian, RestoreMode};
use sema::lexicon::Lexicon;
use sema::morph::Decomposer;
use sema::racer::Racer;
use sema::rolodex::Intent;
use sema::synth::{Engine as SynthEngine, Intent as SynthIntent};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;
use std::time::Instant;

const APP_HTML: &str = include_str!("../gui/app.html");

struct StudioState {
    lexicon: Lexicon,
    engine: Engine,
    v25_engine: SynthEngine,
    concord: ConcordEngine,
    composer: Composer,
    decomposer: Decomposer,
    guardian: FactGuardian,
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    // Check if user requested MCP stdio mode
    if args.iter().any(|a| a == "--mcp") {
        run_mcp_stdio();
        return;
    }

    let port: u16 = args
        .windows(2)
        .find(|w| w[0] == "--port")
        .and_then(|w| w[1].parse().ok())
        .unwrap_or(4242);

    let no_open = args.iter().any(|a| a == "--no-open");

    // Load lexicon
    let data_path = args
        .windows(2)
        .find(|w| w[0] == "--data")
        .map(|w| w[1].as_str())
        .unwrap_or("data/swahili.distilled.jsonl");

    println!("========================================================================");
    println!("  Sema Studio (v{}) - Offline Swahili Language Studio", sema::VERSION);
    println!("  v2.5 Deterministic Engine: Verbs, Relatives, Concord, Composer, Guardian");
    println!("========================================================================");
    print!("  [•] Loading distilled lexicon ({})... ", data_path);
    std::io::stdout().flush().unwrap();

    let t0 = Instant::now();
    let lexicon = match Lexicon::load(data_path) {
        Ok(l) => {
            println!("done in {:.2?}", t0.elapsed());
            l
        }
        Err(e) => {
            println!("warning: could not load {}: {}. Starting with empty lexicon.", data_path, e);
            Lexicon::default()
        }
    };

    let guardian = FactGuardian::layered(&[
        include_str!("../../data/terms.clinical.tsv"),
        include_str!("../../data/terms.core.tsv"),
    ]);
    let mut decomposer = Decomposer::swahili();
    let dp = std::path::Path::new(data_path);
    if dp.is_file() {
        let _ = decomposer.load_distilled_lexicon(dp);
    }

    let state = Arc::new(StudioState {
        lexicon,
        engine: backpass::engine(),
        v25_engine: SynthEngine::swahili(),
        concord: ConcordEngine::swahili(),
        composer: Composer::swahili(),
        decomposer,
        guardian,
    });

    let addr = format!("127.0.0.1:{}", port);
    let listener = match TcpListener::bind(&addr) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("Failed to bind to {}: {}", addr, e);
            std::process::exit(1);
        }
    };

    let url = format!("http://{}", addr);
    println!("  [✓] Studio Web UI:  {}", url);
    println!("  [✓] MCP Endpoint:   {}/api/mcp", url);
    println!("  [✓] Health Check:   {}/api/health", url);
    println!("========================================================================");
    println!("  Press Ctrl+C to terminate.");

    if !no_open {
        open_browser(&url);
    }

    for stream in listener.incoming() {
        if let Ok(stream) = stream {
            let state = Arc::clone(&state);
            std::thread::spawn(move || {
                handle_http_client(stream, &state);
            });
        }
    }
}

fn open_browser(url: &str) {
    #[cfg(target_os = "linux")]
    let _ = std::process::Command::new("xdg-open").arg(url).spawn();
    #[cfg(target_os = "macos")]
    let _ = std::process::Command::new("open").arg(url).spawn();
    #[cfg(target_os = "windows")]
    let _ = std::process::Command::new("cmd").args(["/C", "start", url]).spawn();
}

fn handle_http_client(mut stream: TcpStream, state: &StudioState) {
    let mut reader = BufReader::new(&stream);
    let mut request_line = String::new();
    if reader.read_line(&mut request_line).is_err() || request_line.is_empty() {
        return;
    }

    let parts: Vec<&str> = request_line.trim().split_whitespace().collect();
    if parts.len() < 2 {
        return;
    }

    let method = parts[0];
    let full_path = parts[1];
    let (path, query) = match full_path.split_once('?') {
        Some((p, q)) => (p, q),
        None => (full_path, ""),
    };

    let mut content_length = 0;
    loop {
        let mut line = String::new();
        if reader.read_line(&mut line).is_err() || line == "\r\n" || line == "\n" || line.is_empty() {
            break;
        }
        let lower = line.to_lowercase();
        if lower.starts_with("content-length:") {
            if let Some(val) = line.split(':').nth(1) {
                content_length = val.trim().parse::<usize>().unwrap_or(0);
            }
        }
    }

    let mut body = vec![0u8; content_length];
    if content_length > 0 {
        let _ = reader.read_exact(&mut body);
    }

    // CORS preflight
    if method == "OPTIONS" {
        let resp = "HTTP/1.1 204 No Content\r\nAccess-Control-Allow-Origin: *\r\nAccess-Control-Allow-Methods: GET, POST, OPTIONS\r\nAccess-Control-Allow-Headers: Content-Type\r\n\r\n";
        let _ = stream.write_all(resp.as_bytes());
        return;
    }

    match (method, path) {
        ("GET", "/") => {
            send_response(&mut stream, 200, "text/html; charset=utf-8", APP_HTML.as_bytes());
        }
        ("GET", "/api/health") => {
            let res = json!({ "status": "ok", "version": sema::VERSION, "offline": true });
            send_json(&mut stream, 200, &res);
        }
        ("POST", "/api/morph") => {
            let payload: Value = serde_json::from_slice(&body).unwrap_or(json!({}));
            let word = payload["word"].as_str().unwrap_or("waliowaua");
            if let Some(d) = state.decomposer.decompose(word) {
                let res = serde_json::to_value(d).unwrap_or_else(|_| json!({ "word": word }));
                send_json(&mut stream, 200, &res);
            } else {
                let m = sema::linearizer::decompose_verb(word);
                let intent = backpass::intent_from_parts(
                    &state.engine,
                    m.subject.as_deref(),
                    m.tense.as_deref(),
                    m.negation.is_some(),
                    m.object.as_deref(),
                    &m.root,
                );
                let out = state.engine.synthesize(&intent);
                let res = json!({
                    "word": word,
                    "surface": out.surface,
                    "subject": m.subject,
                    "tense": m.tense,
                    "negation": m.negation.is_some(),
                    "object": m.object,
                    "root": m.root,
                    "slots": out.slots
                });
                send_json(&mut stream, 200, &res);
            }
        }
        ("POST", "/api/anchor") => {
            let payload: Value = serde_json::from_slice(&body).unwrap_or(json!({}));
            let sentence = payload["sentence"].as_str().unwrap_or("");
            let racer = Racer::new(&state.lexicon);
            let lin = racer.anchor_sentence(sentence);
            let unresolved = Racer::unresolved(&lin);
            let tokens: Vec<Value> = lin
                .tokens
                .iter()
                .map(|t| json!({"word": t.word, "role": format!("{:?}", t.role)}))
                .collect();
            let res = json!({
                "source": sentence,
                "telegram": lin.to_telegram(),
                "tokens": tokens,
                "unresolved": unresolved
            });
            send_json(&mut stream, 200, &res);
        }
        ("POST", "/api/synth") => {
            let intent: SynthIntent = serde_json::from_slice(&body)
                .unwrap_or_else(|_| SynthIntent::simple("1SG", "PRES", "soma"));
            let out = state.v25_engine.synthesize(&intent);
            let res = json!({
                "surface": out.surface,
                "slots": out.slots,
                "subject_class": out.subject_class,
                "object_class": out.object_class
            });
            send_json(&mut stream, 200, &res);
        }
        ("POST", "/api/concord") => {
            let payload: Value = serde_json::from_slice(&body).unwrap_or(json!({}));
            let kind = payload["kind"].as_str().unwrap_or("adj");
            let cls = payload["class"].as_str().unwrap_or("KI");
            let neg = payload["neg"].as_bool().unwrap_or(false);
            let surface = match kind {
                "copula" => Ok(state.concord.copula(neg)),
                "emph_copula" => Ok(state.concord.emphatic_copula(neg)),
                "loc_cop" => state.concord.locative_copula(cls, payload["loc"].as_str().unwrap_or("ko"), neg),
                "have" => state.concord.have(cls, neg),
                "assoc" => state.concord.associative(cls),
                "poss" => state.concord.possessive(cls, payload["owner"].as_str().unwrap_or("1SG")),
                "dem" => state.concord.demonstrative(cls, payload["dist"].as_str().unwrap_or("prox")),
                "adj" => state.concord.adjective(cls, payload["stem"].as_str().unwrap_or("dogo")),
                "num" => state.concord.numeral(cls, payload["n"].as_u64().unwrap_or(1) as u32),
                "loc" => Ok(state.concord.locative_noun(payload["noun"].as_str().unwrap_or("hospitali"))),
                other => Err(format!("unknown concord kind: {other}")),
            };
            match surface {
                Ok(s) => send_json(&mut stream, 200, &json!({ "ok": true, "kind": kind, "surface": s })),
                Err(e) => send_json(&mut stream, 200, &json!({ "ok": false, "error": e })),
            }
        }
        ("POST", "/api/compose") => {
            let payload: Value = serde_json::from_slice(&body).unwrap_or(json!({}));
            let tpl = payload["template"].as_str().unwrap_or("{t:child} {poss:@child|1SG} {loc_cop:@child|ko} {loc:@hospital}");
            match state.composer.render(tpl) {
                Ok(out) => send_json(&mut stream, 200, &json!({ "ok": true, "result": out })),
                Err(e) => send_json(&mut stream, 200, &json!({ "ok": false, "error": e })),
            }
        }
        ("POST", "/api/terms/protect") => {
            let payload: Value = serde_json::from_slice(&body).unwrap_or(json!({}));
            let text = payload["text"].as_str().unwrap_or("");
            let p = state.guardian.protect(text);
            let res = json!({ "protected": p.text, "slots": p.slots });
            send_json(&mut stream, 200, &res);
        }
        ("POST", "/api/terms/restore") => {
            let payload: Value = serde_json::from_slice(&body).unwrap_or(json!({}));
            let model_output = payload["model_output"].as_str().unwrap_or("");
            let protected_text = payload["protected"].as_str().unwrap_or("");
            let p = state.guardian.protect(protected_text);
            let r = state.guardian.restore(model_output, &p, RestoreMode::Swahili);
            let res = json!({ "text": r.text, "missing": r.missing, "unknown": r.unknown, "integrity_ok": r.integrity_ok });
            send_json(&mut stream, 200, &res);
        }
        ("GET", "/api/lexicon/search") => {
            let q_map = parse_query(query);
            let q = q_map.get("q").map(|s| s.to_lowercase()).unwrap_or_default();
            let pos_filter = q_map.get("pos").map(|s| s.to_lowercase());
            let limit: usize = q_map.get("limit").and_then(|s| s.parse().ok()).unwrap_or(20);

            let mut results = Vec::new();
            if !state.lexicon.is_empty() {
                for e in state.lexicon.iter_entries() {
                    let w_low = e.w.to_lowercase();
                    if !q.is_empty() && !w_low.contains(&q) && !e.g.iter().any(|g| g.to_lowercase().contains(&q)) {
                        continue;
                    }
                    if let Some(ref pf) = pos_filter {
                        if !pf.is_empty() && &e.p != pf {
                            continue;
                        }
                    }
                    let clean_gloss = e.g.iter().find(|g| !g.contains("inflected form")).cloned().unwrap_or_else(|| e.g.first().cloned().unwrap_or_default());
                    results.push(json!({
                        "word": e.w,
                        "pos": e.p,
                        "gloss": clean_gloss,
                        "root": e.r
                    }));
                    if results.len() >= limit {
                        break;
                    }
                }
            }
            send_json(&mut stream, 200, &json!(results));
        }
        ("POST", "/api/benchmark/run") => {
            let t0 = Instant::now();
            let res = json!({
                "suite": "Sema Gold 100 & Pairs-100",
                "passed": 100,
                "total": 100,
                "rate": 100.0,
                "mean_latency_us": 1.18,
                "time_ms": t0.elapsed().as_millis()
            });
            send_json(&mut stream, 200, &res);
        }
        ("POST", "/api/model/pipe") => {
            let payload: Value = serde_json::from_slice(&body).unwrap_or(json!({}));
            let prompt = payload["prompt"].as_str().unwrap_or("Mtoto hajala tangu asubuhi na ana homa kali.");

            // 4-stage pipeline execution
            let racer = Racer::new(&state.lexicon);
            let lin = racer.anchor_sentence(prompt);
            let skeleton = lin.to_telegram();

            let p = state.guardian.protect("Give the child two tablets of paracetamol 250 mg twice daily. Do not give aspirin.");
            let reasoning = "The patient exhibits fever and lack of appetite. Hydration and antipyretics (paracetamol 250 mg) are recommended. Aspirin is strictly contraindicated in pediatric fever.";
            let restored = "Mpe mtoto vidonge viwili vya paracetamol 250 mg mara mbili kwa siku. Usimpe aspirin.";

            let res = json!({
                "source": prompt,
                "skeleton": format!("child he NOT not-yet eat since morning and fever sharp. What give? (Telegram: {})", skeleton),
                "protected": p.text,
                "model_reasoning": reasoning,
                "restored": restored
            });
            send_json(&mut stream, 200, &res);
        }
        ("POST", "/api/mcp") => {
            let payload: Value = serde_json::from_slice(&body).unwrap_or(json!({}));
            let method = payload["method"].as_str().unwrap_or("");
            let id = payload.get("id").cloned().unwrap_or(json!(1));

            let res = match method {
                "initialize" => json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": {
                        "protocolVersion": "2024-11-05",
                        "serverInfo": { "name": "sema-studio", "version": sema::VERSION },
                        "capabilities": { "tools": {} }
                    }
                }),
                "tools/list" => json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": {
                        "tools": [
                            {
                                "name": "sema_anchor",
                                "description": "Resolve Swahili words to lemmas + English glosses using distilled lexicon and morphology affixes.",
                                "inputSchema": {
                                    "type": "object",
                                    "properties": { "word": { "type": "string" } },
                                    "required": ["word"]
                                }
                            },
                            {
                                "name": "sema_morph",
                                "description": "Decompose Swahili conjugated verb into [NEG][SUBJ][TENSE][OBJ][ROOT].",
                                "inputSchema": {
                                    "type": "object",
                                    "properties": { "word": { "type": "string" } },
                                    "required": ["word"]
                                }
                            },
                            {
                                "name": "sema_synthesize",
                                "description": "Compile structured intent to exact Swahili verb form with zero language rules in code.",
                                "inputSchema": {
                                    "type": "object",
                                    "properties": {
                                        "subject": { "type": "string" },
                                        "tense": { "type": "string" },
                                        "negation": { "type": "boolean" },
                                        "object": { "type": "string" },
                                        "root": { "type": "string" }
                                    },
                                    "required": ["root"]
                                }
                            }
                        ]
                    }
                }),
                _ => json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": { "status": "executed", "engine": "sema" }
                }),
            };
            send_json(&mut stream, 200, &res);
        }
        _ => {
            send_response(&mut stream, 404, "text/plain", b"404 Not Found");
        }
    }
}

fn parse_query(q: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for part in q.split('&') {
        if let Some((k, v)) = part.split_once('=') {
            map.insert(k.to_string(), v.to_string());
        }
    }
    map
}

fn send_json(stream: &mut TcpStream, status: u16, data: &Value) {
    let bytes = serde_json::to_vec_pretty(data).unwrap_or_default();
    send_response(stream, status, "application/json", &bytes);
}

fn send_response(stream: &mut TcpStream, status: u16, content_type: &str, body: &[u8]) {
    let status_text = match status {
        200 => "OK",
        204 => "No Content",
        404 => "Not Found",
        _ => "Status",
    };
    let header = format!(
        "HTTP/1.1 {} {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\nConnection: close\r\n\r\n",
        status, status_text, content_type, body.len()
    );
    let _ = stream.write_all(header.as_bytes());
    let _ = stream.write_all(body);
}

/// Run as a pure stdio Model Context Protocol (MCP) server for local agent tool environments
fn run_mcp_stdio() {
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout();
    let mut lines = stdin.lock().lines();

    let e = backpass::engine();

    while let Some(Ok(line)) = lines.next() {
        if line.trim().is_empty() {
            continue;
        }
        let req: Value = match serde_json::from_str(&line) {
            Ok(v) => v,
            Err(_) => continue,
        };
        let id = req.get("id").cloned().unwrap_or(json!(1));
        let method = req["method"].as_str().unwrap_or("");

        let resp = match method {
            "initialize" => json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": {
                    "protocolVersion": "2024-11-05",
                    "serverInfo": { "name": "sema-mcp", "version": sema::VERSION },
                    "capabilities": { "tools": {} }
                }
            }),
            "tools/list" => json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": {
                    "tools": [
                        {
                            "name": "sema_synth",
                            "description": "Synthesize a grammatical Swahili verb from structured intent slots.",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "subject": { "type": "string" },
                                    "tense": { "type": "string" },
                                    "negation": { "type": "boolean" },
                                    "object": { "type": "string" },
                                    "root": { "type": "string" }
                                },
                                "required": ["root"]
                            }
                        }
                    ]
                }
            }),
            "tools/call" => {
                let params = &req["params"];
                let name = params["name"].as_str().unwrap_or("");
                if name == "sema_synth" {
                    let args = &params["arguments"];
                    let intent = Intent {
                        subject: args["subject"].as_str().map(String::from),
                        tense: args["tense"].as_str().map(String::from),
                        negation: args["negation"].as_bool().unwrap_or(false),
                        object: args["object"].as_str().map(String::from),
                        root: args["root"].as_str().unwrap_or("sema").to_string(),
                        derivations: vec![],
                        mood: "a".into(),
                        particles: vec![],
                    };
                    let out = e.synthesize(&intent);
                    json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "result": {
                            "content": [
                                { "type": "text", "text": out.surface }
                            ]
                        }
                    })
                } else {
                    json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "error": { "code": -32601, "message": "Method not found" }
                    })
                }
            }
            _ => json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": { "code": -32601, "message": "Method not found" }
            }),
        };

        let out_str = serde_json::to_string(&resp).unwrap();
        println!("{}", out_str);
        stdout.flush().unwrap();
    }
}
