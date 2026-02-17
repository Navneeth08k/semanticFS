# Benchmark Harness

## Goals
- Provide a repeatable baseline before ONNX/LanceDB tuning.
- Validate key E2E behavior in one command.
- Emit measurable soak latency and error signals.
- Use optimized binaries for truthful performance numbers.

## Build profile
1. Use `cargo run --release -p semanticfs-cli -- ...` for benchmark/tuning/gates.
2. Debug profile (`cargo run`) is for functional validation only.

## Command
1. `cargo run --release -p semanticfs-cli -- --config config/local.toml benchmark run --soak-seconds 60`

## Fixture corpus mode
1. `cargo run --release -p semanticfs-cli -- --config config/local.toml benchmark run --fixture-repo tests/fixtures/benchmark_repo --soak-seconds 20`

## LanceDB tuning sweep
1. `cargo run --release -p semanticfs-cli -- --config config/local.toml benchmark tune-lancedb --fixture-repo tests/fixtures/benchmark_repo --soak-seconds 10`
2. Runs fixed passes for:
- backend: `sqlite`, `lancedb`
- `retrieval.topn_vector`: `10`, `20`, `40`
3. Emits per-pass:
- query-bench P50/P95/max
- soak P50/P95/max + errors
- RSS
- ONNX counters snapshot

## ONNX throughput sweep
1. `cargo run --release -p semanticfs-cli -- --config config/local.toml benchmark tune-onnx --fixture-repo tests/fixtures/benchmark_repo --samples 1000 --rounds 5 --batch-sizes 16,32,64 --max-lengths 128,256,384,512`
2. Requires ONNX env to be configured:
- `SEMANTICFS_ONNX_MODEL`
- `SEMANTICFS_ONNX_TOKENIZER` (or colocated tokenizer next to model)
3. Emits per-pass:
- provider, max_length, batch_size
- texts/sec throughput
- sidecar telemetry: requests, failures, latency, queue depth

## Long soak gate
1. `cargo run --release -p semanticfs-cli -- --config config/local.toml benchmark soak --duration-seconds 1800 --fixture-repo tests/fixtures/benchmark_repo --max-soak-p95-ms 250 --max-errors 0 --max-rss-mb 2048`
2. Use this as the pre-RC stability sign-off command.
3. Exits non-zero on threshold breach.

## Release gate
1. `cargo run --release -p semanticfs-cli -- --config config/local.toml benchmark release-gate --refresh --fixture-repo tests/fixtures/benchmark_repo`
2. Checks:
- latest benchmark E2E pass
- latest soak error count + p95 threshold
- latest RSS threshold
- tuning report presence + backend coverage
- tuning query/soak errors
- worst-case tuning query/soak p95 thresholds
3. Exits non-zero on failure (CI friendly).

## What it checks
1. Search markdown path renders.
2. Map overview renders.
3. Grounded path from search can be read via `/raw`.
4. Health virtual file renders.

## Soak metrics emitted
1. operation count
2. error count
3. latency P50/P95/max
4. process RSS
5. ONNX telemetry: requests/batches/texts, failures, queue depth current/max, latency sum/count/max

## Output artifact
1. `.semanticfs/bench/latest.json`
2. `.semanticfs/bench/lancedb_tuning.json`
3. `.semanticfs/bench/release_gate.json`
4. `.semanticfs/bench/soak_latest.json`
5. `.semanticfs/bench/onnx_tuning.json`
