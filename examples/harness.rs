use sema::backpass::{engine, intent_from_parts, Intent};
use sema::lexicon::segment_words;
use sema::linearizer::{decompose_verb, linearize_sentence};
use sema::{Lexicon, Racer};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::time::Instant;

fn s(v: &Value, k: &str) -> Option<String> {
    v.get(k).and_then(|x| x.as_str()).map(String::from)
}
fn intent_of(v: &Value) -> Intent {
    Intent {
        subject: s(v, "subject"),
        tense: s(v, "tense"),
        negation: v.get("negation").and_then(|x| x.as_bool()).unwrap_or(false),
        object: s(v, "object"),
        root: s(v, "root").unwrap_or_default(),
        derivations: v.get("derivations").and_then(|x| x.as_array())
            .map(|a| a.iter().filter_map(|x| x.as_str().map(String::from)).collect()).unwrap_or_default(),
        mood: s(v, "mood").unwrap_or_else(|| "a".into()),
        particles: vec![],
    }
}
fn csv_line(line: &str) -> Vec<String> {
    let (mut out, mut cur, mut q) = (vec![], String::new(), false);
    let mut it = line.chars().peekable();
    while let Some(c) = it.next() {
        if q { if c == '"' { if it.peek() == Some(&'"') { cur.push('"'); it.next(); } else { q = false } } else { cur.push(c) } }
        else if c == '"' { q = true } else if c == ',' { out.push(std::mem::take(&mut cur)) } else { cur.push(c) }
    }
    out.push(cur); out
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let e = engine();
    match a[1].as_str() {
        "back" => {
            let gold: Vec<Value> = serde_json::from_str(&std::fs::read_to_string(&a[2]).unwrap()).unwrap();
            let mut out = vec![];
            for g in &gold {
                let r = e.synthesize(&intent_of(g));
                let exp = s(g, "expected").unwrap();
                out.push(json!({"id":g["id"],"cat":g["cat"],"input":{"subject":g["subject"],"tense":g["tense"],"negation":g["negation"],"object":g["object"],"root":g["root"],"mood":g["mood"],"derivations":g.get("derivations")},
                    "expected":exp,"got":r.surface,"ok":r.surface.trim()==exp,"slots":r.slots}));
            }
            std::fs::write(&a[3], serde_json::to_string_pretty(&out).unwrap()).unwrap();
        }
        "bench_back" => {
            // shipped benchmark: EN_TO_SW rows, intent_json -> expected_verb
            let raw = std::fs::read_to_string(&a[2]).unwrap();
            let raw = raw.trim_start_matches('\u{feff}');
            let mut out = vec![];
            for (i, l) in raw.lines().enumerate() {
                if i == 0 || l.trim().is_empty() { continue }
                let f = csv_line(l);
                if f.len() < 14 || f[1] != "EN_TO_SW" || f[12].is_empty() { continue }
                let v: Value = serde_json::from_str(&f[12]).unwrap();
                let r = e.synthesize(&intent_of(&v));
                out.push(json!({"id":f[0],"cat":f[6],"intent":v,"expected":f[13],"got":r.surface,"ok":r.surface.trim()==f[13].trim()}));
            }
            std::fs::write(&a[3], serde_json::to_string_pretty(&out).unwrap()).unwrap();
        }
        "fwd" => {
            let lex = Lexicon::load("data/swahili.distilled.jsonl").unwrap();
            let racer = Racer::new(&lex);
            let txt = std::fs::read_to_string(&a[2]).unwrap();
            let mut out = vec![];
            for line in txt.lines().filter(|l| !l.trim().is_empty()) {
                let words = segment_words(line);
                let mut skels = vec![]; let mut wj = vec![];
                for w in words.iter().filter(|w| w.chars().any(|c| c.is_alphabetic())) {
                    let sk = lex.skeleton_for(w);
                    let d = decompose_verb(w);
                    wj.push(json!({"w":w,"anchor": sk.as_ref().map(|k| json!({"lemma":k.lemma,"pos":k.pos,"gloss":k.gloss,"via":k.resolved_via_form})),
                        "decomp":{"neg":d.negation,"subj":d.subject,"tense":d.tense,"obj":d.object,"root":d.root}}));
                    if let Some(k) = sk { skels.push(k) }
                }
                let lin = linearize_sentence(&skels, line);
                let t0 = Instant::now();
                let rr = racer.race_sentence(line);
                let dt = t0.elapsed().as_secs_f64() * 1e6;
                let anch = racer.anchor_sentence(line);
                out.push(json!({"sentence":line,"words":wj,"telegram":lin.to_telegram(),"anchor_telegram":anch.to_telegram(),"anchor_tokens":anch.tokens.iter().map(|t| format!("{:?}:{}",t.role,t.word)).collect::<Vec<_>>(),
                    "race":rr.iter().map(|r| json!({"racer":r.racer_id,"score":r.score,"tokens":r.tokens.iter().map(|t| format!("{:?}:{}",t.role,t.word)).collect::<Vec<_>>()})).collect::<Vec<_>>(),"race_us":dt}));
            }
            std::fs::write(&a[3], serde_json::to_string_pretty(&out).unwrap()).unwrap();
        }
        "rt" => {
            let lex = Lexicon::load("data/swahili.distilled.jsonl").unwrap();
            let raw = std::fs::read_to_string("data/swahili.distilled.jsonl").unwrap();
            let mut roots = vec![];
            for l in raw.lines() {
                let v: Value = serde_json::from_str(l).unwrap();
                if v["p"] != "verb" { continue }
                let w = v["w"].as_str().unwrap();
                let g0 = v["g"][0].as_str().unwrap_or("");
                if w.contains(' ') || w.len() < 2 || g0.starts_with("infinitive of") || g0.starts_with("inflection of") || w.starts_with("ku") && g0.starts_with("infinitive") { continue }
                roots.push(w.to_string());
            }
            roots.retain(|r| "aeiou".contains(r.chars().last().unwrap()));
            roots.sort(); roots.dedup();
            let subjects = ["1SG","2SG","3SG","1PL","2PL","3PL"];
            let subj_surf = [("1SG","ni"),("2SG","u"),("3SG","a"),("1PL","tu"),("2PL","m"),("3PL","wa")];
            let tenses = ["PRES","PAST","FUT","PERF"];
            let objs: [Option<&str>;3] = [None, Some("ku"), Some("wa")];
            #[derive(Default)] struct St { n: u64, resynth: u64, strict: u64, anchor: u64, anchor_n: u64 }
            let mut tot = St::default();
            let mut by: BTreeMap<String, St> = BTreeMap::new();
            let mut fails: Vec<Value> = vec![];
            let mut tax: BTreeMap<String, u64> = BTreeMap::new();
            let t0 = Instant::now();
            for r in &roots {
                let mono = r.chars().count() <= 2 && !r.contains(|_c| false);
                let vowel_init = "aeiou".contains(r.chars().next().unwrap());
                for sj in subjects { for t in tenses { for neg in [false, true] { for o in objs {
                    let intent = Intent { subject: Some(sj.into()), tense: Some(t.into()), negation: neg, object: o.map(String::from), root: r.clone(), derivations: vec![], mood: "a".into(), particles: vec![] };
                    let syn = e.synthesize(&intent).surface;
                    let d = decompose_verb(&syn);
                    let ri = intent_from_parts(&e, d.subject.as_deref(), d.tense.as_deref(), d.negation.is_some(), d.object.as_deref(), &d.root);
                    let re = e.synthesize(&ri).surface;
                    let want_subj = subj_surf.iter().find(|x| x.0 == sj).unwrap().1;
                    let root_ok = d.root == *r || sema::linearizer::root_candidates(&d).iter().any(|c| c == r);
                    let strict = root_ok && d.subject.as_deref() == Some(want_subj) && d.negation.is_some() == neg && d.object.as_deref() == o;
                    let ok_re = re == syn;
                    let mut keys = vec!["ALL".to_string(), format!("neg={neg}"), format!("tense={t}"), format!("obj={}", o.unwrap_or("none")), format!("class={}", if mono {"mono(<=2 chars)"} else if vowel_init {"vowel-initial"} else {"regular"})];
                    keys.dedup();
                    for k in &keys { let st = if k == "ALL" { &mut tot } else { by.entry(k.clone()).or_default() }; st.n += 1; if ok_re { st.resynth += 1 } if strict { st.strict += 1 } }
                    if !strict {
                        let kind = if !ok_re {"resynth_mismatch"} else if !root_ok { let (x,y)=(d.root.chars().collect::<Vec<_>>(), r.chars().collect::<Vec<_>>()); if x.len()==y.len() && x[..x.len()-1]==y[..y.len()-1] {"root_final_vowel_not_restored"} else if d.tense.as_deref()==Some("ku") || d.object.as_deref()==Some("ku") {"root_eats_or_misassigns_ku"} else {"root_wrong_other"} } else if d.subject.as_deref() != Some(want_subj) {"subject_wrong"} else if (d.negation.is_some()) != neg {"neg_wrong"} else {"object_wrong"};
                        *tax.entry(kind.into()).or_default() += 1;
                        if fails.len() < 4000 && (tax[kind] <= 40) { fails.push(json!({"kind":kind,"in":format!("{sj}/{t}/neg={neg}/obj={:?}/{r}",o),"surface":syn,"decomp":format!("neg={:?} subj={:?} tense={:?} obj={:?} root={}",d.negation,d.subject,d.tense,d.object,d.root),"resynth":re})); }
                    }
                }}}}
                // forward anchor on a fixed set of surfaces per verb
                for (sj, t) in [("1SG","PRES"),("3SG","PAST"),("3PL","FUT"),("2SG","PERF")] {
                    let syn = e.synthesize(&Intent { subject: Some(sj.into()), tense: Some(t.into()), negation: false, object: None, root: r.clone(), derivations: vec![], mood: "a".into(), particles: vec![] }).surface;
                    tot.anchor_n += 1;
                    let hit = lex.skeleton_for(&syn).map(|k| k.lemma == *r || k.root.as_deref().map(|x| x.trim_start_matches('-')) == Some(r.as_str())).unwrap_or(false);
                    if hit { tot.anchor += 1 }
                }
            }
            let dt = t0.elapsed().as_secs_f64();
            let mut bj = serde_json::Map::new();
            for (k, v) in &by { bj.insert(k.clone(), json!({"n":v.n,"resynth_pct":100.0*v.resynth as f64/v.n as f64,"strict_pct":100.0*v.strict as f64/v.n as f64})); }
            let res = json!({"verbs":roots.len(),"cases":tot.n,"seconds":dt,"resynth_pct":100.0*tot.resynth as f64/tot.n as f64,"strict_pct":100.0*tot.strict as f64/tot.n as f64,
              "anchor_lemma_hit_pct":100.0*tot.anchor as f64/tot.anchor_n as f64,"anchor_n":tot.anchor_n,"by":bj,"taxonomy":tax,"fail_samples":fails});
            std::fs::write(&a[2], serde_json::to_string_pretty(&res).unwrap()).unwrap();
        }
        "perf" => {
            let lex = Lexicon::load("data/swahili.distilled.jsonl").unwrap();
            let n = 200_000u32;
            let i = Intent::negated("3PL","FUT","jibu");
            let t = Instant::now(); let mut x = 0usize; for _ in 0..n { x += e.synthesize(&i).surface.len() } let synth = t.elapsed().as_secs_f64()*1e6/n as f64;
            let t = Instant::now(); for _ in 0..n { x += decompose_verb("hawatakujibu").root.len() } let dec = t.elapsed().as_secs_f64()*1e6/n as f64;
            let t = Instant::now(); for _ in 0..n { x += lex.skeleton_for("umefikia").map(|k| k.gloss.len()).unwrap_or(0) } let anc = t.elapsed().as_secs_f64()*1e6/n as f64;
            let t = Instant::now(); let t0 = Instant::now(); let _ = Lexicon::load("data/swahili.distilled.jsonl").unwrap(); let load = t0.elapsed().as_secs_f64()*1e3; let _ = t;
            println!("{}", json!({"synth_us":synth,"decompose_us":dec,"anchor_us":anc,"lexicon_load_ms":load,"sink":x}));
        }
        _ => panic!("mode"),
    }
}
