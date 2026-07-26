# Eval suite version

**suite_id:** `prism-eval-suite@0.5.0`  
**frozen_for:** Phase 5 public report  
**date:** 2026-07-26

## Contents

| Path | Role |
|---|---|
| `eval/tasks/T*.json` | Gold task cards (≥20) |
| `eval/labeling/packs/` | Context precision proxy labels |
| `fixtures/plans/`, `fixtures/packs/` | Planner / pack goldens |
| `fixtures/precise/oracle/` | T2 call-resolution oracle |
| `fixtures/languages/` | Extractor conformance |
| `fixtures/repos/*.md` | Pilot SHA pins |

## Change policy

Bump `suite_id` minor when adding tasks without changing scoring.  
Bump major when changing accepted_answer_criteria semantics or proxy formulas.
