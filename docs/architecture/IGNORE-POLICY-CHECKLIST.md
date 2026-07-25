# Ignore policy review checklist (P0 Stage A)

- [x] `.gitignore` honored via `ignore` crate
- [x] Vendor heuristics: `node_modules`, `vendor`, `target`, `.venv`, `dist`, `build`
- [x] Secret defaults never indexed: `.env*`, `credentials.json`, `*.pem`, SSH keys
- [x] `.prism/` runtime dir excluded from discover
- [x] Index size measured on pilot repos after first cold walk
- [x] Dirty vs clean commit identity distinguishable (`SnapshotId.dirty`)

## Pilot cold-walk measurements (2026-07-25)

| Repo (pinned SHA) | discovered | hashed | cold / warm ms | `.prism/` size |
|---|---:|---:|---|---|
| httpx `b5addb6` | 124 | 124 | 91 / 53 | 88K |
| ripgrep `f9c05a9` | 236 | 236 | 110 / 72 | 136K |

Warm walk skips 100% unchanged files (per-file hash match). Vendor trees and
`.git/` excluded by walker; no secret-sensitive files present in either pilot.
