# Release artifact contract (P11 Stage A)

**Status:** Active — governs GitHub Releases produced by `.github/workflows/release.yml`  
**Consumers:** `scripts/install.sh`, `scripts/install.ps1`, Homebrew/Scoop manifests, agent ensure-install

## Versioning

- Git tag: `vMAJOR.MINOR.PATCH` (semver), matching `[workspace.package].version` when cutting a release.
- Artifact version string: tag **without** the leading `v` (e.g. tag `v0.0.1` → `0.0.1`).

## Target triples (minimum matrix)

| Triple | Runner (CI) | Archive |
|---|---|---|
| `aarch64-apple-darwin` | `macos-latest` | `.tar.gz` |
| `x86_64-apple-darwin` | `macos-13` | `.tar.gz` |
| `x86_64-unknown-linux-gnu` | `ubuntu-latest` | `.tar.gz` |
| `aarch64-unknown-linux-gnu` | `ubuntu-24.04-arm` | `.tar.gz` |
| `x86_64-pc-windows-msvc` | `windows-latest` | `.zip` |

**Stretch (not required for P11 gate):** `*-unknown-linux-musl`, `aarch64-pc-windows-msvc`.

## Archive layout

Each archive contains a **single** binary at the root:

| Platform | Archive name | Binary path inside archive |
|---|---|---|
| Unix | `prism-{version}-{triple}.tar.gz` | `prism` |
| Windows | `prism-{version}-{triple}.zip` | `prism.exe` |

No nested directories. No README inside the archive (docs live in the repo / release notes).

## Checksums

Release asset `SHA256SUMS` (no extension), one line per archive:

```text
<hex-sha256>  prism-0.0.1-aarch64-apple-darwin.tar.gz
```

Installers **must** verify the checksum before installing. Fail closed on mismatch.

## Download URL pattern

```text
https://github.com/${PRISM_GITHUB_REPO}/releases/download/v{version}/{asset}
```

Default `PRISM_GITHUB_REPO` for scripts: value of `PRISM_GITHUB_REPO` env, else `example/prism` until the public org is set. CI always uploads to the repository that runs the workflow (`github.repository`).

## Install destinations (scripts)

| OS | Default bindir |
|---|---|
| macOS / Linux | `$HOME/.local/bin` |
| Windows | `%LOCALAPPDATA%\Prism\bin` |

Installers print PATH guidance when the bindir is not on `PATH`.

## Non-goals

- Signing is optional for P11 Stage A (preferred later: cosign / minisign).
- SBOM attachment follows [docs/security/RELEASE-CHECKLIST.md](../security/RELEASE-CHECKLIST.md) when cutting a gate tag.
- Team/shared index artifacts are **out of scope** (P10).
