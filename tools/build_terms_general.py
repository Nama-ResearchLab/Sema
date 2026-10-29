#!/usr/bin/env python3
"""Build data/terms.general.tsv (English -> Swahili nouns) from the distilled
Wiktionary lexicon. Precision over recall:
  * nouns only, first two glosses, short clean glosses (no parens/colons/lists)
  * drop an English key if it is ambiguous among Swahili nouns
  * drop an English key that is ALSO a gloss/base word of any verb, adj, adv,
    num, pron, prep, conj entry (homograph guard: saw, light, back, ...)
Output columns: english, swahili, plural(empty), class(empty).
Data licence: CC BY-SA 4.0 (Wiktionary via kaikki.org), see DATA-LICENSE."""
import json, re, sys, collections
src = sys.argv[1] if len(sys.argv) > 1 else "data/swahili.distilled.jsonl"
dst = sys.argv[2] if len(sys.argv) > 2 else "data/terms.general.tsv"
other = set()
nouns = []
for l in open(src, encoding="utf-8"):
    d = json.loads(l)
    p, w, gs = d["p"], d["w"], d.get("g", [])
    if p == "noun":
        nouns.append((w, gs))
        continue
    for g in gs:
        g = re.sub(r"\(.*?\)", "", g).strip().lower()
        g = re.sub(r"^(to|a|an|the)\s+", "", g)
        if g:
            other.add(g.split(",")[0].strip())
            other.add(g.split()[0])
BAD = re.compile(r"[():;,/\[\]0-9]|\bof\b|\bform\b|class|plural|diminutive|augmentative|inflect|alternative|abbreviation|^(a|an|the) *$|initialism|misspelling|obsolete|nonstandard|dialect")
cand = collections.defaultdict(set)
for w, gs in nouns:
    if " " in w or len(w) < 3 or not w.isalpha() or w[0].isupper():
        continue
    for g in gs[:2]:
        g0 = g.strip()
        if BAD.search(g0.lower()):
            continue
        g0 = re.sub(r"^(a|an|the)\s+", "", g0.lower())
        n = len(g0.split())
        if n == 0 or n > 3 or len(g0) > 28 or not re.fullmatch(r"[a-z' -]+", g0):
            continue
        cand[g0].add(w)
HOMO = set('saw left found lay rose rode wound bore felt fell fed led lit met read sat sent set shot slept spent stood stuck sung swore taught threw told took wore won wrote bit blew broke brought built bought caught chose came cost cut dealt did drew drank drove ate flew forgot gave went grew had heard hid held hurt kept knew laid let lost made meant paid put ran said saw sold shut sang sank spoke swam tore woke can may will shall must might could would should do does done be been being am is are was were'.split())
KNOWN_BAD = {'difficulty', 'omen', 'liquor', 'bump', 'wilderness', 'tallow'}  # reviewed by hand: wrong sense in source
rows = []
for en, sws in sorted(cand.items()):
    if len(sws) != 1:
        continue
    if en in KNOWN_BAD:
        continue
    if en in HOMO or en.split()[0] in HOMO and len(en.split()) == 1:
        continue
    if en in other or en.split()[0] in other and len(en.split()) == 1:
        continue
    rows.append((en, next(iter(sws))))
with open(dst, "w", encoding="utf-8", newline="\n") as f:
    f.write("# General English->Swahili nouns auto-built from Wiktionary (CC BY-SA 4.0) by tools/build_terms_general.py\n")
    f.write("# Precision-first, UNREVIEWED. Curated files override these. english<TAB>swahili<TAB>plural<TAB>class\n")
    for en, sw in rows:
        f.write(f"{en}\t{sw}\t\t\n")
print(len(rows), "entries ->", dst)
