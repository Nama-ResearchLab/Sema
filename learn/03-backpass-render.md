# Lesson 3: back-pass rendering (meaning to Swahili)

Goal: understand how structured fields become an exact Swahili verb.
This is the longest lesson because verbs carry the most grammar. Take
it slot by slot.

## 1. The template

Every verb is assembled left to right through eight slots:

    [NEG][SUBJ][TENSE][OBJ][ROOT][DERIV][MOOD]

NEG is the standalone negation marker (often empty because negation
fuses into SUBJ). SUBJ is the subject concord. TENSE is the TAM
marker. OBJ is the object infix (empty when there is no object). ROOT
is the verb core. DERIV holds valence-changing suffixes (empty for
plain verbs). MOOD is the final vowel.

The assembler is generic: it reads each slot's contents from data
tables in one TOML file. No Swahili rule lives in code.

## 2. Subjects, including the negative fusions

Positive concords: ni (I), u (you), a (he/she), tu (we), m (you all),
wa (they), plus noun-class concords li, ya, pa, ki, ji, zi, i.

Negation does not prepend "not". It fuses: ni becomes si, u becomes
hu, a becomes ha, tu becomes hatu, m becomes ham, wa becomes hawa.
So "I did not" starts with "si", and "they will not" starts with
"hawa". Beginners should memorize these six fusions first; they
explain half of all negative verbs.

## 3. Tenses, including the negation overrides

Basic markers: li (past), na (present), me (perfect, finished), ta
(future). Under negation, past li becomes ku and perfect me becomes
ja. That is why "sikufanya" contains "ku" and "sijaona" contains "ja".

## 4. Objects, vowels, and short stems

Object infixes sit between tense and root: ni (me), ku (you), m
(him/her), ki (it, class 7), wa (them). A vowel-initial root after
"m" glides: m + ona becomes "mwona" (sijamwona, "I have not seen
him").

Monosyllabic roots (la eat, ja come, pa give) take an intrusive "ku"
after the tense marker: a + na + la becomes "anakula", never "anala".

Borrowed roots keep their final vowel always: "jibu" never shifts,
so the negative present is "hajibu", not "hajibi".

## 5. Moods and the final vowel

Default final vowel is "a". Subjunctive mood uses "e" (tusisome, "let
us not read" patterns), hortative "o". Present negative on regular
-a stems shifts to "i": "sisomi" (I do not read).

## 6. Derivations (growing new verbs from old ones)

Five suffixes sit between root and final vowel, changing who does
what to whom:

- Passive -w-: piga (hit) to pigwa (be hit): "nilipigwa".
- Applicative -i-: andika (write) to andikia (write to): "nitaandikia".
- Causative -ish-/-esh-: fanya to fanyisha (make do); soma to
  somesha (feed). Rule of thumb: mid stem vowels e/o take -esh-,
  others take -ish-. The conditioning vowel is the stem vowel (the
  root minus its final -a), not the final letter.
- Reciprocal -an-: ona to onana (see each other): "nilionana".
- Stative -ik-: vunja to vunjika (be broken): "kilivunjika".

Short stems take passive -iwa instead of -w-: la to liwa
("nilikuliwa", and note the single final -a, never doubled).

## 7. Noun classes, concretely

Swahili nouns fall into classes and the verb agrees. The engine
reports the class with every synthesis: KI subjects are class 7
(kilivunja, "it broke"), LI class 5, YA class 6, PA class 16
(locative), ZI class 10, I class 9. Person concords (I, you, they)
report no class because they agree with people, not classes.

## 8. Three full builds, slowly

sikufanya: negated 1SG (si) + past-under-negation (ku) + fanya + a.
anakula: 3SG (a) + present (na) + intrusive ku (short stem la) + a.
hawatakujibu: negated 3PL (hawa) + future (ta) + object ku + jibu + a.

Open the [verb explorer](verbs.html) and rebuild each one by picking
slots. Every form shown was produced by the real engine.

Next: [Lesson 4: proof, not promises](04-proof.md).
