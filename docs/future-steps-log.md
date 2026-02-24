# Future Steps Log

Last updated: February 24, 2026

Purpose:
1. Keep future work discussed in chat from being lost.
2. Separate active queue from historical completions.

Status legend:
1. `queued`
2. `active`
3. `done`
4. `deferred`

## Current Queue (Only Open Items)
1. Calendar-night stability trend run and drift triage
   - phase: v1.2
   - status: active
   - source: v1.2 acceptance criteria
   - summary: same-day run-count target is done (current counts: `head-to-head semanticfs_repo_v1=13`, `ai_testgen_repo_v1=11`), date-separated progress is now `5/7` nights complete, continue until 7 nights and analyze relevance/latency/RSS drift.

2. Curated larger-repo validation suites (post-bootstrap)
   - phase: v1.2
   - status: active
   - source: daytime exploratory expansion request
   - summary: suites were expanded to curated `40`-query splits (`30` symbol + `10` non-symbol) for both `buckit` and `tensorflow/models`; ambiguity/easy-query filtering is now landed, and the next step is continued query-quality tightening while preserving strict holdout isolation.

3. Filesystem-scope exploratory coverage expansion
   - phase: v1.2
   - status: active
   - source: filesystem-wide goal alignment
   - summary: discovery tooling is in place and external strict signals now include `rlbeta`, `stockguessr_v1`, `stockguessr_v2`, `repo8872pp`, `syntaxless`, `apex_scholars`, `flutter_tools`, and `pseudolang`; a filesystem backlog artifact now ranks repos by state (`uncovered`, `covered_gap`, `covered_partial`, `covered_ok`); current quality gaps remain on `repo8872pp`, `syntaxless`, and `flutter_tools`; `flutter_v2` needs bounded completion.

## Deferred
1. Per-commit vector snapshots at repository scale
   - phase: v3+
   - status: deferred
   - source: systems-vs-AI discussion

2. Full multimodal retrieval default (code + design/image)
   - phase: v3+
   - status: deferred
   - source: systems-vs-AI discussion

## Recently Completed
1. Relevance threshold support in `release-gate` (optional enforcement mode).
2. Multi-suite relevance evaluation (`--golden-dir`) and history snapshots.
3. Nightly benchmark automation scaffold (`scripts/nightly_bench.ps1`).
4. Two real-repo golden suites (`semanticfs_repo.json`, `ai_testgen_repo.json`).
5. Daytime smoke command (`scripts/daytime_smoke.ps1`).
6. Search breadcrumb contract in `/search` output.
7. MCP session-level snapshot pinning + refresh control.
8. Branch-swap queue planning + indexing-in-progress signaling.
9. Anti-shadowing ranking priors (file-type + recency).
10. Head-to-head benchmark harness (`benchmark head-to-head` vs `rg`).
11. Release-gate relevance threshold hardening for representative suites (`20 / 0.90 / 0.99 / 0.80`).
12. FUSE long-lived session pin semantics with explicit refresh/status control files.
13. Representative nightly orchestration script (`scripts/nightly_representative.ps1`).
14. Accelerated same-day representative sequence completed with strict release-gate passing and 7+ head-to-head snapshots per target dataset.
15. Mounted Linux FUSE workflow validation completed in WSL long-lived session (`/.well-known/session.json` + `/.well-known/session.refresh`, `VALIDATION_OK`).
16. Daytime smoke rerun completed after Linux FUSE fixes (`scripts/daytime_smoke.ps1 -SoakSeconds 2`, both representative relevance suites green).
17. Date-separated representative nightly run completed (February 19, 2026) with strict release-gate passing and no drift trigger (`scripts/nightly_representative.ps1 -SoakSeconds 30`).
18. Mounted Linux FUSE session validation rerun completed after nightly execution (`scripts/wsl_run_fuse_session_validation.sh`, `VALIDATION_OK`, `138 -> 139` refresh transition verified).
19. Drift-triage automation script added (`scripts/drift_summary.ps1`) with date coverage, history counts, and last-N delta summaries (`.semanticfs/bench/drift_summary_latest.json`).
20. Linux FUSE session status regression tests added and validated in WSL (`cargo test -p fuse-bridge`: 7 passed, including new `linux_mount` session tests).
21. LanceDB small-dataset warning reduction landed by skipping ANN index creation under `65_536` rows; short `tune-lancedb` rerun showed KMeans empty-cluster warning spam no longer appearing.
22. Daytime expansion head-to-head executed on additional local repos using bootstrap golden generation:
   - `buckit_bootstrap_v1`: SemanticFS outperformed baseline on MRR/symbol-hit and latency while matching recall.
   - `tensorflow_models_bootstrap_v1`: baseline slightly ahead on recall/MRR/symbol-hit; SemanticFS significantly faster on p95 latency.
