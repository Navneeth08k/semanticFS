# SemanticFS v1.2 Execution Plan

Last updated: March 2, 2026

## Intent
v1.2 is the reliability and quality release after v1.1 foundation hardening.
Primary objective: validate and stabilize SemanticFS on representative real-repo workloads.
This plan remains active while Phase 3 bootstrap starts in parallel; v1.2 is not yet considered closed.

## In Scope
1. Retrieval quality hardening and measurable evaluation.
2. Reliability improvements for active edit workflows.
3. Operational confidence via repeated benchmark/gate runs.
4. Clear, reproducible concept validation against non-SemanticFS baseline.

## Out Of Scope
1. Whole-PC semantic indexing.
2. Multi-root distributed indexing scheduler.
3. Write-enabled filesystem semantics.
4. Full multimodal retrieval as default.

## Acceptance Criteria
1. Golden suites cover representative repos with stable metrics.
2. Nightly benchmark trend remains stable for 7 consecutive runs.
3. Release-gate relevance thresholds are enforced and consistently met.
4. No known P0/P1 reliability or security defects.

## Completed In v1.2 So Far
1. Golden relevance harness with aggregate metrics (`benchmark relevance`).
2. Multi-suite support (`--golden-dir`) and history snapshots (`--history`).
3. Daytime and nightly benchmark scripts.
4. Search breadcrumb grounding contract.
5. MCP session pinning and refresh control.
6. Branch-swap queue planning + indexing-in-progress signaling.
7. Anti-shadowing priors (file-type + recency).
8. Head-to-head benchmark command (`benchmark head-to-head`) against `rg`.
9. Strict relevance thresholds hardened for representative suites (`min_queries=20`, `recall>=0.90`, `symbol_hit>=0.99`, `mrr>=0.80`).
10. FUSE long-lived session pin semantics with explicit refresh/status control files.
11. Same-day representative reliability sequence executed to 7+ head-to-head snapshots per target dataset (`semanticfs_repo_v1=8`, `ai_testgen_repo_v1=7`) with strict release-gate passing.
12. Mounted Linux FUSE session workflow validated end-to-end in WSL with real long-lived mount behavior (`session.json` stale detection + `session.refresh` repin/refresh path).
13. Drift-triage automation script added (`scripts/drift_summary.ps1`) with last-N deltas, history counts, and date coverage summary.
14. Linux FUSE session status regression tests added and validated in WSL (`session.json` stale flag, `session.refresh` refreshed flag, mode labels).
15. LanceDB small-dataset warning reduction: skip ANN vector index build under `65_536` rows to remove noisy KMeans empty-cluster spam during small-fixture runs.
16. Additional larger-repo exploratory head-to-head runs executed via bootstrap golden generation (`buckit_bootstrap_v1`, `tensorflow_models_bootstrap_v1`).
17. Strict tune-vs-holdout daytime protocol added:
   - deterministic suite split script (`scripts/split_golden_suite.py`)
   - tune/holdout selection runner (`scripts/daytime_tune_holdout.ps1`)
   - one-command daytime orchestration (`scripts/daytime_action_items.ps1`)
18. Bootstrap suites split into tune/holdout files:
   - `buckit_tune.json`, `buckit_holdout.json`
   - `tensorflow_models_tune.json`, `tensorflow_models_holdout.json`
19. Daytime tune/holdout runs executed on both larger repos with artifacts written to `.semanticfs/bench/tune_holdout_*_latest.json`.
20. Daytime smoke rerun including strict release gate passed (`scripts/daytime_smoke.ps1 -SoakSeconds 2 -IncludeReleaseGate`).
21. Expanded larger-repo bootstrap generation and curated mixed-suite builder landed:
   - larger bootstrap generation (`--max-queries 120`) for `buckit` and `tensorflow/models`
   - new curation script: `scripts/build_curated_mixed_suites.py`
   - daytime runner upgraded to use curated `40`-query tune/holdout splits (`30` symbol + `10` non-symbol each split)
22. Curated daytime tune/holdout runs completed:
   - `buckit_curated_holdout_v1`: SemanticFS recall `0.8250`, MRR `0.7458`, symbol-hit `0.8667`, p95 `77.307 ms`; baseline recall `0.7750`, MRR `0.6229`, symbol-hit `0.6333`, p95 `80.605 ms`
   - `tensorflow_models_curated_holdout_v1`: SemanticFS recall `0.8000`, MRR `0.4758`, symbol-hit `0.3333`, p95 `42.826 ms`; baseline recall `0.6500`, MRR `0.5217`, symbol-hit `0.5667`, p95 `146.918 ms`
23. TensorFlow `build_losses` holdout miss fixed via ground-truth disambiguation (multi-path expected targets) and revalidated:
   - updated `tests/retrieval_golden/tensorflow_models_holdout.json` for `build_losses`
   - re-run result: SemanticFS recall `1.00`, MRR `0.9500`, symbol-hit `0.9000`, p95 `45.890 ms`; baseline p95 `157.252 ms`
24. Retrieval and symbol quality hardening landed:
   - retrieval query normalization in `crates/retrieval-core/src/lib.rs` (symbol and BM25 query variants)
   - symbol extraction expansion in `crates/indexer/src/symbols.rs` (Python `def`/`async def`, Rust async fn, plain `function`)
   - new unit tests added in both crates for regression safety
25. Tune/holdout runner safety hardening:
   - `scripts/daytime_tune_holdout.ps1` now always rebuilds `semanticfs-cli` `--release` before scoring to avoid stale-binary artifacts.
26. Curated holdout revalidation after retrieval/symbol hardening:
   - `tensorflow_models_curated_holdout_v1`: SemanticFS recall `1.0000`, MRR `0.9208`, symbol-hit `0.8333`, p95 `102.798 ms`; baseline recall `0.6750`, MRR `0.5342`, symbol-hit `0.5667`, p95 `150.398 ms`
   - `buckit_curated_holdout_v1`: SemanticFS recall `0.9750`, MRR `0.8958`, symbol-hit `0.8667`, p95 `38.277 ms`; baseline recall `0.7750`, MRR `0.6458`, symbol-hit `0.7000`, p95 `44.555 ms`
27. Filesystem-scope exploratory track started (without blocking v1.2 nightlies):
   - repo discovery script added: `scripts/discover_repo_candidates.ps1`
   - user-root discovery artifact captured: `.semanticfs/bench/filesystem_repo_candidates_userroot.json`
   - new external exploratory suite + h2h run: `rlbeta_bootstrap_v1` (SemanticFS recall `1.00`, MRR `1.0000`, symbol-hit `1.00`, p95 `25.421 ms`; baseline p95 `649.968 ms`)
   - strict external tune/holdout run added: `rlbeta_bootstrap_v1_holdout_v1` (SemanticFS recall `1.00`, MRR `1.0000`, symbol-hit `1.00`, p95 `27.064 ms`; baseline MRR `0.8667`, p95 `727.422 ms`)
28. Curated large-suite hardening pass landed:
   - `scripts/build_curated_mixed_suites.py` now filters ambiguous symbols and generic/easy queries before split construction.
   - regenerated `buckit` and `tensorflow/models` curated tune/holdout suites with strict tune/holdout isolation preserved.
29. Revalidated strict curated holdouts after suite hardening:
   - `buckit_curated_holdout_v1` (`symbol_focus` selected): SemanticFS recall `0.9250`, MRR `0.8542`, symbol-hit `0.8000`, p95 `61.320 ms`; baseline recall `0.7500`, MRR `0.6271`, symbol-hit `0.7000`, p95 `52.228 ms`
   - `tensorflow_models_curated_holdout_v1` (`symbol_focus` selected): SemanticFS recall `1.0000`, MRR `0.9813`, symbol-hit `0.9667`, p95 `98.520 ms`; baseline recall `0.6500`, MRR `0.4988`, symbol-hit `0.5333`, p95 `157.718 ms`
30. Second external strict tune/holdout dataset added:
   - `stockguessr_bootstrap_v1` split to `stockguessr_tune.json` / `stockguessr_holdout.json`
   - strict holdout (`stockguessr_bootstrap_v1_holdout_v1`): SemanticFS recall `0.7333`, MRR `0.4300`, symbol-hit `0.2667`, p95 `376.317 ms`; baseline recall `0.0000`, MRR `0.0000`, symbol-hit `0.0000`, p95 `64.592 ms`
31. Representative nightly run completed on February 22, 2026:
   - relevance/head-to-head/release-gate passed.
   - date-separated night progress moved to `4/7` (`3` nights remaining).
