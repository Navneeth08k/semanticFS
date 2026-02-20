# New Chat Handoff

Last updated: February 20, 2026

This file is the fastest way to restore full working context in a new chat.

## 1) Project Intent
SemanticFS is an intelligence layer for coding agents:
1. Discover relevant code semantically (`/search`).
2. Keep structure/context summarized (`/map`).
3. Verify exact bytes before edits (`/raw`).

Primary principle:
1. Retrieval can be probabilistic.
2. Execution and edits must be grounded and deterministic.

## 2) What Is Implemented
Core platform:
1. Hybrid retrieval (symbol-first + BM25 + vector + RRF).
2. Two-phase index publish with snapshot reads.
3. FUSE virtual path renderer (`/raw`, `/search`, `/map`, health).
4. MCP minimal tool/resource surface.
5. Policy guard on index and retrieval paths.

Recent v1.2 reliability/quality work completed:
1. Search breadcrumb contract (`Source`, `Symbol`, `Hash`, `Snapshot`, `Trust`).
2. MCP session-level snapshot pinning with explicit refresh.
3. Branch-swap queue planning and indexing-in-progress signaling.
4. Anti-shadowing ranking priors (file-type + recency).
5. Head-to-head benchmark harness (`SemanticFS` vs `rg` baseline).
6. FUSE long-lived session pin semantics with explicit refresh/status files:
   - `/.well-known/session.refresh`
   - `/.well-known/session.json`
7. Mounted Linux FUSE workflow validation pass in WSL long-lived session.
8. Drift-triage automation script (`scripts/drift_summary.ps1`) with last-N deltas and date coverage summary.
9. Linux FUSE session regression tests in `linux_mount` (`stale` / `refreshed` / mode payload checks), validated in WSL.
10. LanceDB small-dataset warning reduction by skipping ANN index creation when row count is below `65_536`.
11. Strict daytime tune-vs-holdout workflow:
   - deterministic split tool (`scripts/split_golden_suite.py`)
   - tune/holdout benchmark runner (`scripts/daytime_tune_holdout.ps1`)
   - daytime orchestration (`scripts/daytime_action_items.ps1`)
12. Larger-repo tune/holdout suites created from bootstrap:
   - `tests/retrieval_golden/buckit_tune.json` / `tests/retrieval_golden/buckit_holdout.json`
   - `tests/retrieval_golden/tensorflow_models_tune.json` / `tests/retrieval_golden/tensorflow_models_holdout.json`

## 3) Latest Measured Snapshot
Head-to-head runs on real suites (release mode, February 20, 2026):
1. `semanticfs_repo.json`:
   - SemanticFS (latest): recall `0.90`, MRR `0.8667`, symbol-hit `1.00`, p95 `11.767 ms`.
   - Baseline `rg` (latest): recall `0.80`, MRR `0.3958`, symbol-hit `0.0714`, p95 `28.375 ms`.
2. `ai_testgen_repo.json`:
   - SemanticFS (latest): recall `1.00`, MRR `0.9125`, symbol-hit `1.00`, p95 `10.217 ms`.
   - Baseline `rg` (latest): recall `0.90`, MRR `0.7542`, symbol-hit `1.00`, p95 `26.717 ms`.
3. Artifacts:
   - `.semanticfs/bench/head_to_head_latest.json`
   - `.semanticfs/bench/history/head_to_head_latest_*.json`
4. Representative sequence status:
   - Head-to-head history counts: `semanticfs_repo_v1=9`, `ai_testgen_repo_v1=8`.
   - Relevance history counts: `semanticfs_repo_v1=21`, `ai_testgen_repo_v1=20`.
5. Last-7 head-to-head delta summary (`SemanticFS - rg`):
   - `semanticfs_repo_v1`: delta MRR `min/avg=0.3833/0.4387`, delta recall `0.1000/0.1000`, delta symbol-hit `0.7143/0.8776`, delta p95 `-16.675/-14.038 ms`.
   - `ai_testgen_repo_v1`: delta MRR `min/avg=0.0875/0.1179`, delta recall `0.1000/0.1143`, delta symbol-hit `0.0000/0.0000`, delta p95 `-18.261/-16.094 ms`.
6. Mounted Linux validation (February 19, 2026):
   - Command path: `scripts/wsl_run_fuse_session_validation.sh`
   - Result: `VALIDATION_OK`
   - Verified stale/refresh behavior through real version transitions (`136 -> 137`, `138 -> 139`) for `/.well-known/session.json` and `/.well-known/session.refresh`.
7. Date-separated nightly trend progress:
   - Calendar-night progress is now `2/7` complete.
   - Remaining for v1.2 confidence target: 5 additional date-separated nights.
8. Additional daytime larger-repo exploratory head-to-head (bootstrap suites):
   - `buckit_bootstrap_v1`: SemanticFS recall `1.00`, MRR `0.9417`, symbol-hit `0.90`, p95 `35.004 ms`; baseline `rg` recall `1.00`, MRR `0.7875`, symbol-hit `0.60`, p95 `52.288 ms`.
   - `tensorflow_models_bootstrap_v1`: SemanticFS recall `0.90`, MRR `0.7542`, symbol-hit `0.65`, p95 `49.465 ms`; baseline `rg` recall `0.95`, MRR `0.7892`, symbol-hit `0.70`, p95 `143.105 ms`.
   - Use as exploratory signal only until bootstrap queries are curated into stable acceptance-grade suites.
