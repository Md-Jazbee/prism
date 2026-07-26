# P11 phase gate scorecard

**Date:** 2026-07-26  
**Phase:** Install & Distribution  
**Status:** **PARTIAL** — Stage A+B complete; Stage C cold-VM matrix deferred until a public GitHub Release exists

| Gate item | Result | Evidence |
|---|---|---|
| Release artifact contract | ✅ | `docs/architecture/RELEASE-ARTIFACTS.md` |
| `install.sh` / `install.ps1` (+ dry-run/uninstall) | ✅ | `scripts/install.*` |
| Release CI matrix + SHA256SUMS | ✅ | `.github/workflows/release.yml` |
| Homebrew + Scoop drafts | ✅ | `packaging/` |
| PRODUCT-SETUP leads with installers | ✅ | `docs/architecture/PRODUCT-SETUP.md` |
| `prism self-update` | ✅ | CLI wraps installer |
| Host adapters (≥3) | ✅ | cursor / vscode / claude / generic |
| Doctor checklist v2 | ✅ | binary path/version + hosts + hook |
| Ensure-install in generated assets | ✅ | `prism-agent` assets + `/prism-ensure-install` |
| `prism hook install` | ✅ | append-only post-commit |
| Local install smoke (in CI) | ✅ | `scripts/install-smoke.sh` + `ci.yml` `install-smoke` job |
| Installer verifies checksum end-to-end | ✅ | simulated release via `PRISM_DOWNLOAD_BASE=file://…` — download → verify → install → binary runs |
| Installer fails closed on tamper | ✅ | corrupted `SHA256SUMS` rejected (smoke tamper test) |
| Mirror / air-gapped install path | ✅ | `PRISM_DOWNLOAD_BASE` override in both installers |
| Cold macOS/Linux/Windows VM → MCP | ⬜ | Blocked on real `PRISM_GITHUB_REPO` + first `v*` tag |
| Upgrade path N→N+1 | ⬜ | Needs two published releases |
| Uninstall leaves no broken MCP | ◐ | Host uninstall tested locally; full VM pending |

## Commands

```bash
cargo test -p prism-cli -- hook::
cargo test -p prism-agent -- assets::
./scripts/install-smoke.sh
```

## Residual for Stage C exit

1. Set real GitHub remote / `PRISM_GITHUB_REPO`.
2. Tag `v0.0.1` (or current workspace version) to exercise `release.yml`.
3. Fill Homebrew/Scoop sha256 placeholders from `SHA256SUMS`.
4. Run cold-VM matrix; attach logs to this scorecard and flip status to **PASS**.

**Note:** the checksum/verify/install mechanics are already proven locally and in CI against a simulated release (`PRISM_DOWNLOAD_BASE`); the residual items only swap the source of truth from `file://` to GitHub Releases.
