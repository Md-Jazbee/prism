#!/usr/bin/env bash
# Prism installer (macOS / Linux) — P11 Stage A
# Contract: docs/architecture/RELEASE-ARTIFACTS.md
#
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/${PRISM_GITHUB_REPO}/main/scripts/install.sh | bash
#   ./scripts/install.sh [--version VERSION] [--dry-run] [--uninstall] [--bin-dir DIR]
#
# Env:
#   PRISM_GITHUB_REPO    owner/repo (default: example/prism)
#   PRISM_VERSION        override version (without leading v)
#   PRISM_BIN_DIR        override install directory
#   PRISM_DOWNLOAD_BASE  override asset base URL (e.g. file:///tmp/rel or an
#                        internal mirror); skips GitHub. Requires --version.

set -euo pipefail

PRISM_GITHUB_REPO="${PRISM_GITHUB_REPO:-example/prism}"
BIN_DIR="${PRISM_BIN_DIR:-${HOME}/.local/bin}"
VERSION="${PRISM_VERSION:-}"
DRY_RUN=0
UNINSTALL=0

usage() {
  cat <<'EOF'
Usage: install.sh [options]

  --version VERSION   Install a specific release (e.g. 0.0.1). Default: latest.
  --bin-dir DIR       Install directory (default: ~/.local/bin)
  --dry-run           Print actions without writing files
  --uninstall         Remove the prism binary from the bindir
  -h, --help          Show this help
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --version)
      VERSION="${2:?}"
      shift 2
      ;;
    --bin-dir)
      BIN_DIR="${2:?}"
      shift 2
      ;;
    --dry-run)
      DRY_RUN=1
      shift
      ;;
    --uninstall)
      UNINSTALL=1
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "unknown option: $1" >&2
      usage >&2
      exit 2
      ;;
  esac
done

need_cmd() {
  command -v "$1" >/dev/null 2>&1 || {
    echo "error: required command not found: $1" >&2
    exit 1
  }
}

detect_triple() {
  local os arch
  os="$(uname -s)"
  arch="$(uname -m)"
  case "$os" in
    Darwin)
      case "$arch" in
        arm64|aarch64) echo "aarch64-apple-darwin" ;;
        x86_64) echo "x86_64-apple-darwin" ;;
        *) echo "unsupported macOS arch: $arch" >&2; exit 1 ;;
      esac
      ;;
    Linux)
      case "$arch" in
        x86_64|amd64) echo "x86_64-unknown-linux-gnu" ;;
        aarch64|arm64) echo "aarch64-unknown-linux-gnu" ;;
        *) echo "unsupported Linux arch: $arch" >&2; exit 1 ;;
      esac
      ;;
    *)
      echo "unsupported OS: $os (use install.ps1 on Windows)" >&2
      exit 1
      ;;
  esac
}

sha256_file() {
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$1" | awk '{print $1}'
  elif command -v shasum >/dev/null 2>&1; then
    shasum -a 256 "$1" | awk '{print $1}'
  else
    echo "error: need sha256sum or shasum" >&2
    exit 1
  fi
}

run() {
  if [[ "$DRY_RUN" -eq 1 ]]; then
    echo "+ $*"
  else
    "$@"
  fi
}

if [[ "$UNINSTALL" -eq 1 ]]; then
  target="${BIN_DIR}/prism"
  if [[ -e "$target" ]]; then
    run rm -f "$target"
    echo "removed $target"
  else
    echo "nothing to uninstall at $target"
  fi
  exit 0
fi

need_cmd curl
need_cmd tar
need_cmd mktemp

TRIPLE="$(detect_triple)"
API="https://api.github.com/repos/${PRISM_GITHUB_REPO}/releases"
DOWNLOAD_BASE="${PRISM_DOWNLOAD_BASE:-}"

if [[ -z "$VERSION" ]]; then
  if [[ -n "$DOWNLOAD_BASE" ]]; then
    echo "error: PRISM_DOWNLOAD_BASE requires an explicit --version" >&2
    exit 2
  fi
  echo "resolving latest release from ${PRISM_GITHUB_REPO}…"
  TAG="$(curl -fsSL "${API}/latest" | sed -n 's/.*"tag_name":[[:space:]]*"\([^"]*\)".*/\1/p' | head -1)"
  if [[ -z "$TAG" ]]; then
    echo "error: could not resolve latest release for ${PRISM_GITHUB_REPO}" >&2
    echo "hint: set PRISM_GITHUB_REPO and/or --version once a release exists" >&2
    exit 1
  fi
  VERSION="${TAG#v}"
else
  TAG="v${VERSION#v}"
  VERSION="${TAG#v}"
fi

ASSET="prism-${VERSION}-${TRIPLE}.tar.gz"
BASE="${DOWNLOAD_BASE:-https://github.com/${PRISM_GITHUB_REPO}/releases/download/${TAG}}"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

echo "downloading ${ASSET}…"
if [[ "$DRY_RUN" -eq 1 ]]; then
  echo "+ curl -fsSL ${BASE}/${ASSET} -o ${TMP}/${ASSET}"
  echo "+ curl -fsSL ${BASE}/SHA256SUMS -o ${TMP}/SHA256SUMS"
  echo "+ verify checksum + install to ${BIN_DIR}/prism"
  echo "dry-run complete (version=${VERSION} triple=${TRIPLE})"
  exit 0
fi

curl -fsSL "${BASE}/${ASSET}" -o "${TMP}/${ASSET}"
curl -fsSL "${BASE}/SHA256SUMS" -o "${TMP}/SHA256SUMS"

EXPECTED="$(awk -v f="$ASSET" '$2 == f { print $1; exit }' "${TMP}/SHA256SUMS")"
if [[ -z "$EXPECTED" ]]; then
  echo "error: ${ASSET} not listed in SHA256SUMS" >&2
  exit 1
fi
ACTUAL="$(sha256_file "${TMP}/${ASSET}")"
if [[ "$EXPECTED" != "$ACTUAL" ]]; then
  echo "error: checksum mismatch for ${ASSET}" >&2
  echo "  expected: ${EXPECTED}" >&2
  echo "  actual:   ${ACTUAL}" >&2
  exit 1
fi

tar -xzf "${TMP}/${ASSET}" -C "$TMP"
if [[ ! -f "${TMP}/prism" ]]; then
  echo "error: archive missing prism binary" >&2
  exit 1
fi

mkdir -p "$BIN_DIR"
install -m 0755 "${TMP}/prism" "${BIN_DIR}/prism"

echo "installed ${BIN_DIR}/prism (${VERSION}, ${TRIPLE})"
case ":${PATH}:" in
  *":${BIN_DIR}:"*) ;;
  *)
    echo "note: ${BIN_DIR} is not on PATH — add it, then re-open your shell"
    echo "  export PATH=\"${BIN_DIR}:\$PATH\""
    ;;
esac
echo "next: cd <your-repo> && prism setup . && prism doctor --ready"
