#!/usr/bin/env bash
# Generate CycloneDX SBOMs for Prism release artifacts (P15 REL-7 prep).
#
# Requires: cargo-cyclonedx (cargo install cargo-cyclonedx --locked)
#
# Outputs:
#   dist/sbom/prism.cdx.json       — shipped `prism` binary dependency tree
#   dist/sbom/prism-cli.cdx.json   — prism-cli crate (all targets)
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

if ! command -v cargo-cyclonedx >/dev/null 2>&1; then
  echo "error: cargo-cyclonedx not found. Install with:" >&2
  echo "  cargo install cargo-cyclonedx --locked" >&2
  exit 1
fi

mkdir -p dist/sbom

echo "# generating workspace crate SBOMs (temporary under crates/*/)"
cargo cyclonedx --format json --spec-version 1.5 --describe crate --quiet

echo "# generating binary SBOMs (prism, prismd, prism-lsp)"
( cd crates/prism-cli && cargo cyclonedx --format json --spec-version 1.5 --describe binaries --quiet )

cp crates/prism-cli/prism-cli.cdx.json dist/sbom/prism-cli.cdx.json
cp crates/prism-cli/prism_bin.cdx.json dist/sbom/prism.cdx.json

find crates -name '*.cdx.json' -delete

echo "# SBOM written:"
ls -lh dist/sbom/
echo "# done"
