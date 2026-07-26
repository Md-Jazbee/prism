# Optional runtime enrichment (P4 Stage C — design only)

**Status:** Experimental design; **not required** for Phase 4 gate  
**Policy:** Never delete static T1/T2/T3 edges when ingesting observations

---

## Goal

Weight debug slices toward **actually executed** paths using OTEL traces, coverage, or profiler samples — without making runtime a dependency of `compile_context`.

---

## Ingest model

```text
OBSERVED_CALLS / OBSERVED_SPAN  (tier = observed, confidence = observed)
  → overlay alongside static CALLS
  → slicer may prefer observed edges when present
  → static edges remain queryable forever
```

Storage (proposal):

```text
.prism/runtime/
  traces/<run_id>.json
  coverage/<commit>.json
```

Not mixed into hot adjacency until a future promotion step with certificates.

---

## Planner hook (future)

```text
if runtime available for snapshot_id:
  Slice.prefer_observed = true
else:
  static Slice only  # Stage B/C default
```

---

## Non-goals for gate

- No OTEL collector in-tree  
- No requirement that CI produce coverage for debug packs  
- No deletion / demotion of static edges based on missing coverage
