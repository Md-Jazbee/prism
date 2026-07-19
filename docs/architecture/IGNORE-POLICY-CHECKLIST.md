# Ignore policy review checklist (P0 Stage A)

- [ ] `.gitignore` honored via `ignore` crate
- [ ] Vendor heuristics: `node_modules`, `vendor`, `target`, `.venv`, `dist`, `build`
- [ ] Secret defaults never indexed: `.env*`, `credentials.json`, `*.pem`, SSH keys
- [ ] `.prism/` runtime dir excluded from discover
- [ ] Index size measured on pilot repos after first cold walk
- [ ] Dirty vs clean commit identity distinguishable (`SnapshotId.dirty`)