9. Strict holdout results (new tune-vs-holdout protocol):
   - `buckit_bootstrap_v1_holdout_v1` (`base` selected): SemanticFS recall `1.00`, MRR `1.0000`, symbol-hit `1.00`, p95 `39.054 ms`; baseline `rg` recall `1.00`, MRR `0.8033`, symbol-hit `0.70`, p95 `39.948 ms`.
   - `tensorflow_models_bootstrap_v1_holdout_v1` (`code_focus` selected): SemanticFS recall `0.90`, MRR `0.8500`, symbol-hit `0.80`, p95 `48.205 ms`; baseline `rg` recall `0.90`, MRR `0.8200`, symbol-hit `0.80`, p95 `177.929 ms`.
10. Daytime smoke with strict release gate (February 20, 2026):
   - `scripts/daytime_smoke.ps1 -SoakSeconds 2 -IncludeReleaseGate` passed.
   - semanticFS relevance: recall `0.95`, MRR `0.8917`, symbol-hit `1.00`.
   - ai-testgen relevance: recall `1.00`, MRR `0.9125`, symbol-hit `1.00`.

Note:
1. Measurements include both representative real suites with 7+ same-day head-to-head snapshots each.
2. Calendar-night drift confidence is in progress (`2/7` date-separated nights complete).
3. Holdout protocol is now active for larger-repo daytime tuning, reducing overfit risk.

## 4) Exact Next Steps (Ordered)
1. Continue one representative run per calendar night until 7 date-separated nights are green (`5 additional nights required`).
2. Triage any nightly drift (relevance/head-to-head/release-gate) and adjust priors only if drift appears.
3. Expand larger-repo tune/holdout suites from `10/10` splits to curated acceptance-grade sets (`>=30` per split with mixed symbol and non-symbol queries).
4. Investigate TensorFlow-models holdout miss (`build_losses`) and improve without regressing latency.

## 5) Execution Plan For Next Session
1. Continue representative nightly trend sequence (one run per night):
```powershell
powershell -ExecutionPolicy Bypass -File scripts/nightly_representative.ps1 -SoakSeconds 30
```
2. Run daytime action sequence (recommended):
```powershell
powershell -ExecutionPolicy Bypass -File scripts/daytime_action_items.ps1 -SoakSeconds 2
```
3. Optional manual single-suite recheck (semanticFS):
```bash
cargo run --release -p semanticfs-cli -- --config config/relevance-real.toml benchmark head-to-head --fixture-repo /abs/path/semanticFS --golden tests/retrieval_golden/semanticfs_repo.json --history
```
4. Optional manual single-suite recheck (ai-testgen):
```bash
cargo run --release -p semanticfs-cli -- --config config/relevance-ai-testgen.toml benchmark head-to-head --fixture-repo /abs/path/ai-testgen --golden tests/retrieval_golden/ai_testgen_repo.json --history
```
5. Optional strict tune/holdout run for one larger repo:
```powershell
powershell -ExecutionPolicy Bypass -File scripts/daytime_tune_holdout.ps1 -Label tensorflow_models -RepoRoot C:\path\to\tensorflow\models -BaseConfig config/relevance-real.toml -TuneGolden tests/retrieval_golden/tensorflow_models_tune.json -HoldoutGolden tests/retrieval_golden/tensorflow_models_holdout.json -History
```
6. Optional mounted Linux refresh re-validation (after FUSE/session code changes):
```powershell
wsl -d Ubuntu -- bash -lc 'cd /mnt/c/path/to/semanticFS && bash scripts/wsl_run_fuse_session_validation.sh'
```
7. Release gate strict thresholds now in use:
1. `min_relevance_queries = 20`
2. `min_recall_at_5 = 0.90`
3. `min_symbol_hit_rate = 0.99`
4. `min_mrr = 0.80`

## 6) Definition Of Done For v1.2
1. 7 consecutive calendar-night runs green (`same-day run-count target already met on February 18, 2026`).
2. Stable relevance metrics on representative suites.
3. Head-to-head trend remains favorable or regressions understood/fixed.
4. No P0/P1 reliability/security gaps open.
5. FUSE session stability semantics validated in mounted Linux workflow.

## 7) Command Cheat Sheet
1. Daytime smoke:
```powershell
powershell -ExecutionPolicy Bypass -File scripts/daytime_smoke.ps1 -SoakSeconds 2
```
2. Relevance:
```bash
cargo run --release -p semanticfs-cli -- --config config/local.toml benchmark relevance --fixture-repo /abs/repo --golden-dir tests/retrieval_golden --history
```
3. Head-to-head:
```bash
cargo run --release -p semanticfs-cli -- --config config/local.toml benchmark head-to-head --fixture-repo /abs/repo --golden-dir tests/retrieval_golden --history
```
4. Release gate:
```bash
cargo run --release -p semanticfs-cli -- --config config/local.toml benchmark release-gate --refresh --fixture-repo tests/fixtures/benchmark_repo --enforce-relevance --min-relevance-queries 20 --min-recall-at-5 0.90 --min-symbol-hit-rate 0.99 --min-mrr 0.80
```
5. Mounted Linux FUSE session validation (WSL):
```powershell
wsl -d Ubuntu -- bash -lc 'cd /mnt/c/path/to/semanticFS && bash scripts/wsl_run_fuse_session_validation.sh'
```
6. Drift summary (history counts + deltas + date coverage):
```powershell
powershell -ExecutionPolicy Bypass -File scripts/drift_summary.ps1
```

## 8) Key Files To Read For Context
1. `README.md`
2. `docs/v1_2_execution_plan.md`
3. `docs/future-steps-log.md`
4. `docs/benchmark.md`
5. `crates/indexer/src/lib.rs`
6. `crates/retrieval-core/src/lib.rs`
7. `crates/fuse-bridge/src/lib.rs`
8. `crates/mcp/src/lib.rs`
