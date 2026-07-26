# Semantic analysis crash policy (P4)

**Rule:** Broken or partial source must **never** crash the agent / CLI path.

| Failure | Behavior |
|---|---|
| Unreadable file | Skip; note in manifest / stderr |
| Tree-sitter parse error | Return empty `functions` + `notes: ["parse_error"]`; exit 0 for batch |
| Unsupported construct | Best-effort blocks; note `partial_cfg` |
| Criterion outside any function | Empty slice + `notes: ["criterion_not_in_function"]` |
| Panic in analyzer | Caught at CLI boundary → `SEMANTIC_PARTIAL` JSON error, non-zero only if `--strict` |

Agent-facing tools should treat missing semantic shards as a **gap**, not a hard failure — fall back to T1 neighborhood until Stage C debug recipes require slices.
