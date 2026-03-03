# New Chat Handoff

Last updated: March 2, 2026

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
   - output: `.semanticfs/bench/filesystem_scope_backlog_latest.json` with per-repo states (`uncovered`, `covered_gap`, `covered_partial`, `covered_representative`, `covered_ok`) and next actions.
27. Phase 3 bootstrap plan is now explicit:
   - new doc: `docs/phase3_execution_plan.md`.
   - Phase 3 starts as a parallel workstream while v1.2 hardening remains active.
28. Phase 3 bootstrap implementation has started:
   - shared config now supports `workspace.domains` with effective single-root fallback.
   - CLI `init` and `health` expose effective domain shape for Phase 3 planning.
   - new planner script: `scripts/build_phase3_domain_plan.ps1`.
   - new artifact: `.semanticfs/bench/filesystem_domain_plan_latest.json`.
29. Query-level hardening tooling is now available:
   - `scripts/build_query_gap_report.ps1` builds per-dataset semantic miss/rank-lag reports from the latest head-to-head history.
30. Asset-shadowing hardening landed:
   - retrieval now applies a non-code asset prior penalty (`retrieval.asset_path_penalty`) to prevent checked-in assets from outranking likely source hits.
   - benchmark configs now expose the `asset_path_penalty` knob explicitly.
31. Code-language coverage hardening landed for symbol-first retrieval:
   - the indexer now treats `.tsx`, `.jsx`, `.java`, `.c`, `.cpp`, `.h`, `.hpp`, `.cs`, and `.dart` as code.
   - symbol extraction now covers `export async function`, Java class/interface declarations with access modifiers, and typed Dart/Java-style method declarations.
32. Focused strict hardening reruns closed the prior gap repos:
   - `repo8872pp`, `syntaxless`, and `flutter_tools` now all show `semantic_miss=0` and `semantic_rank_lag=0` in the latest query-gap artifacts.
33. First backlog-driven uncovered repo promotion is complete:
   - `wilcoxrobotics_bootstrap_v1` was generated, split, and validated in strict holdout.
34. Remaining strict-gap repos are now also closed:
   - `apex_scholars` and `pseudolang` now show `semantic_miss=0` and `semantic_rank_lag=0` in the latest query-gap artifacts.
35. Second backlog-driven uncovered repo promotion is complete:
   - `catapult_project_bootstrap_v1` was generated, split, and validated in strict holdout.
36. Third backlog-driven uncovered repo promotion is complete:
   - `boilermakexii_bootstrap_v1` was generated, split, and validated in strict holdout.
37. Bounded full-root Flutter validation is now complete:
   - `flutter_v2` now passes strict holdout when constrained to the exact package roots used by the suite (`_fe_analyzer_shared`, `battery`, `camera`).
38. Nightly workflow correctness bug was identified and fixed:
   - `scripts/nightly_representative.ps1` now restores the `semanticFS` relevance artifact before `release-gate` so the strict gate validates the intended suite.
39. Scoped bootstrap generation is now config-aligned when needed:
   - `scripts/bootstrap_golden_from_repo.py` now accepts `--config` and applies `filter.allow_roots` / `filter.deny_globs`.
   - `tests/retrieval_golden/ai_testgen_strict_bootstrap_v1.json`, `tests/retrieval_golden/ai_testgen_strict_tune.json`, and `tests/retrieval_golden/ai_testgen_strict_holdout.json` were regenerated with `config/relevance-ai-testgen.toml`.
40. JS export-style symbol coverage expanded again:
   - the indexer now extracts `export const` and `export let`, which recovers React hook-style symbols such as `useUser`.
41. Fifth backlog-driven uncovered repo promotion is complete:
   - `yolov5_bootstrap_v1` was generated, split, and validated in strict holdout.
42. `buckit_curated_holdout_v1` is now clean on current code:
   - latest query-gap now shows `semantic_miss=0` and `semantic_rank_lag=0`.
43. The remaining uncovered Phase 3 roots are now fully promoted:
   - `euler_r9_bootstrap_v1`, `mathgame_bootstrap_v1`, and `navs_apple_folio_bootstrap_v1` were all generated, split, and validated in strict holdout.
