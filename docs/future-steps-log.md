# Future Steps Log

Last updated: March 3, 2026

## Purpose
This is the active next-steps queue.
Keep it short. Move finished work into the phase doc or commit history instead of growing this file indefinitely.

## Current Queue
1. `active` Phase 8 scaling and pressure hardening
   - keep `v9` through `v13` green
   - keep `v14` green as the active broadened baseline
   - keep warmed repeated-query latency acceptable on `v14`
   - scheduler visibility is now landed in `health`
   - head-to-head hotspot summaries are now landed in the benchmark artifact
   - cold-path latency work is now secondary unless it becomes the bottleneck again
2. `active` Bounded home-directory pilot expansion
   - `home_pilot_v1` is now green on a broader bounded home profile of `C:\Users\navneeth`
   - current domains: `home_meta`, `home_rules`, `home_skills`, `home_root_text`, `home_desktop`, `home_lmstudio`, `home_cursor`, `home_vscode`, `home_downloads`
   - the benchmark baseline now uses explicit `scan_targets()` for small explicit multi-root sets, so home head-to-head runs stay bounded to the configured scope
   - hidden-root enforcement is now real: hidden home roots must explicitly opt in with `allow_hidden_paths = true`
   - keep the pilot bounded and explicit; do not treat this as permission for an unbounded home sweep
   - the exact-file home gate now spans multiple real user-space areas: `.codex`, `.lmstudio`, `.cursor`, `.vscode`, top-level home text files, `Desktop/Fall Guys.url`, and two exact `Downloads` text files
   - `home_sweep_v1` is now also green as the first budgeted broader home sweep on top of that gate
   - runtime fixes are now landed for the broader sweep: bounded scan-target decomposition, indexer respect for `WatchTarget.recursive`, and bounded `rg` recursion in the benchmark baseline
   - current broader sweep state: `7` domains, `16` scan targets, relevance `1.0000 / 0.9545 / 1.0000`, head-to-head `1.0000 / 0.9545 / 0.002 ms` vs `rg` `0.8182 / 0.8182 / 443.052 ms`
   - live cap probes now confirm excluded files stay out of the active SemanticFS index while bounded `rg` still finds them
   - `home_full_v1` is now also landed as the first uncapped single-root home proof
   - generated files: `.semanticfs/bench/home_full.toml`, `.semanticfs/bench/home_full_fixture.json`, `.semanticfs/bench/home_full_manifest.json`
   - `home_full_v1` uses one domain (`home_full`) rooted at `C:\Users\navneeth` with `allow_roots = ["**"]` and `max_indexed_files = 0`
   - the latest live uncapped run (`.semanticfs/bench/home_full_v12.db`) reached `4,153` file rows before manual stop: `4,106` indexed, `37` metadata-only, `10` too-large
   - the next home move is no longer proving uncapped full-home capability; it is turning that capability into a faster production profile
3. `active` Representative nightly maintenance
   - run the representative nightlies only after material retrieval/ranking changes or when drift needs reconfirmation
4. `active` Curated larger-repo monitor mode
   - keep the existing larger-repo suites in monitor mode unless retrieval changes cause drift
5. `queued` Next bounded expansion batch
   - the current batch (`telemetry`, `admission`, `triage`) is now landed
   - define the next low-risk multi-domain batch only if we want to keep broadening immediately from the new `v14` baseline

## Deferred
1. `deferred` Per-commit vector snapshots at repository scale
2. `deferred` Full multimodal retrieval default (code + design/image)

## Recent Baseline
1. Phase 3 through Phase 7 are complete and now serve as frozen baselines.
2. The current broadened baseline is `tests/retrieval_golden/semanticfs_multiroot_explicit_v14.json`.
3. The current open problem is broader-batch latency, not correctness.
4. Latest Phase 8 slice keeps `v9` through `v13` green and promotes `v14` on `active_version=199`.
5. A snapshot-scoped search-result cache is now active for repeated identical queries on the same snapshot; on the warmed benchmark path, `v14` head-to-head now measures `SemanticFS p95 0.002 ms` vs `rg` `73.005 ms`.
6. New bounded domains `telemetry`, `admission`, and `triage` are now active, and direct cap probes confirm their third `z_` files remain excluded.
7. Two earlier retrieval-side latency experiments were tested and reverted; the durable wins from this phase are better measurement plus the warmed-query cache.
8. A real home-user-space pilot is now landed and broadened:
   - generator: `scripts/build_home_pilot.ps1`
   - runner: `scripts/run_home_pilot.ps1`
   - relevance on `.semanticfs/bench/home_pilot_fixture.json`: recall `1.0000`, MRR `1.0000`, symbol-hit `1.0000`
   - broadened home profile now includes `.codex`, `.lmstudio`, `.cursor`, `.vscode`, exact top-level home text files, `Desktop/Fall Guys.url`, and two exact `Downloads` files
   - bounded head-to-head on the explicit home scan targets: SemanticFS recall `1.0000`, MRR `1.0000`, p95 `0.001 ms` vs `rg` recall `0.8824`, MRR `0.8529`, p95 `291.812 ms`
9. The current repo-side broadened baseline (`v14`) remains green after the baseline-target logic change:
   - SemanticFS recall `1.0000`, MRR `1.0000`, symbol-hit `1.0000`, p95 `0.001 ms`
   - `rg` recall `0.9535`, MRR `0.9031`, symbol-hit `0.6000`, p95 `34.149 ms`
10. A first budgeted broader home sweep is now also landed:
   - generator: `scripts/build_home_sweep.ps1`
   - runner: `scripts/run_home_sweep.ps1`
   - current broader sweep config/fixture/manifest: `.semanticfs/bench/home_sweep.toml`, `.semanticfs/bench/home_sweep_fixture.json`, `.semanticfs/bench/home_sweep_manifest.json`
   - runtime fixes that made it viable: bounded scan-target decomposition, indexer respect for `WatchTarget.recursive`, and bounded `rg` recursion in the benchmark baseline
   - relevance on `home_sweep_v1`: recall `1.0000`, MRR `0.9545`, symbol-hit `1.0000`
   - bounded head-to-head on the broader sweep: SemanticFS recall `1.0000`, MRR `0.9545`, p95 `0.002 ms` vs `rg` recall `0.8182`, MRR `0.8182`, p95 `443.052 ms`
11. Live cap probes now confirm the broader sweep is enforcing exclusion in the active index:
   - `sweep_lmstudio/mcp.json` is still found by bounded `rg`, but not by SemanticFS
   - `sweep_lmstudio/.internal/download-jobs-info.json` is still found by bounded `rg`, but not by SemanticFS
12. The remaining gap toward a stronger whole-home claim is no longer whether an uncapped home-root pass works:
   - `home_full_v1` now proves it can run as a single uncapped root under policy guardrails
   - the open problem is turnaround time on a monolithic full-home pass
   - the likely production next step is a mixed profile: one home catchall root plus explicit large-root domains
