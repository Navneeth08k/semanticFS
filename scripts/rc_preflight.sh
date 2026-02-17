#!/usr/bin/env bash
set -euo pipefail

CONFIG_PATH="${1:-config/semanticfs.sample.toml}"
FIXTURE_REPO="${2:-tests/fixtures/benchmark_repo}"
SOAK_SECONDS="${SOAK_SECONDS:-30}"

echo "[semanticfs] rc preflight starting"
echo "  config:  ${CONFIG_PATH}"
echo "  fixture: ${FIXTURE_REPO}"
echo "  soak:    ${SOAK_SECONDS}s"

cargo fmt --check
cargo test --workspace

cargo run --release -p semanticfs-cli -- --config "${CONFIG_PATH}" benchmark release-gate \
  --refresh \
  --fixture-repo "${FIXTURE_REPO}" \
  --soak-seconds 5

cargo run --release -p semanticfs-cli -- --config "${CONFIG_PATH}" benchmark soak \
  --duration-seconds "${SOAK_SECONDS}" \
  --fixture-repo "${FIXTURE_REPO}" \
  --max-soak-p95-ms 250 \
  --max-errors 0 \
  --max-rss-mb 2048

echo "[semanticfs] rc preflight passed"
echo "reports:"
echo "  .semanticfs/bench/latest.json"
echo "  .semanticfs/bench/lancedb_tuning.json"
echo "  .semanticfs/bench/release_gate.json"
echo "  .semanticfs/bench/soak_latest.json"
