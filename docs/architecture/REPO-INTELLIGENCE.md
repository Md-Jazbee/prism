# Derived repository intelligence catalog (P5 Stage A)

**Status:** Locked for Stage A  
**Implementation:** `prism-store` (`communities`, `intel`) · MCP `repo_map` / `entrypoints` / `detect_changes`  
**Refresh:** [INTEL-REFRESH.md](./INTEL-REFRESH.md)

---

## Products

| Product | Method | Confidence | MCP / CLI |
|---|---|---|---|
| **Communities** | Path-prefix clustering (dir segments) | Heuristic — not Leiden | `repo_map` |
| **Hubs** | Undirected edge degree on Symbol/File/Module | Heuristic (T1 CALLS inflate degree) | `repo_map` |
| **Entrypoints** | Name/path heuristics (`main`, `__main__`, `cli`, handlers, `app`) | Heuristic | `entrypoints` |
| **Layering hints** | IMPORTS across path-prefix layers; flag “upward” edges | Heuristic | `repo_map.layering` / intel report |
| **Change hotspots** | Git `log --numstat` churn (fallback: high degree files) | Observed if git; else heuristic | `detect_changes` + intel |
| **Ambiguity index** | CALLS heuristic/unresolved rates | Labeled mixture | `precise ambiguity` · auto-T2 policy |
| **Contract surfaces** | High fan-in symbols at community borders | Heuristic | intel report `contracts` |

LLM naming of communities is **optional and not required** for Stage A. Labels = path prefixes (deterministic, memoized by nature).

---

## Architecture pack budget

Architecture-layer fragments stay **tiny by default**: community labels + top hubs + entrypoint names. Bodies are drop_order optional (`deep_module_bodies`).

---

## Ambiguity → auto-require T2

When `AmbiguityIndex.require_t2` is true:

| Intent | Behavior |
|---|---|
| `impact` | Run `UpgradePrecision` (`optional_on_ambiguity`) |
| `architecture` / orientation | Note in intel report; do not block `repo_map` |
| `refactor` | Already mandatory UpgradePrecision |

See [AMBIGUITY-INDEX.md](./AMBIGUITY-INDEX.md) · [UPGRADE-POLICY.md](./UPGRADE-POLICY.md).
