# P12 five-arm accuracy report

**Mode:** `agent_dual_pass_rubric+graphify_live`  
**Date:** 2026-07-27  
**Harness:** `python eval/baselines/p12_live_adjudication.py`

> Live-judged quality from agent dual-pass rubric (1A) + Graphify arm E (2A).
> Protocol: [P12-ADJUDICATION-PROTOCOL.md](./P12-ADJUDICATION-PROTOCOL.md).

## Arms (live-judged Doc-QA subset)

| Arm | Protocol | Tokens (mean) | Quality (mean R2) | Citation validity |
|---|---|---:|---:|---:|
| A | explore | 18000 | 0.70 | 0.55 |
| B | explore | 12000 | 0.62 | 0.50 |
| C | prism | 752 | 0.77 | 0.75 |
| D | prism | 903 | 0.79 | 0.75 |
| E | graphify | 1632 | 0.34 | 0.35 |

## ACC checklist (this run)

- ACC-1 answerable rate: **80.0%** (PASS; gate ≥80%)
- ACC-4 label acceptance: **95.0%** (PASS; gate ≥70%)
- ACC-5 Prism≥Graphify @ ≤½ tokens: **PASS** (Δq=43.3 pts, token ratio=0.461)
- ACC-7 dual-review precision: **99.0%** (κ=1.0, n_packs=20) (PASS; gate ≥70%, κ≥0.6)

## Gate

- Status: **PASS**
- Notes: All live ACC targets met

