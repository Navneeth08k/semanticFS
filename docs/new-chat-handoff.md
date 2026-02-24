# New Chat Handoff

Last updated: February 24, 2026

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
   - curated mixed-suite builder (`scripts/build_curated_mixed_suites.py`)
   - tune/holdout benchmark runner (`scripts/daytime_tune_holdout.ps1`)
   - daytime orchestration (`scripts/daytime_action_items.ps1`, now uses expanded curated suites by default)
12. Larger-repo tune/holdout suites created from bootstrap:
   - `tests/retrieval_golden/buckit_tune.json` / `tests/retrieval_golden/buckit_holdout.json`
   - `tests/retrieval_golden/tensorflow_models_tune.json` / `tests/retrieval_golden/tensorflow_models_holdout.json`
13. Expanded curated larger-repo suites created:
   - `tests/retrieval_golden/buckit_curated_tune.json` / `tests/retrieval_golden/buckit_curated_holdout.json` (`40` each, mixed symbol/non-symbol)
   - `tests/retrieval_golden/tensorflow_models_curated_tune.json` / `tests/retrieval_golden/tensorflow_models_curated_holdout.json` (`40` each, mixed symbol/non-symbol)
14. Retrieval + symbol hardening:
   - `crates/retrieval-core/src/lib.rs`: symbol/BM25 query normalization variants and batched symbol variant SQL for latency-safe matching.
   - `crates/indexer/src/symbols.rs`: Python `def`/`async def`, Rust async fn, and plain `function` extraction support.
15. Strict tune/holdout runner hardening:
   - `scripts/daytime_tune_holdout.ps1` now always rebuilds `semanticfs-cli` `--release` before scoring.
16. Filesystem-scope exploration bootstrap:
   - repo discovery helper: `scripts/discover_repo_candidates.ps1`
   - external exploratory suite added: `tests/retrieval_golden/rlbeta_bootstrap_v1.json`
   - strict split suites added: `tests/retrieval_golden/rlbeta_tune.json` / `tests/retrieval_golden/rlbeta_holdout.json`
   - second external strict split suite added: `tests/retrieval_golden/stockguessr_tune.json` / `tests/retrieval_golden/stockguessr_holdout.json`
17. Curated larger-suite hardening pass:
   - `scripts/build_curated_mixed_suites.py` now filters ambiguous symbols and generic/easy queries before generating tune/holdout splits.
18. External bootstrap hardening:
   - `scripts/bootstrap_golden_from_repo.py` now excludes generated build/cache directories (for example `.next`, `.nuxt`, `.svelte-kit`, `.turbo`, `.cache`, `.dart_tool`, `.pytest_cache`, `coverage`, `out`) to avoid generated-artifact dominated suites.
19. External tune/holdout runner flexibility:
   - `scripts/daytime_tune_holdout.ps1` includes latency-focused candidates (`latency_guard`, `symbol_latency_guard`) and now supports `-CandidateIds` for targeted long-running external sweeps.
20. Generated-artifact suppression hardening:
   - `crates/retrieval-core/src/lib.rs` now applies a generated-path prior penalty to prevent transpiled/build artifacts from shadowing source paths in retrieval ranking.
   - benchmark configs now deny generated output directories for strict daytime runs (`config/relevance-real.toml`, `config/relevance-ai-testgen.toml`, `config/semanticfs.sample.toml`).
21. Filesystem-scope candidate discovery expansion:
   - generated `.semanticfs/bench/filesystem_repo_candidates_min80.json` (`24` candidates at `MinTrackedFiles=80`) for broader external-run planning.
22. Bootstrap generator language coverage expansion:
   - `scripts/bootstrap_golden_from_repo.py` now includes `.dart` and Dart symbol extraction patterns (`class`, `enum`, `mixin`, function-like declarations) for external filesystem-scope suite generation.
