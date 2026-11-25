# Lesson 6: Relatives, Conditionals, Concords & Clauses (`v0.2.5`)

Goal: move beyond isolated finite verbs into full Swahili noun-class agreement, relative infixes, conditionals, deterministic clause composition, and zero-leak fact protection.

## 1. Relative Clauses Inside the Verb (`Slot 2b`)

In Swahili, relative markers attach inside the verb directly after the tense marker. Future `-ta-` automatically shifts to `-taka-` when followed by a relative infix:

```text
[SUBJ] + [TENSE (-ta- -> -taka-)] + [REL (-ye-, -o-, -cho-, -po-)] + [OBJ] + [ROOT] + [MOOD]
  wa   +         li               +          o                      +  wa   +   ua   +   a   -> waliowaua
  wa   +        taka              +          o                      +       +  jibu  +   a   -> watakaojibu
  ni   +        taka              +         po                      +       +  fika  +   a   -> nitakapofika
```

Negative conditionals replace `-ki-`/`-nge-`/`-ngali-` with `-sipo-`/`-singe-`/`-singali-` while keeping **positive** subject prefixes (`ni-sipo-soma`, `ni-singe-soma`, not `*si-sipo-soma`).

## 2. Non-Verbal Concords (`ConcordEngine`)

Swahili sentences rely on class agreement (`ngeli` C1-C18) across adjectives, numerals, demonstratives, possessives, associatives, and locative copulas:

- Copula / emphatic: `ni` / `si`, `ndiyo` / `siyo`
- Locative copula: `C1 + ko -> yuko` (`hayuko`), `C7 + ko -> kiko` (`hakiko`)
- Existential / possessive `-na`: `C16 (neg) -> hapana`, `C17 (neg) -> hakuna`
- Noun-class adjectives & numerals: `C7 + dogo -> kidogo`, `C8 + dogo -> vidogo`, `C8 + 2 -> viwili`
- Locative nouns: `hospitali -> hospitalini`, `nyumba -> nyumbani`

## 3. Deterministic Clause Composition (`Composer`)

`Composer` binds terms (`{t:child}`), nouns' tracked classes (`@child`), concords, locatives, and verified verbs into complete grammatical clauses:

```text
Template: "{t:child} {poss:@child|1SG} {loc_cop:@child|ko} {loc:@hospital}"
Output:   "mtoto wangu yuko hospitalini"
```

## 4. Zero-Leak Fact Protection (`FactGuardian`)

`FactGuardian` tokenizes numbers, decimals (`2,5`, `2.5`), ranges (`500 to 1000`), fractions (`nusu kidonge`), Swahili number words (`mia tano`, `elfu moja`), clinical brands (`Panadol`, `Septra`), and bounded Damerau-Levenshtein typos (`paracetemol` -> `paracetamol`), cutting unshielded boundary leaks from 54.2% to **0.0% (0/24)**.

Back to the [course index](README.md) or try the live widgets in the [interactive course hub](index.html) and [verb explorer](verbs.html).
