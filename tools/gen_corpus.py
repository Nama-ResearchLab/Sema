# Script: generate Sema_Benchmark_1000.csv corpus.
# Produces exactly 1,000 structured benchmark rows:
#   500 SW -> EN (forward pass linearizer)
#   500 EN -> SW (back pass rolodex)
#
# Usage:
#   python tools/gen_corpus.py [output.csv]
#
# Output columns:
#   id, direction, source_text, target_gold, morph_breakdown,
#   expected_telegraphic, category, complexity_tier, actual_output,
#   racer_winner, latency_ms, notes_comments
import csv
import json
import sys
import os

# Categories and Morphological Pattern Generators for Sema Benchmark
sw_templates = [
    ("Hawajazipata", "They have not yet received them", "ha-(NEG)+wa-(3PL)+ja-(PERF_NEG)+zi-(OBJ:C10)+pata", "[they] [not yet] [them] [get]", "pro_drop", 1),
    ("Nilipofika hospitalini", "When I arrived at the hospital", "ni-li-po-(REL_TIME)+fika+hospitali-ni", "[when I] [arrive] [hospital in]", "relative_clause", 2),
    ("Watoto wote wazuri walikula", "All good children ate", "wa-toto+wa-ote+wa-zuri+wa-li-ku-la", "children all good [they] [did] [eat]", "noun_class_concord", 2),
    ("Hamkunijibu", "You (pl) did not answer me", "ha-(NEG)+m-(2PL)+ku-(PAST_NEG)+ni-(OBJ:1SG)+jibu", "[you all] [did not] [me] [answer]", "negation_infix", 1),
    ("Inakuwaje leo?", "How are things today?", "i-na-kuwa-je+leo", "[how is it] [today]", "idiom_slang", 1),
    ("Hatutawafundisha", "We will not teach them", "ha-(NEG)+tu-(1PL)+ta-(FUT)+wa-(OBJ:3PL)+fundisha", "[we] [will not] [them] [teach]", "future_negation", 1),
    ("Waliondoka bila kusema", "They left without speaking", "wa-li-ondoka+bila+ku-sema", "[they] [did] [leave] without to speak", "infinitive_clause", 2),
    ("Vitu vyote vimeharibika", "All things are broken", "vi-tu+vi-ote+vi-me-haribika", "things all [they] [have] [spoil]", "noun_class_concord", 2),
    ("Sijui kama watakuja", "I don't know if they will come", "si-(1SG_NEG)+jui+kama+wa-ta-ku-ja", "[I] [not know] if [they] [will] [come]", "complex_clause", 2),
    ("Anakwenda sokoni sasa", "He/She is going to the market now", "a-na-kw-enda+soko-ni+sasa", "[he/she] [is] [go] [market in] now", "monosyllabic_verb", 1),
]