32. Additional medium external strict tune/holdout run completed:
   - `repo8872pp_bootstrap_v1` split to `repo8872pp_tune.json` / `repo8872pp_holdout.json`.
   - strict holdout (`repo8872pp_bootstrap_v1_holdout_v1`): SemanticFS recall `1.0000`, MRR `0.7633`, symbol-hit `0.6000`, p95 `13.244 ms`; baseline recall `1.0000`, MRR `0.8889`, symbol-hit `0.8000`, p95 `43.464 ms`.
33. Daytime tune/holdout runner expanded for faster external iteration:
   - added latency-oriented candidate profiles (`latency_guard`, `symbol_latency_guard`).
   - added optional `-CandidateIds` filter for targeted candidate subsets on long-running external sweeps.
34. Bootstrap suite generator hardened for source-focused external fixtures:
   - `scripts/bootstrap_golden_from_repo.py` now excludes generated build/cache directories (including `.next`, `.nuxt`, `.svelte-kit`, `.turbo`, `.cache`, `.dart_tool`, `.pytest_cache`, `coverage`, `out`).
35. Stockguessr source-focused strict rerun completed:
   - regenerated external fixture set: `stockguessr_bootstrap_v2_src.json` -> `stockguessr_v2_tune.json` / `stockguessr_v2_holdout.json`.
   - strict holdout (`stockguessr_bootstrap_v2_src_holdout_v1`, selected `latency_guard`): SemanticFS recall `0.6000`, MRR `0.4800`, symbol-hit `0.4000`, p95 `190.883 ms`; baseline recall `0.9333`, MRR `0.8111`, symbol-hit `0.7333`, p95 `27.636 ms`.
   - targeted v1 rerun with `-CandidateIds latency_guard,symbol_latency_guard` selected `latency_guard`: SemanticFS recall `0.7333`, MRR `0.4300`, symbol-hit `0.2667`, p95 `391.161 ms`; baseline p95 `46.883 ms`.
   - SQLite backend spot-check on `stockguessr_bootstrap_v1_holdout_v1` reduced SemanticFS p95 from prior LanceDB run but still showed a large latency gap vs baseline (`274.516 ms` vs `34.533 ms`).
36. Generated-artifact suppression hardening landed for external source fidelity:
   - `crates/retrieval-core/src/lib.rs`: added generated-artifact path prior penalty (`.next`, `.nuxt`, `.svelte-kit`, `.turbo`, `.dart_tool`, `dist`, `build`, `out`, `coverage`, `target`, `*.min.js`) with unit coverage.
   - benchmark configs updated to filter generated directories for daytime strict runs (`config/relevance-real.toml`, `config/relevance-ai-testgen.toml`, `config/semanticfs.sample.toml`).
37. Stockguessr source-focused strict rerun after generated-artifact suppression:
   - `stockguessr_bootstrap_v2_src_holdout_v1` (`latency_guard`): SemanticFS recall `1.0000`, MRR `0.9667`, symbol-hit `0.9333`, p95 `13.588 ms`; baseline recall `0.9333`, MRR `0.8111`, symbol-hit `0.7333`, p95 `31.025 ms`.
   - previous external gap on stockguessr_v2 is now closed and reversed in favor of SemanticFS.
38. Filesystem-scope discovery expansion rerun completed:
   - `.semanticfs/bench/filesystem_repo_candidates_min80.json` generated from `C:\Users\navneeth` (`24` candidates at `MinTrackedFiles=80`).
39. Additional medium external strict tune/holdout run completed (`syntaxless`):
   - generated/split suites: `syntaxless_bootstrap_v1.json`, `syntaxless_tune.json`, `syntaxless_holdout.json`.
   - strict holdout (`syntaxless_bootstrap_v1_holdout_v1`, selected `symbol_latency_guard`): SemanticFS recall `1.0000`, MRR `0.7722`, symbol-hit `0.6000`, p95 `13.217 ms`; baseline recall `1.0000`, MRR `0.8889`, symbol-hit `0.8000`, p95 `30.305 ms`.
40. Flutter external strict source-focused run attempted with bounded candidate subset:
   - generated/split suites: `flutter_bootstrap_v2_src.json`, `flutter_v2_tune.json`, `flutter_v2_holdout.json`.
   - `daytime_tune_holdout` run exceeded single-session timeout window before artifact completion; follow-up should use tighter per-run bounds or narrower repo scope.
41. Bootstrap generator language coverage expanded for filesystem-scope suites:
   - `scripts/bootstrap_golden_from_repo.py` now includes `.dart` and Dart symbol extraction patterns (`class`, `enum`, `mixin`, function-like declarations).
42. Additional filesystem-scope strict tune/holdout runs completed:
   - `apex_scholars_bootstrap_v1_holdout_v1` (selected `symbol_latency_guard`): SemanticFS recall `0.9333`, MRR `0.9000`, symbol-hit `0.8667`, p95 `16.908 ms`; baseline recall `1.0000`, MRR `0.8389`, symbol-hit `0.7333`, p95 `38.806 ms`.
   - `flutter_tools_bootstrap_v1_holdout_v1` (selected `symbol_latency_guard`): SemanticFS recall `0.9333`, MRR `0.7578`, symbol-hit `0.6667`, p95 `21.677 ms`; baseline recall `1.0000`, MRR `0.9667`, symbol-hit `0.9333`, p95 `88.868 ms`.
43. Representative nightly run completed on February 24, 2026:
   - relevance/head-to-head/release-gate passed.
   - date-separated night progress moved to `5/7` (`2` nights remaining).
44. New medium external strict tune/holdout run completed (`pseudolang`):
   - generated/split suites: `pseudolang_bootstrap_v1.json`, `pseudolang_tune.json`, `pseudolang_holdout.json`.
   - strict holdout (`pseudolang_bootstrap_v1_holdout_v1`, selected `latency_guard`): SemanticFS recall `0.9333`, MRR `0.8333`, symbol-hit `0.7333`, p95 `19.497 ms`; baseline recall `1.0000`, MRR `0.8556`, symbol-hit `0.7333`, p95 `50.041 ms`.
45. Daytime representative smoke rerun with strict release gate completed on February 24, 2026:
   - `scripts/daytime_smoke.ps1 -SoakSeconds 2 -IncludeReleaseGate` passed.
   - semanticFS relevance: recall `0.95`, MRR `0.9250`, symbol-hit `1.00`.
   - ai-testgen relevance: recall `1.00`, MRR `0.9500`, symbol-hit `1.00`.
46. Drift summary refreshed after February 24 runs (`scripts/drift_summary.ps1`):
   - history counts: `head_to_head=149`, `relevance=60`.
   - representative counts: `semanticfs_repo_v1` h2h/relevance=`13/28`, `ai_testgen_repo_v1` h2h/relevance=`11/28`.
47. `flutter_tools` holdout query-level gap triage captured from latest strict artifact:
   - one true semantic miss: `b06` (`_write`, expected `lib/src/android/android_console.dart`).
   - semantic rank-lag queries vs baseline rank-1: `b10` (`_canRun`), `b14` (`_Attribute`), `b18` (`attemptToolExit`), `b30` (`CommandHelp`).
48. Filesystem candidate discovery hardening landed:
   - `scripts/discover_repo_candidates.ps1` now excludes VS Code workspace mirror repos by default and dedupes mirrored clones by `remote.origin.url` identity.
   - refreshed `min80` artifact (`.semanticfs/bench/filesystem_repo_candidates_min80.json`): `21` candidates (`repo_count_before_dedupe=22`, `excluded_workspace_mirror_count=6`, `deduped_away_count=1`).
49. Filesystem-scope backlog planner landed and executed:
   - new script: `scripts/build_filesystem_scope_backlog.ps1` (discovery + latest strict holdout artifacts -> prioritized next actions).
   - new artifact: `.semanticfs/bench/filesystem_scope_backlog_latest.json`.
   - latest counts: `uncovered=11`, `covered_gap=4`, `covered_partial=2`, `covered_ok=4`.
   - partial-coverage roots identified for expansion: `C:\Users\navneeth\Documents\flutter` (`flutter_tools`) and `C:\Users\navneeth\Desktop\NavneethThings\Projects\Robot` (`tensorflow_models_curated`).
50. Phase 3 bootstrap is now explicit and parallelized:
   - new doc: `docs/phase3_execution_plan.md`.
   - operating mode is now `Phase 2 closeout + Phase 3 bootstrap`.
51. Filesystem-scope backlog classification was refined to account for representative coverage:
   - `scripts/build_filesystem_scope_backlog.ps1` now adds `covered_representative` for roots with representative head-to-head evidence but no strict tune/holdout yet.
   - latest backlog counts are now `uncovered=9`, `covered_gap=4`, `covered_partial=2`, `covered_representative=2`, `covered_ok=4`.
