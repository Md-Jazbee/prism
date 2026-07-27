#!/usr/bin/env bash
# Language extractor conformance — golden fixtures for every registered language.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"

echo "# prism plugin conformance"
cargo test -p prism-extract-python golden_simple_module_conformance -- --nocapture
cargo test -p prism-extract-rust golden_simple_mod_conformance -- --nocapture
cargo test -p prism-extract-java golden_simple_class_conformance -- --nocapture
cargo test -p prism-extract-perl golden_simple_module_conformance -- --nocapture
cargo test -p prism-extract -- --nocapture
echo "# conformance OK (python + rust + java + perl + registry)"
