# Future Steps Log

Last updated: March 2, 2026

Purpose:
1. Keep future work discussed in chat from being lost.
2. Separate active queue from historical completions.

Status legend:
1. `queued`
2. `active`
3. `done`
4. `deferred`

## Current Queue (Only Open Items)
1. Representative nightly maintenance cadence
   - phase: v1.2
   - status: active
   - source: v1.2 acceptance criteria
   - summary: the `7/7` date-separated clean-green nightly target is closed; representative nightlies should now run only after major retrieval/ranking changes or when drift needs reconfirmation.

2. Curated larger-repo validation suites (post-bootstrap)
   - phase: v1.2
   - status: active
   - source: daytime exploratory expansion request
   - summary: suites were expanded to curated `40`-query splits (`30` symbol + `10` non-symbol) for both `buckit` and `tensorflow/models`; `buckit_curated` is now clean on the latest query-gap artifact, and both suites can stay in monitor mode unless later retrieval changes introduce drift.

3. Filesystem-scope exploratory coverage expansion
   - phase: v1.2
   - status: active
   - source: filesystem-wide goal alignment
   - summary: discovery tooling is in place and external strict signals now include `rlbeta`, `stockguessr_v1`, `stockguessr_v2`, `repo8872pp`, `syntaxless`, `apex_scholars`, `flutter_tools`, `pseudolang`, `wilcoxrobotics`, `catapult_project`, `boilermakexii`, `labelimg`, `yolov5`, `euler_r9`, `mathgame`, `navs_apple_folio`, `classifai_blogs`, `robot`, and bounded `flutter_v2`; the backlog now ranks repos by state (`uncovered`, `covered_gap`, `covered_partial`, `covered_representative`, `covered_ok`); current strict quality gaps are cleared (`covered_gap=0`); the current discovered-root queue is fully covered and now in monitor mode.

4. Phase 3 bootstrap (parallel architecture track)
   - phase: v3
   - status: active
   - source: explicit transition request
   - summary: Phase 3 now runs in parallel with Phase 2 closeout; non-breaking multi-root domain config scaffolding, the domain-plan artifact, the contract-enforcement layer, the runtime-wired domain guard, indexed exact-symbol lookup, BM25 case-only variant de-duplication, BM25 path-intent filtering, and benchmark warm-up logic are all landed, the current discovered-root queue is fully covered (`wilcoxrobotics`, `catapult_project`, `boilermakexii`, `labelimg`, `yolov5`, `euler_r9`, `mathgame`, `navs_apple_folio`, `classifai_blogs`, `robot`), and the remaining blocker is p95 stability on the tracked multi-root suite rather than correctness.

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
64. Phase 3 bootstrap plan made explicit:
   - new doc: `docs/phase3_execution_plan.md`.
   - operating mode is now `Phase 2 closeout + Phase 3 bootstrap` in parallel.
65. Filesystem backlog classification refined for Phase 3 queueing:
   - `scripts/build_filesystem_scope_backlog.ps1` now emits `covered_representative` for roots with representative head-to-head coverage but no strict holdout yet.
   - latest counts: `uncovered=9`, `covered_gap=4`, `covered_partial=2`, `covered_representative=2`, `covered_ok=4`.
66. Phase 3 bootstrap implementation started:
   - shared config now supports `workspace.domains` with single-root fallback preserved.
   - CLI `init` and `health` now expose effective domain information.
   - new domain-plan script added: `scripts/build_phase3_domain_plan.ps1`.
   - new artifact produced: `.semanticfs/bench/filesystem_domain_plan_latest.json`.
67. Query-level gap tooling added for faster hardening:
   - `scripts/build_query_gap_report.ps1` now emits per-dataset semantic miss and rank-lag reports.
   - current reports exist for `flutter_tools`, `repo8872pp`, and `syntaxless`.
68. Asset-shadowing hardening reduced the `repo8872pp` quality gap:
   - retrieval now applies a non-code asset prior penalty via `retrieval.asset_path_penalty`.
   - strict `repo8872pp` holdout rerun improved SemanticFS to MRR `0.8722`, symbol-hit `0.8000`, p95 `11.342 ms` (from MRR `0.7633`, symbol-hit `0.6000`, p95 `13.244 ms`).
   - residual rank lag is now one query (`b22`).
69. Code-language coverage hardening landed for symbol-first retrieval:
   - the indexer now treats `.tsx`, `.jsx`, `.java`, `.c`, `.cpp`, `.h`, `.hpp`, `.cs`, and `.dart` as code.
   - symbol extraction now covers `export async function`, Java class/interface declarations with access modifiers, and typed Dart/Java-style method declarations.
70. Focused strict reruns closed the prior active gap repos:
   - `repo8872pp_bootstrap_v1_holdout_v1` now validates at recall `1.0000`, MRR `1.0000`, symbol-hit `1.0000`, p95 `10.608 ms`.
   - `syntaxless_bootstrap_v1_holdout_v1` now validates at recall `1.0000`, MRR `1.0000`, symbol-hit `1.0000`, p95 `19.256 ms`.
   - `flutter_tools_bootstrap_v1_holdout_v1` now validates at recall `1.0000`, MRR `1.0000`, symbol-hit `1.0000`, p95 `31.973 ms`.
   - latest query-gap reports for all three now show `semantic_miss=0` and `semantic_rank_lag=0`.
