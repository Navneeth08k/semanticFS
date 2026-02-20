# Future Steps Log

Last updated: February 20, 2026

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
   - summary: same-day run-count target is done (`head-to-head: semanticfs_repo_v1=8, ai_testgen_repo_v1=7`), date-separated progress is now `2/7` nights complete, continue until 7 nights and analyze relevance/latency/RSS drift.

2. Curated larger-repo validation suites (post-bootstrap)
   - phase: v1.2
   - status: active
   - source: daytime exploratory expansion request
   - summary: strict tune/holdout files now exist and were exercised for `buckit` and `tensorflow/models`; next step is to expand beyond `10/10` bootstrap splits into stable curated acceptance-grade suites and keep holdout isolated from tuning.

3. TensorFlow-models targeted relevance follow-up
   - phase: v1.2
   - status: active
   - source: daytime tune-vs-holdout run
   - summary: holdout is now favorable vs baseline on MRR/latency, but one holdout query (`build_losses`) still misses; investigate symbol extraction/ranking behavior and improve without regressions.

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
