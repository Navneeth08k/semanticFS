#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="/mnt/c/Users/navneeth/Desktop/NavneethThings/Projects/semanticFS"
CONFIG_PATH="config/wsl-local.toml"
MOUNTPOINT="/tmp/semanticfs-mnt"

cd "$REPO_ROOT"

# Keep cleanup idempotent for retries.
cleanup() {
  fusermount3 -uz "$MOUNTPOINT" >/dev/null 2>&1 || true
  if [[ -n "${FUSE_PID:-}" ]]; then
    kill "$FUSE_PID" >/dev/null 2>&1 || true
    wait "$FUSE_PID" >/dev/null 2>&1 || true
  fi
}
trap cleanup EXIT

mkdir -p "$MOUNTPOINT"
fusermount3 -uz "$MOUNTPOINT" >/dev/null 2>&1 || true

echo "[1/4] build index (release)"
cargo run --release -p semanticfs-cli -- --config "$CONFIG_PATH" index build >/tmp/semanticfs-index-pre.log 2>&1

echo "[2/4] start fuse mount"
target/release/semanticfs --config "$CONFIG_PATH" serve fuse >/tmp/semanticfs-fuse.log 2>&1 &
FUSE_PID="$!"

echo "[3/4] run session pin/refresh validator"
python3 scripts/validate_wsl_fuse_session.py \
  --repo "$REPO_ROOT" \
  --config "$CONFIG_PATH" \
  --mountpoint "$MOUNTPOINT" \
  --wait-seconds 120

echo "[4/4] complete"