23. Strict tune-vs-holdout protocol implemented for daytime tuning:
   - `scripts/split_golden_suite.py`
   - `scripts/daytime_tune_holdout.ps1`
   - `scripts/daytime_action_items.ps1`
24. Larger-repo bootstrap suites split and locked:
   - `buckit_tune.json` / `buckit_holdout.json`
   - `tensorflow_models_tune.json` / `tensorflow_models_holdout.json`
25. Daytime tune-vs-holdout runs completed:
   - `buckit` holdout (`base` selected): SemanticFS recall `1.00`, MRR `1.0000`, symbol-hit `1.00`, p95 `39.054 ms`; baseline MRR `0.8033`, symbol-hit `0.70`, p95 `39.948 ms`.
   - `tensorflow/models` holdout (`code_focus` selected): SemanticFS recall `0.90`, MRR `0.8500`, symbol-hit `0.80`, p95 `48.205 ms`; baseline recall `0.90`, MRR `0.8200`, symbol-hit `0.80`, p95 `177.929 ms`.
26. Daytime smoke rerun with strict release gate succeeded (`scripts/daytime_smoke.ps1 -SoakSeconds 2 -IncludeReleaseGate`):
   - semanticFS relevance remained above threshold (recall `0.95`, MRR `0.8917`, symbol-hit `1.00`).
   - ai-testgen relevance remained stable (recall `1.00`, MRR `0.9125`, symbol-hit `1.00`).
   - release-gate checks passed (`.semanticfs/bench/release_gate.json`).
27. Expanded curated suite generation landed and executed end-to-end:
   - generated larger bootstrap inputs (`buckit_bootstrap_v2_full.json`, `tensorflow_models_bootstrap_v2_full.json`, `120` queries each).
   - added `scripts/build_curated_mixed_suites.py` to produce deterministic mixed splits (`40` queries/split with `30` symbol + `10` non-symbol).
   - updated `scripts/daytime_action_items.ps1` to use expanded curated workflow by default.
28. Curated tune/holdout benchmark results recorded:
   - `buckit_curated_holdout_v1`: SemanticFS recall `0.8250`, MRR `0.7458`, symbol-hit `0.8667`, p95 `77.307 ms`; baseline recall `0.7750`, MRR `0.6229`, symbol-hit `0.6333`, p95 `80.605 ms`.
   - `tensorflow_models_curated_holdout_v1`: SemanticFS recall `0.8000`, MRR `0.4758`, symbol-hit `0.3333`, p95 `42.826 ms`; baseline recall `0.6500`, MRR `0.5217`, symbol-hit `0.5667`, p95 `146.918 ms`.
29. TensorFlow `build_losses` holdout miss resolved in legacy split:
   - updated expected paths in `tests/retrieval_golden/tensorflow_models_holdout.json` to disambiguate multi-definition symbol.
   - revalidated with `scripts/daytime_tune_holdout.ps1` on legacy split: SemanticFS holdout recall `1.00`, MRR `0.9500`, symbol-hit `0.9000`, p95 `45.890 ms` (no latency regression).
30. Representative nightly run completed on February 21, 2026 (`scripts/nightly_representative.ps1 -SoakSeconds 30`):
   - relevance/head-to-head/release-gate all passed.
   - calendar-night trend progress moved to `3/7` complete (`4` nights remaining).
31. Retrieval + symbol hardening landed and validated:
   - `crates/retrieval-core/src/lib.rs`: symbol/BM25 query normalization variants + batched symbol variant SQL (`IN`/`LIKE OR`) to recover latency.
   - `crates/indexer/src/symbols.rs`: added parsing for Python `def`/`async def`, Rust async fn, and plain `function`.
   - unit tests added and passing for both crates.
32. Tune/holdout runner hardening landed:
   - `scripts/daytime_tune_holdout.ps1` now always rebuilds `semanticfs-cli` in `--release` before benchmark scoring (prevents stale-binary artifacts).