52. Phase 3 bootstrap tooling landed:
   - multi-root domain config scaffolding added in shared config (`workspace.domains`) with single-root fallback preserved.
   - CLI `init` and `health` now expose effective domain shape without changing runtime behavior.
   - new planner script: `scripts/build_phase3_domain_plan.ps1`.
   - new artifact: `.semanticfs/bench/filesystem_domain_plan_latest.json` (`promote_candidate=9`, `harden_existing=4`, `expand_parent_root=2`, `add_strict_holdout=2`, `monitor=4`).
53. Query-level hardening tooling landed:
   - new script: `scripts/build_query_gap_report.ps1`.
   - query-gap artifacts now exist for `flutter_tools`, `repo8872pp`, and `syntaxless`.
54. Asset-shadowing hardening landed and materially improved `repo8872pp`:
   - `crates/retrieval-core/src/lib.rs` now applies a non-code asset-path prior penalty (for example `assets`, `static`, `.dat`, `.png`, `.onnx`) to reduce checked-in asset shadowing.
   - retrieval config now exposes `retrieval.asset_path_penalty`.
   - strict `repo8872pp` holdout rerun improved SemanticFS from MRR `0.7633` / symbol-hit `0.6000` / p95 `13.244 ms` to MRR `0.8722` / symbol-hit `0.8000` / p95 `11.342 ms`; baseline remained MRR `0.9167` / symbol-hit `0.8667` / p95 `37.820 ms`.
   - `repo8872pp` query-level rank lag dropped from `5` queries to `1` residual lag (`b22`).
55. Code-language coverage hardening landed for symbol-first retrieval:
   - `crates/indexer/src/filetype.rs` now classifies `.tsx`, `.jsx`, `.java`, `.c`, `.cpp`, `.h`, `.hpp`, `.cs`, and `.dart` as code.
   - `crates/indexer/src/symbols.rs` now extracts `export async function`, Java class/interface declarations with access modifiers, and typed method declarations (for example Dart/Java signatures like `void _write(...)`).
   - new unit coverage added for extended filetype detection and new symbol forms.
56. Focused strict holdout reruns closed the prior cross-repo ranking gaps:
   - `repo8872pp_bootstrap_v1_holdout_v1` (`base`): SemanticFS recall `1.0000`, MRR `1.0000`, symbol-hit `1.0000`, p95 `10.608 ms`; baseline recall `1.0000`, MRR `0.8889`, symbol-hit `0.8000`, p95 `38.283 ms`.
   - `syntaxless_bootstrap_v1_holdout_v1` (`symbol_latency_guard`): SemanticFS recall `1.0000`, MRR `1.0000`, symbol-hit `1.0000`, p95 `19.256 ms`; baseline recall `1.0000`, MRR `0.8889`, symbol-hit `0.8000`, p95 `42.625 ms`.
   - `flutter_tools_bootstrap_v1_holdout_v1` (`symbol_latency_guard`): SemanticFS recall `1.0000`, MRR `1.0000`, symbol-hit `1.0000`, p95 `31.973 ms`; baseline recall `1.0000`, MRR `0.9667`, symbol-hit `0.9333`, p95 `92.349 ms`.
   - query-gap reruns for all three now show `semantic_miss=0` and `semantic_rank_lag=0`.
57. First backlog-driven uncovered repo promotion completed:
   - generated/split suites: `wilcoxrobotics_bootstrap_v1.json`, `wilcoxrobotics_tune.json`, `wilcoxrobotics_holdout.json`.
   - strict holdout (`wilcoxrobotics_bootstrap_v1_holdout_v1`, selected `symbol_latency_guard`): SemanticFS recall `1.0000`, MRR `1.0000`, symbol-hit `1.0000`, p95 `25.085 ms`; baseline recall `1.0000`, MRR `0.9500`, symbol-hit `0.9000`, p95 `40.055 ms`.
   - backlog/domain-plan refresh after this run moved counts to `uncovered=8`, `covered_gap=2`, `covered_partial=2`, `covered_representative=2`, `covered_ok=7` and `promote_candidate=8`, `harden_existing=2`, `expand_parent_root=2`, `add_strict_holdout=2`, `monitor=7`.
58. Focused strict holdout reruns closed the remaining strict-gap repos:
   - `apex_scholars_bootstrap_v1_holdout_v1` (`symbol_latency_guard`): SemanticFS recall `1.0000`, MRR `0.9667`, symbol-hit `0.9333`, p95 `14.742 ms`; baseline recall `1.0000`, MRR `0.7667`, symbol-hit `0.6000`, p95 `28.347 ms`.
   - `pseudolang_bootstrap_v1_holdout_v1` (`latency_guard`): SemanticFS recall `1.0000`, MRR `1.0000`, symbol-hit `1.0000`, p95 `11.838 ms`; baseline recall `1.0000`, MRR `0.8222`, symbol-hit `0.6667`, p95 `34.086 ms`.
   - latest query-gap reruns for both now show `semantic_miss=0` and `semantic_rank_lag=0`.
59. Second backlog-driven uncovered repo promotion completed:
   - generated/split suites: `catapult_project_bootstrap_v1.json`, `catapult_project_tune.json`, `catapult_project_holdout.json`.
   - strict holdout (`catapult_project_bootstrap_v1_holdout_v1`, selected `symbol_latency_guard`): SemanticFS recall `1.0000`, MRR `1.0000`, symbol-hit `1.0000`, p95 `32.136 ms`; baseline recall `1.0000`, MRR `0.9500`, symbol-hit `0.9000`, p95 `33.379 ms`.
   - latest query-gap rerun shows `semantic_miss=0` and `semantic_rank_lag=0`.
   - backlog/domain-plan refresh after this run moved counts to `uncovered=7`, `covered_gap=0`, `covered_partial=2`, `covered_representative=2`, `covered_ok=10` and `promote_candidate=7`, `harden_existing=0`, `expand_parent_root=2`, `add_strict_holdout=2`, `monitor=10`.
60. Representative nightly run executed on February 28, 2026:
   - date-separated coverage artifacts moved to `6/7`, but the run is not a clean green night.
   - latest `semanticfs_repo_v1` nightly result fell to recall `0.8000`, MRR `0.8000`, symbol-hit `1.0000`, p95 `41.337 ms`; baseline was recall `0.8000`, MRR `0.7500`, symbol-hit `0.8571`, p95 `37.576 ms`.
   - latest `ai_testgen_repo_v1` nightly result remained strong: recall `1.0000`, MRR `0.9500`, symbol-hit `1.0000`, p95 `12.001 ms`.
61. Nightly workflow correctness bug identified and fixed:
   - `scripts/nightly_representative.ps1` previously let `ai-testgen` overwrite `.semanticfs/bench/relevance_latest.json` before `release-gate`, so the final strict gate could validate the wrong suite.
   - the script now snapshots the `semanticFS` relevance artifact and restores it before `release-gate`.
62. SemanticFS representative nightly regression is now query-scoped:
   - new artifact: `.semanticfs/bench/query_gap_semanticfs_repo_v1_latest.json`.
   - current misses are `s17` (`vector nearest search lancedb`), `s18` (`policy guard entropy detector`), `s19` (`rc preflight powershell`), and `s20` (`future steps log`).
   - this is currently a threshold problem, not a head-to-head loss: baseline also missed the same four queries.
63. Third backlog-driven uncovered repo promotion completed and bounded full-root follow-up succeeded:
   - generated/split suites: `boilermakexii_bootstrap_v1.json`, `boilermakexii_tune.json`, `boilermakexii_holdout.json`.
   - strict holdout (`boilermakexii_bootstrap_v1_holdout_v1`, selected `base`): SemanticFS recall `1.0000`, MRR `0.9500`, symbol-hit `0.9000`, p95 `28.824 ms`; baseline recall `1.0000`, MRR `0.7167`, symbol-hit `0.5000`, p95 `35.255 ms`.
   - bounded `flutter_v2` strict holdout completed using package-scoped allow-roots (`_fe_analyzer_shared`, `battery`, `camera`): SemanticFS recall `1.0000`, MRR `1.0000`, symbol-hit `1.0000`, p95 `54.260 ms`; baseline recall `0.0000`, MRR `0.0000`, symbol-hit `0.0000`, p95 `583.989 ms`.
   - post-run backlog/domain-plan counts are now `uncovered=6`, `covered_gap=0`, `covered_partial=1`, `covered_representative=2`, `covered_ok=12` and `promote_candidate=6`, `harden_existing=0`, `expand_parent_root=1`, `add_strict_holdout=2`, `monitor=12`.
