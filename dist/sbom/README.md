# Software Bill of Materials (SBOM)

Generated CycloneDX JSON for Prism release provenance ([P15 REL-7](../docs/planning/PLANNING-AND-IMPLEMENTATION.md#22-phase-15--reliability-governance--release-trust)).

## Files

| File | Description |
|---|---|
| `prism.cdx.json` | **Release artifact** — dependency tree for the shipped `prism` CLI binary (`--describe binaries`) |
| `prism-cli.cdx.json` | Full `prism-cli` crate SBOM (all Cargo targets) |

- **Format:** CycloneDX JSON **spec 1.5**
- **Tool:** [cargo-cyclonedx](https://github.com/CycloneDX/cargo-cyclonedx) (reads `Cargo.lock`)
- **Version:** matches `[workspace.package].version` in root `Cargo.toml`

## Regenerate

```bash
./scripts/generate-sbom.sh
```

Install the generator once:

```bash
cargo install cargo-cyclonedx --locked
```

## Verify (optional)

```bash
# component count
python3 -c "import json; d=json.load(open('dist/sbom/prism.cdx.json')); print(len(d.get('components',[])), 'components')"

# SPDX/CycloneDX validators (if installed)
# cyclonedx validate --input-file dist/sbom/prism.cdx.json
```

## Release attachment

Attach `dist/sbom/prism.cdx.json` to GitHub Releases alongside `SHA256SUMS` when cutting a gate tag (see [RELEASE-ARTIFACTS.md](../docs/architecture/RELEASE-ARTIFACTS.md)).