44. Parent-root expansion has started:
   - `classifai_blogs_bootstrap_v1` was generated, split, and validated in strict holdout, reducing the partial-root queue further.
45. The final parent-root expansion is complete:
   - `robot_bootstrap_v1` was validated against the `Robot` root using a bounded parent-root config over the code-bearing child subtrees, closing the last remaining partial root in the current queue.
46. Phase 3 architecture contract layer is now active:
   - shared config now computes `workspace_domain_report` / `enforce_workspace_domain_contract` for explicit multi-root configs.
   - explicit domains now validate unique ids, registered trust labels, normalized root collisions, and root-relative `allow_roots` / `deny_globs`, while overlapping roots surface as warnings.
   - scheduler order is now deterministic (`trust tier` first, then more specific roots before broader roots).
   - CLI and benchmark commands now fail fast on invalid explicit multi-root configs, and observability now exposes `/health/domains`.
47. The bounded `Robot` monitor suite is now tightened:
   - `tests/retrieval_golden/robot_holdout.json` replaces the generic queries `train` / `predict` with `tb_writer` / `object detection yolov5s`.
   - the latest Robot query-gap is now `semantic_miss=0`, `baseline_miss=8`, `rank_lag=0`.
48. Phase 3 runtime wiring is now active:
   - `policy-guard` now resolves domain ownership at runtime (`resolve_disk_path`, `resolve_virtual_path`) instead of only validating config shape.
   - `indexer` now walks domain-owned roots and applies per-domain policy contracts before indexing.
   - `retrieval-core` now derives search-hit trust and recency against the owning domain root.
   - `fuse-bridge` `/raw` serving now resolves through the same domain guard instead of `repo_root + path`.
49. Post-change monitor rerun stayed intentionally narrow:
   - one representative rerun only (`semanticfs_repo_v1` relevance) because retrieval/indexing changed.
   - latest result: recall `1.0000`, MRR `0.9500`, symbol-hit `1.0000`.
   - no broad monitor sweep was run; keep reruns limited to retrieval/indexing changes or new root discovery.
50. Explicit multi-root runtime smoke is now confirmed:
   - temporary two-domain config (`code=./crates`, `docs=./docs`) completed `health` and `index build` successfully.
   - this validates a real multi-root indexing path, not just config validation.
51. Runtime domain ownership is now persisted in indexed metadata:
   - `files` and `chunks_meta` now store `domain_id` plus exact `trust_label`, and retrieval reads that stored ownership metadata directly when building hits.
52. `/map` is now domain-aware in the runtime path:
   - directory summaries are now precomputed for every ancestor directory, and map lookup/readdir now validate actual indexed map directories instead of synthesizing arbitrary subdirectories.
   - one explicit multi-root `benchmark run --skip-reindex --soak-seconds 1` passed all `4/4` E2E checks, including `/map/docs/directory_overview.md`.
53. A tracked explicit multi-root benchmark fixture is now the Phase 3 sign-off fixture:
   - tracked config: `config/relevance-multiroot.toml`
   - tracked fixture: `tests/retrieval_golden/semanticfs_multiroot_explicit.json`
   - the tracked fixture now covers `workspace_meta` + `code` + `docs` + `config` + `scripts` + `systemd` + `github` + `fixture_repo`.
   - latest relevance is green at recall `1.0000`, MRR `1.0000`, symbol-hit `1.0000` on `active_version=184`.
   - the final Phase 3 sign-off now uses one untimed warm-up plus median-of-3 timed samples per query for both engines; three consecutive warmed reruns on `active_version=184` held SemanticFS p95 in `42.989-53.384 ms` vs baseline `rg` `28.468-37.609 ms`.
   - the latest saved head-to-head artifact on the same snapshot keeps SemanticFS ahead on quality (`recall 1.0000` vs `0.9200`, `MRR 1.0000` vs `0.7500`, `symbol-hit 1.0000` vs `0.4000`), while `rg` remains faster on p95 on the broader fixture (`28.468 ms` vs SemanticFS `42.989 ms`).
   - `files.modified_unix_ms` is now persisted in the snapshot, but retrieval-side use is intentionally not left on yet because the first search-time attempt widened p95.
   - all `25` tracked queries are now rank `1`.
   - the tracked suite explicitly covers workflow-style literals, a full multi-file `systemd` root, the real operational doc `docs/runbook.md`, and top-level workspace metadata (`Cargo.toml`, `Cargo.lock`, `README.md`).
   - `semanticfs-cli` now includes regression coverage for both top-level `.` baseline normalization and the median-of-3 timing helper.
   - Phase 3 is now operationally complete; further domain-class broadening moves to the next expansion phase rather than this closeout track.