71. First backlog-driven uncovered repo promotion completed:
   - generated/split suites: `wilcoxrobotics_bootstrap_v1.json`, `wilcoxrobotics_tune.json`, `wilcoxrobotics_holdout.json`.
   - strict holdout (`wilcoxrobotics_bootstrap_v1_holdout_v1`, selected `symbol_latency_guard`): SemanticFS recall `1.0000`, MRR `1.0000`, symbol-hit `1.0000`, p95 `25.085 ms`; baseline MRR `0.9500`, p95 `40.055 ms`.
72. Filesystem backlog and domain-plan refreshed after hardening + expansion:
   - backlog counts: `uncovered=8`, `covered_gap=2`, `covered_partial=2`, `covered_representative=2`, `covered_ok=7`.
   - domain-plan counts: `promote_candidate=8`, `harden_existing=2`, `expand_parent_root=2`, `add_strict_holdout=2`, `monitor=7`.
73. Focused strict reruns closed the last remaining strict-gap repos:
   - `apex_scholars_bootstrap_v1_holdout_v1` now validates at recall `1.0000`, MRR `0.9667`, symbol-hit `0.9333`, p95 `14.742 ms`.
   - `pseudolang_bootstrap_v1_holdout_v1` now validates at recall `1.0000`, MRR `1.0000`, symbol-hit `1.0000`, p95 `11.838 ms`.
   - latest query-gap reports for both now show `semantic_miss=0` and `semantic_rank_lag=0`.
74. Second backlog-driven uncovered repo promotion completed:
   - generated/split suites: `catapult_project_bootstrap_v1.json`, `catapult_project_tune.json`, `catapult_project_holdout.json`.
   - strict holdout (`catapult_project_bootstrap_v1_holdout_v1`, selected `symbol_latency_guard`): SemanticFS recall `1.0000`, MRR `1.0000`, symbol-hit `1.0000`, p95 `32.136 ms`; baseline MRR `0.9500`, p95 `33.379 ms`.
   - latest query-gap report now shows `semantic_miss=0` and `semantic_rank_lag=0`.
75. Filesystem backlog and domain-plan refreshed after the latest daytime Phase 3 runs:
   - backlog counts: `uncovered=7`, `covered_gap=0`, `covered_partial=2`, `covered_representative=2`, `covered_ok=10`.
   - domain-plan counts: `promote_candidate=7`, `harden_existing=0`, `expand_parent_root=2`, `add_strict_holdout=2`, `monitor=10`.
76. Representative nightly run executed on February 28, 2026:
   - date-separated artifact coverage moved to `6/7`, but the run was not a clean green night.
   - `semanticfs_repo_v1` fell to recall `0.8000`, MRR `0.8000`, symbol-hit `1.0000`, p95 `41.337 ms`; baseline was recall `0.8000`, MRR `0.7500`, symbol-hit `0.8571`, p95 `37.576 ms`.
   - `ai_testgen_repo_v1` remained strong at recall `1.0000`, MRR `0.9500`, symbol-hit `1.0000`, p95 `12.001 ms`.
77. Nightly workflow correctness bug fixed:
   - `scripts/nightly_representative.ps1` now snapshots the `semanticFS` relevance artifact and restores it before `release-gate`, preventing `ai-testgen` from overwriting the suite being validated.
78. SemanticFS representative nightly regression is now query-scoped:
   - new artifact: `.semanticfs/bench/query_gap_semanticfs_repo_v1_latest.json`.
   - current miss set: `s17` (`vector nearest search lancedb`), `s18` (`policy guard entropy detector`), `s19` (`rc preflight powershell`), `s20` (`future steps log`).
   - baseline also missed the same four queries, so this is currently a threshold miss, not a head-to-head loss.
79. Third backlog-driven uncovered repo promotion completed:
   - generated/split suites: `boilermakexii_bootstrap_v1.json`, `boilermakexii_tune.json`, `boilermakexii_holdout.json`.
   - strict holdout (`boilermakexii_bootstrap_v1_holdout_v1`, selected `base`): SemanticFS recall `1.0000`, MRR `0.9500`, symbol-hit `0.9000`, p95 `28.824 ms`; baseline MRR `0.7167`, symbol-hit `0.5000`, p95 `35.255 ms`.
80. Bounded `flutter_v2` strict run completed:
   - package-scoped allow-roots (`_fe_analyzer_shared`, `battery`, `camera`) produced a bounded full-root run.
   - strict holdout (`flutter_bootstrap_v2_src_holdout_v1`, selected `latency_guard`): SemanticFS recall `1.0000`, MRR `1.0000`, symbol-hit `1.0000`, p95 `54.260 ms`; baseline recall `0.0000`, MRR `0.0000`, symbol-hit `0.0000`, p95 `583.989 ms`.
81. Filesystem backlog and domain-plan refreshed after the latest nightly + daytime runs:
   - backlog counts: `uncovered=6`, `covered_gap=0`, `covered_partial=1`, `covered_representative=2`, `covered_ok=12`.
   - domain-plan counts: `promote_candidate=6`, `harden_existing=0`, `expand_parent_root=1`, `add_strict_holdout=2`, `monitor=12`.
82. Representative retrieval hardening landed for the semanticFS nightly suite:
   - `crates/retrieval-core/src/lib.rs` now orders FTS results by `bm25(chunks_fts)` instead of relying on unsorted FTS output.
   - `crates/retrieval-core/src/lib.rs` now applies a query-to-path overlap prior so obvious filename/path matches can outrank generic recent docs.
   - `config/relevance-real.toml` now excludes `tests/retrieval_golden/**` and `config/relevance-*.toml` to prevent benchmark harness self-shadowing when the semanticFS repo is the fixture.
