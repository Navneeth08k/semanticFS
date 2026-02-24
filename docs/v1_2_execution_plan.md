# SemanticFS v1.2 Execution Plan

Last updated: February 24, 2026

## Intent
v1.2 is the reliability and quality release after v1.1 foundation hardening.
Primary objective: validate and stabilize SemanticFS on representative real-repo workloads.

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

## Latest Progress Snapshot (February 24, 2026)
1. Relevance history counts:
   - `semanticfs_repo_v1=28`
   - `ai_testgen_repo_v1=28`
2. Head-to-head history counts:
   - `semanticfs_repo_v1=13`
   - `ai_testgen_repo_v1=11`
3. Last-7 head-to-head delta trend (`SemanticFS - rg`):
   - `semanticfs_repo_v1`: delta MRR `min/avg=0.1833/0.3244`, delta recall `0.1000/0.1286`, delta symbol-hit `0.1429/0.5408`, delta p95 `-16.608/-9.253 ms`.
   - `ai_testgen_repo_v1`: delta MRR `min/avg=0.0875/0.1456`, delta recall `0.1000/0.1143`, delta symbol-hit `0.0000/0.0000`, delta p95 `-19.976/-17.105 ms`.
4. Mounted Linux FUSE validation:
   - Real mounted workflow now passes end-to-end in WSL (`VALIDATION_OK`) for `/.well-known/session.json` and `/.well-known/session.refresh`.
   - Verified stale detection and refresh behavior across real index version transitions (`136 -> 137`, `138 -> 139`).
5. Calendar-night representative run status:
   - Date-separated nightly coverage now includes February 18, February 19, February 21, February 22, and February 24 (`5/7` complete).
   - Representative nightly runs on covered dates passed relevance, head-to-head, and strict release-gate with no drift trigger observed.
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
   - `buckit_curated_holdout_v1` (latest strict holdout): SemanticFS leads baseline on recall/MRR/symbol-hit, with a p95 latency tradeoff (`61.320 ms` vs `52.228 ms`).
   - `tensorflow_models_curated_holdout_v1` (latest strict holdout): SemanticFS leads baseline on recall/MRR/symbol-hit and p95 after retrieval + symbol hardening.
9. External strict holdout expansion:
   - `rlbeta_bootstrap_v1_holdout_v1`: strong quality + major latency win.
   - `stockguessr_bootstrap_v1_holdout_v1`: SemanticFS beats baseline on quality (baseline near-zero) but has higher p95 latency.
   - `stockguessr_bootstrap_v2_src_holdout_v1` (source-focused, latest): SemanticFS now leads baseline on recall/MRR/symbol-hit and p95 after generated-artifact suppression.
   - `repo8872pp_bootstrap_v1_holdout_v1`: SemanticFS is much faster than baseline but trails on MRR and symbol hit-rate.
   - `syntaxless_bootstrap_v1_holdout_v1`: SemanticFS matches recall and wins p95 latency, while baseline currently leads on MRR/symbol-hit.
   - `apex_scholars_bootstrap_v1_holdout_v1`: SemanticFS leads on MRR/symbol-hit and latency, with small recall deficit.
   - `flutter_tools_bootstrap_v1_holdout_v1`: SemanticFS leads strongly on latency but trails baseline on recall/MRR/symbol-hit.
   - `pseudolang_bootstrap_v1_holdout_v1`: SemanticFS is faster with near-par quality (small recall/MRR deficit, symbol-hit parity).
10. `build_losses` disambiguation check:
   - previous TensorFlow holdout miss was due ambiguous ground truth (`build_losses` appears across many files).
   - after expected-path disambiguation, both engines hit and SemanticFS keeps strong latency advantage (`p95 45.890 ms` vs `157.252 ms`).
11. Interpretation:
   - Same-day reliability trend is favorable.
   - Mounted Linux session semantics are now validated in a real long-lived session.
   - Date-separated overnight trend evidence is in progress (`5/7` complete).
   - Curated TensorFlow holdout quality objective is now met with preserved latency advantage in latest strict run.
   - Holdout protocol remains in place, reducing overfit risk for daytime tuning.
   - Generated-artifact suppression closed the stockguessr_v2 external source gap.
   - Remaining external quality gaps are now concentrated in `repo8872pp`, `syntaxless`, and `flutter_tools`, while `flutter_v2` still needs bounded completion.
12. Filesystem-scope planning status:
   - discovery noise is reduced (workspace mirrors + mirrored clone dedupe).
   - backlog now separates repos into `uncovered`, `covered_gap`, `covered_partial`, and `covered_ok`, enabling deterministic daytime queueing.
   - immediate backlog focus should be: highest-size `uncovered` repos, then `covered_gap` repos.

## Active Remaining Work
1. Calendar-night stability confirmation: continue one representative run per night for 7 date-separated nights, then triage relevance/latency/RSS drift (`2 additional nights required`).
2. Larger-repo validation hardening: keep curated suite quality improving (reduce ambiguous/easy queries and add stronger non-symbol intent coverage) before using these suites as release-gate evidence.
3. Filesystem-scope prep track: external strict coverage now includes `rlbeta`, `stockguessr_v1`, `stockguessr_v2`, `repo8872pp`, `syntaxless`, `apex_scholars`, `flutter_tools`, and `pseudolang`; backlog artifact now tracks `uncovered/gap/partial/ok` state. Next step is backlog-driven expansion on top uncovered repos plus quality recovery on `repo8872pp`/`syntaxless`/`flutter_tools`, and bounded completion for `flutter_v2`.

## Current Risk Register
1. Observer-effect write loop: mitigated on MCP and FUSE pinning paths; mounted Linux refresh semantics are now validated, continue overnight soak watch.
2. Branch-swap blackout: queue planning is implemented, now needs continued soak validation at scale.
3. Semantic shadowing: priors are implemented, with positive same-day trend; still requires date-separated nightly evidence.
4. Determinism vs probability: architecture is grounded by `/raw`; continue enforcing search-then-raw-verify loop in docs/tests/prompts.
5. Latency regression risk from richer symbol matching: mitigated by batched symbol-variant SQL (`IN` / `LIKE OR`) and revalidated on curated holdout.

## Execution Order (Next Sessions)
1. Run exactly one representative nightly sequence per calendar night and collect 7 date-separated trend artifacts.
2. Triage drift and regressions from `relevance/head_to_head/release_gate` history.
3. Refine curated larger-repo suites (`buckit`, `tensorflow/models`) and rerun strict holdout evaluations when query sets are updated.
4. Use `.semanticfs/bench/filesystem_scope_backlog_latest.json` as daytime queue source: run top uncovered repos first, then covered-gap repos.
5. Prioritize external-gap triage on `repo8872pp` and `syntaxless`, and complete one bounded `flutter_v2` strict run (narrowed scope) before adding further external suites.
6. Add targeted triage pass for `flutter_tools` holdout misses (query-level root cause and candidate/profile refinement) while preserving latency advantage.
7. If Linux FUSE session code changes, rerun mounted validation for `session.json` / `session.refresh`.

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
