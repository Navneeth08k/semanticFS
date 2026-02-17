# SemanticFS v1.1 Scaffold

This repository contains a production-oriented scaffold for SemanticFS v1.1.

## Workspace crates
- `semanticfs-common`: shared config/types/audit/health structures.
- `policy-guard`: allow/deny filtering, secret detection, and redaction.
- `indexer`: indexing pipeline, schema management, symbol extraction, map precompute, two-phase publish.
- `retrieval-core`: symbol-first planner + BM25 + plain RRF fusion.
- `map-engine`: deterministic map summary retrieval + optional enrichment merge.
- `fuse-bridge`: virtual path router with inode/content LRU caches and snapshot-scoped reads.
- `mcp`: minimal MCP-compatible HTTP surface.
- `semanticfs-cli`: user-facing commands.

## Current implementation status
SemanticFS now works as a local intelligence layer around your repository rather than a plain file index.  
In bigger-picture terms, the system already supports the core grounded loop: semantic discovery (`/search`), deterministic orientation (`/map`), and byte-accurate verification (`/raw`) on snapshot-versioned data.  
Operationally, it includes policy enforcement, index publishing, MCP exposure, and observability surfaces needed to run this as an always-on developer service.

Implemented now:
- Full config model.
- SQLite schema and index versioning.
- Symbol table extraction and lookup.
- Policy guard + audit event model.
- Search and map virtual rendering paths.
- Vector persistence + retrieval path with RRF integration.
- Optional LanceDB vector backend (index sync + nearest-neighbor retrieval).
- Optional ONNX embedding backend via persistent Python sidecar with startup health check and batched embedding requests.
- MCP minimal tools/resources/prompt endpoints.
- systemd service templates.
- Automated benchmark harness (`benchmark run`) with E2E checks and soak latency reporting.
- Long-soak gate command (`benchmark soak`) for stability sign-off with thresholded pass/fail output.
- ONNX telemetry instrumentation (batch latency, failures, queue depth) exposed in benchmark output and `/metrics`.
- CI release pipeline (`.github/workflows/ci.yml`) running fmt/test/release-gate/soak on each PR/push.
- RC preflight helper: `scripts/rc_preflight.sh` and checklist at `docs/release-v1_1_0-rc1.md`.

Stubbed next increments:
- ONNX throughput tuning on larger real-world corpora and model-host optimization.

Implemented async `/map` enrichment:
- Deterministic map base summaries are written at index time.
- After publish, a background enrichment worker computes optional enrichment per directory.
- `/map/.../directory_overview.md` serves base immediately and appends enrichment only when available.
- Manual fallback command: `semanticfs index enrich --version <n>`.

## Quickstart (after Rust toolchain install)
1. `cargo build`
2. `cp config/semanticfs.sample.toml local.toml` and edit `repo_root`/`mount_point`
3. `cargo run -p semanticfs-cli -- --config local.toml index build`
4. `cargo run -p semanticfs-cli -- --config local.toml serve mcp`
5. `cargo run -p semanticfs-cli -- --config local.toml index watch` for continuous reindexing
6. `cargo run -p semanticfs-cli -- --config local.toml serve observability` for `/health/*` and `/metrics`
7. `cargo run --release -p semanticfs-cli -- --config local.toml benchmark run --soak-seconds 30` for accurate E2E + soak report
8. `cargo run --release -p semanticfs-cli -- --config local.toml benchmark tune-lancedb --fixture-repo tests/fixtures/benchmark_repo` for backend tuning passes
9. `cargo run --release -p semanticfs-cli -- --config local.toml benchmark tune-onnx --fixture-repo tests/fixtures/benchmark_repo` for real ONNX throughput sweeps
10. `cargo run --release -p semanticfs-cli -- --config local.toml benchmark release-gate --refresh --fixture-repo tests/fixtures/benchmark_repo` for RC gate evaluation
11. `cargo run --release -p semanticfs-cli -- --config local.toml benchmark soak --duration-seconds 1800 --fixture-repo tests/fixtures/benchmark_repo` for long-soak sign-off

Vector backend selection:
1. Default v1.1 profile is LanceDB when `SEMANTICFS_VECTOR_BACKEND` is unset.
2. `SEMANTICFS_VECTOR_BACKEND=sqlite` to force SQLite vector fallback.
3. `SEMANTICFS_VECTOR_BACKEND=lancedb` to explicitly pin LanceDB mode.
4. Optional LanceDB location override: `SEMANTICFS_LANCEDB_URI=./.semanticfs/lancedb`