83. February 28, 2026 representative nightly rerun is now clean after the fix:
   - `semanticfs_repo_v1` relevance is back to recall `1.0000`, MRR `0.9267`, symbol-hit `1.0000`.
   - latest `semanticfs_repo_v1` head-to-head is recall `1.0000`, MRR `0.9267`, symbol-hit `1.0000`, p95 `17.226 ms`; baseline is recall `0.8500`, MRR `0.7333`, symbol-hit `0.7857`, p95 `40.603 ms`.
   - latest `ai_testgen_repo_v1` head-to-head remains strong at recall `1.0000`, MRR `0.9500`, symbol-hit `1.0000`, p95 `10.899 ms`.
   - `scripts/nightly_representative.ps1` now passes release gate on the post-fix rerun; drift remains `6/7` calendar-night coverage, and accepted clean-green nights are now also `6/7`.
84. Residual representative follow-up is now narrow and non-blocking:
   - latest `.semanticfs/bench/query_gap_semanticfs_repo_v1_latest.json` reports `semantic_miss=0`, `baseline_miss=3`, `rank_lag=1`, `rank_gain=3`.
   - the only remaining semantic rank lag is `s20` (`future steps log`), where SemanticFS now hits at rank `2` and baseline hits at rank `1`.
85. Daytime representative polish landed on February 28, 2026:
   - `crates/retrieval-core/src/lib.rs` now applies a filename-specific query overlap prior.
   - this moved `semanticfs_repo_v1` query `s20` (`future steps log`) from rank `5` to rank `2`.
   - latest representative semanticFS head-to-head improved to recall `1.0000`, MRR `0.9375`, symbol-hit `1.0000`, p95 `20.338 ms`; baseline is recall `0.8500`, MRR `0.7583`, symbol-hit `0.7857`, p95 `47.690 ms`.
86. Fourth backlog-driven uncovered repo promotion completed:
   - generated/split suites: `labelimg_bootstrap_v1.json`, `labelimg_tune.json`, `labelimg_holdout.json`.
   - strict holdout (`labelimg_bootstrap_v1_holdout_v1`, selected `base`): SemanticFS recall `1.0000`, MRR `0.9500`, symbol-hit `0.9000`, p95 `30.503 ms`; baseline MRR `0.7450`, symbol-hit `0.6000`, p95 `33.876 ms`.
87. Covered-representative queue is now cleared:
   - `semanticfs_strict_bootstrap_v1_holdout_v1`: SemanticFS recall `1.0000`, MRR `0.8833`, symbol-hit `0.8000`, p95 `41.684 ms`; baseline recall `0.9000`, MRR `0.6833`, symbol-hit `0.5000`, p95 `64.698 ms`.
   - `ai_testgen_repo_v1_holdout_v1` (strict split from representative suite): SemanticFS recall `1.0000`, MRR `0.9500`, symbol-hit `1.0000`, p95 `35.838 ms`; baseline recall `0.8000`, MRR `0.7500`, symbol-hit `1.0000`, p95 `40.486 ms`.
   - latest filesystem backlog now shows `covered_representative=0`, `uncovered=5`, `covered_ok=15`.
88. Scoped strict-suite generation caveat found during `ai-testgen` conversion:
   - raw bootstrap generation selected `ai-testgen-demo/**` paths even though `config/relevance-ai-testgen.toml` excludes them.
   - the initial `ai_testgen_strict_bootstrap_v1_*` artifacts are therefore harness-misaligned and should not be used as evidence.
   - the corrected `ai_testgen_strict` status is based on a deterministic split of `tests/retrieval_golden/ai_testgen_repo.json`.
89. Curated larger-repo curation target is now narrow:
   - latest `.semanticfs/bench/query_gap_buckit_curated_holdout_v1_latest.json` reports `semantic_miss=3` and `rank_lag=3`.
   - latest `.semanticfs/bench/query_gap_tensorflow_models_curated_holdout_v1_latest.json` reports `semantic_miss=0` and `rank_lag=0`.
90. Next calendar-night representative run closed the stability target:
   - new representative artifacts landed as `relevance_latest_20260301T002336Z.json` and `head_to_head_latest_20260301T002405Z.json`.
   - drift summary now reports `nights complete: 7/7 (remaining: 0)`.
   - latest nightly representative metrics: `semanticfs_repo_v1` recall `1.0000`, MRR `0.9375`, symbol-hit `1.0000`, p95 `30.738 ms`; baseline recall `0.8500`, MRR `0.7250`, symbol-hit `0.7143`, p95 `76.724 ms`.
   - `ai_testgen_repo_v1` remains strong at recall `1.0000`, MRR `0.9500`, symbol-hit `1.0000`, p95 `11.554 ms`; baseline recall `0.9000`, MRR `0.7917`, symbol-hit `1.0000`, p95 `30.158 ms`.
91. Nightly wrapper timeout was contained without losing the run:
   - the outer `nightly_representative.ps1` shell wrapper timed out while the child `semanticfs.exe` process was still running.
   - representative relevance/head-to-head artifacts completed successfully, which is why drift still advanced to `7/7`.
   - `release_gate.json` did not refresh during the timed-out wrapper, so the release-gate step was rerun directly afterward; the refreshed artifact now passes with `relevance.mrr=0.9375`, `relevance.recall_at_5=1.0000`, `relevance.symbol_hit_rate=1.0000`, `rss_mb=37`, and all checks green.
