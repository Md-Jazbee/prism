#!/usr/bin/env bash
# P11 Stage C — local install smoke (no GitHub Release required).
# Validates installers dry-run, host adapters, hooks, doctor, and asset bootstrap.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

PRISM_BIN="${PRISM_BIN:-}"
if [[ -z "$PRISM_BIN" ]]; then
  if [[ -x "$ROOT/target/debug/prism" ]]; then
    PRISM_BIN="$ROOT/target/debug/prism"
  else
    echo "building prism-cli…"
    cargo build -p prism-cli -q
    PRISM_BIN="$ROOT/target/debug/prism"
  fi
fi

echo "== install.sh --dry-run =="
"$ROOT/scripts/install.sh" --dry-run --version 0.0.1

echo "== host adapters =="
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT
# Minimal fake workspace for host files (no full index required for host cmds)
mkdir -p "$TMP"
"$PRISM_BIN" host install cursor "$TMP" >/dev/null
"$PRISM_BIN" host install vscode "$TMP" >/dev/null
"$PRISM_BIN" host install claude "$TMP" >/dev/null
"$PRISM_BIN" host install generic "$TMP" >/dev/null
"$PRISM_BIN" host status "$TMP" --json | grep -q '"registered": true'
"$PRISM_BIN" host uninstall cursor "$TMP" >/dev/null
"$PRISM_BIN" host uninstall vscode "$TMP" >/dev/null
"$PRISM_BIN" host uninstall claude "$TMP" >/dev/null
"$PRISM_BIN" host uninstall generic "$TMP" >/dev/null

echo "== git hook =="
GIT_TMP="$(mktemp -d)"
git -C "$GIT_TMP" init -q
"$PRISM_BIN" hook install "$GIT_TMP" >/dev/null
"$PRISM_BIN" hook status "$GIT_TMP" --json | grep -q '"installed": true'
"$PRISM_BIN" hook uninstall "$GIT_TMP" >/dev/null
"$PRISM_BIN" hook status "$GIT_TMP" --json | grep -q '"installed": false'
rm -rf "$GIT_TMP"

echo "== generate-assets ensure-install =="
ASSET_TMP="$(mktemp -d)"
"$PRISM_BIN" agent generate-assets "$ASSET_TMP" >/dev/null
grep -q "Ensure install" "$ASSET_TMP/AGENTS.md"
grep -q "/prism-ensure-install" "$ASSET_TMP/.prism/agent/skills.md"
rm -rf "$ASSET_TMP"

echo "== doctor (this workspace) =="
"$PRISM_BIN" doctor --json >/dev/null

echo "P11 install smoke: PASS"