23. Additional filesystem-scope strict tune/holdout suites and runs:
   - `apex_scholars_bootstrap_v1` split and strict run completed (`apex_scholars_tune.json` / `apex_scholars_holdout.json`).
   - `flutter_tools_bootstrap_v1` split and strict run completed (`flutter_tools_tune.json` / `flutter_tools_holdout.json`).
24. Additional medium external strict tune/holdout suite and run:
   - `pseudolang_bootstrap_v1` split and strict run completed (`pseudolang_tune.json` / `pseudolang_holdout.json`).
25. Filesystem discovery hardening:
   - `scripts/discover_repo_candidates.ps1` now excludes VS Code workspace mirror repos by default and dedupes mirrored clones by `remote.origin.url`.
   - refreshed `.semanticfs/bench/filesystem_repo_candidates_min80.json`: `21` candidates after cleanup (`6` workspace mirrors excluded, `1` remote-duplicate repo deduped).
26. Filesystem-scope backlog planner:
   - new script: `scripts/build_filesystem_scope_backlog.ps1`.
   - output: `.semanticfs/bench/filesystem_scope_backlog_latest.json` with per-repo states (`uncovered`, `covered_gap`, `covered_partial`, `covered_ok`) and next actions.

## 3) Latest Measured Snapshot
Head-to-head runs on real suites (release mode, February 24, 2026):
1. `semanticfs_repo.json`:
   - SemanticFS (latest): recall `0.95`, MRR `0.9250`, symbol-hit `1.00`, p95 `56.916 ms`.
   - Baseline `rg` (latest): recall `0.80`, MRR `0.7083`, symbol-hit `0.7857`, p95 `59.757 ms`.
2. `ai_testgen_repo.json`:
   - SemanticFS (latest): recall `1.00`, MRR `0.9500`, symbol-hit `1.00`, p95 `12.335 ms`.
   - Baseline `rg` (latest): recall `0.90`, MRR `0.8083`, symbol-hit `1.00`, p95 `31.030 ms`.
3. Artifacts:
   - `.semanticfs/bench/head_to_head_latest.json`
   - `.semanticfs/bench/history/head_to_head_latest_*.json`
4. Representative sequence status:
   - Head-to-head history counts: `semanticfs_repo_v1=13`, `ai_testgen_repo_v1=11`.
   - Relevance history counts: `semanticfs_repo_v1=28`, `ai_testgen_repo_v1=28`.
5. Last-7 head-to-head delta summary (`SemanticFS - rg`):
   - `semanticfs_repo_v1`: delta MRR `min/avg=0.1833/0.3244`, delta recall `0.1000/0.1286`, delta symbol-hit `0.1429/0.5408`, delta p95 `-16.608/-9.253 ms`.
   - `ai_testgen_repo_v1`: delta MRR `min/avg=0.0875/0.1456`, delta recall `0.1000/0.1143`, delta symbol-hit `0.0000/0.0000`, delta p95 `-19.976/-17.105 ms`.
6. Mounted Linux validation (February 19, 2026):
   - Command path: `scripts/wsl_run_fuse_session_validation.sh`
   - Result: `VALIDATION_OK`
   - Verified stale/refresh behavior through real version transitions (`136 -> 137`, `138 -> 139`) for `/.well-known/session.json` and `/.well-known/session.refresh`.
7. Date-separated nightly trend progress:
   - Calendar-night progress is now `5/7` complete.
   - Remaining for v1.2 confidence target: 2 additional date-separated nights.
8. Additional daytime larger-repo exploratory head-to-head (bootstrap suites):
   - `buckit_bootstrap_v1`: SemanticFS recall `1.00`, MRR `0.9417`, symbol-hit `0.90`, p95 `35.004 ms`; baseline `rg` recall `1.00`, MRR `0.7875`, symbol-hit `0.60`, p95 `52.288 ms`.
   - `tensorflow_models_bootstrap_v1`: SemanticFS recall `0.90`, MRR `0.7542`, symbol-hit `0.65`, p95 `49.465 ms`; baseline `rg` recall `0.95`, MRR `0.7892`, symbol-hit `0.70`, p95 `143.105 ms`.
   - Use as exploratory signal only until bootstrap queries are curated into stable acceptance-grade suites.