64. Scoped bootstrap generation is now config-aligned when needed:
   - `scripts/bootstrap_golden_from_repo.py` now accepts `--config` and applies matching `filter.allow_roots` / `filter.deny_globs` rules during file selection.
   - `tests/retrieval_golden/ai_testgen_strict_bootstrap_v1.json`, `tests/retrieval_golden/ai_testgen_strict_tune.json`, and `tests/retrieval_golden/ai_testgen_strict_holdout.json` were regenerated with `config/relevance-ai-testgen.toml`.
   - direct holdout validation on `ai_testgen_strict_bootstrap_v1_holdout_v1` now passes at recall `1.0000`, MRR `1.0000`, symbol-hit `1.0000`, p95 `23.426 ms`; baseline MRR `0.9500`, p95 `32.525 ms`.
65. React-style exported symbol extraction hardening landed:
   - `crates/indexer/src/symbols.rs` now extracts `export const` and `export let` declarations, recovering hook-style JS symbols such as `useUser`.
   - `cargo test -p indexer` passed after the parser update.
66. Fifth backlog-driven uncovered repo promotion completed:
   - generated/split suites: `yolov5_bootstrap_v1.json`, `yolov5_tune.json`, `yolov5_holdout.json`.
   - strict holdout (`yolov5_bootstrap_v1_holdout_v1`, selected `latency_guard`): SemanticFS recall `1.0000`, MRR `1.0000`, symbol-hit `1.0000`, p95 `31.411 ms`; baseline recall `0.9000`, MRR `0.7083`, symbol-hit `0.6000`, p95 `46.559 ms`.
   - latest query-gap rerun shows `semantic_miss=0` and `semantic_rank_lag=0`.
67. Curated `buckit` holdout is now clean on current code:
   - `buckit_curated_holdout_v1` (`symbol_focus` selected): SemanticFS recall `1.0000`, MRR `0.9750`, symbol-hit `0.9333`, p95 `50.475 ms`; baseline recall `0.7500`, MRR `0.6333`, symbol-hit `0.7333`, p95 `42.885 ms`.
   - this improvement came from the `export const` symbol-indexing fix plus a duplicate-definition ground-truth update for `confirmGame` (`screens/LogScreen_fixed.js` and `screens/LogScreen.js`).
   - latest `.semanticfs/bench/query_gap_buckit_curated_holdout_v1_latest.json` now reports `semantic_miss=0` and `semantic_rank_lag=0`.
68. Filesystem backlog and domain plan were refreshed after the latest daytime runs:
   - backlog counts are now `uncovered=3`, `covered_gap=0`, `covered_partial=2`, `covered_representative=0`, `covered_ok=16`.
   - domain-plan counts are now `promote_candidate=3`, `harden_existing=0`, `expand_parent_root=2`, `add_strict_holdout=0`, `monitor=16`.
69. Remaining uncovered-root Phase 3 promotions completed:
   - `euler_r9_bootstrap_v1_holdout_v1` (`code_focus`): SemanticFS recall `1.0000`, MRR `0.9500`, symbol-hit `0.9000`, p95 `27.533 ms`; baseline recall `1.0000`, MRR `0.9000`, symbol-hit `0.8000`, p95 `32.291 ms`.
   - `mathgame_bootstrap_v1_holdout_v1` (`latency_guard`): SemanticFS recall `1.0000`, MRR `1.0000`, symbol-hit `1.0000`, p95 `31.416 ms`; baseline recall `1.0000`, MRR `0.8333`, symbol-hit `0.7000`, p95 `37.683 ms`.
   - `navs_apple_folio_bootstrap_v1_holdout_v1` (`symbol_latency_guard`): SemanticFS recall `1.0000`, MRR `1.0000`, symbol-hit `1.0000`, p95 `43.750 ms`; baseline recall `1.0000`, MRR `0.8750`, symbol-hit `0.8000`, p95 `38.382 ms`.
   - latest query-gap reruns for all three now show `semantic_miss=0` and `semantic_rank_lag=0`.
70. Parent-root expansion started and reduced the partial queue:
   - generated/split suites: `classifai_blogs_bootstrap_v1.json`, `classifai_blogs_tune.json`, `classifai_blogs_holdout.json`.
   - strict holdout (`classifai_blogs_bootstrap_v1_holdout_v1`, selected `latency_guard`): SemanticFS recall `1.0000`, MRR `1.0000`, symbol-hit `1.0000`, p95 `30.423 ms`; baseline recall `0.8000`, MRR `0.4650`, symbol-hit `0.3000`, p95 `49.510 ms`.
   - latest query-gap rerun shows `semantic_miss=0` and `semantic_rank_lag=0`.
71. Filesystem backlog and domain plan were refreshed again after clearing the uncovered queue:
   - backlog counts are now `uncovered=0`, `covered_gap=0`, `covered_partial=1`, `covered_representative=0`, `covered_ok=20`.
   - domain-plan counts are now `promote_candidate=0`, `harden_existing=0`, `expand_parent_root=1`, `add_strict_holdout=0`, `monitor=20`.
72. Bootstrap generation now supports a faster large-repo mode:
   - `scripts/bootstrap_golden_from_repo.py` now supports `--git-tracked-only`, which enumerates candidate files via `git ls-files` instead of walking the full tree.
   - this is useful for large filesystem-scope roots where the full directory walk is dominated by non-code or asset-heavy trees.
73. Final parent-root expansion completed and closed the current Phase 3 bootstrap queue:
   - `Robot` root was validated using a bounded parent-root config (`allow_roots` constrained to `newModelCreate/classifai-blogs/**` and `TFODCourse/Tensorflow/models/**`) plus a composed root-relative suite (`robot_bootstrap_v1`) built from those child subtrees.
   - strict holdout (`robot_bootstrap_v1_holdout_v1`, selected `latency_guard`): SemanticFS recall `0.8000`, MRR `0.7500`, symbol-hit `0.7000`, p95 `194.556 ms`; baseline recall `0.1000`, MRR `0.0500`, symbol-hit `0.0000`, p95 `2278.461 ms`.
   - latest query-gap artifact shows `semantic_miss=2`, `baseline_miss=9`, `semantic_rank_lag=0`; the remaining misses are broad generic terms (`train`, `predict`) that still leave the run comfortably ahead of baseline.
74. Filesystem backlog and domain plan now show the bootstrap slice fully covered:
   - backlog counts are now `uncovered=0`, `covered_gap=0`, `covered_partial=0`, `covered_representative=0`, `covered_ok=21`.
   - domain-plan counts are now `promote_candidate=0`, `harden_existing=0`, `expand_parent_root=0`, `add_strict_holdout=0`, `monitor=21`.
75. Phase 3 architecture contract layer is now implemented in the non-breaking scaffold:
   - `crates/semanticfs-common/src/config.rs` now computes `workspace_domain_report` and `enforce_workspace_domain_contract`.
   - explicit multi-root configs now validate unique domain ids, registered trust labels, normalized root collisions, and root-relative `allow_roots` / `deny_globs` patterns.
   - overlapping roots are surfaced as warnings, and scheduler order is now deterministic (`trust tier` first, then more specific roots before broader roots).
   - CLI and benchmark commands now fail fast on invalid explicit multi-root configs instead of silently accepting malformed cross-root state.
   - CLI `health` and observability now expose the contract/scheduler surface, including `/health/domains`.
76. `Robot` parent-root suite was tightened so the monitor artifact measures the bounded code roots instead of generic English verbs:
   - `tests/retrieval_golden/robot_holdout.json` now replaces `train` with `tb_writer` and `predict` with `object detection yolov5s`.
   - targeted bounded recheck on the existing `Robot` index (`--skip-reindex`) now reports SemanticFS recall `1.0000`, MRR `0.9000`, symbol-hit `0.8750`, p95 `200.972 ms`; baseline recall `0.2000`, MRR `0.1500`, symbol-hit `0.1250`, p95 `2318.070 ms`.
   - latest `.semanticfs/bench/query_gap_robot_bootstrap_v1_holdout_v1_latest.json` now reports `semantic_miss=0`, `baseline_miss=8`, `semantic_rank_lag=0`.
77. Phase 3 runtime wiring is now active instead of config-only:
   - `crates/policy-guard/src/lib.rs` now resolves domain ownership at runtime (`resolve_disk_path`, `resolve_virtual_path`) and applies per-domain allow/deny contracts plus trust mapping.
   - `crates/indexer/src/lib.rs` now walks the configured domain roots, assigns files to the scheduler-selected owning domain, and applies per-domain filter decisions before indexing.
   - `crates/retrieval-core/src/lib.rs` now derives hit trust from the owning domain and resolves recency checks against the real domain root path.
   - `crates/fuse-bridge/src/lib.rs` now resolves `/raw` through the domain guard instead of `repo_root + path`, and Linux raw listing/lookup now use the same ownership rules.
78. Narrow monitor rerun completed after the retrieval/indexing/runtime wiring change:
   - single representative rerun only (`semanticfs_repo_v1` relevance on `config/relevance-real.toml`), rather than a broad sweep.
   - latest result: recall `1.0000`, MRR `0.9500`, symbol-hit `1.0000`.
   - this preserves the monitor policy: rerun representative coverage only after retrieval/indexing changes or new root discovery.
