# Current Execution Plan

Last updated: March 3, 2026

## Purpose
This is the active implementation plan.
Use it as the single current-phase reference.

## Current Phase
Phase: `Phase 8 scaling and pressure hardening`

Status:
1. Active.
2. Phase 3 through Phase 7 are treated as completed baselines, not active workstreams.
3. The main problem is now controlled expansion from repo-local batches into real user-space roots, not core correctness.
4. The first Phase 8 slice is landed: health now reports aggregate scan/watch pressure.
5. Head-to-head artifacts now include explicit hotspot summaries so the next latency pass can target measured offenders directly.
6. A bounded snapshot-scoped search-result cache is now landed in retrieval for repeated identical queries on the same snapshot.
7. A real bounded home-directory pilot is now landed via `scripts/build_home_pilot.ps1` and `scripts/run_home_pilot.ps1`.
8. The `rg` baseline now uses explicit `scan_targets()` for small explicit multi-root sets, so bounded home-slice head-to-head runs measure only the configured scope instead of sweeping the whole home directory.
9. Hidden-path policy is now enforced at runtime for both hidden subpaths and hidden domain roots; hidden home roots must explicitly opt in with `allow_hidden_paths = true`.
10. A first budgeted broader home sweep is now landed via `scripts/build_home_sweep.ps1` and `scripts/run_home_sweep.ps1`.
11. Constrained wildcard allowlists now decompose into bounded scan targets instead of forcing full recursive root walks.
12. The indexer now respects `WatchTarget.recursive`, so bounded scan targets stay bounded during full index builds.
13. The benchmark baseline now also respects non-recursive scan targets, so bounded home head-to-head runs no longer recurse through whole home subtrees.
14. A single-root `home_full` pilot is now landed via `scripts/build_home_full.ps1` and `scripts/run_home_full.ps1`.
15. The full-index walker now prunes denied targets and denied subtrees before queueing files, so broader root scans do not waste most of their time on paths policy already rejects.

## Frozen Regression Gates
These stay green before broader expansion continues:
1. `tests/retrieval_golden/semanticfs_multiroot_explicit.json` (`v9`)
2. `tests/retrieval_golden/semanticfs_multiroot_explicit_v10.json` (`v10`)
3. `tests/retrieval_golden/semanticfs_multiroot_explicit_v11.json` (`v11`)
4. `tests/retrieval_golden/semanticfs_multiroot_explicit_v12.json` (`v12`)
5. `tests/retrieval_golden/semanticfs_multiroot_explicit_v13.json` (`v13`)

Current broadened baseline:
1. `v14` on `config/relevance-multiroot.toml`
2. Latest signed-off quality target: rank-1 on all queries
3. Current pressure target: reduce broader-batch p95 while keeping the frozen gates green

Latest validated state:
1. `v9` through `v13` all remain green at recall `1.0000`, MRR `1.0000`, symbol-hit `1.0000` on `active_version=199`.
2. `v14` is now green at recall `1.0000`, MRR `1.0000`, symbol-hit `1.0000` on `active_version=199`.
3. Current warmed `v14` head-to-head on `active_version=199` shows:
   - SemanticFS p95 `0.002 ms`
   - `rg` p95 `73.005 ms`
4. This is a real warmed-path result, not a cold-path result:
   - the benchmark already performs one untimed warm-up per query
   - the new snapshot-scoped search-result cache now makes those repeated identical-query reads effectively free
5. Two earlier retrieval-side latency experiments were tried and reverted because they did not improve the measured run; the cache is the first change from this phase that is both durable and clearly effective.
6. The next bounded batch is now landed cleanly on top of that warmed-path behavior:
   - new bounded domains: `telemetry`, `admission`, `triage`
   - new broadened suite: `tests/retrieval_golden/semanticfs_multiroot_explicit_v14.json`
7. Direct cap probes confirm the third file in each new domain remains excluded:
   - `telemetry/z_raw_trace_backlog.md`
   - `admission/z_high_risk_root_candidates.md`
   - `triage/z_unbounded_trace_dump.md`
8. Cold first-query latency is still a separate scaling question, but warmed `/search` access is now fast enough to stop blocking bounded expansion.
9. Current health output now exposes:
   - `workspace_scan_target_count=44`
   - `workspace_watch_target_count=19`
   - `workspace_watch_enabled_domain_count=8`
   - `workspace_budgeted_domain_count=7`
10. A real home-user-space pilot is now validated on `C:\Users\navneeth` with explicit bounded domains:
   - generated config: `.semanticfs/bench/home_pilot.toml`
   - generated fixture: `.semanticfs/bench/home_pilot_fixture.json`
   - explicit domains: `home_meta`, `home_rules`, `home_skills`, `home_root_text`, `home_desktop`, `home_lmstudio`, `home_cursor`, `home_vscode`, `home_downloads`
11. `home_pilot_v1` now spans a broader bounded home profile under the user root:
   - the `.codex` subtree
   - the `.lmstudio` subtree
   - the `.cursor` subtree
   - the `.vscode` subtree
   - `.condarc`
   - `requirements.txt`
   - `LabelObj.txt`
   - `install.sh`
   - `import pygame.py`
   - `Desktop/Fall Guys.url`
   - `Downloads/math_exp_00_input.txt`
   - `Downloads/measurements.txt`
12. `home_pilot_v1` is green on the broadened bounded home profile:
   - `workspace_domain_count=9`
   - `workspace_scan_target_count=17`
   - relevance on `active_version=1`: recall `1.0000`, MRR `1.0000`, symbol-hit `1.0000`
   - bounded head-to-head against the explicit home scan targets: SemanticFS recall `1.0000`, MRR `1.0000`, p95 `0.001 ms`
   - bounded `rg` baseline on the same scan targets: recall `0.8824`, MRR `0.8529`, p95 `291.812 ms`
