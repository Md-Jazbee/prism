# Intent recipe catalog v1 (P2 Stage A)

**Source:** ADD §17.2 seed rules + §18.1 drop order  
**Plugin card:** [`schemas/plugins/IntentRecipe.md`](../../schemas/plugins/IntentRecipe.md)  
**Implementation:** `prism_plan::recipe::recipe_for`

Recipes are **deterministic**. Each produces a plan without an LLM.

| Intent | Seeds | Expand | Must-include | Min tier notes |
|---|---|---|---|---|
| `repo_qa` | named symbols / paths | def + 1-hop signatures | primary definition + signature | T1 |
| `debug` | stack + error | backward slice + recent diff | error/stack verbatim + primary frame body | T1 now; Slice→P4 |
| `impact` | changed / named symbols | forward IMPACTS depth 1–3 | seed symbols + depth-1 cone | T1 heuristic |
| `refactor` | target symbol | all REFERENCES (T2+) | target def + reference list | warn without T2 |
| `generate` | target file locus | types + one exemplar | insertion neighborhood + type sigs | T1 |
| `review` | PR / worktree diff | impact cone + tests | diff hunks + depth-1 impact | T1 |
| `architecture` | communities | hubs + boundaries | community map + hubs | T1; anchors optional |

## Drop order (shared default, ADD §18.1)

1. Low-confidence embedding seeds  
2. Depth-3+ impact / neighbor nodes  
3. Neighbor bodies (keep signatures)  
4. Secondary exemplars  
5. Architecture prose  
6. **Never drop** primary criterion slice or error/stack verbatim  

Per-intent `drop_order` strings in the Plan IR name these priorities for Stage B enforcement.

## Classification heuristics

`classify_intent` uses keyword / hint rules (stack frames ⇒ debug, “impact of” ⇒ impact, …). Agents may pass `--intent` to override.

## Ambiguous / refuse

Questions with **no anchors** (and intent ≠ `architecture`) → `SCOPE_UNRESOLVED` asking for symbol, path, stack/error, or changed path.

## Fixtures

| Case | Path |
|---|---|
| Repo-QA | `fixtures/plans/repo_qa/` |
| Debug | `fixtures/plans/debug/` |
| Impact | `fixtures/plans/impact/` |
| Refactor | `fixtures/plans/refactor/` |
| Generate | `fixtures/plans/generate/` |
| Review | `fixtures/plans/review/` |
| Architecture | `fixtures/plans/architecture/` |
| Ambiguous refuse | `fixtures/plans/ambiguous/` |