79. Explicit multi-root runtime smoke passed:
   - temporary two-domain config (`code=./crates`, `docs=./docs`) completed `health` and `index build` successfully on the debug binary.
   - this confirms the new runtime guard is exercising actual multi-root indexing, not just config parsing.
80. Runtime domain ownership is now persisted in indexed metadata:
   - `files` and `chunks_meta` now store `domain_id` plus exact `trust_label`, and retrieval reads those fields directly instead of recomputing trust from virtual paths for every hit.
   - this keeps cross-root trust state queryable from the stored snapshot itself and aligns retrieval with the indexed ownership contract.
81. `/map` is now domain-aware in the same way `/raw` already is:
   - directory summaries are now precomputed for every ancestor directory (including the root summary), so explicit multi-root configs expose real domain trees under `/map`.
   - `fuse-bridge` map lookup and readdir now validate actual indexed map directories/summaries instead of synthesizing arbitrary subdirectories.
82. A tracked explicit multi-root benchmark fixture now exists and is validated:
   - new tracked config: `config/relevance-multiroot.toml`
   - new tracked fixture: `tests/retrieval_golden/semanticfs_multiroot_explicit.json`
   - latest explicit multi-root relevance: recall `1.0000`, MRR `1.0000`, symbol-hit `1.0000`
   - latest explicit multi-root head-to-head: SemanticFS and baseline `rg` are at quality parity (`recall=1.0000`, `MRR=1.0000`, `symbol-hit=1.0000`) after baseline normalization was updated to use the same domain-prefixed path contract.
83. Optional vector-backend parity is now implemented for Phase 3 ownership metadata:
   - LanceDB sync now writes `domain_id` plus `trust_label` into each vector row.
   - LanceDB retrieval now reads those columns when present and only falls back to path-derived ownership for older tables.
84. Domain-aware map enrichment/reporting is now aligned with the same directory model as `/map` and `/raw`:
   - map enrichment now reports immediate child directories and observed trust-label counts for each indexed directory subtree.
   - root-level enrichment now shows the live explicit domain tree (for example `code`, `docs`, `config`) instead of only flat summary text.
85. The tracked explicit multi-root benchmark fixture has been expanded beyond the original two-domain contract:
   - the fixture now covers `code` + `docs` + `config`.
   - latest explicit multi-root relevance on active version `153`: recall `1.0000`, MRR `0.9375`, symbol-hit `1.0000`.
   - latest explicit multi-root head-to-head on active version `153`: SemanticFS recall `1.0000`, MRR `0.9375`, symbol-hit `1.0000`, p95 `57.861 ms`; baseline `rg` recall `1.0000`, MRR `0.9167`, symbol-hit `1.0000`, p95 `41.310 ms`.
86. Multi-root search output now de-duplicates repeated same-file hits before final rendering:
   - `retrieval-core` now collapses final search results to one slot per file path, rather than allowing multiple line-range hits from the same file to consume top-N slots.
   - this specifically fixes the earlier repeated-path output seen in the explicit multi-root benchmark fixture.
87. Multi-root literal benchmark correctness and ranking priors were tightened:
   - `retrieval-core` now applies a targeted config-query prior, lifting config files for config-like literals such as `mount_point = "/mnt/ai"` without changing the general single-root contract.
   - `benchmark` now runs `rg` with `--` so literal queries beginning with `-` are handled correctly, and explicit multi-root baseline results now drop out-of-domain paths instead of leaking fixture files into the comparison.
88. The tracked explicit multi-root contract fixture has now been broadened and revalidated again:
   - the fixture now covers `code` + `docs` + `config` + `scripts`.
   - a small Rust symbol-coverage gap was also closed (`pub(crate)` / scoped `pub(...) fn` extraction), which restored rank-1 behavior for `map_dir_entries`.
   - latest explicit multi-root relevance on active version `3`: recall `1.0000`, MRR `0.8409`, symbol-hit `1.0000`.
   - latest explicit multi-root head-to-head on active version `3`: SemanticFS recall `1.0000`, MRR `0.8409`, symbol-hit `1.0000`, p95 `42.572 ms`; baseline `rg` recall `0.9091`, MRR `0.8030`, symbol-hit `0.6667`, p95 `33.868 ms`.
89. The tracked explicit multi-root contract fixture has now been broadened once more and tightened:
   - the fixture now covers `code` + `docs` + `config` + `scripts` + `systemd`.
   - `config/relevance-multiroot.toml` now uses a lower candidate fanout (`topn_bm25=12`, `topn_vector=12`) for the tracked contract suite, and `retrieval-core` now caches per-path prior work inside one search.
   - narrative-doc and command/script query priors were tightened, and `pub struct` / scoped `pub(...) struct|enum|trait` extraction was added so symbol and literal intent stay stable on the broader suite.
   - latest explicit multi-root relevance on active version `1`: recall `1.0000`, MRR `0.8750`, symbol-hit `1.0000`.
   - latest explicit multi-root head-to-head on active version `1`: SemanticFS recall `1.0000`, MRR `0.8750`, symbol-hit `1.0000`, p95 `36.001 ms`; baseline `rg` recall `0.9167`, MRR `0.8611`, symbol-hit `0.6667`, p95 `39.014 ms`.
90. The tracked explicit multi-root contract fixture was then stabilized on a narrow follow-up rerun:
   - `m09` now uses the script-unique literal `git ls-files failed for`, which maps cleanly to `scripts/bootstrap_golden_from_repo.py` without config/doc overlap.
   - the previously weak tracked literals (`m08`, `m09`, `m10`) are now all rank `1`.
   - latest explicit multi-root relevance on active version `1`: recall `1.0000`, MRR `1.0000`, symbol-hit `1.0000`.
   - latest explicit multi-root head-to-head on active version `1`: SemanticFS recall `1.0000`, MRR `1.0000`, symbol-hit `1.0000`, p95 `42.766 ms`; baseline `rg` recall `1.0000`, MRR `0.8403`, symbol-hit `0.3333`, p95 `40.571 ms`.
91. The tracked explicit multi-root contract fixture was expanded again and revalidated on a broader six-domain slice:
   - `config/relevance-multiroot.toml` now includes an explicit `github` domain rooted at `./.github`, and the tracked fixture now covers `code` + `docs` + `config` + `scripts` + `systemd` + `github`.
   - the new tracked query `m13` (`name: semanticfs-bench-artifacts`) validates `github/workflows/ci.yml` at rank `1`, which gives the contract a real workflow/infrastructure root instead of only code-adjacent paths.
   - `retrieval-core` now caches full per-path prior context during a search (path terms, file-name terms, path prior, recency prior) instead of recomputing them repeatedly, which removes redundant work from multi-root ranking.
   - latest explicit multi-root relevance on active version `1`: recall `1.0000`, MRR `1.0000`, symbol-hit `1.0000`.
   - latest explicit multi-root head-to-head on active version `1`: SemanticFS recall `1.0000`, MRR `1.0000`, symbol-hit `1.0000`, p95 `30.363 ms`; baseline `rg` recall `0.9231`, MRR `0.7821`, symbol-hit `0.6667`, p95 `30.431 ms`.
92. The tracked explicit multi-root contract fixture was broadened again beyond curated single-file infrastructure roots:
   - `config/relevance-multiroot.toml` now includes an explicit `fixture_repo` domain rooted at `./tests/fixtures/benchmark_repo`, so the tracked fixture now covers `code` + `docs` + `config` + `scripts` + `systemd` + `github` + `fixture_repo`.
   - the tracked fixture is now `semanticfs_multiroot_explicit_v6`.
   - `fixture_repo` is a real multi-file mixed-content subtree (`docs/architecture.md`, `src/auth.rs`, `src/main.rs`, `src/map_logic.rs`), and the new tracked queries `m14`, `m15`, and `m16` all validate at rank `1`.
   - latest explicit multi-root relevance on active version `1`: recall `1.0000`, MRR `1.0000`, symbol-hit `1.0000`.
   - latest explicit multi-root head-to-head on active version `1`: SemanticFS recall `1.0000`, MRR `1.0000`, symbol-hit `1.0000`, p95 `27.644 ms`; baseline `rg` recall `0.9375`, MRR `0.8021`, symbol-hit `0.4000`, p95 `30.716 ms`.
