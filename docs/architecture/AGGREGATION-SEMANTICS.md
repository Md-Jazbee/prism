# Aggregation semantics

**Phase:** P7 Stage A  
**Audience:** renderer + projection authors

## What a collapsed super-node means

A super-node (community, file group, collapsed LOD cluster) **stands for a set of member nodes**. It is an orientation affordance, not a new KG fact.

| Property | Rule |
|---|---|
| Label | Human path/prefix or “N symbols” |
| Tier | **Weakest** (highest T-number) among members — never upgrade |
| Confidence | **Weakest** among members (`heuristic` < `extracted` < `precise` < `observed` for display honesty: show the least trustworthy) |
| Citation | Union of member `node_ids` (capped); prefer a representative file path |
| Heat | max(member heats) when present |

**Display honesty:** a T1-heuristic community must never look like a T2-precise symbol.

## What a collapsed edge means

When endpoints collapse, parallel edges between the same super-node pair merge:

| Property | Rule |
|---|---|
| Weight / thickness | Member edge count (visual only) |
| Kind | Dominant kind if unanimous; else `AGGREGATED` |
| Confidence | **Weakest** member confidence |
| Tier | Weakest member tier |
| Citation | Sample of member edge citations (not claimed complete) |

## Forbidden implications

- Do not draw aggregated edges as solid “precise” when any member is heuristic.
- Do not hide the legend when aggregation is active.
- Do not invent call edges between communities that have no underlying member edges.
