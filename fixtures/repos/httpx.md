# Pilot: encode/httpx

| Field | Value |
|---|---|
| Upstream | https://github.com/encode/httpx |
| **Pinned SHA** | `b5addb64f0161ff6bfe94c124ef76f6a1fba5254` |
| Pin date | 2026-02-23 (upstream commit date) · frozen 2026-07-25 |
| License | BSD-3-Clause (Encode OSS Ltd) |
| Approx LOC | ~8.8k (package `httpx/`), ~21k with tests |
| Languages | Python |
| Why | Clean modular client; strong CALLS/IMPORTS for T1 extractors |
| Local snapshot | `fixtures/repos/snapshots/httpx` (gitignored; re-clone at SHA to reproduce) |

## Reproduce

```bash
git clone https://github.com/encode/httpx fixtures/repos/snapshots/httpx
git -C fixtures/repos/snapshots/httpx checkout b5addb64f0161ff6bfe94c124ef76f6a1fba5254
```

## Cold-walk stats (P0, `prism index`)

| Metric | Value |
|---|---|
| files_discovered | 124 |
| files_hashed | 124 |
| files_secret_skipped | 0 |
| wall_time_ms cold / warm | 91 / 53 (warm = 124 skipped_unchanged) |
| `.prism/` size after cold walk | 88K |
| tree_fingerprint | `1078e278fdd04bf9…` |

## Ignore review notes

- Skip `.venv`, `site-packages`, large docs images if present.
- Never index `.env` / credential fixtures if any.
