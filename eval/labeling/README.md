# Context precision labeling (P2 Stage B)

Process for measuring **context precision** (≥60% gate in Stage C / phase gate).

## Goal

For a sample of compiled Evidence Packs, label each fragment as **necessary** or **unnecessary** for answering the gold task. Precision = necessary kept / all kept.

## Sample size

Target **≥20 packs** across intents (repo-QA, impact, architecture, debug best-effort). Start with tasks T001–T011 where gold hints exist.

## Procedure

1. Freeze pack algorithm version (`PACK_SCHEMA_VERSION` + plan recipe notes).  
2. Run `prism compile "<task prompt>"` on the pinned pilot snapshot; save pack JSON.  
3. Two reviewers mark each fragment: `necessary` | `unnecessary` | `unsure`.  
4. Resolve `unsure` with discussion; record disagreement rate.  
5. Store labels under `eval/labeling/packs/<task_id>.json` (versioned).  

## Label file shape (draft)

```json
{
  "task_id": "T001",
  "plan_id": "…",
  "pack_schema": "0.0.1",
  "fragments": [
    { "fragment_id": "frag:…", "label": "necessary", "note": "" }
  ]
}
```

## Rules

- Do not change gold task answers silently after a published scorecard — cut a new suite version.  
- Prefer dual review on the precision sample.  
- Necessary-span labels are versioned with the pack algorithm.
