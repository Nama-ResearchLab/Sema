# Lesson 1: first principles

Goal: understand the problem, the idea, and every word the rest of
the course uses. No code in this lesson.

## 1. The problem, plainly

Swahili is spoken by around 100 million people. Almost all modern
language AI is trained overwhelmingly in English. That creates three
practical gaps:

- Small models barely speak Swahili. Ask one to conjugate a verb and
  it guesses frequent words instead.
- Training a model properly costs money and data that low-resource
  languages do not have.
- Translation websites need an internet connection. A phone with no
  signal gets nothing.

So the target is: useful Swahili AI that runs fully offline, on an
ordinary CPU, with no model training required.

## 2. Words you need (glossary for the whole course)

- Morphology: how words are built from smaller meaningful parts.
- Morpheme: one such part. In "sikufanya", the parts are si (not, I),
  ku (past, under negation), fanya (do), a (ending vowel).
- Root (or stem): the core of the verb that carries its meaning.
  "fanya" means do. Everything else around it is grammar.
- Lemma: the dictionary headword of a word. The lemma of "umefikia"
  is "fikia".
- Affix: a part glued to the front (prefix) or back (suffix) of a root.
- Concord: a little agreement marker. Swahili verbs start with one
  that says who or what is acting: ni (I), u (you), a (he/she),
  tu (we), m (you all), wa (they).
- Agglutination: building words by gluing many parts in a fixed order.
  Swahili verbs are strongly agglutinative: ha-wa-ta-ku-jibu stacks
  not, they, will, you, answer in one word.
- TAM: tense, aspect, mood. The slot that says when and how: li (past),
  na (present), me (perfect, done), ta (future).
- Polarity: positive or negative. Negation in Swahili fuses with the
  subject concord (ni plus not becomes si, not "ha-ni").
- Noun class: Swahili nouns belong to classes (like genders but more
  of them), and verbs agree with the class: ki-tabu ki-dogo (small
  book, class 7), li- for class 5, ya- for class 6, and so on.
- Lemma, gloss: a gloss is the English meaning note attached to a
  dictionary entry, for example fikia glossed "to arrive at".
- Skeleton: our name for a resolved word: surface form plus lemma,
  part of speech, gloss, and root.
- Intent: our name for a structured request to build a verb: subject,
  tense, polarity, object, root, extras, mood.
- Surface (form): the final spelled word, for example "sikufanya".
- Offline: runs on your machine with no network calls. Everything in
  this course is offline after a one-time data download.
- Deterministic: same input always gives same output. No sampling,
  no temperature, no surprises.

## 3. Why small models fail at Swahili verbs

A small model reads text in chunks called tokens. It learned those
chunks mostly from English. A Swahili verb like "hawatakujibu" gets
chopped into pieces that mean nothing, so the model treats the whole
word as noise. Measured directly: two small offline models scored 0
out of 15 exact when asked to translate verb phrases (see Lesson 4).
They stutter, guess common words like "habari", or flip positive and
negative.

## 4. The idea that fixes it

Give each side the language it handles best:

1. Anchor (deterministic, offline): resolve each Swahili word to an
   English skeleton. "umefikia" becomes fikia, verb, "to arrive at".
2. Reason (small model, in English): the model thinks over clean
   English meaning instead of noisy Swahili strings.
3. Render (deterministic, offline): compile the model's structured
   answer back into a grammatical Swahili verb. "I did not do" plus
   first person plus past plus "do" becomes "sikufanya", exactly,
   every time.

The model never back-translates prose. Facts and numbers travel
inside fields and templates, so translation cannot corrupt them.

## 5. A full pass, slowly

Input: "umefikia wapi?" (Where have you arrived?).

Anchor step. "umefikia" strips to u (you) + me (perfect) + fikia
(arrive). "wapi" is an adverb meaning where. Skeleton:

    umefikia[fikia:verb 'to arrive at'] + wapi[wapi:adv 'where']

Reason step. The model reads the skeleton and decides the answer in
English, returning structured fields rather than Swahili prose.

Render step. Fields such as subject 1SG, past tense, negated, root
"fanya" assemble slot by slot into "sikufanya".

You now know the whole system at concept level. The next lessons open
each box: what data it uses, what rules it applies, and how to run it.

Next: [Lesson 2: forward anchoring](02-forward-anchor.md).