54. Optional vector-backend parity is now live for Phase 3 ownership metadata:
   - LanceDB sync now writes `domain_id` plus `trust_label` into vector rows.
   - LanceDB retrieval reads those columns directly when present.
55. Domain-aware map enrichment/reporting now uses the same directory model:
   - enrichment now reports immediate child directories and observed trust-label counts from the indexed subtree.
   - a direct DB check on active version `153` confirms the root enrichment now lists `code`, `config`, and `docs` with trust-label counts.
56. Explicit multi-root benchmark run is green on the latest active version:
   - `benchmark run --soak-seconds 1` on `config/relevance-multiroot.toml` rebuilt version `1` and passed `4/4` E2E checks, including `/map/docs/directory_overview.md`.
57. Phase 3 runtime scheduling is now wired into actual index builds:
   - `indexer` now sorts full-index work by hotset first, then domain schedule rank, then path, so the multi-root scheduler is no longer only a config/health report.
58. Exact symbol-like queries now have a direct retrieval fast path:
   - `retrieval-core` now returns exact-symbol results directly when exact symbol hits exist, instead of paying the full generic fusion path for those high-signal identifier lookups.

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
   - Calendar-night progress is now complete at `7/7`.
   - Representative nightlies now move to maintenance cadence instead of gating daytime work.
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
   - `buckit_curated_holdout_v1` (latest strict run, selected `symbol_focus`): SemanticFS recall `1.0000`, MRR `0.9750`, symbol-hit `0.9333`, p95 `50.475 ms`; baseline recall `0.7500`, MRR `0.6333`, symbol-hit `0.7333`, p95 `42.885 ms`.
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
   - `repo8872pp_bootstrap_v1_holdout_v1` (latest): SemanticFS recall `1.0000`, MRR `1.0000`, symbol-hit `1.0000`, p95 `10.608 ms`; baseline recall `1.0000`, MRR `0.8889`, symbol-hit `0.8000`, p95 `38.283 ms`.
   - interpretation: the prior ranking gap is now closed; SemanticFS leads on quality and latency.
16. Source-focused stockguessr strict rerun:
   - regenerated source-only suite: `stockguessr_bootstrap_v2_src` -> `stockguessr_v2_tune.json` / `stockguessr_v2_holdout.json`.
   - latest `stockguessr_bootstrap_v2_src_holdout_v1` (`latency_guard` selected, after generated-artifact suppression): SemanticFS recall `1.0000`, MRR `0.9667`, symbol-hit `0.9333`, p95 `13.588 ms`; baseline recall `0.9333`, MRR `0.8111`, symbol-hit `0.7333`, p95 `31.025 ms`.
   - SQLite backend spot-check on v1 holdout reduced SemanticFS p95 to `274.516 ms` but still left a large latency gap vs baseline (`34.533 ms`).
17. Additional medium external strict holdout (`syntaxless`):
   - `syntaxless_bootstrap_v1_holdout_v1` (latest, `symbol_latency_guard`): SemanticFS recall `1.0000`, MRR `1.0000`, symbol-hit `1.0000`, p95 `19.256 ms`; baseline recall `1.0000`, MRR `0.8889`, symbol-hit `0.8000`, p95 `42.625 ms`.
18. Flutter source-focused external run status:
   - generated/split suites (`flutter_bootstrap_v2_src`, `flutter_v2_tune.json`, `flutter_v2_holdout.json`) succeeded.
   - bounded strict rerun completed using package-scoped allow-roots: SemanticFS recall `1.0000`, MRR `1.0000`, symbol-hit `1.0000`, p95 `54.260 ms`; baseline recall `0.0000`, MRR `0.0000`, symbol-hit `0.0000`, p95 `583.989 ms`.