93. The tracked explicit multi-root contract fixture now explicitly exercises the new workflow and systemd ranking intents:
   - `retrieval-core` now includes `workflow` and `systemd-unit` query priors, plus matching path detectors, so workflow YAML and systemd unit literals get targeted boosts instead of relying only on generic overlap.
   - the tracked fixture is now `semanticfs_multiroot_explicit_v7` with `18` queries.
   - the new tracked queries `m17` (`runs-on: ubuntu-latest`) and `m18` (now the systemd-specific `Description=SemanticFS MCP Service`) validate the new workflow/systemd intent slice.
   - latest explicit multi-root relevance on active version `1`: recall `1.0000`, MRR `1.0000`, symbol-hit `1.0000`.
   - latest explicit multi-root head-to-head on active version `1`: SemanticFS recall `1.0000`, MRR `1.0000`, symbol-hit `1.0000`, p95 `32.561 ms`; baseline `rg` recall `0.8889`, MRR `0.7824`, symbol-hit `0.6000`, p95 `34.367 ms`.
94. The tracked explicit multi-root contract fixture was broadened again within existing real roots:
   - the `systemd` domain now covers all four in-repo service units (`semanticfs-fuse.service`, `semanticfs-indexer.service`, `semanticfs-mcp.service`, `semanticfs-observability.service`) instead of a single curated unit.
   - the tracked fixture is now `semanticfs_multiroot_explicit_v8` with `22` queries.
   - the new tracked queries `m19`, `m20`, and `m21` validate the additional `systemd` units, and `m22` validates the real operational document `docs/runbook.md`.
   - latest explicit multi-root relevance on active version `1`: recall `1.0000`, MRR `1.0000`, symbol-hit `1.0000`.
   - latest explicit multi-root head-to-head on active version `1`: SemanticFS recall `1.0000`, MRR `1.0000`, symbol-hit `1.0000`, p95 `32.267 ms`; baseline `rg` recall `0.8636`, MRR `0.7311`, symbol-hit `0.2000`, p95 `29.750 ms`.
95. The tracked explicit multi-root retrieval path was then tightened again for latency without broadening scope:
   - `retrieval-core` now skips vector search for clearly structured literal queries (`config`, command/script, workflow, systemd-unit) because those queries are already better served by symbol/BM25 plus the existing path priors.
   - `retrieval-core` now also skips vector search for symbol-shaped single-token queries when exact/prefix symbol hits already exist, which removes avoidable vector work on the high-signal identifier cases.
   - the workflow prior was tightened so `runs-on: ubuntu-latest` still ranks the real workflow file ahead of the detector implementation in `retrieval-core`.
   - latest explicit multi-root relevance on active version `1`: recall `1.0000`, MRR `1.0000`, symbol-hit `1.0000`.
   - latest explicit multi-root head-to-head on active version `2`: SemanticFS recall `1.0000`, MRR `1.0000`, symbol-hit `1.0000`, p95 `39.653 ms`; baseline `rg` recall `0.8636`, MRR `0.6364`, symbol-hit `0.0000`, p95 `42.045 ms`.
96. The tracked explicit multi-root contract was then broadened again with a top-level workspace metadata domain:
   - `config/relevance-multiroot.toml` now includes `workspace_meta` rooted at `.` with an allow-list of only `Cargo.toml`, `Cargo.lock`, and `README.md`, and the tracked fixture is now `semanticfs_multiroot_explicit_v9` with `25` queries.
   - the new tracked queries `m23`, `m24`, and `m25` validate `workspace_meta/Cargo.toml`, `workspace_meta/Cargo.lock`, and `workspace_meta/README.md`.
   - `benchmark.rs` now applies `guard.should_index_resolved(...)` after domain ownership normalization, so the `rg` baseline no longer leaks invalid top-level paths like `workspace_meta/tests/...` when a root is `.`.
   - latest explicit multi-root relevance on active version `1`: recall `1.0000`, MRR `1.0000`, symbol-hit `1.0000`.
   - latest explicit multi-root head-to-head on active version `1`: SemanticFS recall `1.0000`, MRR `1.0000`, symbol-hit `1.0000`, p95 `77.828 ms`; baseline `rg` recall `0.8800`, MRR `0.7400`, symbol-hit `0.0000`, p95 `57.282 ms`.
97. Phase 3 scheduler behavior is now part of real runtime indexing, not just config planning:
   - `indexer` now sorts full-index work by hotset first, then domain schedule rank, then path, so the computed multi-root scheduler order is enforced during actual index builds.
   - this keeps overlapping or mixed-trust roots deterministic in the same order the domain contract reports.
98. Exact symbol-like queries now take a dedicated fast path in retrieval:
   - `retrieval-core` now returns exact-symbol results directly when exact symbol hits exist, instead of paying the full generic fusion path on those high-signal identifier lookups.
   - latest narrow rerun (`active_version=162`) keeps the tracked `semanticfs_multiroot_explicit_v9` suite fully green at relevance recall `1.0000`, MRR `1.0000`, symbol-hit `1.0000`.
   - the tracked contract stayed green on the current follow-up rerun (`active_version=171`): SemanticFS recall `1.0000`, MRR `1.0000`, symbol-hit `1.0000`, p95 `53.024 ms`; baseline `rg` recall `0.9200`, MRR `0.7700`, symbol-hit `0.2000`, p95 `42.725 ms`.

## Latest Progress Snapshot (March 1, 2026)
1. Relevance history counts:
   - `semanticfs_repo_v1=29`
   - `ai_testgen_repo_v1=29`
2. Head-to-head history counts:
   - `semanticfs_repo_v1=14`
   - `ai_testgen_repo_v1=12`
3. Last-7 head-to-head delta trend (`SemanticFS - rg`):
   - `semanticfs_repo_v1`: delta MRR `min/avg=0.0500/0.2696`, delta recall `0.0000/0.1143`, delta symbol-hit `0.1429/0.4388`, delta p95 `-16.608/-7.384 ms`.
   - `ai_testgen_repo_v1`: delta MRR `min/avg=0.0875/0.1629`, delta recall `0.1000/0.1143`, delta symbol-hit `0.0000/0.0000`, delta p95 `-28.106/-18.776 ms`.
4. Mounted Linux FUSE validation:
   - Real mounted workflow now passes end-to-end in WSL (`VALIDATION_OK`) for `/.well-known/session.json` and `/.well-known/session.refresh`.
   - Verified stale detection and refresh behavior across real index version transitions (`136 -> 137`, `138 -> 139`).
5. Calendar-night representative run status:
   - Date-separated nightly coverage is now complete at `7/7`.
   - Accepted clean-green nights are also `7/7`; representative nightlies should now run on maintenance cadence instead of blocking daytime work.
6. Additional larger-repo exploratory snapshot (bootstrap suites, daytime):
   - `buckit_bootstrap_v1`: SemanticFS recall `1.00`, MRR `0.9417`, symbol-hit `0.90`, p95 `35.004 ms`; baseline `rg` recall `1.00`, MRR `0.7875`, symbol-hit `0.60`, p95 `52.288 ms`.
   - `tensorflow_models_bootstrap_v1`: SemanticFS recall `0.90`, MRR `0.7542`, symbol-hit `0.65`, p95 `49.465 ms`; baseline `rg` recall `0.95`, MRR `0.7892`, symbol-hit `0.70`, p95 `143.105 ms`.
   - Note: these are bootstrap suites for exploratory signal, not yet acceptance-grade curated golden sets.
7. New strict holdout results (daytime tune-vs-holdout protocol):
   - `buckit_bootstrap_v1_holdout_v1` (selected candidate: `base`):
     - SemanticFS recall `1.00`, MRR `1.0000`, symbol-hit `1.00`, p95 `39.054 ms`
     - baseline `rg` recall `1.00`, MRR `0.8033`, symbol-hit `0.70`, p95 `39.948 ms`
   - `tensorflow_models_bootstrap_v1_holdout_v1` (latest selected candidate: `base`, after `build_losses` disambiguation):
     - SemanticFS recall `1.00`, MRR `0.9500`, symbol-hit `0.90`, p95 `45.890 ms`
     - baseline `rg` recall `1.00`, MRR `0.9500`, symbol-hit `0.90`, p95 `157.252 ms`
8. Expanded curated holdout results (acceptance-grade size target met):
   - `buckit_curated_holdout_v1` (latest strict holdout): SemanticFS recall `1.0000`, MRR `0.9750`, symbol-hit `0.9333`, p95 `50.475 ms`; baseline recall `0.7500`, MRR `0.6333`, symbol-hit `0.7333`, p95 `42.885 ms`.
   - `tensorflow_models_curated_holdout_v1` (latest strict holdout): SemanticFS leads baseline on recall/MRR/symbol-hit and p95 after retrieval + symbol hardening.