33. Curated strict holdout reruns completed after hardening:
   - `tensorflow_models_curated_holdout_v1` (selected `base`): SemanticFS recall `1.0000`, MRR `0.9208`, symbol-hit `0.8333`, p95 `102.798 ms`; baseline recall `0.6750`, MRR `0.5342`, symbol-hit `0.5667`, p95 `150.398 ms`.
   - `buckit_curated_holdout_v1` (selected `code_focus`): SemanticFS recall `0.9750`, MRR `0.8958`, symbol-hit `0.8667`, p95 `38.277 ms`; baseline recall `0.7750`, MRR `0.6458`, symbol-hit `0.7000`, p95 `44.555 ms`.
34. Filesystem-scope prep track execution started:
   - new discovery script: `scripts/discover_repo_candidates.ps1`
   - discovery artifacts produced: `.semanticfs/bench/filesystem_repo_candidates_latest.json`, `.semanticfs/bench/filesystem_repo_candidates_userroot.json`
   - new exploratory external dataset: `tests/retrieval_golden/rlbeta_bootstrap_v1.json`
   - exploratory head-to-head result: SemanticFS recall `1.00`, MRR `1.0000`, symbol-hit `1.00`, p95 `25.421 ms`; baseline p95 `649.968 ms`.
   - strict tune/holdout result (`scripts/daytime_tune_holdout.ps1 -Label rlbeta`): holdout SemanticFS recall `1.0000`, MRR `1.0000`, symbol-hit `1.0000`, p95 `27.064 ms`; baseline MRR `0.8667`, p95 `727.422 ms`.
35. Daytime representative smoke rerun completed after retrieval/indexer changes:
   - `scripts/daytime_smoke.ps1 -SoakSeconds 2` passed.
   - semanticFS relevance remains strong (recall `0.95`, MRR `0.9250`, symbol-hit `1.00`).
   - ai-testgen relevance remains strong (recall `1.00`, MRR `0.9500`, symbol-hit `1.00`).
36. Curated mixed-suite hardening pass completed:
   - updated `scripts/build_curated_mixed_suites.py` to filter ambiguous symbols and generic/easy queries before split generation.
   - regenerated `buckit_curated_*` and `tensorflow_models_curated_*` strict tune/holdout suites with isolation preserved.
37. Curated strict holdout reruns completed after hardening:
   - `buckit_curated_holdout_v1` (`symbol_focus` selected): SemanticFS recall `0.9250`, MRR `0.8542`, symbol-hit `0.8000`, p95 `61.320 ms`; baseline recall `0.7500`, MRR `0.6271`, symbol-hit `0.7000`, p95 `52.228 ms`.
   - `tensorflow_models_curated_holdout_v1` (`symbol_focus` selected): SemanticFS recall `1.0000`, MRR `0.9813`, symbol-hit `0.9667`, p95 `98.520 ms`; baseline recall `0.6500`, MRR `0.4988`, symbol-hit `0.5333`, p95 `157.718 ms`.
38. Second external strict tune/holdout dataset added:
   - `stockguessr_bootstrap_v1` split into `stockguessr_tune.json` / `stockguessr_holdout.json`.
   - holdout result (`stockguessr_bootstrap_v1_holdout_v1`): SemanticFS recall `0.7333`, MRR `0.4300`, symbol-hit `0.2667`, p95 `376.317 ms`; baseline recall `0.0000`, MRR `0.0000`, symbol-hit `0.0000`, p95 `64.592 ms`.
   - attempted `flutter` strict sweep exceeded a single-session window (timed out), so `stockguessr` was used for the second external strict track.
39. Representative nightly run completed on February 22, 2026 (`scripts/nightly_representative.ps1 -SoakSeconds 30`):
   - relevance/head-to-head/release-gate all passed.
   - calendar-night trend progress moved to `4/7` complete (`3` nights remaining).
40. Drift summary refreshed after nightly + daytime strict runs (`scripts/drift_summary.ps1`):
   - history counts: `head_to_head=108`, `relevance=55`.
   - representative counts: `semanticfs_repo_v1` h2h/relevance=`12/26`, `ai_testgen_repo_v1` h2h/relevance=`10/25`.
   - last-7 delta averages: `semanticfs_repo_v1` MRR `0.3482`, recall `0.1214`, symbol-hit `0.6122`, p95 `-11.123 ms`; `ai_testgen_repo_v1` MRR `0.1379`, recall `0.1143`, symbol-hit `0.0000`, p95 `-16.742 ms`.
41. External strict tune/holdout runner expanded for daytime iteration speed:
   - `scripts/daytime_tune_holdout.ps1` added latency-focused candidates (`latency_guard`, `symbol_latency_guard`).
   - `scripts/daytime_tune_holdout.ps1` now supports `-CandidateIds` to run targeted candidate subsets on long external sweeps.