# Each EN->SW template carries: (source_en, target_gold, morph_breakdown,
# expected_telegraphic, category, tier, intent_dict, expected_verb).
# `intent_dict` is the rolodex Intent payload for the BACKPASS and
# `expected_verb` is the gold finite verb the rolodex should synthesize.
en_templates = [
    ("No, I did not.", "Hapana, sikufanya.", "hapana+si-(1SG_NEG)+ku-(PAST_NEG)+fanya", "No [I] [did not] [do]", "rolodex_synthesis", 1,
     {"subject": "1SG", "tense": "PAST", "negation": True, "root": "fanya", "mood": "a"}, "sikufanya"),
    ("They will not answer you.", "Hawatakujibu.", "ha-(NEG)+wa-(3PL)+ta-(FUT)+ku-(OBJ:2SG)+jibu", "[they] [will not] [you] [answer]", "rolodex_synthesis", 1,
     {"subject": "3PL", "tense": "FUT", "negation": True, "object": "2SG", "root": "jibu", "mood": "a"}, "hawatakujibu"),
    ("Have you eaten?", "Umekula?", "u-(2SG)+me-(PERF)+ku-(INF)+la", "[you] [have] [eat]?", "monosyllabic_verb", 1,
     {"subject": "2SG", "tense": "PERF", "negation": False, "root": "la", "mood": "a"}, "umekula"),
    ("The patient left early.", "Mgonjwa alitoka mapema.", "m-gonjwa+a-li-toka+mapema", "patient [he/she] [did] [leave] early", "subject_agreement", 2,
     {"subject": "3SG", "tense": "PAST", "negation": False, "root": "toka", "mood": "a"}, "alitoka"),
    ("I am coming back tomorrow.", "Nitarejea kesho.", "ni-ta-rejea+kesho", "[I] [will] [return] tomorrow", "future_tense", 1,
     {"subject": "1SG", "tense": "FUT", "negation": False, "root": "rejea", "mood": "a"}, "nitarejea"),
    ("We cannot do this.", "Hatuwezi kufanya hivi.", "ha-(NEG)+tu-(1PL)+wezi+ku-fanya+hi-vi", "[we] [cannot] to do this", "modal_negation", 2,
     {"subject": "1PL", "tense": "PRES", "negation": True, "root": "weza", "mood": "a"}, "hatuwezi"),
    ("Did they find the keys?", "Walipata funguo?", "wa-li-pata+funguo", "[they] [did] [get] keys", "interrogative_past", 1,
     {"subject": "3PL", "tense": "PAST", "negation": False, "root": "pata", "mood": "a"}, "walipata"),
    ("I haven't seen him yet.", "Sijamwona bado.", "si-(1SG_NEG)+ja-(PERF_NEG)+m-(OBJ:C1)+ona+bado", "[I] [not yet] [him] [see] yet", "aspect_negation", 2,
     {"subject": "1SG", "tense": "PERF", "negation": True, "object": "m", "root": "ona", "mood": "a"}, "sijamwona"),
    ("Where did you put it?", "Ulikieweka wapi?", "u-li-ki-(OBJ:C7)+eweka+wapi", "[you] [did] [it] [put] where", "object_infix_placement", 2,
     {"subject": "2SG", "tense": "PAST", "negation": False, "object": "3SG_KI", "root": "weka", "mood": "a"}, "ulikiweka"),
    ("Don't worry, brother.", "Usijali, ndugu.", "u-si-(SUBJ_NEG)+jali+ndugu", "[you] [do not] [worry] brother", "imperative_negation", 1,
     {"subject": "2SG", "tense": "SUBJ", "negation": True, "root": "jali", "mood": "e"}, "usijali"),
]

headers = [
    "id", "direction", "source_text", "target_gold", "morph_breakdown",
    "expected_telegraphic", "category", "complexity_tier", "actual_output",
    "racer_winner", "latency_ms", "notes_comments",
    "intent_json", "expected_verb"
]

DEFAULT_OUT = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "bench", "Sema_Benchmark_1000.csv")


def main():
    filename = sys.argv[1] if len(sys.argv) > 1 else DEFAULT_OUT
    os.makedirs(os.path.dirname(filename) or ".", exist_ok=True)

    with open(filename, mode='w', newline='', encoding='utf-8') as f:
        writer = csv.writer(f)
        writer.writerow(headers)

        # 500 SW -> EN rows (Forward Pass). Each of the 10 templates appears
        # 50 times with clean Swahili source_text (no "(var N)" pollution) so
        # the linearizer/racer sees the same well-formed surface repeatedly.
        for i in range(1, 501):
            t = sw_templates[(i - 1) % len(sw_templates)]
            writer.writerow([
                f"SW-ENG-{i:03d}", "SW_TO_EN", t[0], t[1],
                t[2], t[3], t[4], t[5], "", "", "", "Forward pass linearizer benchmark",
                "", ""
            ])

        # 500 EN -> SW rows (Back Pass Rolodex).
        for i in range(1, 501):
            t = en_templates[(i - 1) % len(en_templates)]
            intent = t[6]
            intent_json = json.dumps(intent, separators=(",", ":"))
            writer.writerow([
                f"ENG-SW-{i:03d}", "EN_TO_SW", t[0], t[1],
                t[2], t[3], t[4], t[5], "", "", "", "Back pass rolodex benchmark",
                intent_json, t[7]
            ])

    print(f"Done! {filename} generated with exactly 1,000 structured benchmark rows.")


if __name__ == "__main__":
    main()