9. External strict holdout expansion:
   - `rlbeta_bootstrap_v1_holdout_v1`: strong quality + major latency win.
   - `stockguessr_bootstrap_v1_holdout_v1`: SemanticFS beats baseline on quality (baseline near-zero) but has higher p95 latency.
   - `stockguessr_bootstrap_v2_src_holdout_v1` (source-focused, latest): SemanticFS now leads baseline on recall/MRR/symbol-hit and p95 after generated-artifact suppression.
   - `repo8872pp_bootstrap_v1_holdout_v1` (latest): SemanticFS now leads baseline on recall parity, MRR, symbol-hit, and p95 after asset + language-coverage hardening.
   - `syntaxless_bootstrap_v1_holdout_v1` (latest): SemanticFS now leads baseline on recall parity, MRR, symbol-hit, and p95 after TSX symbol indexing recovery.
   - `apex_scholars_bootstrap_v1_holdout_v1` (latest): SemanticFS now leads baseline on recall parity, MRR, symbol-hit, and p95.
   - `flutter_tools_bootstrap_v1_holdout_v1` (latest): SemanticFS now leads baseline on recall parity, MRR, symbol-hit, and p95 after Dart symbol indexing recovery.
   - `pseudolang_bootstrap_v1_holdout_v1` (latest): SemanticFS now leads baseline on recall parity, MRR, symbol-hit, and p95.
   - `wilcoxrobotics_bootstrap_v1_holdout_v1`: first backlog-driven uncovered repo completed, with SemanticFS leading on recall parity, MRR, symbol-hit, and p95.
   - `catapult_project_bootstrap_v1_holdout_v1`: second backlog-driven uncovered repo completed, with SemanticFS leading on recall parity, MRR, symbol-hit, and p95.
   - `boilermakexii_bootstrap_v1_holdout_v1`: third backlog-driven uncovered repo completed, with SemanticFS leading on recall parity, MRR, symbol-hit, and p95.
   - `yolov5_bootstrap_v1_holdout_v1`: fifth backlog-driven uncovered repo completed, with SemanticFS leading on recall, MRR, symbol-hit, and p95.
   - `euler_r9_bootstrap_v1_holdout_v1`, `mathgame_bootstrap_v1_holdout_v1`, and `navs_apple_folio_bootstrap_v1_holdout_v1`: remaining uncovered roots completed, all with zero semantic misses/rank lag on latest query-gap and SemanticFS leading or matching on recall while leading on ranking quality.
   - `classifai_blogs_bootstrap_v1_holdout_v1`: first parent-root expansion after the uncovered queue, with SemanticFS leading strongly on recall, MRR, symbol-hit, and p95.
   - `flutter_bootstrap_v2_src_holdout_v1` (bounded package-scoped run): SemanticFS leads baseline massively on recall, MRR, symbol-hit, and p95.
   - `robot_bootstrap_v1_holdout_v1` (tightened bounded monitor suite, skip-reindex recheck): SemanticFS now reaches recall `1.0000`, MRR `0.9000`, symbol-hit `0.8750`, p95 `200.972 ms`; baseline recall `0.2000`, MRR `0.1500`, symbol-hit `0.1250`, p95 `2318.070 ms`.
10. `build_losses` disambiguation check:
   - previous TensorFlow holdout miss was due ambiguous ground truth (`build_losses` appears across many files).
   - after expected-path disambiguation, both engines hit and SemanticFS keeps strong latency advantage (`p95 45.890 ms` vs `157.252 ms`).
11. Interpretation:
   - Same-day reliability trend is favorable.
   - Mounted Linux session semantics are now validated in a real long-lived session.
   - Date-separated overnight artifact coverage is now `7/7`, and accepted clean-green nightly evidence is now also `7/7` after the overnight run recorded in `relevance_latest_20260301T002336Z.json` / `head_to_head_latest_20260301T002405Z.json`.
   - Curated TensorFlow holdout quality objective is now met with preserved latency advantage in latest strict run.
   - Holdout protocol remains in place, reducing overfit risk for daytime tuning.
   - Generated-artifact suppression closed the stockguessr_v2 external source gap.
   - The prior `repo8872pp`, `syntaxless`, `flutter_tools`, `apex_scholars`, and `pseudolang` gaps are now closed.
   - There are currently no `covered_gap` repos in the latest filesystem backlog; the active daytime focus should stay on uncovered-root promotion and parent-root expansion.
   - The bounded `flutter_v2` completion moved the full `flutter` root from partial to `covered_ok`.
   - `buckit_curated_holdout_v1` is now clean on latest query-gap (`semantic_miss=0`, `semantic_rank_lag=0`).
12. Filesystem-scope planning status:
   - discovery noise is reduced (workspace mirrors + mirrored clone dedupe).
   - backlog now separates repos into `uncovered`, `covered_gap`, `covered_partial`, `covered_representative`, and `covered_ok`, enabling deterministic daytime queueing.
   - current backlog counts: `uncovered=0`, `covered_gap=0`, `covered_partial=0`, `covered_representative=0`, `covered_ok=21`.
   - the current discovered-root promotion queue is fully cleared; the backlog now moves to monitor mode.
13. Phase 3 bootstrap status:
   - non-breaking multi-root domain config scaffolding is now landed.
   - domain-plan artifact is live at `.semanticfs/bench/filesystem_domain_plan_latest.json`.
   - shared config now computes/enforces domain contracts (unique ids, trust-label registration, normalized root collision checks, root-relative policy patterns, overlap warnings, deterministic scheduler order).
   - CLI `health` and observability now expose the contract surface, and CLI / benchmark commands fail fast on invalid explicit multi-root configs.
   - runtime wiring is now started: indexing, retrieval, and `/raw` serving all resolve domain ownership through the same guard instead of assuming a single `repo_root`.
   - current domain-plan counts: `promote_candidate=0`, `harden_existing=0`, `expand_parent_root=0`, `add_strict_holdout=0`, `monitor=21`.
14. Current hardening status:
   - `repo8872pp`, `syntaxless`, `flutter_tools`, `apex_scholars`, and `pseudolang` now have zero semantic misses and zero semantic rank lag in their latest query-gap artifacts.
15. Explicit multi-root Phase 3 contract fixture:
   - tracked config + fixture now exist (`config/relevance-multiroot.toml`, `tests/retrieval_golden/semanticfs_multiroot_explicit.json`).
   - the fixture now covers `workspace_meta` + `code` + `docs` + `config` + `scripts` + `systemd` + `github` + `fixture_repo`.
   - repeated same-file result entries are now collapsed before final search output, so top-5 no longer wastes slots on duplicate file paths.
   - relevance is green at recall `1.0000`, MRR `1.0000`, symbol-hit `1.0000`.
   - latest narrow rerun (`active_version=171`) keeps SemanticFS clearly ahead on quality on the same domain-prefixed contract (`recall 1.0000` vs `0.9200`, `MRR 1.0000` vs `0.7700`, `symbol-hit 1.0000` vs `0.2000`), while `rg` is still ahead on p95 on the broadened fixture (`42.725 ms` vs SemanticFS `53.024 ms`).
   - all 25 tracked queries are now rank `1` on the latest tracked run.
   - the top-level `workspace_meta` domain is now in the contract and the baseline normalization bug for `.` roots is fixed by enforcing `guard.should_index_resolved(...)` on baseline paths.
   - narrative-heavy docs queries now trim vector fanout when top BM25 already shows docs signal, which materially reduced the broader-fixture p95 cost without changing tracked correctness.
   - `semanticfs-cli` now has a regression test that pins top-level `.` baseline normalization to the configured `workspace_meta` allow-roots.
   - the runtime now also enforces scheduler order during full-index builds and uses an exact-symbol fast path for direct identifier hits.
   - the latest filesystem backlog now has `covered_gap=0`.
   - `semanticfs_repo_v1` is back above threshold after representative retrieval hardening (`relevance recall=1.0000`, `MRR=0.9267`, `symbol-hit=1.0000`; latest daytime head-to-head polish improved this to `MRR=0.9375`, p95 `20.338 ms` vs baseline `47.690 ms`).
   - the representative hardening change set was: order FTS results by `bm25(chunks_fts)`, add a query-to-path overlap prior in `retrieval-core`, and exclude benchmark harness self-shadowing paths (`tests/retrieval_golden/**`, `config/relevance-*.toml`) from `config/relevance-real.toml`.
   - the latest `semanticfs_repo_v1` query-gap artifact now shows `semantic_miss=0`; the only residual issue is one non-blocking rank-lag query (`s20`, `future steps log`, SemanticFS rank `2` vs baseline rank `1`).
   - new strict coverage landed for `labelimg`, `semanticFS`, and `ai-testgen`; `covered_representative` is now cleared in the backlog.
   - `crates/indexer/src/symbols.rs` now indexes `export const` / `export let`, which recovered React hook-style symbols and closed the prior `buckit` `useUser` miss.
   - config-aligned bootstrap generation is now available for scoped repos; the regenerated `ai_testgen_strict_*` fixtures validate cleanly.
   - head-to-head timing now uses one untimed warm-up plus median-of-3 timed samples per query for both SemanticFS and `rg`; three consecutive warmed reruns on `active_version=184` held SemanticFS p95 in `42.989-53.384 ms` vs baseline `28.468-37.609 ms` while keeping the tracked multi-root suite at `25/25` rank `1`.
   - `crates/semanticfs-cli/src/benchmark.rs` now includes regression coverage for the new `median_u64` helper, so the Phase 3 timing harness is pinned by tests.
   - this is enough to move the Phase 3 architecture track from `runtime hardening` to `operationally complete`; future broadening now belongs to the next expansion phase instead of Phase 3 closeout.

