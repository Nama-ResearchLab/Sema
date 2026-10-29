use sema::rolodex::{Intent, Rolodex};
use serde_json::Value;

fn run(file: &str) -> Vec<String> {
    let p = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("bench/gold").join(file);
    let v: Vec<Value> = serde_json::from_str(&std::fs::read_to_string(p).unwrap()).unwrap();
    let r = Rolodex::swahili();
    let s = |x: &Value| x.as_str().map(String::from);
    let mut fails = vec![];
    for row in &v {
        let i = Intent {
            subject: s(&row["subject"]),
            tense: s(&row["tense"]),
            negation: row["negation"].as_bool().unwrap_or(false),
            object: s(&row["object"]),
            root: row["root"].as_str().unwrap().into(),
            derivations: row["derivations"].as_array().map(|a| a.iter().filter_map(|x| s(x)).collect()).unwrap_or_default(),
            mood: s(&row["mood"]).unwrap_or("a".into()),
            particles: vec![],
        };
        let got = r.synthesize(&i).surface;
        let want = row["expected"].as_str().unwrap();
        if got != want {
            fails.push(format!("{} {:?}: got {got} want {want}", row["id"], i));
        }
    }
    fails
}

#[test]
fn gold_back100() {
    let f = run("back100.json");
    assert!(f.is_empty(), "{} fails:\n{}", f.len(), f.join("\n"));
}

#[test]
fn gold_derivations_stretch() {
    let f = run("back_stretch_derivations.json");
    assert!(f.is_empty(), "{} fails:\n{}", f.len(), f.join("\n"));
}
