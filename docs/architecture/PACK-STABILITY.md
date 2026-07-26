# Pack stability property (whitespace-only)

**Status:** Specified + tested (P5 Stage B)  
**Test:** `prism-compile::tests::whitespace_only_change_keeps_must_include_stable`

---

## Property

Given the same question, anchors, intent, and budget:

1. Compile pack A from candidates derived from source S.  
2. Mutate S → S' by **whitespace-only** changes that do not alter extractor symbol ids (or use identical synthetic candidates with whitespace in optional noise only).  
3. Must-include fragment **ids** and **roles** in pack A and pack A' are equal (set equality).  
4. Optional fragments may differ in text/token estimates.

## Rationale

Agents and IDE peeks must not churn citations when the user only reformats code. Semantic/T1 identity is content-hash based for index invalidation, but Evidence Pack must-include should stay citation-stable when the selected graph nodes are unchanged.

## Out of scope

- Refactors that rename symbols  
- Budget changes  
- Intent changes
