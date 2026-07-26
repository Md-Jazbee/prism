# P12 phase gate — Accuracy & Grounding

**Suite:** Stage D five-arm + ACC-1…ACC-7  
**Harness:** `python eval/baselines/p12_live_adjudication.py` → `eval/baselines/p12-live-adjudication/latest.json`  
**Gold:** `eval/tasks/doc-qa/DQ*.json` · `eval/tasks/seed-grounding/AG*.json`  
**Adjudication:** [`docs/eval/P12-ADJUDICATION-PROTOCOL.md`](../docs/eval/P12-ADJUDICATION-PROTOCOL.md)  
**Report:** [`docs/eval/P12-FIVE-ARM-REPORT.md`](../docs/eval/P12-FIVE-ARM-REPORT.md)  
**Status date:** 2026-07-27 · **Gate: PASS** (1A agent dual-pass + 2A live Graphify)

## Checklist

- [x] ACC-1 Doc-QA answerability ≥80% from one pack (live-judged) — **PASS** 80.0% (20/25); mean quality 0.77 — `eval/baselines/p12-live-adjudication/latest.json`
- [x] ACC-2 Placeholder-fragment rate = 0 (invariant test)
- [x] ACC-3 Seed-grounding precision ≥90% — **PASS** on AG sample (`eval/baselines/seed-grounding/latest.json`, precision=1.0)
- [x] ACC-4 Top-10 hubs contain 0 language builtins; Louvain communities; label dual-review ≥70% — **PASS** 95% (`eval/labeling/community-labels-p12-sample.json`)
- [x] ACC-5 Prism ≥ Graphify arm on shared narrative set at ≤½ tokens — **PASS** Δq=+43.3 pts, token ratio 0.461 (Prism 752 vs Graphify 1632 mean)
- [x] ACC-6 No vendored/fixture fragments unless anchored
- [x] ACC-7 Dual-reviewed precision sample n≥20 still ≥70% — **PASS** precision 0.99, κ=1.0, n_packs=20 (`eval/labeling/packs/DQ*.dual.json`)
- [x] Five-arm report published (live-judged + ablations) — [P12-FIVE-ARM-REPORT.md](../docs/eval/P12-FIVE-ARM-REPORT.md)
- [x] Ablations recorded (docs / communities / lexical)

## How to run

```bash
python eval/baselines/five_arm.py
python eval/baselines/seed_grounding_score.py
python eval/baselines/doc_qa_pack_answerability.py
python eval/baselines/p12_live_adjudication.py --budget 850
cargo test -p prism-compile -p prism-store -p prism-core --lib
./target/release/prism index .
./target/release/prism query repo-map .
graphify update . --no-cluster   # refresh arm E graph if needed
```

## Gate notes (2026-07-27)

- Judge mode **1A**: agent dual-pass rubric (R1 loose / R2 strict). Worksheets under `eval/baselines/p12-live-adjudication/` remain available for optional human **1C** spot-check.
- Graphify arm **2A**: live `graphify query --budget 2000` on updated `graphify-out/`.
- Residual honesty: explore arms A/B remain scripted placeholders; narrative ACC gate uses live C vs E.
