//! Sema v2.5 grammar tests: relatives, conditionals, subjectless HAB/INF,
//! imperatives, short-stem object elision, vowel-cluster & borrowed derivations,
//! strict validation, and pure-data decoupling proof.

use sema::synth::{Engine, Intent, Morphology};

fn synth(i: Intent) -> String {
    Engine::swahili().synthesize(&i).surface
}

#[test]
fn relative_clauses_and_future_taka() {
    assert_eq!(
        synth(Intent::simple("3PL", "PAST", "ua").with_relative("C2").with_object("wa")),
        "waliowaua"
    );
    assert_eq!(
        synth(Intent::simple("C10", "PAST", "jaa").with_relative("C10")),
        "zilizojaa"
    );
    assert_eq!(
        synth(Intent::simple("C9", "PRES", "toa").with_relative("C9")),
        "inayotoa"
    );
    assert_eq!(
        synth(Intent::simple("C9", "PRES", "pendeza").with_relative("C9")),
        "inayopendeza"
    );
    assert_eq!(
        synth(
            Intent::simple("3PL", "PRES", "panda")
                .with_relative("C2")
                .with_derivations(["passive"])
        ),
        "wanaopandwa"
    );
    assert_eq!(
        synth(Intent::simple("3PL", "PRES", "weza").with_relative("C2")),
        "wanaoweza"
    );
    assert_eq!(
        synth(Intent::simple("3SG", "PAST", "fika").with_relative("C16")),
        "alipofika"
    );
    assert_eq!(
        synth(Intent::simple("3PL", "FUT", "jibu").with_relative("3PL")),
        "watakaojibu"
    );
    assert_eq!(
        synth(Intent::simple("1SG", "FUT", "fika").with_relative("C16")),
        "nitakapofika"
    );
    assert_eq!(
        synth(Intent::simple("3SG", "PRES", "la").with_relative("C1")),
        "anayekula"
    );
}

#[test]
fn conditionals_and_negative_conditionals() {
    assert_eq!(
        synth(Intent::simple("3SG", "COND", "husisha").with_object("m")),
        "akimhusisha"
    );
    assert_eq!(
        synth(
            Intent::simple("3SG", "COND", "husu")
                .with_object("m")
                .with_derivations(["causative"])
        ),
        "akimhusisha"
    );
    assert_eq!(synth(Intent::simple("1SG", "COND_PRES", "soma")), "ningesoma");
    assert_eq!(synth(Intent::simple("1SG", "COND_PAST", "soma")), "ningalisoma");
    assert_eq!(synth(Intent::negated("1SG", "COND", "soma")), "nisiposoma");
    assert_eq!(synth(Intent::negated("3SG", "COND_PRES", "fanya")), "asingefanya");
    assert_eq!(synth(Intent::negated("1PL", "COND_PAST", "fika")), "tusingalifika");
    assert_eq!(synth(Intent::simple("1SG", "COND_PRES", "la")), "ningekula");
    assert_eq!(synth(Intent::negated("1SG", "COND_PRES", "la")), "nisingekula");
}

#[test]
fn subjectless_habitual_and_infinitive_with_object_glides() {
    // Subject tag provided on HAB/INF must be suppressed cleanly.
    assert_eq!(synth(Intent::simple("3SG", "HAB", "soma")), "husoma");
    assert_eq!(synth(Intent::infinitive("soma")), "kusoma");
    assert_eq!(synth(Intent::infinitive("ua").with_object("m")), "kumuua");
    assert_eq!(synth(Intent::infinitive("kabili").with_object("ya")), "kuyakabili");
    assert_eq!(synth(Intent::infinitive("vaa").with_object("i")), "kuivaa");
    assert_eq!(synth(Intent::infinitive("ishi").with_object("i")), "kuiishi");
    assert_eq!(synth(Intent::infinitive("penda").with_object("ji")), "kujipenda");
    assert_eq!(
        synth(Intent::simple("3SG", "PAST", "uliza").with_object("m")),
        "alimuuliza"
    );
}

#[test]
fn short_stem_object_elision() {
    assert_eq!(synth(Intent::simple("3SG", "PAST", "pa").with_object("ni")), "alinipa");
    assert_eq!(synth(Intent::simple("3SG", "PRES", "pa").with_object("m")), "anampa");
    assert_eq!(synth(Intent::simple("1SG", "FUT", "pa").with_object("ku")), "nitakupa");
    assert_eq!(synth(Intent::simple("3PL", "PAST", "la").with_object("m")), "walimla");
    // Without object infix, intrusive -ku- remains.
    assert_eq!(synth(Intent::simple("3SG", "PRES", "la")), "anakula");
    assert_eq!(synth(Intent::negated("1PL", "FUT", "la")), "hatutakula");
    assert_eq!(
        synth(Intent::simple("1SG", "PAST", "la").with_derivations(["passive"])),
        "nilikuliwa"
    );
    assert_eq!(
        synth(Intent::simple("3SG", "PAST", "ja").with_derivations(["passive"])),
        "alikujiwa"
    );
}

#[test]
fn imperatives_singular_plural_and_object() {
    assert_eq!(synth(Intent::imperative("soma")), "soma");
    assert_eq!(synth(Intent::imperative("enda")), "nenda");
    assert_eq!(synth(Intent::imperative("ja")), "njoo");
    assert_eq!(synth(Intent::imperative("la")), "kula");
    assert_eq!(synth(Intent::imperative("nywa")), "kunywa");

    assert_eq!(synth(Intent::imperative_plural("soma")), "someni");
    assert_eq!(synth(Intent::imperative_plural("jibu")), "jibuni");
    assert_eq!(synth(Intent::imperative_plural("enda")), "nendeni");
    assert_eq!(synth(Intent::imperative_plural("ja")), "njooni");
    assert_eq!(synth(Intent::imperative_plural("la")), "kuleni");

    assert_eq!(synth(Intent::imperative("pa").with_object("m")), "mpe");
    assert_eq!(synth(Intent::imperative("leta").with_object("ki")), "kilete");
    assert_eq!(synth(Intent::imperative_plural("pa").with_object("m")), "mpeni");
}

