# Time-to-orient task set

**Phase:** P7 Stage A / eval  
**Purpose:** Measure whether budgeted graph views beat text-only orientation — honestly, including where they do not.

## Protocol

1. **Cold repo** with a fresh index (`prism index`).
2. **Two arms** (within-subject, counterbalanced order):
   - **Text:** `prism compile` / `repo_map` JSON + file reads only
   - **Visual:** `prism view` / HTTP `/v1/view` rendered in `prism-graph-view` (+ optional pack panel)
3. Cap each trial at **10 minutes**. Record time-to-correct-answer, wrong answers, and give-ups.
4. Answers must cite a **file:line** or symbol id (same bar as packs).

## Tasks

| ID | Task | Correct answer shape | Text baseline | Visual view |
|---|---|---|---|---|
| TTO-1 | Name the top-level subsystems | 3–7 path prefixes | `repo_map` JSON | `architecture_map` |
| TTO-2 | Find the highest-degree hub symbol | symbol id + file | hubs JSON | `architecture_map` + focus |
| TTO-3 | List files likely hit by changing seed S | paths within depth 2 | `impact` JSON | `impact_cone` |
| TTO-4 | Trace who calls helper H (1–2 hops) | caller symbols | neighbors / compile | `slice_path` |
| TTO-5 | Spot duplicate symbol names | name + ≥2 files | ambiguity intel | `ambiguity_heat` |
| TTO-6 | Explain why fragment F was dropped from a pack | EXPLAIN reason code | pack JSON `drops` | `pack_map` + visual EXPLAIN |

## Metrics

| Metric | Definition |
|---|---|
| Time-to-orient | Median seconds to correct citation |
| Accuracy | % correct within time cap |
| Budget adherence | Views with `nodes_used ≤ max_nodes` (must be 100%) |
| Refusal quality | Oversized asks return `VIEW_TOO_LARGE` with usable anchors |

## Reporting

Publish deltas in [`eval/scorecards/p7-phase-gate.md`](../../eval/scorecards/p7-phase-gate.md). If visual loses on a task, say so — do not average away failures.
