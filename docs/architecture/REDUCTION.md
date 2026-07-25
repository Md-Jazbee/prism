# Reduction techniques catalog (P2 Stage B)

Lossy compilation with named risk (ADD §18).

| Technique | Keeps | Drops | When |
|---|---|---|---|
| Span slice | criterion + local window | unrelated fns | primary definitions / frames |
| Signature skeleton | API shape (name, file, kind) | bodies | neighbors, impact cone |
| Diff hunks | changed paths / hunks | unchanged | review intent |
| Community one-liner | orientation | detail | architecture layer |
| Dedup by fragment id | one copy | repeats | after KG enrich |
| CFG path summary | predicates on path | other branches | **P4** (placeholder) |

## Drop order under budget (ADD §18.1)

1. Low-confidence embedding seeds  
2. Depth-3+ impact / neighbor nodes  
3. Neighbor bodies (keep signatures)  
4. Secondary exemplars  
5. Architecture prose  
6. **Never drop** primary criterion slice or error/stack verbatim  

Implemented as ascending `drop_priority` fill; must-include forced to priority `0`.

## Extractive default

Code fragments are **extractive** (source windows / signatures). Abstractive LLM summaries are banned for code in v0; docs/ADR summaries deferred.
