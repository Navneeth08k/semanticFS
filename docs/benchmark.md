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
4. Small-dataset behavior:
- LanceDB ANN index build is skipped under `65_536` rows to reduce non-actionable KMeans empty-cluster warning noise on fixture-scale runs.

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
2. Optional relevance enforcement:
- defaults are now strict for representative suites: `min_relevance_queries=20`, `min_recall_at_5=0.90`, `min_symbol_hit_rate=0.99`, `min_mrr=0.80`
- add explicit flags to override when needed
- requires `.semanticfs/bench/relevance_latest.json` to exist
3. Checks:
- latest benchmark E2E pass
- latest soak error count + p95 threshold
- latest RSS threshold
- tuning report presence + backend coverage
- tuning query/soak errors
- worst-case tuning query/soak p95 thresholds
- optional relevance thresholds (when enabled)
4. Exits non-zero on failure (CI friendly).

## Relevance metrics (golden queries)
1. `cargo run --release -p semanticfs-cli -- --config config/local.toml benchmark relevance --fixture-repo tests/fixtures/benchmark_repo --golden tests/retrieval_golden/benchmark_repo.json`
2. Multi-suite mode:
- `cargo run --release -p semanticfs-cli -- --config config/local.toml benchmark relevance --fixture-repo /abs/repo --golden-dir tests/retrieval_golden`
3. Emits:
- `recall_at_5`
- `mrr`
- `symbol_hit_rate`
- per-query retrieved top-5 paths and first relevant rank

## Head-to-head validation (SemanticFS vs baseline)
1. Run:
- `cargo run --release -p semanticfs-cli -- --config config/local.toml benchmark head-to-head --fixture-repo /abs/repo --golden-dir tests/retrieval_golden --history`
2. Baseline:
- `rg` fixed-string search (`-F`) over the same repo and same query suite.
3. Emits:
- per-engine `recall_at_topn`, `mrr`, `symbol_hit_rate`
- per-engine latency `p50/p95/max`
- delta block (`semanticfs - baseline`) for quick concept validation
4. Output:
- `.semanticfs/bench/head_to_head_latest.json`

## Retrieval prior knobs (anti-shadowing)
1. `retrieval.code_path_boost`
2. `retrieval.docs_path_penalty`
3. `retrieval.test_path_penalty`
4. `retrieval.recency_half_life_hours`
5. `retrieval.recency_min_boost` / `retrieval.recency_max_boost`
6. Built-in generated-artifact suppression is applied in retrieval prior scoring for paths such as `.next`, `.nuxt`, `.svelte-kit`, `.turbo`, `dist`, `build`, `out`, and `coverage`.
7. Tune these against golden suites to reduce verbose-doc/build-artifact shadowing.

## History artifacts (nightly trendline)
1. Add `--history` to `benchmark run`, `benchmark tune-lancedb`, `benchmark tune-onnx`, `benchmark soak`, or `benchmark relevance`.
2. Latest artifact is still written to `.semanticfs/bench/*.json`.
3. Timestamped snapshots are additionally written under `.semanticfs/bench/history/`.
4. Nightly helper script (Windows):
- `powershell -ExecutionPolicy Bypass -File scripts/nightly_bench.ps1 -ConfigPath config/semanticfs.sample.toml -FixtureRepo tests/fixtures/benchmark_repo -GoldenDir tests/retrieval_golden -SoakSeconds 30`
5. Representative nightly helper (Windows, real suites + strict gate):
- `powershell -ExecutionPolicy Bypass -File scripts/nightly_representative.ps1 -SoakSeconds 30`
6. Drift summary helper (counts + last-N deltas + date coverage):
- `powershell -ExecutionPolicy Bypass -File scripts/drift_summary.ps1`
- Output: `.semanticfs/bench/drift_summary_latest.json`

## Additional repo bootstrap suites
1. Discover larger local git repos for system-scope exploratory coverage:
- `powershell -ExecutionPolicy Bypass -File scripts/discover_repo_candidates.ps1 -Roots C:\Users\<user> -MinTrackedFiles 500 -TopN 30`
- Output: `.semanticfs/bench/filesystem_repo_candidates_latest.json` (or custom `-OutputPath`).
2. Discovery defaults now reduce duplicate noise:
- excludes VS Code Java workspace mirror repos by default (`AppData\Roaming\Code\User\workspaceStorage\...`).
- dedupes mirrored clones by `remote.origin.url` identity by default.
- disable either behavior explicitly when needed:
  - include workspace mirrors: `-IncludeWorkspaceMirrors`
  - disable remote dedupe: `-DisableRemoteDedupe`