42. Medium external strict tune/holdout run completed (`repo8872pp`):
   - generated/split suites: `repo8872pp_bootstrap_v1.json`, `repo8872pp_tune.json`, `repo8872pp_holdout.json`.
   - holdout result (`repo8872pp_bootstrap_v1_holdout_v1`): SemanticFS recall `1.0000`, MRR `0.7633`, symbol-hit `0.6000`, p95 `13.244 ms`; baseline recall `1.0000`, MRR `0.8889`, symbol-hit `0.8000`, p95 `43.464 ms`.
43. External bootstrap generation hardened for source-focused suites:
   - `scripts/bootstrap_golden_from_repo.py` now excludes generated build/cache directories (for example `.next`, `.nuxt`, `.svelte-kit`, `.turbo`, `.cache`, `.dart_tool`, `.pytest_cache`, `coverage`, `out`).
44. Stockguessr source-focused strict rerun completed:
   - generated/split suites: `stockguessr_bootstrap_v2_src.json`, `stockguessr_v2_tune.json`, `stockguessr_v2_holdout.json`.
   - holdout result (`stockguessr_bootstrap_v2_src_holdout_v1`, selected `latency_guard`): SemanticFS recall `0.6000`, MRR `0.4800`, symbol-hit `0.4000`, p95 `190.883 ms`; baseline recall `0.9333`, MRR `0.8111`, symbol-hit `0.7333`, p95 `27.636 ms`.
45. Additional stockguessr backend check completed:
   - SQLite vector backend spot-check on `stockguessr_bootstrap_v1_holdout_v1` lowered SemanticFS p95 to `274.516 ms` but still remained far above baseline (`34.533 ms`), so backend alone does not close the latency gap.
46. Drift summary refreshed after expanded daytime external runs (`scripts/drift_summary.ps1`):
   - history counts: `head_to_head=131`, `relevance=55`.
   - representative counts unchanged: `semanticfs_repo_v1` h2h/relevance=`12/26`, `ai_testgen_repo_v1` h2h/relevance=`10/25`.
47. Targeted stockguessr v1 rerun completed with latency-focused candidate subset:
   - command used `-CandidateIds latency_guard,symbol_latency_guard`.
   - selected `latency_guard`; holdout SemanticFS recall `0.7333`, MRR `0.4300`, symbol-hit `0.2667`, p95 `391.161 ms`; baseline p95 `46.883 ms`.
48. Generated-artifact suppression hardening landed for external source fidelity:
   - `crates/retrieval-core/src/lib.rs` now applies generated-artifact path prior penalty (`.next`, `.nuxt`, `.svelte-kit`, `.turbo`, `.dart_tool`, `dist`, `build`, `out`, `coverage`, `target`, `*.min.js`).
   - benchmark configs updated with matching deny globs: `config/relevance-real.toml`, `config/relevance-ai-testgen.toml`, `config/semanticfs.sample.toml`.
49. Stockguessr source-focused strict rerun after suppression:
   - `stockguessr_bootstrap_v2_src_holdout_v1` (`latency_guard`): SemanticFS recall `1.0000`, MRR `0.9667`, symbol-hit `0.9333`, p95 `13.588 ms`; baseline recall `0.9333`, MRR `0.8111`, symbol-hit `0.7333`, p95 `31.025 ms`.
50. Filesystem candidate discovery expanded:
   - generated `.semanticfs/bench/filesystem_repo_candidates_min80.json` with `24` candidates (`MinTrackedFiles=80`, root `C:\Users\navneeth`).
51. New medium external strict tune/holdout run completed (`syntaxless`):
   - generated/split suites: `syntaxless_bootstrap_v1.json`, `syntaxless_tune.json`, `syntaxless_holdout.json`.
   - holdout result (`syntaxless_bootstrap_v1_holdout_v1`, selected `symbol_latency_guard`): SemanticFS recall `1.0000`, MRR `0.7722`, symbol-hit `0.6000`, p95 `13.217 ms`; baseline recall `1.0000`, MRR `0.8889`, symbol-hit `0.8000`, p95 `30.305 ms`.
52. Flutter source-focused strict run preparation completed; execution exceeded session window:
   - generated/split suites: `flutter_bootstrap_v2_src.json`, `flutter_v2_tune.json`, `flutter_v2_holdout.json`.
   - strict tune/holdout run with targeted candidate subset timed out before artifact completion; follow-up should run with tighter runtime bounds or narrower scope.
