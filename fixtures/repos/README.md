# Pilot repos (Phase 0 Stage A)

Snapshot SHAs are `PIN_ME` until frozen. Gold tasks reference these docs.

| Repo | Approx LOC | Languages | Role |
|---|---|---|---|
| [httpx](./httpx.md) | ~15–25k | Python | HTTP client — symbol / impact / architecture tasks |
| [ripgrep](./ripgrep.md) | ~40–60k | Rust | Search tool — structural + ignore-policy tasks |

## Freeze checklist

1. Clone at a tagged release or known-good commit.
2. Record SHA + date + license here.
3. Replace `PIN_ME` in `eval/tasks/*.json`.
4. Prefer content-addressed vendor under `fixtures/repos/snapshots/` only if license allows redistribution.