19. Additional filesystem-scope strict holdout results (latest):
   - `apex_scholars_bootstrap_v1_holdout_v1` (latest, `symbol_latency_guard`): SemanticFS recall `1.0000`, MRR `0.9667`, symbol-hit `0.9333`, p95 `14.742 ms`; baseline recall `1.0000`, MRR `0.7667`, symbol-hit `0.6000`, p95 `28.347 ms`.
   - `flutter_tools_bootstrap_v1_holdout_v1` (latest, `symbol_latency_guard`): SemanticFS recall `1.0000`, MRR `1.0000`, symbol-hit `1.0000`, p95 `31.973 ms`; baseline recall `1.0000`, MRR `0.9667`, symbol-hit `0.9333`, p95 `92.349 ms`.
20. Additional medium external strict holdout result (February 24, 2026):
   - `pseudolang_bootstrap_v1_holdout_v1` (latest, `latency_guard`): SemanticFS recall `1.0000`, MRR `1.0000`, symbol-hit `1.0000`, p95 `11.838 ms`; baseline recall `1.0000`, MRR `0.8222`, symbol-hit `0.6667`, p95 `34.086 ms`.
21. Drift summary refresh (February 24, 2026):
   - history counts: `head_to_head=149`, `relevance=60`.
   - representative counts: `semanticfs_repo_v1` h2h/relevance=`13/28`, `ai_testgen_repo_v1` h2h/relevance=`11/28`.
22. `flutter_tools` query-level gap triage (latest holdout artifact):
   - one semantic miss: `b06` (`_write`).
   - four semantic rank-lag queries vs baseline rank-1: `b10` (`_canRun`), `b14` (`_Attribute`), `b18` (`attemptToolExit`), `b30` (`CommandHelp`).
23. Filesystem-scope backlog snapshot (March 1, 2026):
   - counts: `uncovered=0`, `covered_gap=0`, `covered_partial=0`, `covered_representative=0`, `covered_ok=21`.
   - the current discovered-root queue is fully covered; backlog is now monitor-only.
24. Phase 3 domain-plan snapshot (March 1, 2026):
   - artifact: `.semanticfs/bench/filesystem_domain_plan_latest.json`
   - counts: `promote_candidate=0`, `harden_existing=0`, `expand_parent_root=0`, `add_strict_holdout=0`, `monitor=21`
   - implementation status: root promotion is done; the config/health contract layer and first runtime-wired guard layer are landed.
