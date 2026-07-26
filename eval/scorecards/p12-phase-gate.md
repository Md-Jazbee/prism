# P12 phase gate — Accuracy & Grounding

**Suite:** Stage D five-arm + ACC-1…ACC-7  
**Harness:** `python eval/baselines/five_arm.py` → `eval/baselines/five-arm/latest.json`  
**Gold:** `eval/tasks/doc-qa/DQ*.json`  
**Adjudication:** [`docs/eval/P12-ADJUDICATION-PROTOCOL.md`](../docs/eval/P12-ADJUDICATION-PROTOCOL.md)

## Checklist

- [ ] ACC-1 Doc-QA answerability ≥80% from one pack (live-judged)
- [ ] ACC-2 Placeholder-fragment rate = 0 (invariant test)
- [ ] ACC-3 Seed-grounding precision ≥90%
- [ ] ACC-4 Top-10 hubs contain 0 language builtins; label acceptance ≥70%
- [ ] ACC-5 Prism ≥ Graphify arm on shared narrative set at ≤½ tokens
- [ ] ACC-6 No vendored/fixture fragments unless anchored
- [ ] ACC-7 Dual-reviewed precision sample n≥20 still ≥70%
- [ ] Five-arm report published (scripted proxy OK interim; live judge for gate)
- [ ] Ablations recorded (docs / communities / lexical seeds)

## How to run

```bash
python eval/baselines/five_arm.py
cargo test -p prism-compile --lib
cargo test -p prism-store communities
```

## Interim status (2026-07-26)

Stages A–C code landed: markdown Doc/Section indexing, honest `gaps[]`, path-class filter, resolved-only hubs, analyzer-pipeline re-extract. Stage D harness + **25/25** DQ tasks authored; adjudication protocol + scorecard present. Live-judged ACC gates remain open (scripted five-arm is interim only).