53. Drift summary refreshed after stockguessr fix + filesystem expansion (`scripts/drift_summary.ps1`):
   - history counts: `head_to_head=138`, `relevance=55`.
   - representative counts unchanged: `semanticfs_repo_v1` h2h/relevance=`12/26`, `ai_testgen_repo_v1` h2h/relevance=`10/25`.
54. Representative nightly run completed on February 24, 2026 (`scripts/nightly_representative.ps1 -SoakSeconds 30`):
   - relevance/head-to-head/release-gate all passed.
   - calendar-night trend progress moved to `5/7` complete (`2` nights remaining).
55. Bootstrap generator language coverage expanded for filesystem-scope fixtures:
   - `scripts/bootstrap_golden_from_repo.py` now includes `.dart` and Dart symbol extraction patterns (`class`, `enum`, `mixin`, function-like declarations).
56. Additional filesystem-scope strict tune/holdout runs completed:
   - `apex_scholars_bootstrap_v1_holdout_v1` (`symbol_latency_guard`): SemanticFS recall `0.9333`, MRR `0.9000`, symbol-hit `0.8667`, p95 `16.908 ms`; baseline recall `1.0000`, MRR `0.8389`, symbol-hit `0.7333`, p95 `38.806 ms`.
   - `flutter_tools_bootstrap_v1_holdout_v1` (`symbol_latency_guard`): SemanticFS recall `0.9333`, MRR `0.7578`, symbol-hit `0.6667`, p95 `21.677 ms`; baseline recall `1.0000`, MRR `0.9667`, symbol-hit `0.9333`, p95 `88.868 ms`.
57. New medium external strict tune/holdout run completed (`pseudolang`):
   - generated/split suites: `pseudolang_bootstrap_v1.json`, `pseudolang_tune.json`, `pseudolang_holdout.json`.
   - holdout result (`pseudolang_bootstrap_v1_holdout_v1`, selected `latency_guard`): SemanticFS recall `0.9333`, MRR `0.8333`, symbol-hit `0.7333`, p95 `19.497 ms`; baseline recall `1.0000`, MRR `0.8556`, symbol-hit `0.7333`, p95 `50.041 ms`.
58. Daytime representative smoke rerun with strict release gate completed (February 24, 2026):
   - `scripts/daytime_smoke.ps1 -SoakSeconds 2 -IncludeReleaseGate` passed.
   - semanticFS relevance: recall `0.95`, MRR `0.9250`, symbol-hit `1.00`.
   - ai-testgen relevance: recall `1.00`, MRR `0.9500`, symbol-hit `1.00`.
59. Drift summary refreshed after February 24 runs (`scripts/drift_summary.ps1`):
   - history counts: `head_to_head=149`, `relevance=60`.
   - representative counts: `semanticfs_repo_v1` h2h/relevance=`13/28`, `ai_testgen_repo_v1` h2h/relevance=`11/28`.
60. `flutter_tools` holdout query-level gap triage captured from latest strict artifact:
   - one semantic miss: `b06` (`_write`, expected `lib/src/android/android_console.dart`).
   - semantic rank-lag queries vs baseline rank-1: `b10` (`_canRun`), `b14` (`_Attribute`), `b18` (`attemptToolExit`), `b30` (`CommandHelp`).
61. Filesystem candidate discovery hardening landed:
   - `scripts/discover_repo_candidates.ps1` now excludes VS Code workspace mirror repos by default and dedupes mirrored clones by `remote.origin.url`.
   - refreshed `.semanticfs/bench/filesystem_repo_candidates_min80.json`: `21` candidates (`repo_count_before_dedupe=22`, `excluded_workspace_mirror_count=6`, `deduped_away_count=1`).
62. Filesystem-scope backlog planner added and executed:
   - new script: `scripts/build_filesystem_scope_backlog.ps1`.
   - artifact: `.semanticfs/bench/filesystem_scope_backlog_latest.json`.
   - latest counts: `uncovered=11`, `covered_gap=4`, `covered_partial=2`, `covered_ok=4`.
63. Partial-coverage roots identified for filesystem expansion queueing:
   - `C:\Users\navneeth\Documents\flutter` currently covered via child dataset `flutter_tools`.
   - `C:\Users\navneeth\Desktop\NavneethThings\Projects\Robot` currently covered via child dataset `tensorflow_models_curated`.