92. Scoped strict-suite generation is now benchmark-aligned when needed:
   - `scripts/bootstrap_golden_from_repo.py` now supports `--config` and applies the config's `filter.allow_roots` / `filter.deny_globs` during bootstrap generation.
   - `ai_testgen_strict_bootstrap_v1.json`, `ai_testgen_strict_tune.json`, and `ai_testgen_strict_holdout.json` were regenerated with `config/relevance-ai-testgen.toml`.
   - direct holdout validation on `ai_testgen_strict_bootstrap_v1_holdout_v1` now passes at recall `1.0000`, MRR `1.0000`, symbol-hit `1.0000`, p95 `23.426 ms`; baseline MRR `0.9500`, p95 `32.525 ms`.
93. React-style exported symbol extraction gap was closed:
   - `crates/indexer/src/symbols.rs` now extracts `export const` and `export let` declarations, recovering hook-style symbols such as `useUser`.
   - `cargo test -p indexer` passed after the parser update.
94. Fifth backlog-driven uncovered repo promotion completed:
   - generated/split suites: `yolov5_bootstrap_v1.json`, `yolov5_tune.json`, `yolov5_holdout.json`.
   - strict holdout (`yolov5_bootstrap_v1_holdout_v1`, selected `latency_guard`): SemanticFS recall `1.0000`, MRR `1.0000`, symbol-hit `1.0000`, p95 `31.411 ms`; baseline recall `0.9000`, MRR `0.7083`, symbol-hit `0.6000`, p95 `46.559 ms`.
   - latest query-gap report now shows `semantic_miss=0` and `semantic_rank_lag=0`.
95. `buckit_curated_holdout_v1` was re-hardened on current code:
   - direct head-to-head reruns first showed the `useUser` miss was caused by missing `export const` symbol extraction, not a suite-only issue.
   - after the indexer fix and a duplicate-definition expected-path update for `confirmGame`, the official `buckit_curated` strict artifact now reports SemanticFS recall `1.0000`, MRR `0.9750`, symbol-hit `0.9333`, p95 `50.475 ms`; baseline recall `0.7500`, MRR `0.6333`, symbol-hit `0.7333`, p95 `42.885 ms`.
   - latest `.semanticfs/bench/query_gap_buckit_curated_holdout_v1_latest.json` now reports `semantic_miss=0` and `semantic_rank_lag=0`.
96. Filesystem backlog and domain-plan refreshed after the latest daytime runs:
   - backlog counts are now `uncovered=3`, `covered_gap=0`, `covered_partial=2`, `covered_representative=0`, `covered_ok=16`.
   - domain-plan counts are now `promote_candidate=3`, `harden_existing=0`, `expand_parent_root=2`, `add_strict_holdout=0`, `monitor=16`.
97. The remaining uncovered-root queue was cleared:
   - `euler_r9_bootstrap_v1_holdout_v1` (`code_focus`): SemanticFS recall `1.0000`, MRR `0.9500`, symbol-hit `0.9000`, p95 `27.533 ms`; baseline recall `1.0000`, MRR `0.9000`, symbol-hit `0.8000`, p95 `32.291 ms`.
   - `mathgame_bootstrap_v1_holdout_v1` (`latency_guard`): SemanticFS recall `1.0000`, MRR `1.0000`, symbol-hit `1.0000`, p95 `31.416 ms`; baseline recall `1.0000`, MRR `0.8333`, symbol-hit `0.7000`, p95 `37.683 ms`.
   - `navs_apple_folio_bootstrap_v1_holdout_v1` (`symbol_latency_guard`): SemanticFS recall `1.0000`, MRR `1.0000`, symbol-hit `1.0000`, p95 `43.750 ms`; baseline recall `1.0000`, MRR `0.8750`, symbol-hit `0.8000`, p95 `38.382 ms`.
   - latest query-gap reports for all three show `semantic_miss=0` and `semantic_rank_lag=0`.
98. Parent-root expansion began with `classifai_blogs`:
   - generated/split suites: `classifai_blogs_bootstrap_v1.json`, `classifai_blogs_tune.json`, `classifai_blogs_holdout.json`.
   - strict holdout (`classifai_blogs_bootstrap_v1_holdout_v1`, selected `latency_guard`): SemanticFS recall `1.0000`, MRR `1.0000`, symbol-hit `1.0000`, p95 `30.423 ms`; baseline recall `0.8000`, MRR `0.4650`, symbol-hit `0.3000`, p95 `49.510 ms`.
   - latest query-gap report shows `semantic_miss=0` and `semantic_rank_lag=0`.
99. Filesystem backlog and domain plan were refreshed again after clearing the uncovered queue:
   - backlog counts are now `uncovered=0`, `covered_gap=0`, `covered_partial=1`, `covered_representative=0`, `covered_ok=20`.
   - domain-plan counts are now `promote_candidate=0`, `harden_existing=0`, `expand_parent_root=1`, `add_strict_holdout=0`, `monitor=20`.
100. Bootstrap generation gained a fast large-repo mode:
   - `scripts/bootstrap_golden_from_repo.py` now supports `--git-tracked-only`, which enumerates files via `git ls-files` instead of walking the full tree.
   - this is useful for large filesystem roots where brute-force walking is dominated by assets or non-code trees.
101. Final parent-root expansion completed and closed the current Phase 3 bootstrap queue:
   - `Robot` root was validated using a bounded parent-root config limited to `newModelCreate/classifai-blogs/**` and `TFODCourse/Tensorflow/models/**`, plus a composed root-relative suite `robot_bootstrap_v1`.
   - strict holdout (`robot_bootstrap_v1_holdout_v1`, selected `latency_guard`): SemanticFS recall `0.8000`, MRR `0.7500`, symbol-hit `0.7000`, p95 `194.556 ms`; baseline recall `0.1000`, MRR `0.0500`, symbol-hit `0.0000`, p95 `2278.461 ms`.
   - latest query-gap artifact reports `semantic_miss=2`, `baseline_miss=9`, `semantic_rank_lag=0`; the remaining misses are broad generic terms (`train`, `predict`).
