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

sha256_file() {
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$1" | awk '{print $1}'
  else
    shasum -a 256 "$1" | awk '{print $1}'
  fi
}

detect_triple() {
  case "$(uname -s)/$(uname -m)" in
    Darwin/arm64|Darwin/aarch64) echo "aarch64-apple-darwin" ;;
    Darwin/x86_64) echo "x86_64-apple-darwin" ;;
    Linux/x86_64|Linux/amd64) echo "x86_64-unknown-linux-gnu" ;;
    Linux/aarch64|Linux/arm64) echo "aarch64-unknown-linux-gnu" ;;
    *) echo "unsupported: $(uname -s)/$(uname -m)" >&2; exit 1 ;;
  esac
}

echo "== install.sh --dry-run =="
"$ROOT/scripts/install.sh" --dry-run --version 0.0.1

echo "== simulated release: full download → verify → install (file:// base) =="
REL_V="0.0.1"
TRIPLE="$(detect_triple)"
REL_TMP="$(mktemp -d)"
REL_ASSET="prism-${REL_V}-${TRIPLE}.tar.gz"
STAGE="$(mktemp -d)"
cp "$PRISM_BIN" "$STAGE/prism"
tar -C "$STAGE" -czf "$REL_TMP/$REL_ASSET" prism
(cd "$REL_TMP" && printf '%s  %s\n' "$(sha256_file "$REL_ASSET")" "$REL_ASSET" > SHA256SUMS)
PRISM_DOWNLOAD_BASE="file://$REL_TMP" "$ROOT/scripts/install.sh" \
  --version "$REL_V" --bin-dir "$REL_TMP/bin"
"$REL_TMP/bin/prism" --version >/dev/null
echo "installed binary runs OK"

echo "== tamper test: corrupted checksum must fail closed =="
printf '%s  %s\n' "0000000000000000000000000000000000000000000000000000000000000000" "$REL_ASSET" > "$REL_TMP/SHA256SUMS"
if PRISM_DOWNLOAD_BASE="file://$REL_TMP" "$ROOT/scripts/install.sh" \
  --version "$REL_V" --bin-dir "$REL_TMP/bin2" 2>/dev/null; then
  echo "ERROR: tampered checksum was accepted" >&2
  exit 1
fi
echo "tampered checksum rejected (fail-closed) OK"
rm -rf "$REL_TMP" "$STAGE"

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
