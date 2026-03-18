#!/usr/bin/env bash
# SemanticFS installer for Linux and macOS.
#
# Downloads the latest pre-built release binary from GitHub Releases and
# installs it to ~/.local/bin/semanticfs (or a path you choose with --prefix).
#
# Usage:
#   curl -sSfL https://raw.githubusercontent.com/YOUR_ORG/semanticfs/main/scripts/install.sh | bash
#   bash scripts/install.sh [--prefix /usr/local] [--version v1.0.0]

set -euo pipefail

REPO="YOUR_ORG/semanticfs"  # ← set to your GitHub org/repo before publishing
BINARY_NAME="semanticfs"
DEFAULT_PREFIX="${HOME}/.local"

# ── Argument parsing ──────────────────────────────────────────────────────────
PREFIX="${DEFAULT_PREFIX}"
VERSION=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --prefix)  PREFIX="$2"; shift 2 ;;
    --version) VERSION="$2"; shift 2 ;;
    *)         echo "Unknown flag: $1"; exit 1 ;;
  esac
done

BIN_DIR="${PREFIX}/bin"

# ── Detect platform ───────────────────────────────────────────────────────────
OS="$(uname -s)"
ARCH="$(uname -m)"

case "${OS}" in
  Linux)
    case "${ARCH}" in
      x86_64) TARGET="x86_64-unknown-linux-gnu" ;;
      *) echo "Unsupported Linux architecture: ${ARCH}"; exit 1 ;;
    esac
    ARCHIVE_EXT="tar.gz"
    ;;
  Darwin)
    case "${ARCH}" in
      x86_64)  TARGET="x86_64-apple-darwin" ;;
      arm64)   TARGET="aarch64-apple-darwin" ;;
      *) echo "Unsupported macOS architecture: ${ARCH}"; exit 1 ;;
    esac
    ARCHIVE_EXT="tar.gz"
    ;;
  *)
    echo "Unsupported OS: ${OS}. Use the Windows PowerShell installer."
    exit 1
    ;;
esac

# ── Resolve version ───────────────────────────────────────────────────────────
if [[ -z "${VERSION}" ]]; then
  echo "Fetching latest release version..."
  VERSION="$(curl -sSfL "https://api.github.com/repos/${REPO}/releases/latest" \
    | grep '"tag_name"' | sed -E 's/.*"tag_name": *"([^"]+)".*/\1/')"
  if [[ -z "${VERSION}" ]]; then
    echo "Could not determine latest version. Pass --version vX.Y.Z explicitly."
    exit 1
  fi
fi

echo "Installing semanticfs ${VERSION} for ${TARGET}..."

# ── Download and install ──────────────────────────────────────────────────────
ARTIFACT="semanticfs-${VERSION}-${TARGET}.${ARCHIVE_EXT}"
URL="https://github.com/${REPO}/releases/download/${VERSION}/${ARTIFACT}"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "${TMP_DIR}"' EXIT

echo "Downloading ${URL}..."
curl -sSfL "${URL}" -o "${TMP_DIR}/${ARTIFACT}"

echo "Extracting..."
tar -xzf "${TMP_DIR}/${ARTIFACT}" -C "${TMP_DIR}"

mkdir -p "${BIN_DIR}"
EXTRACTED_DIR="${TMP_DIR}/semanticfs-${VERSION}-${TARGET}"
install -m 755 "${EXTRACTED_DIR}/${BINARY_NAME}" "${BIN_DIR}/${BINARY_NAME}"

# Also install the Python stdio wrapper alongside the binary
if [[ -f "${EXTRACTED_DIR}/semanticfs_mcp_stdio.py" ]]; then
  cp "${EXTRACTED_DIR}/semanticfs_mcp_stdio.py" "${BIN_DIR}/semanticfs_mcp_stdio.py"
  chmod 644 "${BIN_DIR}/semanticfs_mcp_stdio.py"
fi

# ── Done ──────────────────────────────────────────────────────────────────────
echo ""
echo "semanticfs ${VERSION} installed to ${BIN_DIR}/semanticfs"
echo ""

# Check PATH
if [[ ":${PATH}:" != *":${BIN_DIR}:"* ]]; then
  echo "  NOTE: ${BIN_DIR} is not in your PATH."
  echo "  Add this to your shell profile (~/.bashrc, ~/.zshrc, etc.):"
  echo ""
  echo "    export PATH=\"${BIN_DIR}:\${PATH}\""
  echo ""
fi

echo "Next steps:"
echo "  1. Create a config:  semanticfs init --root /path/to/your/repo"
echo "  2. Build the index:  semanticfs --config semanticfs.toml index build"
echo "  3. Start MCP server: semanticfs --config semanticfs.toml serve mcp-stdio"
echo ""
echo "For Claude Code integration, see: docs/setup_10_minute_agents.md"
