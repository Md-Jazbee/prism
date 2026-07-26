# P12 ACC-4 — Community label dual-review worksheet

**Sample file:** [`eval/labeling/community-labels-p12-sample.json`](../../eval/labeling/community-labels-p12-sample.json)  
**Algorithm:** `louvain_v1+resolved_degree_hubs`  
**Target:** ≥70% dual-reviewed acceptance (n≥20)

## How to review

1. Open the sample JSON (20 communities from a live `prism query repo-map`).
2. Independently set `r1` / `r2` to `accept` | `reject` | `revise`.
3. Agree a `final_label` when revising; set `decision` to `accept` | `reject`.
4. Acceptance rate = rows with `decision=accept` / n.

Until both reviewers fill the sheet, ACC-4 **label** acceptance remains OPEN (hub denylist already ships).
