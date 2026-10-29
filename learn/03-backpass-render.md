# Lesson 3: Deterministic Verb Realization

Goal: understand how structured intent slots become an exact Swahili verb. Verbs carry the most grammar in Bantu languages, so we explore the process slot by slot.

## 1. The Slot Architecture

Every verb form is assembled through structured grammatical positions:

    [NEGATION] [SUBJECT] [TENSE] [OBJECT] [ROOT] [DERIVATION] [MOOD]

Negation marks polarity, often fusing smoothly with the subject concord. Subject indicates who is acting. Tense indicates the time and aspect. Object marks the recipient of the action. Root represents the semantic core. Derivation alters verb valence (causative, applicative, passive). Mood determines the final vowel.

Rules live in clean data tables, allowing the synthesis logic to remain lightweight and extensible.

## 2. Subject Concords and Polarity Fusion

Positive concords: ni (I), u (you), a (he/she), tu (we), m (you all), wa (they), alongside noun class concords like li, ya, ki, vi, and zi.

In negative polarity, Swahili gracefully fuses the negation marker with the subject pronoun:
- ni becomes si (I do not)
- u becomes hu (you do not)
- a becomes ha (he or she does not)
- tu becomes hatu (we do not)
- m becomes ham (you all do not)
- wa becomes hawa (they do not)

Understanding these six fusions provides immediate clarity across everyday conversations.

## 3. Tense Markers and Polarity Harmony

Core markers:
- li (past)
- na (present)
- me (perfective, completed action)
- ta (future)

Under negative polarity, past li changes to ku (for example, sikufanya), and perfective me shifts to ja (for example, sijaona).

## 4. Object Infixes, Vowel Harmony, and Short Stems

Object infixes rest between the tense marker and the verb root:
- ni (me), ku (you), m (him/her), ki (it), wa (them).

When an object infix like m meets a vowel-initial root like ona, it smoothly glides: m + ona becomes mwona (sijamwona: I have not seen him).

Monosyllabic roots such as la (eat), ja (come), and pa (give) naturally receive an epenthetic ku after positive tense markers: a + na + la becomes anakula.

Borrowed verbs of Arabic or foreign origin (such as jibu or fariji) retain their natural ending vowels across all moods.

## 5. Moods and Verbal Extensions

The default final vowel for indigenous Bantu verbs is a. The subjunctive mood adopts e (for example, tusome: let us read).

Verbal extensions expand the core root:
- Applicative (benefactive): fanyia (do for)
- Causative: fanyisha (cause to do)
- Passive: fanywa (be done)
- Reciprocal: fanyana (do with one another)
- Stative: fanyika (be feasible or done)

Back to the [course index](README.md). Next step: [Lesson 4 (Measured Proof)](04-proof.md).