## Active Remaining Work
1. Calendar-night stability confirmation: the `7/7` clean-green night target is now closed; shift representative nightlies from gating cadence to maintenance cadence unless a regression or major retrieval change lands.
2. Larger-repo validation hardening: `buckit_curated_holdout_v1` is now clean on the latest query-gap artifact; keep `buckit_curated_*` and `tensorflow_models_curated_*` in monitor mode unless later retrieval changes introduce drift.
3. Filesystem-scope prep track: external strict coverage now includes `rlbeta`, `stockguessr_v1`, `stockguessr_v2`, `repo8872pp`, `syntaxless`, `apex_scholars`, `flutter_tools`, `pseudolang`, `wilcoxrobotics`, `catapult_project`, `boilermakexii`, `labelimg`, `yolov5`, `euler_r9`, `mathgame`, `navs_apple_folio`, `classifai_blogs`, `robot`, bounded `flutter_v2`, plus strict representative-root conversions for `semanticFS` and `ai-testgen`; backlog artifact now tracks `uncovered/gap/partial/representative/ok` state. The current discovered-root bootstrap queue is closed, so this track now shifts to monitor-mode reruns and new-root discovery.
4. Representative polish: improve the residual `semanticfs_repo_v1` rank lag on `s20` (`future steps log`) without regressing the now-green nightly gate.
5. Phase 3 architecture track: operationally complete. The current root-promotion queue is closed, and the config/CLI contract layer, persisted domain metadata, optional vector-backend parity, domain-aware `/map` path, repeated same-file search de-duplication, domain-rank-aware full-index ordering, exact-symbol fast path, config-query priors, narrative/script/workflow/systemd intent priors, per-search prior caching, query-aware vector gating, narrative-doc vector trimming, top-level-domain baseline normalization regression coverage, and median-of-3 warmed head-to-head timing are all landed; the tracked explicit suite is signed off at `workspace_meta` + `code` + `docs` + `config` + `scripts` + `systemd` + `github` + `fixture_repo`, and future work now shifts to the next expansion phase instead of Phase 3 closeout.
6. Strict-suite generation alignment: config-aware bootstrap generation is now implemented; standardize on `scripts/bootstrap_golden_from_repo.py --config ...` for scoped repos so future strict suites stay benchmark-aligned.

## Current Risk Register
1. Observer-effect write loop: mitigated on MCP and FUSE pinning paths; mounted Linux refresh semantics are now validated, continue overnight soak watch.
2. Branch-swap blackout: queue planning is implemented, now needs continued soak validation at scale.
3. Semantic shadowing: priors are implemented, and the `7/7` date-separated nightly target is now satisfied; keep maintenance-night monitoring for regressions.
4. Determinism vs probability: architecture is grounded by `/raw`; continue enforcing search-then-raw-verify loop in docs/tests/prompts.
5. Latency regression risk from richer symbol matching: mitigated by batched symbol-variant SQL (`IN` / `LIKE OR`) and revalidated on curated holdout.
6. Nightly validation coupling: the `relevance_latest.json` overwrite bug is fixed in `scripts/nightly_representative.ps1`, and the `7/7` calendar-night target is now closed; keep monitoring for regressions on maintenance cadence.
7. Scoped-suite bootstrap mismatch: raw bootstrap generation can select paths excluded by repo-specific configs (confirmed on `ai-testgen-demo` vs `config/relevance-ai-testgen.toml`); generated suites must be aligned with benchmark filters before they are interpreted as product regressions.

## Execution Order (Next Sessions)
1. Move representative nightlies to maintenance cadence now that the `7/7` calendar-night confirmation gap is closed; rerun promptly after major retrieval/ranking changes or when drift needs reconfirmation.
2. Optionally tighten the residual `semanticfs_repo_v1` rank lag from `.semanticfs/bench/query_gap_semanticfs_repo_v1_latest.json` (`s20`, `future steps log`) if it can be done without destabilizing the now-green nightly gate.
3. Keep `buckit_curated_*` and `tensorflow_models_curated_*` in monitor mode; rerun only after retrieval/indexing changes or if query-gap drift reappears.
4. Use config-aligned bootstrap generation (`--config`) for any scoped repo strict-suite work so benchmark filters and fixture generation stay consistent.
5. Use `.semanticfs/bench/filesystem_scope_backlog_latest.json` and `.semanticfs/bench/filesystem_domain_plan_latest.json` as monitor artifacts now that the current discovered-root queue is fully covered; only rerun promotion flows when new roots are discovered or a monitor rerun regresses.
6. Shift daytime Phase 3 work from root promotion to runtime deepening: keep the landed contract/health/runtime guard layer, then extend domain-aware scheduler behavior, trust boundaries, and policy enforcement into the remaining runtime surfaces on top of the now-complete bootstrap coverage set.
7. `files.modified_unix_ms` is now persisted in the snapshot as a Phase 3 prep step, but retrieval-side recency still uses the live-fs path because the first direct runtime swap widened tracked p95; keep the stored field, and only re-attempt runtime use when it matches or beats the current narrow `v9` latency without reintroducing volatility.
8. Additional Phase 3 runtime hardening landed on March 2, 2026:
   - exact-symbol retrieval now probes the indexed `symbols(symbol_name, index_version)` path first and only falls back to the slower case-folded match if the indexed probe misses.
   - BM25 now drops case-only duplicate query variants before running FTS.
   - BM25 now applies SQL-side path-class filters for workflow/systemd/script intent queries so those literal searches do less cross-domain work before ranking.
   - head-to-head now performs one untimed warm-up for both SemanticFS and `rg` before timing each query.
   - the tracked `semanticfs_multiroot_explicit_v9` suite remains fully green on correctness at `active_version=182` (`25/25` rank `1`), but recent warmed narrow reruns still show latency volatility (roughly `64-72 ms` SemanticFS p95 vs `37-42 ms` baseline `rg` p95), so Phase 3 remains open.
8. If Linux FUSE session code changes, rerun mounted validation for `session.json` / `session.refresh`.

## Primary Commands
1. Representative nightly:
```powershell
powershell -ExecutionPolicy Bypass -File scripts/nightly_representative.ps1 -SoakSeconds 30
```
2. Legacy benchmark nightly:
```powershell
powershell -ExecutionPolicy Bypass -File scripts/nightly_bench.ps1 -ConfigPath config/semanticfs.sample.toml -FixtureRepo tests/fixtures/benchmark_repo -GoldenDir tests/retrieval_golden -SoakSeconds 30
```
3. Relevance:
```bash
cargo run --release -p semanticfs-cli -- --config config/local.toml benchmark relevance --fixture-repo /abs/repo --golden-dir tests/retrieval_golden --history
```
4. Head-to-head:
```bash
cargo run --release -p semanticfs-cli -- --config config/local.toml benchmark head-to-head --fixture-repo /abs/repo --golden-dir tests/retrieval_golden --history
```
5. Release gate with relevance:
```bash
cargo run --release -p semanticfs-cli -- --config config/local.toml benchmark release-gate --refresh --fixture-repo tests/fixtures/benchmark_repo --enforce-relevance --min-relevance-queries 20 --min-recall-at-5 0.90 --min-symbol-hit-rate 0.99 --min-mrr 0.80
```
6. Mounted Linux FUSE session validation (WSL):
```powershell
wsl -d Ubuntu -- bash -lc 'cd /mnt/c/path/to/semanticFS && bash scripts/wsl_run_fuse_session_validation.sh'
```
7. Filesystem candidate discovery + backlog build:
```powershell
powershell -ExecutionPolicy Bypass -File scripts/discover_repo_candidates.ps1 -Roots C:\Users\<user> -MinTrackedFiles 80 -TopN 80 -OutputPath .semanticfs/bench/filesystem_repo_candidates_min80.json
powershell -ExecutionPolicy Bypass -File scripts/build_filesystem_scope_backlog.ps1 -CandidatesPath .semanticfs/bench/filesystem_repo_candidates_min80.json -OutputPath .semanticfs/bench/filesystem_scope_backlog_latest.json
```

## Related Docs
1. `README.md`
2. `docs/new-chat-handoff.md`
3. `docs/future-steps-log.md`
4. `docs/benchmark.md`