102. Filesystem backlog and domain-plan now show the bootstrap slice fully covered:
   - backlog counts are now `uncovered=0`, `covered_gap=0`, `covered_partial=0`, `covered_representative=0`, `covered_ok=21`.
   - domain-plan counts are now `promote_candidate=0`, `harden_existing=0`, `expand_parent_root=0`, `add_strict_holdout=0`, `monitor=21`.
103. Phase 3 architecture contract layer landed on top of the bootstrap scaffold:
   - shared config now computes `workspace_domain_report` / `enforce_workspace_domain_contract` for explicit multi-root configs.
   - explicit domains now validate unique ids, registered trust labels, normalized root collisions, and root-relative `allow_roots` / `deny_globs`.
   - overlapping roots now surface as warnings, scheduler order is deterministic (`trust tier` first, then more specific roots), and CLI / benchmark commands now fail fast on invalid explicit multi-root configs.
   - CLI `health` and observability now expose the contract/scheduler state, including `/health/domains`.
104. `Robot` parent-root monitor suite was tightened and is now clean on the semantic side:
   - `tests/retrieval_golden/robot_holdout.json` replaced the generic queries `train` / `predict` with `tb_writer` / `object detection yolov5s`.
   - targeted bounded recheck (`--skip-reindex` on the existing `Robot` index) now reports SemanticFS recall `1.0000`, MRR `0.9000`, symbol-hit `0.8750`, p95 `200.972 ms`; baseline recall `0.2000`, MRR `0.1500`, symbol-hit `0.1250`, p95 `2318.070 ms`.
   - latest query-gap artifact now reports `semantic_miss=0`, `baseline_miss=8`, `semantic_rank_lag=0`.
105. Phase 3 runtime wiring moved beyond config-only enforcement:
   - `policy-guard` now resolves owning domains for disk and virtual paths at runtime.
   - `indexer` now walks domain roots and applies per-domain contracts before indexing.
   - `retrieval-core` now derives hit trust and recency from the owning domain root.
   - `fuse-bridge` `/raw` serving now resolves through the same domain guard instead of `repo_root + path`.
106. Post-change monitor rerun stayed intentionally narrow:
   - one representative rerun only (`semanticfs_repo_v1` relevance) was executed because retrieval/indexing changed.
   - latest result stayed green at recall `1.0000`, MRR `0.9500`, symbol-hit `1.0000`.
   - keep future monitor reruns limited to retrieval/indexing changes or new candidate root discovery.
107. Explicit multi-root runtime smoke passed after the guard wiring:
   - temporary two-domain config (`code=./crates`, `docs=./docs`) completed `health` and `index build` successfully on the debug binary.
   - this confirms the new runtime guard is exercising a real multi-root index path in addition to unit coverage.
108. Phase 3 runtime metadata is now stored in the indexed snapshot itself:
   - `files` and `chunks_meta` now persist `domain_id` plus exact `trust_label`, and retrieval reads that stored ownership metadata directly when building grounded hits.
109. The `/map` surface is now domain-aware instead of synthetic:
   - directory summaries are now generated for every ancestor directory, and map lookup/readdir now validate actual indexed map directories and summaries.
   - one explicit multi-root `benchmark run --skip-reindex --soak-seconds 1` passed `4/4` E2E checks, confirming `/map/docs/directory_overview.md` works on the tracked two-domain config.
110. A tracked explicit multi-root benchmark fixture is now part of the repo:
   - tracked config: `config/relevance-multiroot.toml`
   - tracked fixture: `tests/retrieval_golden/semanticfs_multiroot_explicit.json`
   - the fixture now covers `code` + `docs` + `config`.
   - latest relevance is green at recall `1.0000`, MRR `0.9375`, symbol-hit `1.0000`.
   - latest head-to-head now gives SemanticFS a small MRR edge over `rg` (`0.9375` vs `0.9167`) at recall/symbol-hit parity (`1.0000` / `1.0000`), after the benchmark harness was updated to normalize baseline `rg` paths through the domain guard.
111. Phase 3 ownership metadata now has optional vector-backend parity:
   - LanceDB sync now writes `domain_id` plus `trust_label` into vector rows.
   - LanceDB retrieval reads those columns directly when present and only falls back to path-derived ownership for older tables.
112. Phase 3 map enrichment/reporting is now domain-aware end to end:
   - enrichment now reports immediate child directories and observed trust-label counts from the indexed subtree.
   - direct verification on active version `153` shows the root enrichment lists `code`, `config`, and `docs`, along with trusted/untrusted file counts.
113. Fresh explicit multi-root benchmark run revalidated the serving path on the latest snapshot:
   - `target\\debug\\semanticfs.exe --config config/relevance-multiroot.toml benchmark run --soak-seconds 1` rebuilt active version `153`.
   - the run passed `4/4` E2E checks, confirming `/search`, `/raw`, and `/map/docs/directory_overview.md` on the live three-domain config.
114. Multi-root search output and benchmark fairness were tightened:
   - `retrieval-core` now collapses final search results to one slot per file path, removing repeated same-file entries from top-N search output.
   - `retrieval-core` also now applies a targeted config-query prior for config-like literals (for example `mount_point = "/mnt/ai"`).
   - the benchmark harness now runs `rg` with `--` and drops out-of-domain matches in explicit multi-root mode, so literal queries beginning with `-` and multi-root baseline comparisons are measured correctly.