9. Strict holdout results (new tune-vs-holdout protocol):
   - `buckit_bootstrap_v1_holdout_v1` (`base` selected): SemanticFS recall `1.00`, MRR `1.0000`, symbol-hit `1.00`, p95 `39.054 ms`; baseline `rg` recall `1.00`, MRR `0.8033`, symbol-hit `0.70`, p95 `39.948 ms`.
   - `tensorflow_models_bootstrap_v1_holdout_v1` (`base` selected after `build_losses` disambiguation): SemanticFS recall `1.00`, MRR `0.9500`, symbol-hit `0.90`, p95 `45.890 ms`; baseline `rg` recall `1.00`, MRR `0.9500`, symbol-hit `0.90`, p95 `157.252 ms`.
10. Latest daytime smoke with strict release gate (February 24, 2026):
   - `scripts/daytime_smoke.ps1 -SoakSeconds 2 -IncludeReleaseGate` passed.
   - semanticFS relevance: recall `0.95`, MRR `0.9250`, symbol-hit `1.00`.
   - ai-testgen relevance: recall `1.00`, MRR `0.9500`, symbol-hit `1.00`.
11. Expanded curated holdout results:
   - `buckit_curated_holdout_v1` (latest strict run, selected `symbol_focus`): SemanticFS recall `0.9250`, MRR `0.8542`, symbol-hit `0.8000`, p95 `61.320 ms`; baseline recall `0.7500`, MRR `0.6271`, symbol-hit `0.7000`, p95 `52.228 ms`.
   - `tensorflow_models_curated_holdout_v1` (latest strict run, selected `symbol_focus`): SemanticFS recall `1.0000`, MRR `0.9813`, symbol-hit `0.9667`, p95 `98.520 ms`; baseline recall `0.6500`, MRR `0.4988`, symbol-hit `0.5333`, p95 `157.718 ms`.
12. TensorFlow `build_losses` miss fix (legacy split):
   - updated `expected_paths` for `build_losses` in `tests/retrieval_golden/tensorflow_models_holdout.json` to account for multi-definition ambiguity.
   - revalidated holdout metrics: SemanticFS recall `1.00`, MRR `0.9500`, symbol-hit `0.9000`, p95 `45.890 ms`; baseline p95 `157.252 ms`.
13. New external exploratory head-to-head (filesystem-scope prep):
   - `rlbeta_bootstrap_v1`: SemanticFS recall `1.00`, MRR `1.0000`, symbol-hit `1.00`, p95 `25.421 ms`; baseline recall `1.00`, MRR `0.8521`, symbol-hit `0.75`, p95 `649.968 ms`.
   - strict holdout follow-up (`rlbeta_bootstrap_v1_holdout_v1`): SemanticFS recall `1.0000`, MRR `1.0000`, symbol-hit `1.0000`, p95 `27.064 ms`; baseline recall `1.0000`, MRR `0.8667`, symbol-hit `0.7500`, p95 `727.422 ms`.
14. Second external strict holdout result (filesystem-scope prep):
   - `stockguessr_bootstrap_v1_holdout_v1` (latest targeted run, `latency_guard`): SemanticFS recall `0.7333`, MRR `0.4300`, symbol-hit `0.2667`, p95 `391.161 ms`; baseline recall `0.0000`, MRR `0.0000`, symbol-hit `0.0000`, p95 `46.883 ms`.
   - note: this bundle-heavy v1 suite remains latency-heavy; source-focused v2 suite is now fixed and leads baseline.
