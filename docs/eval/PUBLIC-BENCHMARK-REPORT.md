# Prism public benchmark report (Phase 5)

**Suite version:** [`eval/SUITE-VERSION.md`](../../eval/SUITE-VERSION.md)  
**Date:** generated / maintained with `prism-eval p5-scorecard`  
**Status:** Proxy-complete; LLM four-arm quality pending under `eval/baselines/`

---

## Claims (what we assert today)

| Claim | Evidence | Confidence |
|---|---|---|
| Structural token↓ ≥10× vs explore | P1 proxy **~21.7×** (`p1-scorecard`) | Proxy (hop×token model) |
| Debug token↓ ≥5× vs explore | P4 proxy **40×** (`p4-scorecard`) | Proxy |
| One-shot `compile_context` preferred | AGENT-USAGE + MCP primary tool | Design + product |
| Precise CALLS uplift when overlay present | P3 oracle +50pp precision | Fixture oracle |
| Slice + error never dropped under budget | Pack gates + unit tests | Tested |
| Plugin path without core changes | [plugin-guide.md](../contributing/plugin-guide.md) | Documented |

---

## Four-arm methodology (target)

| Arm | Protocol |
|---|---|
| A | Frontier + explore (grep/read/glob only) |
| B | Medium + explore |
| C | Medium + Prism (`compile_context` first) |
| D | Frontier + Prism |

**Gate target:** quality(C) ≥ quality(A) − 3 pts on the frozen suite.

**Today:** arms A–D LLM scores are **not yet measured**. Interim quality claim is deferred; completeness proxies (necessary_spans, labeled pack precision) stand in. Plan to close: run pinned pilots at frozen SHAs with recorded prompts under `eval/baselines/`, dual-review ≥20 packs, then re-run `p5-scorecard`.

---

## Context precision

| Metric | Value | Gate |
|---|---|---|
| Proxy-v0 mean (n=5 packs) | **60%** | North star ≥70% |
| Interim | Honest gap of ~10pp | Close via dual human review + pack tuning |

See `eval/labeling/README.md`.

---

## Reproducibility

1. Pin task cards in `eval/tasks/` (commit SHAs on pilots).  
2. `cd eval && uv run prism-eval smoke`  
3. Regenerate phase scorecards: `p1`…`p5-scorecard`  
4. Language goldens: `./scripts/plugins/conformance-check.sh`  
5. Do **not** mutate pilot snapshots mid-report.

---

## Caveats / non-goals

- Proxies are not substitute for frontier LLM judgments.  
- T1 CALLS remain heuristic without PreciseIndex.  
- Communities are path-prefix, not Leiden.  
- Overlap with Graphify/GitNexus: Prism differentiates on Evidence Pack + Slice + eval discipline — not “another tree-sitter MCP.”

## Residual risks

See [PROGRAM-RESIDUAL-RISKS.md](./PROGRAM-RESIDUAL-RISKS.md).
