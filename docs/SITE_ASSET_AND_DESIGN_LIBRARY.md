# Sema Public Site, Asset & Curated Component Library

Design and component catalog for the public **Sema (`v0.1.0` -> `v0.2.5`)** web portal (`index.html`, `docs/index.html`, and offline single-file `downloads/sema-studio.html`). Modeled after the discipline of sister system docs in **Cervos** and **KAMBi**: one warm palette system, verified real visual assets, zero dark developer clutter inside reader/studio flows, and honest instrumentation.

---

## 1. Visual Asset Inventory (`assets/` & `docs/assets/`)

| Asset Path | Role on Site | Usage Rules |
|---|---|---|
| `assets/nama-logo.png` | **Nama Research Lab** primary mark | Placed in flush editorial header beside the `Sema` display wordmark; never stretched or recolored. |
| `assets/sema-studio-desk.jpg` | Editorial Studio Photograph | Framed cleanly at `4:3` in crisp editorial borders; zero text pills, badges, or captions overlaid on top of or under the image. |

---

## 2. Version-Coded Palette System (`[data-version-theme]`)

Every public release of Sema wears its own signature palette so visitors can preview releases visually while exploring that version's pack:

| Release Tag | Theme Name | Ground (`--bg`) | Secondary (`--bg-alt`) | Primary Accent (`--accent`) | Dark Ink (`--accent-dark`) | Soft Fill (`--accent-soft`) |
|---|---|---|---|---|---|---|
| `v0.2.5` *(default)* | Warm Terracotta | `#FAF7F2` | `#F2ECE3` | `#C2410C` | `#9A3412` | `#FFF4EC` |
| `v0.2.0` | Lagoon Teal | `#F3FAF9` | `#E7F3F1` | `#0D9488` | `#0F766E` | `#F0FDFA` |
| `v0.1.0` | Archival Indigo | `#F5F6FC` | `#EAEDF8` | `#4338CA` | `#3730A3` | `#EEF2FF` |

All Studio panels, subtabs, waveform canvases, Academy modules, and interactive SVG flowpipes bind dynamically to these CSS variables rather than hardcoded slate/navy colors.

---

## 3. Adaptive Depth & Persona Modes (`[data-depth]`, `[data-role]`)

Guided by a 3-question entrance onboarder (re-openable anytime from the header **Customize View** control or quick mode pill):

1. **`simple` (Everyday):** Plain copy with zero linguistic jargon, focused 3-tab Studio (`Phrase Builder`, `Dosage Shield`, `19k Dictionary`), high-level flowpipe explanations.
2. **`balanced` (Builder):** Healthcare safety & clause integrity framing, interactive clinical directive & clause composer views.
3. **`expert` (Full Spec):** Unlocks all 8 Studio tools (`Verb Decompiler`, `18 Noun Classes`, `50-Intent Batch Bench`, `Morpheme Dependency DAG`, 5-language `Code Exporter`), granular slot specs, and Rust timing instrumentation.

---

## 4. Curated Interactive Components

1. **Conductive Morph-Flowpipe (`#live-flowpipe-svg` & `#lesson-flowpipe-box`):**
   - Curved Bezier vector conduits with live signal bead animations (`stroke-dashoffset` flow + pulse bursts when inputs or scenarios change).
   - Paired directly on Overview with a **3-Second Before/After Clinical Proof** comparator (`Unguarded Model Risk` vs `Protected by Sema`).
   - Automatically injected above every **Academy** module (`Basic 1-5` & `Advanced 1-6`) so learners trace how prefixes, tenses, roots, and dosage shields conduct signals step by step.
2. **Sema Field Dispatches (`#dispatches-board`):**
   - Short bilingual/plain-English field notes covering why numbers drift offline, how 18 Swahili noun classes stay aligned, and why explicit rules complement small edge models.
3. **Interactive Presence Bar (`#live-presence-bar`):**
   - Honest multi-tab session presence indicator (`BroadcastChannel('sema_presence_v1')` + active tab role counters) grouped into `Learners`, `Clinic Builders`, and `Ryoiki Experts`, updating live when the visitor selects or switches their role.
4. **Release Evolution Board (`#versions-showcase`):**
   - `v0.1.0` (`76037c2`), `v0.2.0` (`032d3fe` + `eee506a`), and `v0.2.5` release cards pairing plain-English release notes with direct download packs (`.zip`, `sema-linux-aarch64.tar.gz`, `install.sh`, standalone `sema-studio.html`, `.jsonl`/`.tsv` lexicons).
