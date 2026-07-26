# P12 five-arm accuracy report

**Mode:** `scripted_proxy`  
**Date:** 2026-07-26  
**Harness:** `python eval/baselines/five_arm.py`

> Live-judged quality is required to close the Phase 12 gate. This artifact is the
> scripted-proxy interim plus ablation attribution. See
> [P12-ADJUDICATION-PROTOCOL.md](./P12-ADJUDICATION-PROTOCOL.md).

## Arms (proxy)

| Arm | Protocol | Tokens | Quality | Citation validity |
|---|---|---:|---:|---:|
| A | explore | 18000 | 0.70 | 0.55 |
| B | explore | 12000 | 0.62 | 0.50 |
| C | prism | 800 | 0.72 | 0.85 |
| D | prism | 1200 | 0.74 | 0.85 |
| E | graphify | 2000 | 0.74 | 0.80 |

## ACC-5 (proxy)

- Claim: `quality(Medium+Prism) >= quality(Graphify) - 2pts AND tokens(C) <= 0.5 * tokens(E)`
- Status: **PASS_PROXY**
- Quality Δ (C−E): -2.0 pts
- Token ratio C/E: 0.4

## Ablations (Medium+Prism arm C quality_proxy)

| Config | Quality | Δ vs full |
|---|---:|---:|
| full | 0.720 | +0.0 pts |
| docs_off | 0.680 | -4.0 pts |
| communities_path_prefix | 0.700 | -2.0 pts |
| lexical_off | 0.700 | -2.0 pts |
| all_off | 0.640 | -8.0 pts |

## Doc-QA gold

- Tasks: 25 / 25
- Notes: DQ001–DQ025 authored; live adjudication pending

## Gate honesty

- ACC-1…ACC-7 **live-judged** results: **OPEN** (proxies must not close the gate).
- Residual: run adjudication protocol on DQ sample + Graphify arm E with pinned graph build.
