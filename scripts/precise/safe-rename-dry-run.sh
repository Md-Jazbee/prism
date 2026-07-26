#!/usr/bin/env bash
# Safe rename dry-run demo (P3 Stage C) — read-only; never writes source.
# Usage: ./scripts/precise/safe-rename-dry-run.sh [workspace] [old_name] [new_name]
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
WS="${1:-.}"
OLD="${2:-greet}"
NEW="${3:-hello}"
cd "$ROOT"
echo "# safe-rename-dry-run workspace=$WS old=$OLD new=$NEW (writes=false)"
cargo run -q -p prism-cli -- precise rename-dry-run \
  --symbol "$OLD" \
  --new-name "$NEW" \
  --workspace "$WS"