#[test]
fn extended_derivations_epenthesis_borrowed_and_stacked() {
    // Stative harmony
    assert_eq!(
        synth(Intent::simple("1SG", "PRES", "soma").with_derivations(["stative"])),
        "ninasomeka"
    );
    assert_eq!(
        synth(Intent::simple("C9", "PRES", "elewa").with_derivations(["stative"])),
        "inaeleweka"
    );
    assert_eq!(
        synth(Intent::simple("3SG", "PAST", "samehe").with_derivations(["stative"])),
        "alisameheka"
    );

    // Epenthetic -l- on vowel clusters (-oa, -ea, -ua, -aa)
    assert_eq!(
        synth(Intent::simple("C9", "PAST", "fungua").with_derivations(["passive"])),
        "ilifunguliwa"
    );
    assert_eq!(
        synth(Intent::simple("C9", "PAST", "ondoa").with_derivations(["passive"])),
        "iliondolewa"
    );
    assert_eq!(
        synth(Intent::simple("3SG", "PAST", "toa").with_derivations(["applicative"])),
        "alitolea"
    );
    assert_eq!(
        synth(Intent::simple("3SG", "PAST", "fungua").with_derivations(["applicative"])),
        "alifungulia"
    );
    assert_eq!(
        synth(Intent::simple("3SG", "PAST", "kaa").with_derivations(["applicative"])),
        "alikalia"
    );
    assert_eq!(
        synth(Intent::simple("3SG", "PAST", "potea").with_derivations(["applicative"])),
        "alipotelea"
    );

    // Borrowed passives (-e, -i, -u)
    assert_eq!(
        synth(Intent::simple("C9", "PAST", "jibu").with_derivations(["passive"])),
        "ilijibiwa"
    );
    assert_eq!(
        synth(Intent::simple("C8", "PERF", "ripoti").with_derivations(["passive"])),
        "vimeripotiwa"
    );
    assert_eq!(
        synth(Intent::simple("3SG", "PAST", "samehe").with_derivations(["passive"])),
        "alisamehewa"
    );

    // Short irregular passives in INF and neg PAST
    assert_eq!(
        synth(Intent::infinitive("pa").with_derivations(["passive"])),
        "kupewa"
    );
    assert_eq!(
        synth(Intent::infinitive("nywa").with_derivations(["passive"])),
        "kunywewa"
    );
    assert_eq!(
        synth(Intent::negated("3SG", "PAST", "pa").with_derivations(["passive"])),
        "hakupewa"
    );

    // Stacked derivations
    assert_eq!(
        synth(Intent::simple("YA", "PAST", "fika").with_derivations(["applicative", "passive"])),
        "yalifikiwa"
    );
    assert_eq!(
        synth(
            Intent::simple("3SG", "PAST", "andika")
                .with_object("m")
                .with_derivations(["applicative"])
        ),
        "alimwandikia"
    );
}

#[test]
fn strict_intent_validation_catches_invalid_slots() {
    let e = Engine::swahili();
    assert!(e.synthesize_checked(&Intent::simple("1SG", "PRES", "soma")).is_ok());
    assert!(
        e.synthesize_checked(&Intent::simple("3PL", "FUT", "jibu").with_relative("C2"))
            .is_ok()
    );

    let bad_subj = e.validate(&Intent::simple("BAD_SUBJ", "PRES", "soma")).unwrap_err();
    assert_eq!(bad_subj[0].slot, "subject");

    let bad_tense = e.validate(&Intent::simple("1SG", "BAD_TENSE", "soma")).unwrap_err();
    assert_eq!(bad_tense[0].slot, "tense");

    let bad_rel = e
        .validate(&Intent::simple("1SG", "PRES", "soma").with_relative("ZZ"))
        .unwrap_err();
    assert_eq!(bad_rel[0].slot, "relative");

    let bad_deriv = e
        .validate(&Intent::simple("1SG", "PRES", "soma").with_derivations(["bogus"]))
        .unwrap_err();
    assert_eq!(bad_deriv[0].slot, "derivations");

    let bad_root = e.validate(&Intent::simple("1SG", "PRES", "two words")).unwrap_err();
    assert_eq!(bad_root[0].slot, "root");
}

#[test]
fn v25_features_are_pure_data_from_toml() {
    let custom_toml = r##"
[subjects]
1SG = "zo"
[tenses]
FUT = "da"
HAB = "xe"
[tense_behavior]
subjectless = ["HAB"]
[relatives]
1SG = "qu"
[relative_rules]
future_marker = "da"
future_rel_marker = "daka"
[imperatives]
plural_mood = "eni"
plural_suffix = "nu"
plural_vowel = "e"
[imperatives.irregular_sg]
go = "zek"
[defaults]
final_vowel = "a"
"##;
    let engine = Engine::from_morphology(Morphology::parse(custom_toml).expect("parse custom"));
    assert_eq!(
        engine
            .synthesize(&Intent::simple("1SG", "FUT", "tam").with_relative("1SG"))
            .surface,
        "zodakaqutama"
    );
    assert_eq!(
        engine.synthesize(&Intent::simple("1SG", "HAB", "tam")).surface,
        "xetama"
    );
    assert_eq!(engine.synthesize(&Intent::imperative("go")).surface, "zek");
    assert_eq!(
        engine.synthesize(&Intent::imperative_plural("tam")).surface,
        "tamenu"
    );
}