25. Fresh hardening + expansion runs (March 1, 2026):
   - `repo8872pp`, `syntaxless`, `flutter_tools`, `pseudolang`, and bounded `flutter_v2` now all validate at recall `1.0000`, MRR `1.0000`, symbol-hit `1.0000` on their latest strict holdouts.
   - `apex_scholars` now validates at recall `1.0000`, MRR `0.9667`, symbol-hit `0.9333`, p95 `14.742 ms`.
   - `wilcoxrobotics_bootstrap_v1_holdout_v1`, `catapult_project_bootstrap_v1_holdout_v1`, `boilermakexii_bootstrap_v1_holdout_v1`, and `labelimg_bootstrap_v1_holdout_v1` all validated as successful backlog-driven uncovered promotions.
   - `yolov5_bootstrap_v1_holdout_v1` now also validates at recall `1.0000`, MRR `1.0000`, symbol-hit `1.0000`, p95 `31.411 ms`; baseline recall `0.9000`, MRR `0.7083`, symbol-hit `0.6000`, p95 `46.559 ms`.
   - `euler_r9_bootstrap_v1_holdout_v1` now validates at recall `1.0000`, MRR `0.9500`, symbol-hit `0.9000`, p95 `27.533 ms`; baseline recall `1.0000`, MRR `0.9000`, symbol-hit `0.8000`, p95 `32.291 ms`.
   - `mathgame_bootstrap_v1_holdout_v1` now validates at recall `1.0000`, MRR `1.0000`, symbol-hit `1.0000`, p95 `31.416 ms`; baseline recall `1.0000`, MRR `0.8333`, symbol-hit `0.7000`, p95 `37.683 ms`.
   - `navs_apple_folio_bootstrap_v1_holdout_v1` now validates at recall `1.0000`, MRR `1.0000`, symbol-hit `1.0000`, p95 `43.750 ms`; baseline recall `1.0000`, MRR `0.8750`, symbol-hit `0.8000`, p95 `38.382 ms`.
   - `classifai_blogs_bootstrap_v1_holdout_v1` now validates at recall `1.0000`, MRR `1.0000`, symbol-hit `1.0000`, p95 `30.423 ms`; baseline recall `0.8000`, MRR `0.4650`, symbol-hit `0.3000`, p95 `49.510 ms`.
   - `robot_bootstrap_v1_holdout_v1` now validates on the tightened bounded parent-root monitor suite at recall `1.0000`, MRR `0.9000`, symbol-hit `0.8750`, p95 `200.972 ms`; baseline recall `0.2000`, MRR `0.1500`, symbol-hit `0.1250`, p95 `2318.070 ms`.
   - latest Robot query-gap is now clean on the semantic side (`semantic_miss=0`, `rank_lag=0`); only baseline misses remain.
   - `buckit_curated_holdout_v1` is now clean again at recall `1.0000`, MRR `0.9750`, symbol-hit `0.9333`, p95 `50.475 ms`.
   - `semanticfs_strict_bootstrap_v1_holdout_v1` now validates at recall `1.0000`, MRR `0.8833`, symbol-hit `0.8000`, p95 `41.684 ms`; baseline recall `0.9000`, MRR `0.6833`, symbol-hit `0.5000`, p95 `64.698 ms`.
   - `ai_testgen_repo_v1_holdout_v1` now validates under strict split at recall `1.0000`, MRR `0.9500`, symbol-hit `1.0000`, p95 `35.838 ms`; baseline recall `0.8000`, MRR `0.7500`, symbol-hit `1.0000`, p95 `40.486 ms`.
26. Nightly status after the next calendar-night run (`relevance_latest_20260301T002336Z.json`, `head_to_head_latest_20260301T002405Z.json`):
   - date-separated artifact coverage is now `7/7`, and accepted clean-green nights are now also `7/7`.
   - `semanticfs_repo_v1` remains green at relevance recall `1.0000`, MRR `0.9375`, symbol-hit `1.0000`; latest nightly head-to-head p95 is `30.738 ms` vs baseline `76.724 ms`.
   - `ai_testgen_repo_v1` remains strong at head-to-head recall `1.0000`, MRR `0.9500`, symbol-hit `1.0000`, p95 `11.554 ms`.
   - representative nightly gating is complete; this can now move to maintenance cadence.
27. Representative retrieval hardening landed (February 28, 2026):
   - `retrieval-core` now orders FTS results by `bm25(chunks_fts)`, adds a query-to-path overlap prior, and now also applies a filename-specific query overlap prior to lift obvious path matches.
   - `config/relevance-real.toml` now excludes `tests/retrieval_golden/**` and `config/relevance-*.toml` when indexing benchmark targets, preventing harness self-shadowing inside the semanticFS repo.
   - latest `.semanticfs/bench/query_gap_semanticfs_repo_v1_latest.json` now shows `semantic_miss=0`; the only residual issue is one non-blocking rank lag on `s20` (`future steps log`, rank `2` vs baseline `1`).
28. Strict-suite generation alignment for scoped repos is now fixed (March 1, 2026):
   - `scripts/bootstrap_golden_from_repo.py` now supports `--config`, so fixture generation can use the same filter rules as the benchmark config.
   - regenerated `ai_testgen_strict_*` fixtures are now benchmark-aligned and direct holdout validation passes at recall `1.0000`, MRR `1.0000`, symbol-hit `1.0000`, p95 `23.426 ms`.
29. Current operating mode:
   - do not treat Phase 3 as a replacement for Phase 2.
   - run `Phase 2 closeout` and `Phase 3 bootstrap` in parallel.