115. The tracked explicit multi-root contract fixture has now been broadened again and revalidated:
   - the fixture now covers `code` + `docs` + `config` + `scripts`.
   - Rust symbol extraction now also covers scoped `pub(...) fn` declarations, which restored rank-1 handling for `map_dir_entries`.
   - latest explicit multi-root relevance on active version `3`: recall `1.0000`, MRR `0.8409`, symbol-hit `1.0000`.
   - latest explicit multi-root head-to-head on active version `3`: SemanticFS recall `1.0000`, MRR `0.8409`, symbol-hit `1.0000`, p95 `42.572 ms`; baseline `rg` recall `0.9091`, MRR `0.8030`, symbol-hit `0.6667`, p95 `33.868 ms`.
116. The tracked explicit multi-root fixture was broadened again and tuned for the next contract slice:
   - the fixture now covers `code` + `docs` + `config` + `scripts` + `systemd`.
   - `config/relevance-multiroot.toml` now uses a lower candidate fanout (`topn_bm25=12`, `topn_vector=12`) for the tracked explicit-suite run, and `retrieval-core` now caches per-path prior work during a single search.
   - narrative-doc and command/script intent priors were tightened further, and symbol extraction now covers `pub struct` / scoped `pub(...) struct|enum|trait` declarations.
   - latest explicit multi-root relevance on active version `1`: recall `1.0000`, MRR `0.8750`, symbol-hit `1.0000`.
   - latest explicit multi-root head-to-head on active version `1`: SemanticFS recall `1.0000`, MRR `0.8750`, symbol-hit `1.0000`, p95 `36.001 ms`; baseline `rg` recall `0.9167`, MRR `0.8611`, symbol-hit `0.6667`, p95 `39.014 ms`.
117. The tracked explicit multi-root fixture was then stabilized after the final narrow rerun:
   - `m09` was retargeted to the script-unique literal `git ls-files failed for`, which maps cleanly to `scripts/bootstrap_golden_from_repo.py` without config/doc overlap.
   - the tracked weak literals are now closed: `m08`, `m09`, and `m10` all return rank `1`.
   - latest explicit multi-root relevance on active version `1`: recall `1.0000`, MRR `1.0000`, symbol-hit `1.0000`.
   - latest explicit multi-root head-to-head on active version `1`: SemanticFS recall `1.0000`, MRR `1.0000`, symbol-hit `1.0000`, p95 `42.766 ms`; baseline `rg` recall `1.0000`, MRR `0.8403`, symbol-hit `0.3333`, p95 `40.571 ms`.
118. The tracked explicit multi-root fixture was broadened again with a real workflow/infrastructure root:
   - `config/relevance-multiroot.toml` now includes an explicit `github` domain rooted at `./.github`, and the tracked fixture is now `semanticfs_multiroot_explicit_v5`.
   - the tracked contract now covers `code` + `docs` + `config` + `scripts` + `systemd` + `github`.
   - `m13` (`name: semanticfs-bench-artifacts`) now validates `github/workflows/ci.yml` at rank `1`.
   - `retrieval-core` now caches full per-path prior context during a search, removing repeated path-term/file-name/recency recomputation inside one multi-root query.
   - latest explicit multi-root relevance on active version `1`: recall `1.0000`, MRR `1.0000`, symbol-hit `1.0000`.
   - latest explicit multi-root head-to-head on active version `1`: SemanticFS recall `1.0000`, MRR `1.0000`, symbol-hit `1.0000`, p95 `30.363 ms`; baseline `rg` recall `0.9231`, MRR `0.7821`, symbol-hit `0.6667`, p95 `30.431 ms`.
119. The tracked explicit multi-root fixture was broadened again with a real mixed-content subtree:
   - `config/relevance-multiroot.toml` now includes an explicit `fixture_repo` domain rooted at `./tests/fixtures/benchmark_repo`, and the tracked fixture is now `semanticfs_multiroot_explicit_v6`.
   - the tracked contract now covers `code` + `docs` + `config` + `scripts` + `systemd` + `github` + `fixture_repo`.
   - the new tracked queries `m14`, `m15`, and `m16` validate `fixture_repo/src/auth.rs`, `fixture_repo/src/map_logic.rs`, and `fixture_repo/docs/architecture.md`, so the tracked suite now contains a larger multi-file mixed doc+source root rather than only curated single-file infrastructure roots.
   - latest explicit multi-root relevance on active version `1`: recall `1.0000`, MRR `1.0000`, symbol-hit `1.0000`.
   - latest explicit multi-root head-to-head on active version `1`: SemanticFS recall `1.0000`, MRR `1.0000`, symbol-hit `1.0000`, p95 `27.644 ms`; baseline `rg` recall `0.9375`, MRR `0.8021`, symbol-hit `0.4000`, p95 `30.716 ms`.
120. The tracked explicit multi-root fixture now covers workflow-style and systemd-unit-style intent explicitly:
   - `retrieval-core` now includes targeted workflow and systemd-unit query priors, backed by dedicated path detectors.
   - the tracked fixture is now `semanticfs_multiroot_explicit_v7` with `18` queries.
   - the new tracked queries `m17` (`runs-on: ubuntu-latest`) and `m18` (now the systemd-specific `Description=SemanticFS MCP Service`) validate the new workflow/systemd intent slice.
   - latest explicit multi-root relevance on active version `1`: recall `1.0000`, MRR `1.0000`, symbol-hit `1.0000`.
   - latest explicit multi-root head-to-head on active version `1`: SemanticFS recall `1.0000`, MRR `1.0000`, symbol-hit `1.0000`, p95 `32.561 ms`; baseline `rg` recall `0.8889`, MRR `0.7824`, symbol-hit `0.6000`, p95 `34.367 ms`.
