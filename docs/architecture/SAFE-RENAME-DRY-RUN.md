# Safe rename dry-run (P3 Stage C)

**Status:** Demo procedure — **not** a production rename engine  
**CLI:** `prism precise rename-dry-run`  
**Script:** [`scripts/precise/safe-rename-dry-run.sh`](../../scripts/precise/safe-rename-dry-run.sh)

---

## Goal

Show how Prism would support a **safe rename** once T2 references exist: list every precise `REFERENCES` / `CALLS` site for a symbol **without writing files**.

---

## Prerequisites

1. `prism index <workspace>`
2. Attach PreciseIndex: `prism precise import <index.json> --workspace <workspace>`
3. Symbol id or exact name known (`prism query resolve <name>`)

---

## Procedure

```bash
# Gate: fails with PRECISION_REQUIRED if no T2 overlay
cargo run -p prism-cli -- precise rename-dry-run \
  --symbol greet \
  --new-name hello \
  --workspace .

# Explicit heuristic override (labeled; not a safety claim)
cargo run -p prism-cli -- precise rename-dry-run \
  --symbol greet \
  --new-name hello \
  --allow-heuristic
```

Or:

```bash
./scripts/precise/safe-rename-dry-run.sh . greet hello
```

### Output shape

```json
{
  "mode": "dry_run",
  "writes": false,
  "old_name": "greet",
  "new_name": "hello",
  "tier": "T2",
  "sites": [
    { "edge_kind": "REFERENCES", "confidence": "precise", "src": "…", "file_path": "…", "dst": "…" }
  ],
  "site_count": 1,
  "notes": ["No files modified. Apply rename only after human review."]
}
```

---

## Non-goals

- Applying edits, formatting, or updating imports automatically  
- Cross-repo / published-API renames  
- Treating heuristic callers as complete

---

## Security

No write tools are exposed over MCP in P3. Dry-run is CLI/script only. Any future apply-rename must keep the T2 gate and an explicit user confirm.
