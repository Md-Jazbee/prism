# Pilot: BurntSushi/ripgrep

| Field | Value |
|---|---|
| Upstream | https://github.com/BurntSushi/ripgrep |
| **Pinned SHA** | `f9c05a949d1a0dc8e16dee28ca9605d38611faeb` |
| Pin date | 2026-07-24 (upstream commit date) · frozen 2026-07-25 |
| License | Dual: MIT / Unlicense |
| Approx LOC | ~50k Rust (under `crates/`) |
| Languages | Rust |
| Why | Native Rust codebase matches Prism core language; ignore/globs matter |
| Local snapshot | `fixtures/repos/snapshots/ripgrep` (gitignored; re-clone at SHA to reproduce) |

## Reproduce

```bash
git clone https://github.com/BurntSushi/ripgrep fixtures/repos/snapshots/ripgrep
git -C fixtures/repos/snapshots/ripgrep checkout f9c05a949d1a0dc8e16dee28ca9605d38611faeb
```

## Cold-walk stats (P0, `prism index`)

| Metric | Value |
|---|---|
| files_discovered | 236 |
| files_hashed | 236 |
| files_secret_skipped | 0 |
| wall_time_ms cold / warm | 110 / 72 (warm = 236 skipped_unchanged) |
| `.prism/` size after cold walk | 136K |
| tree_fingerprint | `8f11e5d2f4aeffba…` |

## Ignore review notes

- Skip `target/`, `.git/`.
- Vendor crates under third-party paths if vendored.