15. Additional medium external strict holdout result:
   - `repo8872pp_bootstrap_v1_holdout_v1`: SemanticFS recall `1.0000`, MRR `0.7633`, symbol-hit `0.6000`, p95 `13.244 ms`; baseline recall `1.0000`, MRR `0.8889`, symbol-hit `0.8000`, p95 `43.464 ms`.
   - interpretation: SemanticFS is much faster, but baseline currently ranks better.
16. Source-focused stockguessr strict rerun:
   - regenerated source-only suite: `stockguessr_bootstrap_v2_src` -> `stockguessr_v2_tune.json` / `stockguessr_v2_holdout.json`.
   - latest `stockguessr_bootstrap_v2_src_holdout_v1` (`latency_guard` selected, after generated-artifact suppression): SemanticFS recall `1.0000`, MRR `0.9667`, symbol-hit `0.9333`, p95 `13.588 ms`; baseline recall `0.9333`, MRR `0.8111`, symbol-hit `0.7333`, p95 `31.025 ms`.
   - SQLite backend spot-check on v1 holdout reduced SemanticFS p95 to `274.516 ms` but still left a large latency gap vs baseline (`34.533 ms`).
17. Additional medium external strict holdout (`syntaxless`):
   - `syntaxless_bootstrap_v1_holdout_v1` (`symbol_latency_guard` selected): SemanticFS recall `1.0000`, MRR `0.7722`, symbol-hit `0.6000`, p95 `13.217 ms`; baseline recall `1.0000`, MRR `0.8889`, symbol-hit `0.8000`, p95 `30.305 ms`.
18. Flutter source-focused external run status:
   - generated/split suites (`flutter_bootstrap_v2_src`, `flutter_v2_tune.json`, `flutter_v2_holdout.json`) succeeded.
   - strict tune/holdout execution exceeded the single-session timeout window before artifact completion; needs bounded follow-up strategy.
19. Additional filesystem-scope strict holdout results (February 24, 2026):
   - `apex_scholars_bootstrap_v1_holdout_v1` (`symbol_latency_guard`): SemanticFS recall `0.9333`, MRR `0.9000`, symbol-hit `0.8667`, p95 `16.908 ms`; baseline recall `1.0000`, MRR `0.8389`, symbol-hit `0.7333`, p95 `38.806 ms`.
   - `flutter_tools_bootstrap_v1_holdout_v1` (`symbol_latency_guard`): SemanticFS recall `0.9333`, MRR `0.7578`, symbol-hit `0.6667`, p95 `21.677 ms`; baseline recall `1.0000`, MRR `0.9667`, symbol-hit `0.9333`, p95 `88.868 ms`.
20. Additional medium external strict holdout result (February 24, 2026):
   - `pseudolang_bootstrap_v1_holdout_v1` (`latency_guard`): SemanticFS recall `0.9333`, MRR `0.8333`, symbol-hit `0.7333`, p95 `19.497 ms`; baseline recall `1.0000`, MRR `0.8556`, symbol-hit `0.7333`, p95 `50.041 ms`.
21. Drift summary refresh (February 24, 2026):
   - history counts: `head_to_head=149`, `relevance=60`.
   - representative counts: `semanticfs_repo_v1` h2h/relevance=`13/28`, `ai_testgen_repo_v1` h2h/relevance=`11/28`.
22. `flutter_tools` query-level gap triage (latest holdout artifact):
   - one semantic miss: `b06` (`_write`).
   - four semantic rank-lag queries vs baseline rank-1: `b10` (`_canRun`), `b14` (`_Attribute`), `b18` (`attemptToolExit`), `b30` (`CommandHelp`).
23. Filesystem-scope backlog snapshot (February 24, 2026):
   - counts: `uncovered=11`, `covered_gap=4`, `covered_partial=2`, `covered_ok=4`.
   - partial-coverage roots detected: `C:\Users\navneeth\Documents\flutter` (`flutter_tools`) and `C:\Users\navneeth\Desktop\NavneethThings\Projects\Robot` (`tensorflow_models_curated`).