Embedding backend selection:
1. `embedding.runtime = "hash"` for deterministic local hash embeddings
2. `embedding.runtime = "onnx"` plus:
   - `SEMANTICFS_ONNX_MODEL=/abs/path/model.onnx`
   - optional `SEMANTICFS_ONNX_TOKENIZER=/abs/path/tokenizer_dir`
   - optional `SEMANTICFS_ONNX_PYTHON=python3`
   - optional `SEMANTICFS_ONNX_SCRIPT=scripts/onnx_embed.py`
   - optional `SEMANTICFS_ONNX_PROVIDER=CPUExecutionProvider`
   - optional `SEMANTICFS_ONNX_MAX_LENGTH=512`
   - optional `SEMANTICFS_ONNX_INTRA_THREADS=0`
   - optional `SEMANTICFS_ONNX_INTER_THREADS=0`
3. Install ONNX sidecar dependencies:
   - `pip install -r scripts/requirements-onnx.txt`
4. Tuned baseline on this repo (release build, `.semanticfs/bench/onnx_tuning.json`):
   - `SEMANTICFS_ONNX_PROVIDER=CPUExecutionProvider`
   - `SEMANTICFS_ONNX_MAX_LENGTH=128`
   - `embedding.batch_size=32`

Windows note:
1. LanceDB build requires `protoc` on PATH.
2. ONNX sidecar avoids Rust/MSVC linking issues by running inference in Python.

Linux-only mount:
1. Ensure FUSE userspace tooling is installed and mountpoint exists.
2. Run `cargo run -p semanticfs-cli -- --config local.toml serve fuse`.
3. On non-Linux targets, `serve fuse` exits with a clear unsupported-platform error.

## Security model
Policy guard is enforced in indexing and retrieval/render pathways and supports:
- deny/allow glob filters
- secret heuristics (regex + entropy)
- retrieval redaction

## Recovery
Use:
- `semanticfs recover mount --force-unmount`
- then remount/restart service using systemd

## Observability endpoints
- `GET /health/live`
- `GET /health/ready`
- `GET /health/index`
- `GET /metrics` (Prometheus format)
  - includes ONNX counters/gauges: requests, batches, texts, failures, health checks, queue depth, latency sum/count/max

## Benchmark harness
- Accuracy note:
  - use `--release` for all benchmark/tuning/gate commands; debug builds are only for functional checks.
- Command:
  - `cargo run --release -p semanticfs-cli -- --config <config> benchmark run --soak-seconds 60`
- Deterministic fixture mode:
  - `cargo run --release -p semanticfs-cli -- --config <config> benchmark run --fixture-repo tests/fixtures/benchmark_repo --soak-seconds 20`
- LanceDB tuning sweep:
  - `cargo run --release -p semanticfs-cli -- --config <config> benchmark tune-lancedb --fixture-repo tests/fixtures/benchmark_repo --soak-seconds 10`
- ONNX throughput sweep:
  - `cargo run --release -p semanticfs-cli -- --config <config> benchmark tune-onnx --fixture-repo tests/fixtures/benchmark_repo --samples 1000 --rounds 5 --batch-sizes 16,32,64 --max-lengths 128,256,384,512`
- Release gate (fails non-zero on threshold breach):
  - `cargo run --release -p semanticfs-cli -- --config <config> benchmark release-gate --refresh --fixture-repo tests/fixtures/benchmark_repo --max-query-p95-ms 250 --max-soak-p95-ms 250 --max-rss-mb 2048`
- Long soak gate (fails non-zero on threshold breach):
  - `cargo run --release -p semanticfs-cli -- --config <config> benchmark soak --duration-seconds 1800 --fixture-repo tests/fixtures/benchmark_repo --max-soak-p95-ms 250 --max-errors 0 --max-rss-mb 2048`
- Output report:
  - `.semanticfs/bench/latest.json` with E2E pass/fail, soak P50/P95/max, error count, and RSS.
  - ONNX section includes requests/batches/text volume, failures, queue depth, and latency counters.
  - LanceDB sweep report: `.semanticfs/bench/lancedb_tuning.json` with per-pass query P50/P95/max + soak + RSS.
  - ONNX sweep report: `.semanticfs/bench/onnx_tuning.json` with per-pass throughput and sidecar telemetry.
  - Release gate report: `.semanticfs/bench/release_gate.json` with per-check pass/fail and thresholds.
  - Soak gate report: `.semanticfs/bench/soak_latest.json` with pass/fail checks and thresholds.
