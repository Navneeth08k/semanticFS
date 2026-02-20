# Implemented v1.1 Scaffold

Roadmap reference:
- `docs/big-picture-roadmap.md`

## Implemented
- Workspace and crate decomposition aligned with v1.1 spec.
- Config schema and sample config.
- SQLite schema with two-phase index version publish.
- File filtering and policy guard enforcement.
- File-type routing and chunking strategies.
- Global symbol extraction table.
- Symbol-first retrieval planner + BM25 + plain RRF.
- Vector retrieval pipeline with persisted embeddings and cosine similarity.
- Optional LanceDB vector backend sync + nearest-neighbor query path.
- Optional ONNX embedding backend via persistent Python sidecar (tokenizer + model inference), with startup ping health-check and batched request support.
- ONNX telemetry instrumentation (latency, failures, queue depth, request volume) surfaced in benchmark report and observability metrics.
- ONNX throughput tuning sweep command (`benchmark tune-onnx`) with real sidecar runs and per-pass throughput telemetry artifacts.
- Release-mode ONNX baseline captured on local corpus (`.semanticfs/bench/onnx_tuning.json`) and locked into `config/onnx-test.toml` (`batch_size=16`, `SEMANTICFS_ONNX_MAX_LENGTH=128`, CPU provider).
- Async `/map` enrichment worker triggered post-publish (non-blocking read path, version-scoped cache).
- Virtual path router for `/raw`, `/search`, `/map`, `/.well-known/health.json`.
- Inode/content LRU cache structure.
- MCP minimal surface:
  - tools: `search_codebase`, `get_directory_map`
  - resources: `health`, `search/<query>`, `map/<path>`
  - prompt: `semanticfs_search_then_raw_verify`
- Automated benchmark harness with E2E checks and soak-mode latency reporting.
- LanceDB tuning sweep harness with fixed-corpus sqlite vs lancedb passes and per-pass P50/P95/RSS output.
- Automated release-gate harness evaluating benchmark/tuning artifacts with fail-fast threshold checks for RC readiness.
- Long-soak gate command and artifact (`benchmark soak` -> `soak_latest.json`) for pre-RC stability sign-off.
- Long-soak stability fixes applied:
  - bounded soak latency sampling to prevent OOM on long runs
  - ONNX sidecar input normalization and per-item fallback for malformed batch entries
- systemd units and operational runbook.
- Continuous `index watch` daemon loop with notify events.
- Linux-target mount adapter entrypoint (with Linux-only guard on non-Linux hosts).
- CI workflow executing fmt/test/release-gate/soak and uploading benchmark artifacts.
- RC checklist and preflight script for `v1.1.0-rc1` cut.

## Remaining for full production parity
- Production-grade LanceDB schema/index tuning and large-scale performance profiling.
- Broader ONNX profiling matrix on larger external repos and alternate providers/hardware (AVX512/GPU) for environment-specific default packs.