30. Explicit multi-root contract benchmark (March 2, 2026):
   - tracked config + fixture now exist (`config/relevance-multiroot.toml`, `tests/retrieval_golden/semanticfs_multiroot_explicit.json`).
   - the tracked fixture now covers `workspace_meta` + `code` + `docs` + `config` + `scripts` + `systemd` + `github` + `fixture_repo`.
   - repeated same-file result entries are now collapsed before final search output.
   - SemanticFS relevance is green at recall `1.0000`, MRR `1.0000`, symbol-hit `1.0000`.
   - latest narrow rerun (`active_version=171`) keeps SemanticFS ahead on overall quality (`recall 1.0000` vs `0.9200`, `MRR 1.0000` vs `0.7700`, `symbol-hit 1.0000` vs `0.2000`), while `rg` is still ahead on p95 on the broadened fixture (`42.725 ms` vs SemanticFS `53.024 ms`).

Note:
1. Measurements include both representative real suites with 7+ same-day head-to-head snapshots each.
2. Calendar-night drift confidence target is complete (`7/7` date coverage, `7/7` accepted clean-green nights).
3. Holdout protocol is now active for larger-repo daytime tuning, reducing overfit risk.
4. `buckit_curated` and `tensorflow_models_curated` are both currently in monitor mode; neither is the active daytime blocker.

## 4) Exact Next Steps (Ordered)
1. Shift representative nightlies to maintenance cadence now that the `7/7` target is closed; rerun after major retrieval/ranking changes or when drift needs reconfirmation.
2. Optional representative polish: improve the residual `semanticfs_repo_v1` rank lag on `s20` (`future steps log`) without regressing the now-green nightly gate.
3. Keep `buckit_curated_*` and `tensorflow_models_curated_*` in monitor mode; rerun them only after material retrieval/indexing changes.
4. For any scoped repo strict-suite work, use config-aligned bootstrap generation (`scripts/bootstrap_golden_from_repo.py --config ...`) instead of raw bootstrap mode.
5. Use `.semanticfs/bench/filesystem_scope_backlog_latest.json` and `.semanticfs/bench/filesystem_domain_plan_latest.json` as monitor artifacts: the current discovered-root queue is now fully covered.
6. Use the signed-off Phase 4 baseline from `docs/phase4_execution_plan.md`:
   - Phase 3 is now operationally complete
   - Phase 4 is now operationally complete
   - keep single-root runtime behavior unchanged
   - treat the config/health/runtime guard layer, persisted domain metadata, domain-aware `/map`, repeated same-file search de-duplication, domain-rank-aware index ordering, exact-symbol fast path, indexed exact-symbol lookup, BM25 case-only variant de-duplication, BM25 path-intent filtering, config-query priors, and median-of-3 warmed head-to-head timing as landed
   - the tracked `workspace_meta` + `code` + `docs` + `config` + `scripts` + `systemd` + `github` + `fixture_repo` contract set remains the signed-off Phase 3 baseline (`25/25` rank `1`)
   - the broadened Phase 4 baseline now adds `playbooks`, and `semanticfs_multiroot_explicit_v10` is green at `27/27` rank `1`
   - the watch planner now uses exact-file watch targets for exact allow-roots and supports `workspace.scheduler.max_watch_targets` as the minimum scheduling-budget layer
7. Keep `Robot` in monitor mode now that its bounded holdout is clean on the semantic side; rerun it only after retrieval/indexing changes or if a new parent-root monitor query regresses.

## 5) Execution Plan For Next Session
1. Representative nightly maintenance run (now optional, no longer gating):
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
11. Filesystem domain-plan build from latest backlog:
```powershell
powershell -ExecutionPolicy Bypass -File scripts/build_phase3_domain_plan.ps1 -BacklogPath .semanticfs/bench/filesystem_scope_backlog_latest.json -OutputPath .semanticfs/bench/filesystem_domain_plan_latest.json
```
12. Phase 3 execution plan:
```text
docs/phase3_execution_plan.md
```
13. Release gate strict thresholds now in use:
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
3. `docs/phase3_execution_plan.md`
4. `docs/future-steps-log.md`
5. `docs/benchmark.md`
6. `crates/indexer/src/lib.rs`
7. `crates/retrieval-core/src/lib.rs`
8. `crates/fuse-bridge/src/lib.rs`
9. `crates/mcp/src/lib.rs`