3. Build prioritized filesystem-scope backlog from discovery + strict holdout artifacts:
- `powershell -ExecutionPolicy Bypass -File scripts/build_filesystem_scope_backlog.ps1 -CandidatesPath .semanticfs/bench/filesystem_repo_candidates_latest.json -OutputPath .semanticfs/bench/filesystem_scope_backlog_latest.json`
- Output includes `uncovered`, `covered_gap`, `covered_partial`, and `covered_ok` buckets with per-repo next actions.
4. Generate a bootstrap golden suite from a local repo:
- `python scripts/bootstrap_golden_from_repo.py --repo-root C:\path\repo --output tests/retrieval_golden/repo_bootstrap.json --dataset-name repo_bootstrap_v1 --max-queries 20`
- bootstrap generator now skips common generated directories (for example `.next`, `.nuxt`, `.svelte-kit`, `.turbo`, `.cache`, `coverage`, `out`) so suites stay source-focused.
5. Run exploratory head-to-head on that repo:
- `cargo run --release -p semanticfs-cli -- --config config/relevance-real.toml benchmark head-to-head --fixture-repo C:\path\repo --golden tests/retrieval_golden/repo_bootstrap.json --history`
6. Bootstrap suites are exploratory only; curate/lock queries before using them as release evidence.
7. Expanded bootstrap seed for curated suites:
- `python scripts/bootstrap_golden_from_repo.py --repo-root C:\path\repo --output tests/retrieval_golden/repo_bootstrap_v2_full.json --dataset-name repo_bootstrap_v2_full --max-queries 120`

## Tune vs holdout protocol (strict)
1. Split exploratory suite into deterministic tune/holdout:
- `python scripts/split_golden_suite.py --input tests/retrieval_golden/repo_bootstrap.json --tune-output tests/retrieval_golden/repo_tune.json --holdout-output tests/retrieval_golden/repo_holdout.json --tune-count 10`
2. Curate mixed acceptance-grade splits from expanded bootstrap:
- `python scripts/build_curated_mixed_suites.py --input tests/retrieval_golden/repo_bootstrap_v2_full.json --tune-output tests/retrieval_golden/repo_curated_tune.json --holdout-output tests/retrieval_golden/repo_curated_holdout.json --split-size 40 --non-symbol-per-split 10 --dataset-prefix repo`
3. Tune only on `*_tune.json`; do not read holdout metrics until selection is complete.
4. Run selection + one-shot holdout report:
- `powershell -ExecutionPolicy Bypass -File scripts/daytime_tune_holdout.ps1 -Label repo -RepoRoot C:\path\repo -BaseConfig config/relevance-real.toml -TuneGolden tests/retrieval_golden/repo_tune.json -HoldoutGolden tests/retrieval_golden/repo_holdout.json -History`
5. `scripts/daytime_tune_holdout.ps1` now always rebuilds `semanticfs-cli` in `--release` at start to prevent stale binary benchmark artifacts.
6. Optional targeted candidate sweep (for long-running external repos):
- `powershell -ExecutionPolicy Bypass -File scripts/daytime_tune_holdout.ps1 -Label repo -RepoRoot C:\path\repo -BaseConfig config/relevance-real.toml -TuneGolden tests/retrieval_golden/repo_tune.json -HoldoutGolden tests/retrieval_golden/repo_holdout.json -History -CandidateIds latency_guard,symbol_latency_guard`
7. Artifacts:
- `.semanticfs/bench/tune_holdout_<label>_latest.json`
- `.semanticfs/bench/history/tune_holdout_<label>_*.json`

## Daytime smoke (no overnight run needed)
1. Run both real-repo relevance suites (lightweight default):
- `powershell -ExecutionPolicy Bypass -File scripts/daytime_smoke.ps1 -SoakSeconds 2`
2. Optional custom repo paths:
- `powershell -ExecutionPolicy Bypass -File scripts/daytime_smoke.ps1 -SemanticFsRepo C:\path\semanticFS -AiTestgenRepo C:\path\ai-testgen -SoakSeconds 2`
3. Include heavier release gate explicitly:
- `powershell -ExecutionPolicy Bypass -File scripts/daytime_smoke.ps1 -IncludeReleaseGate -SoakSeconds 2`

## Full daytime action runner
1. Run expanded bootstrap + curated splits + smoke + tune/holdout sweeps + drift summary:
- `powershell -ExecutionPolicy Bypass -File scripts/daytime_action_items.ps1 -SoakSeconds 2`
2. Include strict release gate in the smoke step:
- `powershell -ExecutionPolicy Bypass -File scripts/daytime_action_items.ps1 -SoakSeconds 2 -IncludeReleaseGate`
3. Optional filesystem candidate discovery during daytime run:
- `powershell -ExecutionPolicy Bypass -File scripts/daytime_action_items.ps1 -SoakSeconds 2 -DiscoveryRoots C:\Users\<user> -DiscoveryMinTrackedFiles 500 -DiscoveryTopN 30`
4. By default, daytime action runner now builds `.semanticfs/bench/filesystem_scope_backlog_latest.json` after discovery. Skip with:
- `-SkipFilesystemBacklog`

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
6. long-run memory safety: latency percentiles use bounded sampling during soak

## Output artifact
1. `.semanticfs/bench/latest.json`
2. `.semanticfs/bench/lancedb_tuning.json`
3. `.semanticfs/bench/release_gate.json`
4. `.semanticfs/bench/soak_latest.json`
5. `.semanticfs/bench/onnx_tuning.json`
6. `.semanticfs/bench/relevance_latest.json`
7. `.semanticfs/bench/head_to_head_latest.json`
8. `.semanticfs/bench/drift_summary_latest.json`
9. `.semanticfs/bench/tune_holdout_<label>_latest.json`
10. `.semanticfs/bench/filesystem_scope_backlog_latest.json`
