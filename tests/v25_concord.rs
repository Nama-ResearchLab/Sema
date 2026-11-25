//! Sema v2.5 non-verbal concord, copula, locative, and clause composer tests.

use sema::{Composer, ConcordEngine};

#[test]
fn copulas_and_locative_copulas() {
    let c = ConcordEngine::swahili();
    assert_eq!(c.copula(false), "ni");
    assert_eq!(c.copula(true), "si");
    assert_eq!(c.emphatic_copula(false), "ndiyo");
    assert_eq!(c.emphatic_copula(true), "siyo");

    assert_eq!(c.locative_copula("C1", "ko", false).unwrap(), "yuko");
    assert_eq!(c.locative_copula("C1", "ko", true).unwrap(), "hayuko");
    assert_eq!(c.locative_copula("C2", "ko", false).unwrap(), "wako");
    assert_eq!(c.locative_copula("3PL", "ko", true).unwrap(), "hawako");
    assert_eq!(c.locative_copula("C7", "ko", false).unwrap(), "kiko");
    assert_eq!(c.locative_copula("C8", "ko", false).unwrap(), "viko");
    assert_eq!(c.locative_copula("C5", "po", false).unwrap(), "lipo");
    assert_eq!(c.locative_copula("1SG", "po", false).unwrap(), "nipo");
}

#[test]
fn possessive_existential_have_verbs() {
    let c = ConcordEngine::swahili();
    assert_eq!(c.have("1SG", false).unwrap(), "nina");
    assert_eq!(c.have("1SG", true).unwrap(), "sina");
    assert_eq!(c.have("3SG", true).unwrap(), "hana");
    assert_eq!(c.have("C17", false).unwrap(), "kuna");
    assert_eq!(c.have("C17", true).unwrap(), "hakuna");
    assert_eq!(c.have("C18", true).unwrap(), "hamna");
    assert_eq!(c.have("C16", true).unwrap(), "hapana");
}

#[test]
fn associatives_possessives_and_demonstratives() {
    let c = ConcordEngine::swahili();
    assert_eq!(c.associative("C1").unwrap(), "wa");
    assert_eq!(c.associative("C5").unwrap(), "la");
    assert_eq!(c.associative("C7").unwrap(), "cha");
    assert_eq!(c.associative("C8").unwrap(), "vya");
    assert_eq!(c.associative("C9").unwrap(), "ya");
    assert_eq!(c.associative("C10").unwrap(), "za");
    assert_eq!(c.associative("C17").unwrap(), "kwa");

    assert_eq!(c.possessive("C1", "1SG").unwrap(), "wangu");
    assert_eq!(c.possessive("C7", "1SG").unwrap(), "changu");
    assert_eq!(c.possessive("C8", "1PL").unwrap(), "vyetu");
    assert_eq!(c.possessive("C9", "3SG").unwrap(), "yake");
    assert_eq!(c.possessive("C10", "3PL").unwrap(), "zao");

    assert_eq!(c.demonstrative("C1", "prox").unwrap(), "huyu");
    assert_eq!(c.demonstrative("C1", "dist").unwrap(), "yule");
    assert_eq!(c.demonstrative("C1", "ref").unwrap(), "huyo");
    assert_eq!(c.demonstrative("C7", "prox").unwrap(), "hiki");
    assert_eq!(c.demonstrative("C7", "dist").unwrap(), "kile");
    assert_eq!(c.demonstrative("C7", "ref").unwrap(), "hicho");
    assert_eq!(c.demonstrative("C8", "prox").unwrap(), "hivi");
    assert_eq!(c.demonstrative("C8", "dist").unwrap(), "vile");
    assert_eq!(c.demonstrative("C8", "ref").unwrap(), "hivyo");
}

#[test]
fn adjectives_numerals_and_locative_nouns() {
    let c = ConcordEngine::swahili();
    assert_eq!(c.adjective("C1", "dogo").unwrap(), "mdogo");
    assert_eq!(c.adjective("C2", "dogo").unwrap(), "wadogo");
    assert_eq!(c.adjective("C7", "kubwa").unwrap(), "kikubwa");
    assert_eq!(c.adjective("C8", "zuri").unwrap(), "vizuri");
    assert_eq!(c.adjective("C1", "ema").unwrap(), "mwema");
    assert_eq!(c.adjective("C7", "ema").unwrap(), "chema");
    assert_eq!(c.adjective("C2", "ema").unwrap(), "wema");
    assert_eq!(c.adjective("C8", "ingi").unwrap(), "vingi");

    assert_eq!(c.numeral("C1", 1).unwrap(), "mmoja");
    assert_eq!(c.numeral("C2", 2).unwrap(), "wawili");
    assert_eq!(c.numeral("C7", 1).unwrap(), "kimoja");
    assert_eq!(c.numeral("C8", 2).unwrap(), "viwili");
    assert_eq!(c.numeral("C10", 2).unwrap(), "mbili");
    assert_eq!(c.numeral("C8", 6).unwrap(), "sita");

    assert_eq!(c.locative_noun("hospitali"), "hospitalini");
    assert_eq!(c.locative_noun("nyumba"), "nyumbani");
    assert_eq!(c.locative_noun("nyumbani"), "nyumbani");
}

#[test]
fn deterministic_clause_composer() {
    let composer = Composer::swahili();

    let c1 = composer
        .render("{t:child} {poss:@child|1SG} {loc_cop:@child|ko} {loc:@hospital}")
        .unwrap();
    assert_eq!(c1.surface, "mtoto wangu yuko hospitalini");

    let c2 = composer
        .render("Mpe {t:child} {t:tablet:pl} {num:@tablet:pl|2} {assoc:@tablet:pl} paracetamol 500 mg")
        .unwrap();
    assert_eq!(c2.surface, "Mpe mtoto vidonge viwili vya paracetamol 500 mg");

    let c3 = composer
        .render("{t:chair} {dem:@chair|prox} {v:@chair|PERF|-|-|vunja|stative}")
        .unwrap();
    assert_eq!(c3.surface, "kiti hiki kimevunjika");
    assert_eq!(c3.verbs_verified, 1);

    let c4 = composer
        .render("{t:child} {v:@child|PAST|neg|-|la}")
        .unwrap();
    assert_eq!(c4.surface, "mtoto hakula");
    assert_eq!(c4.verbs_verified, 1);
}
