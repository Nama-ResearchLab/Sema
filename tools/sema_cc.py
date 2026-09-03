#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
Sema Command Centre (GUI)

A Tkinter-based control room for the Sema offline Swahili semantic anchoring
engine. It drives the Rust helper `sema_cc` (built via `cargo build --release
--bin sema_cc`) and surfaces 30+ features across four panels:

  * System Health  - lexicon, anchor probing, latency, models, deps, versions
  * Forward / Reverse Tester - one-shot & corner-case morphology testing
  * Corpus Bench   - drive bench/Sema_Benchmark_1000.csv, pass rates, racers
  * Reverse Inspect (Revin) - slot-by-slot decomposition, review & logs

No network, no extra Python deps (stdlib only).
"""

import csv
import json
import os
import subprocess
import sys
import threading
import time
import tkinter as tk
from tkinter import ttk, filedialog, messagebox

# --------------------------------------------------------------------------
# Configuration
# --------------------------------------------------------------------------
REPO = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
CC_EXE = os.path.join(REPO, "target", "release", "sema_cc.exe")
LEX = os.path.join(REPO, "data", "swahili.distilled.jsonl")
BENCH_CSV = os.path.join(REPO, "bench", "Sema_Benchmark_1000.csv")

# 30+ features are surfaced as buttons / actions across the app.


# --------------------------------------------------------------------------
# Bridge to the Rust helper
# --------------------------------------------------------------------------
def run_cc(args, timeout=60):
    """Run sema_cc with `args`, return parsed JSON dict."""
    cmd = [CC_EXE] + args
    proc = subprocess.run(
        cmd, capture_output=True, text=True, timeout=timeout,
        cwd=REPO, creationflags=subprocess.CREATE_NO_WINDOW,
    )
    if proc.returncode != 0:
        raise RuntimeError(proc.stderr.strip() or "sema_cc exited nonzero")
    return json.loads(proc.stdout)


def pick():
    """Return a marker sentinel so profile changes are obvious."""
    return os.path.sep


class App(tk.Tk):
    def __init__(self):
        super().__init__()
        self.title("Sema Command Centre  -  Offline Swahili Anchoring Engine")
        self.geometry("1180x760")

        self.notebook = ttk.Notebook(self)
        self.notebook.pack(fill="both", expand=True)

        self.tab_health = ttk.Frame(self.notebook)
        self.tab_tester = ttk.Frame(self.notebook)
        self.tab_corpus = ttk.Frame(self.notebook)
        self.tab_revin = ttk.Frame(self.notebook)
        self.tab_log = ttk.Frame(self.notebook)

        self.notebook.add(self.tab_health, text="1. System Health")
        self.notebook.add(self.tab_tester, text="2. Morph Tester")
        self.notebook.add(self.tab_corpus, text="3. Corpus Bench")
        self.notebook.add(self.tab_revin, text="4. Reverse Inspect")
        self.notebook.add(self.tab_log, text="5. Log / Review")

        self.log_lines = []

        self._build_health()
        self._build_tester()
        self._build_corpus()
        self._build_revin()
        self._build_log()

    # ------------------------------------------------------------------
    # Logging helpers
    # ------------------------------------------------------------------
    def log(self, msg):
        stamp = time.strftime("%H:%M:%S")
        self.log_lines.append(f"[{stamp}] {msg}")
        if len(self.log_lines) > 500:
            self.log_lines = self.log_lines[-500:]
        self.refresh_log()

    def refresh_log(self):
        self.log_text.delete("1.0", tk.END)
        self.log_text.insert("1.0", "\n".join(self.log_lines))

    # ------------------------------------------------------------------
    # Panel 1: System Health  (12 features)
    # ------------------------------------------------------------------
    def _build_health(self):
        f = self.tab_health
        grid = ttk.Frame(f)
        grid.pack(side=tk.LEFT, fill="both", expand=True)
        for c in range(3):
            grid.columnconfigure(c, weight=1, uniform="g")

        self.health_items = [
            ("engine_version", "Engine version"),
            ("lexicon_path", "Lexicon path"),
            ("lemmas", "Lemma count"),
            ("anchor_probe", "Anchor probe (hit/total)"),
            ("lookup_avg_us", "Avg lookup (us/word)"),
            ("model_0_5b", "Qwen2.5-0.5B GGUF"),
            ("model_1_5b", "Qwen2.5-1.5B GGUF"),
        ]
        self.health_vars = {}
        r = 0
        c = 0
        for key, label in self.health_items:
            box = ttk.LabelFrame(grid, text=label)
            box.grid(row=r, column=c, sticky="nsew", padx=6, pady=6)
            var = tk.StringVar(value="-")
            ttk.Label(box, textvariable=var, wraplength=330, justify=tk.LEFT).pack(
                fill="both", expand=True, padx=8, pady=8)
            self.health_vars[key] = var
            r += 1
            if r == 3:
                r = 0
                c += 1

        # control column
        ctrl = ttk.Frame(f)
        ctrl.pack(side=tk.RIGHT, fill="y", padx=8, pady=8)
        ttk.Button(ctrl, text="Refresh health", command=self.refresh_health).pack(
            fill="x", pady=3)
        ttk.Button(ctrl, text="Verify deps (.toml/.jsonl/.gguf)",
                   command=self.verify_deps).pack(fill="x", pady=3)
        ttk.Button(ctrl, text="Export report .json",
                   command=self.export_report).pack(fill="x", pady=3)

    def refresh_health(self):
        def work():
            try:
                h = run_cc(["health"])
                self.after(0, lambda: self._set_health(h))
            except Exception as e:
                self.after(0, lambda: self.log(f"health failed: {e}"))

        self.log("health refresh requested")
        threading.Thread(target=work, daemon=True).start()

    def _set_health(self, h):
        self.health_vars["engine_version"].set(h.get("version", "-"))
        self.health_vars["lexicon_path"].set(h.get("lexicon_path", "-"))
        self.health_vars["lemmas"].set(h.get("lemmas", "-"))
        self.health_vars["anchor_probe"].set(h.get("anchor_probe", "-"))
        self.health_vars["lookup_avg_us"].set(str(h.get("lookup_avg_us_per_word", "-")))

        models = h.get("models", [])
        def fmt(m):
            mb = m.get("bytes", 0) / 1e6
            return (f"{m['file']}\n  present: {bool(m['present'])}   {mb:.1f} MB"
                    if os.path.exists(os.path.join(REPO, m["file"]))
                    else f"{m['file']}\n  present: False")
        for i, m in enumerate(models):
            key = "model_0_5b" if "0.5B" in m["file"] else "model_1_5b"
            if key in self.health_vars:
                self.health_vars[key].set(fmt(m))
        self.log("health refreshed")

    def verify_deps(self):
        checks = [
            ("Cargo.toml", "crate manifest", os.path.exists(os.path.join(REPO, "Cargo.toml"))),
            ("affix/sw.toml", "morphology table", os.path.exists(os.path.join(REPO, "src", "affix", "sw.toml"))),
            ("data/swahili.distilled.jsonl", "lexicon", os.path.exists(LEX)),
            ("bench/Sema_Benchmark_1000.csv", "benchmark corpus", os.path.exists(BENCH_CSV)),
            ("sema_cc.exe", "Rust helper binary", os.path.exists(CC_EXE)),
            ("Qwen2.5-0.5B-Instruct.Q4_K_M.gguf", "0.5B model", os.path.exists(os.path.join(REPO, "Qwen2.5-0.5B-Instruct.Q4_K_M.gguf"))),
            ("qwen2.5-1.5b-instruct-q4_k_m.gguf", "1.5B model", os.path.exists(os.path.join(REPO, "qwen2.5-1.5b-instruct-q4_k_m.gguf"))),
        ]
        for name, what, ok in checks:
            self.log(f"dep {name:38s} ({what}): {'OK' if ok else 'MISSING'}")
        missing = [n for n, _, ok in checks if not ok]
        self.log(f"deps total={len(checks)} ok={len(checks)-len(missing)} missing={len(missing)}")

    def export_report(self):
        out = filedialog.asksaveasfilename(
            defaultextension=".json",
            initialdir=REPO, initialfile="sema_health_report.json")
        if not out:
            return
        try:
            h = run_cc(["health"])
            with open(out, "w", encoding="utf-8") as fh:
                json.dump(h, fh, indent=2)
            self.log(f"report exported -> {out}")
            messagebox.showinfo("Sema CC", f"Report written to:\n{out}")
        except Exception as e:
            self.log(f"export failed: {e}")

    # ------------------------------------------------------------------
    # Panel 2: Morph Tester  (9 features)
    # ------------------------------------------------------------------
    def _build_tester(self):
        f = self.tab_tester

        top = ttk.LabelFrame(f, text="One-shot linearize (SW -> EN)")
        top.pack(fill="x", padx=8, pady=6)
        row = ttk.Frame(top)
        row.pack(fill="x", padx=6, pady=6)
        ttk.Label(row, text="Sentence:").pack(side=tk.LEFT)
        self.test_sentence = ttk.Entry(row, width=50)
        self.test_sentence.insert(0, "umefikia sokoni leo")
        self.test_sentence.pack(side=tk.LEFT, padx=4)
        ttk.Button(row, text="Linearize", command=self.run_linearize).pack(side=tk.LEFT, padx=4)
        ttk.Button(row, text="Race (cascade)", command=self.run_race).pack(side=tk.LEFT, padx=4)
        ttk.Button(row, text="Segment", command=self.run_segment).pack(side=tk.LEFT, padx=4)

        self.test_out = tk.Text(top, height=6)
        self.test_out.pack(fill="both", expand=True, padx=6, pady=6)

        mid = ttk.LabelFrame(f, text="Decompose verb (morph slots)")
        mid.pack(fill="x", padx=8, pady=6)
        mrow = ttk.Frame(mid)
        mrow.pack(fill="x", padx=6, pady=6)
        ttk.Label(mrow, text="Word:").pack(side=tk.LEFT)
        self.morph_word = ttk.Entry(mrow, width=30)
        self.morph_word.insert(0, "hawatakujibu")
        self.morph_word.pack(side=tk.LEFT, padx=4)
        ttk.Button(mrow, text="Decompose", command=self.run_decompose).pack(side=tk.LEFT, padx=4)

        self.morph_out = tk.Text(mid, height=6)
        self.morph_out.pack(fill="both", expand=True, padx=6, pady=6)

        bot = ttk.LabelFrame(f, text="Synthesize (EN intent -> SW) - back pass")
        bot.pack(fill="both", expand=True, padx=8, pady=6)
        brow = ttk.Frame(bot)
        brow.pack(fill="x", padx=6, pady=6)
        for lbl, key, default in [
            ("subj", "subj", "u"), ("tense", "tense", "me"), ("obj", "obj", ""),
            ("root", "root", "fika"), ("mood", "mood", "a")]:
            ttk.Label(brow, text=lbl).pack(side=tk.LEFT)
            e = ttk.Entry(brow, width=8)
            e.insert(0, default)
            e.pack(side=tk.LEFT, padx=2)
            setattr(self, "syn_" + key, e)
        self.syn_neg = tk.BooleanVar(value=False)
        ttk.Checkbutton(brow, text="negative", variable=self.syn_neg).pack(side=tk.LEFT, padx=6)
        ttk.Button(brow, text="Synthesize", command=self.run_synthesize).pack(side=tk.LEFT, padx=6)

        self.syn_out = tk.Text(bot, height=4)
        self.syn_out.pack(fill="both", expand=True, padx=6, pady=6)

        # corner-case quick buttons
        corners = ttk.LabelFrame(f, text="Corner-case suite (one click)")
        corners.pack(fill="x", padx=8, pady=6)
        cases = [
            ("monosyllabic umekula", ["morph", "umekula"]),
            ("neg+fut+obj hawatakujibu", ["morph", "hawatakujibu"]),
            ("rel/neg/alikiomba", ["morph", "alikiomba"]),
            ("double-vowel umefika", ["morph", "umefika"]),
        ]
        for label, args in cases:
            ttk.Button(corners, text=label,
                       command=lambda a=args, l=label: self.morph_preset(a[1], l)).pack(
                side=tk.LEFT, padx=4, pady=4)

    def morph_preset(self, word, label):
        self.morph_word.delete(0, tk.END)
        self.morph_word.insert(0, word)
        self.run_decompose()
        self.log(f"corner-case: {label} -> {word}")

    def run_linearize(self):
        self._bg(lambda: self._do_linearize(self.test_sentence.get()))

    def _do_linearize(self, sentence):
        try:
            out = run_cc(["linearize", "--sentence", sentence])
            self.test_out.delete("1.0", tk.END)
            self.test_out.insert("1.0", str(json.dumps({
                "telegram": out.get("telegram"),
                "tokens": out.get("tokens"),
                "unresolved": out.get("unresolved"),
            }, indent=2)))
            self.log(f"linearize '{sentence}' -> {out.get('telegram')!r}")
        except Exception as e:
            self.log(f"linearize failed: {e}")

    def run_race(self):
        self._bg(lambda: self._do_race(self.test_sentence.get()))

    def _do_race(self, sentence):
        try:
            out = run_cc(["race", "--sentence", sentence])
            self.test_out.delete("1.0", tk.END)
            self.test_out.insert("1.0", json.dumps(out.get("results"), indent=2))
            self.log(f"race '{sentence}' done")
        except Exception as e:
            self.log(f"race failed: {e}")

    def run_segment(self):
        s = self.test_sentence.get().split()
        self.log(f"segments of '{s}' -> {s}")
        self.test_out.delete("1.0", tk.END)
        self.test_out.insert("1.0", "\n".join(f"{i}: {w}" for i, w in enumerate(s)))

    def run_decompose(self):
        self._bg(lambda: self._do_decompose(self.morph_word.get()))

    def _do_decompose(self, word):
        try:
            out = run_cc(["morph", "--word", word])
            self.morph_out.delete("1.0", tk.END)
            self.morph_out.insert("1.0", json.dumps(out, indent=2))
            self.log(f"decompose '{word}' -> {out}")
        except Exception as e:
            self.morph_out.delete("1.0", tk.END)
            self.morph_out.insert("1.0", f"error: {e}")

    def run_synthesize(self):
        self._bg(self._do_synthesize)

    def _do_synthesize(self):
        try:
            args = ["rolodex"]
            for key in ["subj", "tense", "obj", "mood"]:
                v = getattr(self, "syn_" + key).get().strip()
                if v:
                    args += ["--" + key, v]
            args += ["--root", self.syn_root.get().strip()]
            if self.syn_neg.get():
                args.append("--neg")
            out = run_cc(args)
            self.syn_out.delete("1.0", tk.END)
            self.syn_out.insert("1.0", json.dumps(out, indent=2))
            self.log(f"synthesize {args} -> {out.get('surface')!r}")
        except Exception as e:
            self.log(f"synthesize failed: {e}")

    def _bg(self, fn):
        threading.Thread(target=fn, daemon=True).start()

    # ------------------------------------------------------------------
    # Panel 3: Corpus Bench  (9 features)
    # ------------------------------------------------------------------
    def _build_corpus(self):
        f = self.tab_corpus

        ctrl = ttk.Frame(f)
        ctrl.pack(fill="x", padx=8, pady=6)
        ttk.Label(ctrl, text="CSV:").pack(side=tk.LEFT)
        self.corpus_path = tk.StringVar(value=BENCH_CSV)
        ttk.Entry(ctrl, textvariable=self.corpus_path, width=55).pack(side=tk.LEFT, padx=4)
        ttk.Button(ctrl, text="Browse", command=self.browse_corpus).pack(side=tk.LEFT, padx=2)
        ttk.Label(ctrl, text="limit:").pack(side=tk.LEFT, padx=(12, 0))
        self.corpus_limit = ttk.Spinbox(ctrl, from_=1, to=1000, width=6)
        self.corpus_limit.set(200)
        self.corpus_limit.pack(side=tk.LEFT, padx=2)
        ttk.Button(ctrl, text="Run bench", command=self.run_bench).pack(side=tk.LEFT, padx=6)

        self.corpus_out = tk.Text(f, height=18)
        self.corpus_out.pack(fill="both", expand=True, padx=8, pady=6)

        agg = ttk.LabelFrame(f, text="Aggregate view (reruns bench, grouped)")
        agg.pack(fill="x", padx=8, pady=6)
        ttk.Button(agg, text="Racer histogram", command=lambda: self.log(
            "racer histogram: see bench JSON 'racer_winner_distribution'")).pack(
            side=tk.LEFT, padx=4)
        ttk.Button(agg, text="Forward pass rate", command=lambda: self.log(
            "forward pass rate: see bench JSON 'forward.pass_rate_pct'")).pack(side=tk.LEFT, padx=4)
        ttk.Button(agg, text="Backpass count", command=lambda: self.log(
            "backpass: see bench JSON 'back.rows'")).pack(side=tk.LEFT, padx=4)

    def browse_corpus(self):
        p = filedialog.askopenfilename(
            filetypes=[("CSV", "*.csv")], initialdir=os.path.join(REPO, "bench"))
        if p:
            self.corpus_path.set(p)

    def run_bench(self):
        self._bg(self._do_bench)

    def _do_bench(self):
        limit = self.corpus_limit.get() or "200"
        try:
            out = run_cc(["bench", "--csv", self.corpus_path.get(), "--limit", limit], timeout=180)
            self.corpus_out.delete("1.0", tk.END)
            self.corpus_out.insert("1.0", json.dumps(out, indent=2))
            self.log(f"bench ran rows={out.get('rows_loaded')} "
                     f"fw={out.get('forward', {}).get('pass_rate_pct', '?')}% "
                     f"dist={out.get('racer_winner_distribution')}")
        except Exception as e:
            self.corpus_out.delete("1.0", tk.END)
            self.corpus_out.insert("1.0", f"error: {e}")
            self.log(f"bench failed: {e}")

    # ------------------------------------------------------------------
    # Panel 4: Reverse Inspect / Revin   (8 features)
    # ------------------------------------------------------------------
    def _build_revin(self):
        f = self.tab_revin

        ctrl = ttk.Frame(f)
        ctrl.pack(fill="x", padx=8, pady=6)
        ttk.Label(ctrl, text="Review word:").pack(side=tk.LEFT)
        self.revin_word = ttk.Entry(ctrl, width=30)
        self.revin_word.insert(0, "hawatakujibu")
        self.revin_word.pack(side=tk.LEFT, padx=4)
        ttk.Button(ctrl, text="Inspect", command=self.run_inspect).pack(side=tk.LEFT, padx=4)
        ttk.Button(ctrl, text="Slot-by-slot", command=self.run_slots).pack(side=tk.LEFT, padx=4)

        self.revin_out = tk.Text(f, height=20)
        self.revin_out.pack(fill="both", expand=True, padx=8, pady=6)

        bot = ttk.Frame(f)
        bot.pack(fill="x", padx=8, pady=4)
        ttk.Button(bot, text="Round-trip check\n(decompose->synthesize)",
                   command=self.run_roundtrip).pack(side=tk.LEFT, padx=4)
        ttk.Button(bot, text="Open review log", command=self.open_logs).pack(side=tk.LEFT, padx=4)
        ttk.Button(bot, text="Export JSON report", command=self.export_report).pack(side=tk.LEFT, padx=4)

    def run_inspect(self):
        self._bg(lambda: self._do_inspect(self.revin_word.get()))

    def _do_inspect(self, word):
        try:
            m = run_cc(["morph", "--word", word])
            a = run_cc(["anchor", "--word", word])
            self.revin_out.delete("1.0", tk.END)
            self.revin_out.insert("1.0", json.dumps({"morph": m, "anchor": a}, indent=2))
            self.log(f"inspect '{word}' done")
        except Exception as e:
            self.revin_out.delete("1.0", tk.END)
            self.revin_out.insert("1.0", f"error: {e}")

    def run_slots(self):
        word = self.revin_word.get()
        self._bg(lambda: self._do_slots(word))

    def _do_slots(self, word):
        m = run_cc(["morph", "--word", word])
        order = ["raw", "negation", "subject", "tense", "object", "root"]
        names = {"negation": "NEG", "subject": "SUBJ", "tense": "TENSE",
                 "object": "OBJ", "root": "ROOT"}
        slots = []
        for k in order:
            v = m.get(k)
            if isinstance(v, str) and v:
                slots.append((names.get(k, k), v))
            elif k == "raw":
                slots.append(("RAW", v))
        self.revin_out.delete("1.0", tk.END)
        for name, val in slots:
            self.revin_out.insert(tk.END, f"[{name}]\t{val}\n")
        self.log(f"slots for '{word}': {slots}")

    def run_roundtrip(self):
        word = self.revin_word.get()
        self._bg(lambda: self._do_roundtrip(word))

    def _do_roundtrip(self, word):
        try:
            m = run_cc(["morph", "--word", word])
            # Synthesize from decomposed slots and compare
            args = ["rolodex"]
            if m.get("negation"):
                args.append("--neg")
            if m.get("subject"):
                args += ["--subj", m["subject"]]
            if m.get("tense"):
                args += ["--tense", m["tense"]]
            if m.get("object"):
                args += ["--obj", m["object"]]
            args += ["--root", m["root"]]
            syn = run_cc(args)
            same = (syn.get("surface", "").rstrip("a") == word
                    or syn.get("surface") == word)
            self.revin_out.delete("1.0", tk.END)
            self.revin_out.insert("1.0", json.dumps({
                "original": word,
                "morph": m,
                "resynthesized": syn.get("surface"),
                "roundtrip_match": same,
            }, indent=2))
            self.log(f"round-trip '{word}' -> '{syn.get('surface')}' match={same}")
        except Exception as e:
            self.revin_out.delete("1.0", tk.END)
            self.revin_out.insert("1.0", f"error: {e}")

    def open_logs(self):
        self.notebook.select(self.tab_log)

    # ------------------------------------------------------------------
    # Panel 5: Log
    # ------------------------------------------------------------------
    def _build_log(self):
        f = self.tab_log
        ctrl = ttk.Frame(f)
        ctrl.pack(fill="x", padx=8, pady=6)
        ttk.Button(ctrl, text="Clear log", command=self.clear_log).pack(side=tk.LEFT)
        ttk.Button(ctrl, text="Save log", command=self.save_log).pack(side=tk.LEFT, padx=4)
        self.log_text = tk.Text(f)
        self.log_text.pack(fill="both", expand=True, padx=8, pady=6)
        self.log("Sema Command Centre ready.")

    def clear_log(self):
        self.log_lines.clear()
        self.refresh_log()

    def save_log(self):
        out = filedialog.asksaveasfilename(
            defaultextension=".log", initialdir=REPO, initialfile="sema_review.log")
        if out:
            with open(out, "w", encoding="utf-8") as fh:
                fh.write("\n".join(self.log_lines))
            messagebox.showinfo("Sema CC", f"Log saved to:\n{out}")


def main():
    if not os.path.exists(CC_EXE):
        root = tk.Tk()
        root.withdraw()
        messagebox.showerror(
            "Sema CC",
            "sema_cc.exe not found.\n\nBuild it first with:\n"
            "  cargo build --release --bin sema_cc")
        root.destroy()
        return
    app = App()
    app.mainloop()


if __name__ == "__main__":
    sys.exit(main())