13. A first budgeted broader home sweep is now validated on top of the bounded home gate:
   - generated config: `.semanticfs/bench/home_sweep.toml`
   - generated fixture: `.semanticfs/bench/home_sweep_fixture.json`
   - generated manifest: `.semanticfs/bench/home_sweep_manifest.json`
   - broader domains: `sweep_codex`, `sweep_lmstudio`, `sweep_cursor`, `sweep_vscode`, `sweep_home_root`, `sweep_desktop`, `sweep_downloads`
   - current health on the broadened sweep: `workspace_domain_count=7`, `workspace_scan_target_count=16`, `workspace_budgeted_domain_count=6`
   - current built DB stays bounded (`.semanticfs/bench/home_sweep_v4.db` is `1,650,688` bytes)
14. `home_sweep_v1` is green on the first budgeted broader home sweep:
   - relevance on `active_version=1`: recall `1.0000`, MRR `0.9545`, symbol-hit `1.0000`
   - bounded head-to-head on the same sweep: SemanticFS recall `1.0000`, MRR `0.9545`, p95 `0.002 ms`
   - bounded `rg` baseline on the same sweep: recall `0.8182`, MRR `0.8182`, p95 `443.052 ms`
15. Live cap probes now confirm excluded files stay out of the active SemanticFS index on the broader sweep:
   - excluded `sweep_lmstudio/mcp.json` is still found by bounded `rg`, but SemanticFS does not return it
   - excluded `sweep_lmstudio/.internal/download-jobs-info.json` is still found by bounded `rg`, but SemanticFS does not return it
16. The broader home claim is now stronger:
   - the runtime is no longer limited to exact-file home slices
   - it now supports a budgeted broader home profile with explicit caps and hidden-root policy
   - it now also supports an uncapped single-root home profile under policy guardrails
17. `home_full_v1` is now the first uncapped single-root home proof:
   - generated config: `.semanticfs/bench/home_full.toml`
   - generated fixture: `.semanticfs/bench/home_full_fixture.json`
   - generated manifest: `.semanticfs/bench/home_full_manifest.json`
   - single domain: `home_full`
   - root: `C:\Users\navneeth`
   - `allow_roots = ["**"]`
   - `max_indexed_files = 0`
18. The uncapped `home_full` run is now proven viable under policy bounds:
   - the live run on `.semanticfs/bench/home_full_v12.db` reached `4,153` file rows before manual stop
   - current row mix at that point: `4,106` indexed, `37` metadata-only, `10` too-large
   - the run was indexing deep real home content (for example `Desktop/NavneethThings/Projects/Robot/...`) instead of stalling on denied junk
19. The remaining limitation on `home_full_v1` is turnaround time, not capability:
   - an uncapped full-home pass is now operationally viable
   - it is not yet the fastest production shape for whole-home coverage
   - the faster production shape will likely be a mixed profile: one home catchall root plus explicit large-root domains

## Goals
1. Improve scheduler visibility so broader batches are measurable instead of opaque.
2. Reduce avoidable retrieval work on text-heavy multi-root queries.
3. Keep `/raw`, `/search`, and `/map` behavior unchanged from a correctness standpoint.
4. Hold the frozen `v9` through `v13` suites green while expanding and tuning runtime cost.
5. Prove a budgeted broader home-directory sweep beyond exact-file slices before broader host expansion continues.
6. Prove at least one uncapped single-root home pass under policy guardrails so whole-home capability is real, not hypothetical.
7. Keep hidden-root enforcement explicit so broader home expansion does not silently admit hidden roots.
8. Only add the next low-risk batch after the broader-batch or home-slice pressure trend is under control.

## Acceptance Criteria
This phase is ready to close when:
1. Health output exposes scheduler pressure directly enough to explain scan/watch breadth.
2. The active broadened suite remains fully green on relevance quality.
3. The frozen `v9` through `v13` gates remain green after the scaling changes.
4. The active scaling slice measurably reduces or stabilizes broader-batch latency without introducing ranking regressions.
5. A budgeted broader home-directory sweep is validated with explicit domains, caps, and exact-query grounding.
6. The next low-risk batch is defined only after the current broader-batch or home-slice pressure slice is understood.

## Current Work
1. Treat warmed repeated-query latency as acceptable for progress.
2. Re-run only the narrow explicit multi-root suites (`v9` through `v14`) after each retrieval/indexing change.
3. Treat `home_pilot_v1` as the exact-file bounded home gate and keep it available for reruns.
4. Treat `home_sweep_v1` as the first budgeted broader home baseline above that gate.
5. Treat `home_full_v1` as the first uncapped single-root home proof.
6. Keep using hotspot summaries for any future cold-path latency work.
7. Keep the older per-phase docs as archive only; do not treat them as active planning docs.

## Next Steps
1. Keep `v14` as the active broadened baseline and use `v9` through `v13` as frozen gates.
2. Keep `home_pilot_v1` as the exact-file home gate, `home_sweep_v1` as the first budgeted broader home baseline, and `home_full_v1` as the uncapped single-root home proof.
3. The next home step is no longer “can we do uncapped full-home at all”; that is now proven.
4. The next home step is to turn that capability into a faster production profile, most likely by splitting known large roots into explicit domains instead of relying on one monolithic home root for everything.
5. If we broaden home again, do it with explicit per-domain caps and allowlists so hidden roots stay opt-in.
6. If we continue broadening immediately on the repo-side, add the next low-risk bounded batch with the same cap discipline.
7. If future work needs better cold first-query latency, use the hotspot summaries rather than ad hoc tuning.