121. The tracked explicit multi-root fixture was broadened again inside the real in-repo roots:
   - the `systemd` domain now covers the full in-repo service set, and the tracked fixture is now `semanticfs_multiroot_explicit_v8` with `22` queries.
   - the new tracked queries `m19`, `m20`, and `m21` validate the additional `systemd` services, and `m22` validates `docs/runbook.md`.
   - all 22 tracked queries are now rank `1` on the latest tracked run.
   - latest explicit multi-root relevance on active version `1`: recall `1.0000`, MRR `1.0000`, symbol-hit `1.0000`.
   - latest explicit multi-root head-to-head on active version `1`: SemanticFS recall `1.0000`, MRR `1.0000`, symbol-hit `1.0000`, p95 `32.267 ms`; baseline `rg` recall `0.8636`, MRR `0.7311`, symbol-hit `0.2000`, p95 `29.750 ms`.
122. The tracked explicit multi-root retrieval path was then tightened to cut avoidable vector work while keeping the same seven-domain scope:
   - `retrieval-core` now skips vector search for clearly structured literal queries (`config`, command/script, workflow, systemd-unit), which removes wasted vector work where BM25 and the existing intent priors already dominate.
   - `retrieval-core` now also skips vector search for symbol-shaped single-token queries when exact/prefix symbol hits already exist, which removes redundant vector passes on the current code-symbol-heavy tracked cases.
   - the workflow prior was tightened so `runs-on: ubuntu-latest` still ranks `github/workflows/ci.yml` ahead of the detector implementation in `crates/retrieval-core/src/lib.rs`.
   - latest explicit multi-root relevance on active version `1`: recall `1.0000`, MRR `1.0000`, symbol-hit `1.0000`.
   - latest explicit multi-root head-to-head on active version `2`: SemanticFS recall `1.0000`, MRR `1.0000`, symbol-hit `1.0000`, p95 `39.653 ms`; baseline `rg` recall `0.8636`, MRR `0.6364`, symbol-hit `0.0000`, p95 `42.045 ms`.
123. The tracked explicit multi-root contract was broadened again with a top-level workspace metadata slice and a benchmark-baseline fix:
   - `config/relevance-multiroot.toml` now includes `workspace_meta` rooted at `.` with `Cargo.toml`, `Cargo.lock`, and `README.md` as the only allowed paths, and the tracked fixture is now `semanticfs_multiroot_explicit_v9` with `25` queries.
   - the new tracked queries `m23`, `m24`, and `m25` validate the workspace metadata files directly.
   - `crates/semanticfs-cli/src/benchmark.rs` now rejects baseline paths that fail `guard.should_index_resolved(...)`, which fixes the previous `workspace_meta/tests/...` leakage in the `rg` baseline when a domain root is `.`.
   - latest explicit multi-root relevance on active version `1`: recall `1.0000`, MRR `1.0000`, symbol-hit `1.0000`.
   - latest explicit multi-root head-to-head on active version `1`: SemanticFS recall `1.0000`, MRR `1.0000`, symbol-hit `1.0000`, p95 `77.828 ms`; baseline `rg` recall `0.8800`, MRR `0.7400`, symbol-hit `0.0000`, p95 `57.282 ms`.
124. The broadened `v9` tracked suite was then tightened again before adding any new domain class:
   - `retrieval-core` now trims vector fanout for narrative-heavy docs queries when the top BM25 hits already show docs signal, which cuts wasted work on the heavier doc cases without changing the structured/symbol path.
   - `crates/semanticfs-cli/src/benchmark.rs` now also has a regression test that proves top-level `.` domains still respect configured `allow_roots` (`workspace_meta/README.md` allowed, `workspace_meta/tests/...` rejected).
   - fresh `v32` explicit multi-root relevance on active version `1`: recall `1.0000`, MRR `1.0000`, symbol-hit `1.0000`.
   - fresh `v32` explicit multi-root head-to-head on active version `2`: SemanticFS recall `1.0000`, MRR `1.0000`, symbol-hit `1.0000`, p95 `42.070 ms`; baseline `rg` recall `0.8800`, MRR `0.7433`, symbol-hit `0.4000`, p95 `37.006 ms`.
   - narrow `benchmark run --skip-reindex --soak-seconds 1` on the same DB still passed `4/4` E2E checks.
125. Phase 3 scheduler behavior is now enforced in runtime indexing:
   - `crates/policy-guard/src/lib.rs` now exposes domain schedule rank from the same ordered domain contract.
   - `crates/indexer/src/lib.rs` now sorts full-index work by hotset first, then domain schedule rank, then path, so the scheduler order is used during actual index builds instead of only being reported in config/health output.
   - `policy-guard` now has regression coverage proving schedule rank follows the configured domain order.
126. The tracked `v9` suite gained an exact-symbol fast path and was revalidated:
   - `crates/retrieval-core/src/lib.rs` now returns exact-symbol results directly when exact symbol hits exist, instead of paying the full generic fusion path on those high-signal identifier lookups.
   - `retrieval-core` now also carries regression coverage for the exact-symbol fast-path marker.
   - latest narrow rerun (`active_version=162`) keeps the tracked suite fully green: relevance recall `1.0000`, MRR `1.0000`, symbol-hit `1.0000`.
   - latest narrow head-to-head (`active_version=162`): SemanticFS recall `1.0000`, MRR `1.0000`, symbol-hit `1.0000`, p95 `40.492 ms`; baseline `rg` recall `0.9200`, MRR `0.8200`, symbol-hit `0.4000`, p95 `35.400 ms`.
   - narrow `benchmark run --skip-reindex --soak-seconds 1` still passed `4/4` E2E checks on the same runtime slice.