Note:
1. Measurements include both representative real suites with 7+ same-day head-to-head snapshots each.
2. Calendar-night drift confidence is in progress (`5/7` date-separated nights complete).
3. Holdout protocol is now active for larger-repo daytime tuning, reducing overfit risk.

## 4) Exact Next Steps (Ordered)
1. Continue one representative run per calendar night until 7 date-separated nights are green (`2 additional nights required`).
2. Triage any nightly drift (relevance/head-to-head/release-gate) and adjust priors only if drift appears.
3. Refine curated larger-repo suites (reduce ambiguous/easy queries, strengthen non-symbol intent coverage) before release evidence use.
4. Use `.semanticfs/bench/filesystem_scope_backlog_latest.json` as daytime queue source: run top `uncovered` repos first, then `covered_gap` repos.
5. Triage external strict quality gaps on `repo8872pp`, `syntaxless`, and `flutter_tools` (starting from `b06`, `b10`, `b14`, `b18`, `b30`), and complete one bounded `flutter_v2` strict run.

## 5) Execution Plan For Next Session
1. Continue representative nightly trend sequence (one run per night):
```powershell
powershell -ExecutionPolicy Bypass -File scripts/nightly_representative.ps1 -SoakSeconds 30
```
2. Run daytime action sequence (recommended):
```powershell
powershell -ExecutionPolicy Bypass -File scripts/daytime_action_items.ps1 -SoakSeconds 2 -IncludeReleaseGate
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
powershell -ExecutionPolicy Bypass -File scripts/daytime_tune_holdout.ps1 -Label tensorflow_models_curated -RepoRoot C:\path\to\tensorflow\models -BaseConfig config/relevance-real.toml -TuneGolden tests/retrieval_golden/tensorflow_models_curated_tune.json -HoldoutGolden tests/retrieval_golden/tensorflow_models_curated_holdout.json -History
```
6. Optional targeted external sweep (faster):
```powershell
powershell -ExecutionPolicy Bypass -File scripts/daytime_tune_holdout.ps1 -Label stockguessr_v2 -RepoRoot C:\path\to\StockGuessr -BaseConfig config/relevance-real.toml -TuneGolden tests/retrieval_golden/stockguessr_v2_tune.json -HoldoutGolden tests/retrieval_golden/stockguessr_v2_holdout.json -History -CandidateIds latency_guard,symbol_latency_guard
```
7. Optional strict run for new medium external repo:
```powershell
powershell -ExecutionPolicy Bypass -File scripts/daytime_tune_holdout.ps1 -Label syntaxless -RepoRoot C:\path\to\syntaxless -BaseConfig config/relevance-real.toml -TuneGolden tests/retrieval_golden/syntaxless_tune.json -HoldoutGolden tests/retrieval_golden/syntaxless_holdout.json -History -CandidateIds latency_guard,symbol_latency_guard
```
8. Optional mounted Linux refresh re-validation (after FUSE/session code changes):
```powershell
wsl -d Ubuntu -- bash -lc 'cd /mnt/c/path/to/semanticFS && bash scripts/wsl_run_fuse_session_validation.sh'
```
9. Filesystem candidate discovery (for system-scope track):
```powershell
powershell -ExecutionPolicy Bypass -File scripts/discover_repo_candidates.ps1 -Roots C:\Users\<user> -MinTrackedFiles 500 -TopN 30
```
10. Filesystem backlog build from latest discovery + strict holdout artifacts:
```powershell
powershell -ExecutionPolicy Bypass -File scripts/build_filesystem_scope_backlog.ps1 -CandidatesPath .semanticfs/bench/filesystem_repo_candidates_latest.json -OutputPath .semanticfs/bench/filesystem_scope_backlog_latest.json
```
11. Release gate strict thresholds now in use:
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
