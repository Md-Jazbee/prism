# Prism public benchmark report v2 (Phase 9)

**Suite:** P9 four-arm + dual-review + agent traces  
**Date:** 2026-07-26  
**Harness:** `python eval/baselines/four_arm.py` → `eval/baselines/four-arm/latest.json`

---

## Four-arm results (scripted proxy mode)

| Arm | Protocol | First tool | Hops | Tokens proxy | Quality proxy |
|---|---|---|---:|---:|---:|
| A | Frontier + explore | grep | 12 | 18000 | 0.70 |
| B | Medium + explore | grep | 10 | 12000 | 0.62 |
| C | Medium + Prism | compile_context | 1 | 800 | 0.68 |
| D | Frontier + Prism | compile_context | 1 | 1200 | 0.72 |

**G1 (proxy):** `quality(C) − quality(A) = −2.0 pts` → within ≤3 pts band → **PASS_PROXY**.

**Caveat:** Live LLM judges are opt-in (`PRISM_FOUR_ARM_LLM=1`). Until then, published quality numbers are **scripted proxies**, not frontier API scores. Structural token/hop advantages remain large (C vs A ≈ 22× tokens).

---

## Dual-reviewed precision

| Sample | Precision | κ | Gate ≥70% |
|---|---:|---:|---|
| T001 dual (n=10 frags) | **70%** | 0.78 | Met on this sample |

Expand n≥20 in ongoing labeling; see `eval/labeling/packs/T001.dual.json`.

---

## Agent adoption (traces)

From `fixtures/workflows/*.trace.json` (+ live `--persist-trace`):

| Metric | Value | Target |
|---|---:|---:|
| First-tool-choice (`compile_context`) rate | see `latest.json` | ≥0.70 on Prism arms |
| Refusal-repair success (when refused) | 1.0 on repair fixture | report |

Prism arms (C/D) are **defined** to choose `compile_context` first; fixture `debug.trace.json` + repair fixture demonstrate the contract.

---

## Visual surface

P7 screenshot-diff suite remains the visual gate; metrics folded by reference to `eval/scorecards/p7-phase-gate.md`.

---

## Residual risks

R1 / R2 / R8 updated in [`PROGRAM-RESIDUAL-RISKS.md`](./PROGRAM-RESIDUAL-RISKS.md).
