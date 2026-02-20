# SemanticFS v1.2 Execution Plan

Last updated: February 20, 2026

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

## Latest Progress Snapshot (February 20, 2026)
1. Relevance history counts:
   - `semanticfs_repo_v1=21`
   - `ai_testgen_repo_v1=20`
2. Head-to-head history counts:
   - `semanticfs_repo_v1=9`
   - `ai_testgen_repo_v1=8`
3. Last-7 head-to-head delta trend (`SemanticFS - rg`):
   - `semanticfs_repo_v1`: delta MRR `min/avg=0.3833/0.4387`, delta recall `0.1000/0.1000`, delta symbol-hit `0.7143/0.8776`, delta p95 `-16.675/-14.038 ms`.
   - `ai_testgen_repo_v1`: delta MRR `min/avg=0.0875/0.1179`, delta recall `0.1000/0.1143`, delta symbol-hit `0.0000/0.0000`, delta p95 `-18.261/-16.094 ms`.
4. Mounted Linux FUSE validation:
   - Real mounted workflow now passes end-to-end in WSL (`VALIDATION_OK`) for `/.well-known/session.json` and `/.well-known/session.refresh`.
   - Verified stale detection and refresh behavior across real index version transitions (`136 -> 137`, `138 -> 139`).
5. Calendar-night representative run status:
   - Date-separated nightly coverage now includes February 18 and February 19 (`2/7` complete).
   - Representative nightly runs on covered dates passed relevance, head-to-head, and strict release-gate with no drift trigger observed.
6. Additional larger-repo exploratory snapshot (bootstrap suites, daytime):
   - `buckit_bootstrap_v1`: SemanticFS recall `1.00`, MRR `0.9417`, symbol-hit `0.90`, p95 `35.004 ms`; baseline `rg` recall `1.00`, MRR `0.7875`, symbol-hit `0.60`, p95 `52.288 ms`.
   - `tensorflow_models_bootstrap_v1`: SemanticFS recall `0.90`, MRR `0.7542`, symbol-hit `0.65`, p95 `49.465 ms`; baseline `rg` recall `0.95`, MRR `0.7892`, symbol-hit `0.70`, p95 `143.105 ms`.
   - Note: these are bootstrap suites for exploratory signal, not yet acceptance-grade curated golden sets.
7. New strict holdout results (daytime tune-vs-holdout protocol):
   - `buckit_bootstrap_v1_holdout_v1` (selected candidate: `base`):
     - SemanticFS recall `1.00`, MRR `1.0000`, symbol-hit `1.00`, p95 `39.054 ms`
     - baseline `rg` recall `1.00`, MRR `0.8033`, symbol-hit `0.70`, p95 `39.948 ms`
   - `tensorflow_models_bootstrap_v1_holdout_v1` (selected candidate: `code_focus`):
     - SemanticFS recall `0.90`, MRR `0.8500`, symbol-hit `0.80`, p95 `48.205 ms`
     - baseline `rg` recall `0.90`, MRR `0.8200`, symbol-hit `0.80`, p95 `177.929 ms`
8. Interpretation:
   - Same-day reliability trend is favorable.
   - Mounted Linux session semantics are now validated in a real long-lived session.
   - Date-separated overnight trend evidence is in progress (`2/7` complete).
   - Holdout protocol is now in place, reducing overfit risk for daytime tuning.

## Active Remaining Work
1. Calendar-night stability confirmation: continue one representative run per night for 7 date-separated nights, then triage relevance/latency/RSS drift (`5 additional nights required`).
2. Larger-repo validation hardening: grow the new tune/holdout suites from `10/10` bootstrap splits to curated acceptance-grade sets (`>=30` queries per split, stable non-symbol + symbol coverage) before using them as release-gate evidence.
3. TensorFlow-models quality tuning follow-up: investigate the remaining holdout miss (`build_losses`) and improve recall/MRR without sacrificing latency advantage.

## Current Risk Register
1. Observer-effect write loop: mitigated on MCP and FUSE pinning paths; mounted Linux refresh semantics are now validated, continue overnight soak watch.
2. Branch-swap blackout: queue planning is implemented, now needs continued soak validation at scale.
3. Semantic shadowing: priors are implemented, with positive same-day trend; still requires date-separated nightly evidence.
4. Determinism vs probability: architecture is grounded by `/raw`; continue enforcing search-then-raw-verify loop in docs/tests/prompts.

## Execution Order (Next Sessions)
1. Run exactly one representative nightly sequence per calendar night and collect 7 date-separated trend artifacts.
2. Triage drift and regressions from `relevance/head_to_head/release_gate` history.
3. Expand and curate larger-repo tune/holdout suites (`buckit`, `tensorflow/models`) and rerun strict holdout evaluations.
4. If Linux FUSE session code changes, rerun mounted validation for `session.json` / `session.refresh`.

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

## Related Docs
1. `README.md`
2. `docs/new-chat-handoff.md`
3. `docs/future-steps-log.md`
4. `docs/benchmark.md`
