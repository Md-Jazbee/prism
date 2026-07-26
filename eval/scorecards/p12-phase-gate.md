# P12 phase gate — Accuracy & Grounding

**Suite:** Stage D five-arm + ACC-1…ACC-7  
**Harness:** `python eval/baselines/five_arm.py` → `eval/baselines/five-arm/latest.json`  
**Gold:** `eval/tasks/doc-qa/DQ*.json` · `eval/tasks/seed-grounding/AG*.json`  
**Adjudication:** [`docs/eval/P12-ADJUDICATION-PROTOCOL.md`](../docs/eval/P12-ADJUDICATION-PROTOCOL.md)  
**Report:** [`docs/eval/P12-FIVE-ARM-REPORT.md`](../docs/eval/P12-FIVE-ARM-REPORT.md)

## Checklist

- [ ] ACC-1 Doc-QA answerability ≥80% from one pack (live-judged) — **OPEN** (gold authored; live judge pending)
- [x] ACC-2 Placeholder-fragment rate = 0 (invariant test)
- [x] ACC-3 Seed-grounding precision ≥90% — **PASS** on AG sample (`eval/baselines/seed-grounding/latest.json`, precision=1.0)
- [x] ACC-4 Top-10 hubs contain 0 language builtins; Louvain communities shipped (label dual-review still open for ≥70%)
- [ ] ACC-5 Prism ≥ Graphify arm on shared narrative set at ≤½ tokens — **proxy only** (see five-arm report)
- [x] ACC-6 No vendored/fixture fragments unless anchored
- [ ] ACC-7 Dual-reviewed precision sample n≥20 still ≥70% — **OPEN**
- [x] Five-arm report published (scripted proxy + ablations) — live judge still required for gate
- [x] Ablations recorded (docs / communities / lexical)

## How to run

```bash
python eval/baselines/five_arm.py
python eval/baselines/seed_grounding_score.py
cargo test -p prism-compile -p prism-store -p prism-core --lib
./target/release/prism index .
```

## Interim status (2026-07-26)

**Code complete for Stages A–C** (docs layer, honest gaps, ACC-3 grounding, Louvain+hubs+bridges, G4 doc-edit incremental test, planted-secret fixture). Stage D harness + reports archived. **Phase gate remains OPEN** until live-judged ACC-1/ACC-5/ACC-7.