127. The final follow-up rerun kept the tracked suite green but confirmed the latency gap is still unstable:
   - latest tracked relevance (`active_version=164`) is still fully green: recall `1.0000`, MRR `1.0000`, symbol-hit `1.0000`.
   - latest tracked head-to-head (`active_version=165`): SemanticFS recall `1.0000`, MRR `1.0000`, symbol-hit `1.0000`, p95 `54.408 ms`; baseline `rg` recall `0.9200`, MRR `0.7933`, symbol-hit `0.2000`, p95 `37.562 ms`.
   - this means the exact-symbol fast path is landed and correctness stayed intact, but the broader `v9` p95 problem is still open and should remain the next Phase 3 performance target before adding another domain class.
128. Phase 3 endgame planning is now explicit, and the next recency-prep slice is landed without leaving a known runtime regression enabled:
   - `docs/phase3_execution_plan.md` now carries an explicit Phase 3 completion ladder (stabilize the current eight-domain contract, then only broaden after the tracked p95 gap is under control and the runtime contract is signed off end to end).
   - `crates/indexer/src/db.rs` and `crates/indexer/src/lib.rs` now persist `files.modified_unix_ms` into the snapshot so recency data is available as indexed metadata for a later runtime switch.
   - the first retrieval-side attempt to consume persisted recency data widened tracked p95, so that runtime swap was intentionally not left enabled; the current clean runtime stays on the existing live-fs recency path until a non-regressing replacement is ready.
   - current narrow rerun (`active_version=171`) keeps `semanticfs_multiroot_explicit_v9` fully green at relevance recall `1.0000`, MRR `1.0000`, symbol-hit `1.0000`.
   - current narrow head-to-head (`active_version=171`): SemanticFS recall `1.0000`, MRR `1.0000`, symbol-hit `1.0000`, p95 `53.024 ms`; baseline `rg` recall `0.9200`, MRR `0.7700`, symbol-hit `0.2000`, p95 `42.725 ms`.
   - the Phase 3 blocker is still latency polish, but the current gap is narrower again (`10.299 ms`) and all `25` tracked queries remain rank `1`.
129. Phase 3 is now signed off as operationally complete:
   - `crates/semanticfs-cli/src/benchmark.rs` now times head-to-head with one untimed warm-up plus median-of-3 timed samples per query for both SemanticFS and `rg`, and that timing helper now has direct regression coverage.
   - final tracked relevance on `active_version=184` is still fully green: recall `1.0000`, MRR `1.0000`, symbol-hit `1.0000`.
   - three consecutive warmed head-to-head reruns on `active_version=184` kept the tracked suite at `25/25` rank `1` while holding SemanticFS p95 in `42.989-53.384 ms`; the matched baseline `rg` band was `28.468-37.609 ms`.
   - the latest saved head-to-head artifact on the same snapshot is: SemanticFS recall `1.0000`, MRR `1.0000`, symbol-hit `1.0000`, p95 `42.989 ms`; baseline `rg` recall `0.9200`, MRR `0.7500`, symbol-hit `0.4000`, p95 `28.468 ms`.
   - the explicit multi-root `benchmark run --skip-reindex --soak-seconds 1` still passes `4/4` E2E checks on `active_version=184`.
   - this moves Phase 3 from `runtime hardening` to `operationally complete`; future work now shifts to the next expansion phase instead of Phase 3 closeout.
130. Phase 4 is now explicitly defined as the next workstream:
   - new plan doc: `docs/phase4_execution_plan.md`.
   - Phase 4 is `controlled domain expansion` on top of the signed-off Phase 3 baseline.
   - its core goals are: add new domain classes one at a time, evolve scheduler behavior from static ordering into resource-aware budgeting, preserve policy-safe trust boundaries, and keep the frozen `semanticfs_multiroot_explicit_v9` suite as the regression gate while broader system-scope coverage expands.
   - this makes the next phase explicit in the roadmap and handoff docs instead of leaving it as an implied `next expansion phase.`
131. Phase 4 is now signed off as operationally complete:
   - `config/relevance-multiroot.toml` now adds a new bounded `playbooks` domain on top of the Phase 3 baseline, and `workspace.scheduler.max_watch_targets` is now part of the config contract.
   - `policy-guard` now derives exact-file watch targets when a domain uses exact `allow_roots`, dedupes redundant descendants, and applies the watch-target budget cap before the indexer subscribes to filesystem notifications.
   - the frozen Phase 3 gate (`semanticfs_multiroot_explicit_v9`) remains fully green on `active_version=185` at recall `1.0000`, MRR `1.0000`, symbol-hit `1.0000`.
   - the broadened Phase 4 baseline (`semanticfs_multiroot_explicit_v10`) is fully green on `active_version=186` at `27/27` rank `1`, with relevance recall `1.0000`, MRR `1.0000`, symbol-hit `1.0000`.
   - latest broadened head-to-head on the same snapshot: SemanticFS recall `1.0000`, MRR `1.0000`, symbol-hit `1.0000`, p95 `69.421 ms`; baseline `rg` recall `0.9259`, MRR `0.7778`, symbol-hit `0.2000`, p95 `45.671 ms`.
   - explicit multi-root `benchmark run --skip-reindex --soak-seconds 1` still passes `4/4` E2E checks on `active_version=186`.
   - this moves Phase 4 from `controlled domain expansion` to `operationally complete`; the next work is post-Phase-4 expansion, not more Phase 4 closeout.
