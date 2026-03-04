# Current Execution Plan

Last updated: March 4, 2026

## Purpose
This is the active implementation reference.
Use it as the single current-phase plan.

## Current Phase
Phase: `Phase 8 scaling, home-scope validation, and productization`

Status:
1. Active.
2. Phase 3 through Phase 7 are treated as completed baselines.
3. The main technical problem is now broadening from bounded multi-root coverage into larger user-space profiles while keeping scope controls explicit.
4. The main product problem is now packaging and simple adoption, not core retrieval correctness.

## Active Baselines
1. Frozen repo regression gates:
   - `tests/retrieval_golden/semanticfs_multiroot_explicit.json` (`v9`)
   - `tests/retrieval_golden/semanticfs_multiroot_explicit_v10.json` (`v10`)
   - `tests/retrieval_golden/semanticfs_multiroot_explicit_v11.json` (`v11`)
   - `tests/retrieval_golden/semanticfs_multiroot_explicit_v12.json` (`v12`)
   - `tests/retrieval_golden/semanticfs_multiroot_explicit_v13.json` (`v13`)
2. Active broadened repo baseline:
   - `tests/retrieval_golden/semanticfs_multiroot_explicit_v14.json` (`v14`)
3. Active home baselines:
   - exact bounded home gate: `home_pilot_v1`
   - budgeted broader home sweep: `home_sweep_v1`
   - uncapped capability proof: `home_full_v1`
   - production-shaped home profile: `home_profile_v1`

## Latest Validated State
1. Repo multi-root baseline is fully green again after removing the repo-side scan-target cap from `config/relevance-multiroot.toml`:
   - `max_scan_targets = 0`
   - `workspace_scan_target_raw_count=70`
   - `workspace_scan_target_count=70`
   - `workspace_scan_target_pruned_count=0`
2. `v14` is fully green on the repaired fixture:
   - query count `43`
   - recall `1.0000`
   - MRR `1.0000`
   - symbol-hit `1.0000`
3. The profile matrix is now clean again:
   - artifact: `.semanticfs/bench/profile_matrix_latest.json`
   - `repo_multiroot`: `1.0000 / 1.0000 / 1.0000`
   - `home_pilot`: `1.0000 / 1.0000 / 1.0000`
   - `home_sweep`: `1.0000 / 0.9545 / 1.0000`
   - `home_profile`: `1.0000 / 0.8542 / 1.0000`
4. `home_profile_v1` is now the current production-shaped home baseline:
   - domains: `home_catchall`, `home_windsurf_tooling`, `home_desktop_shallow`, `home_downloads_shallow`, `home_documents_shallow`, `home_vscode_tooling`, `home_cursor_tooling`, `home_lmstudio_tooling`, `home_codex`, `home_projects`, `home_school`, `home_robot`
   - `workspace_domain_count=12`
   - `workspace_scan_target_raw_count=25`
   - `workspace_scan_target_count=25`
   - `workspace_scan_target_pruned_count=0`
   - `workspace_scan_target_limit=32`
   - `workspace_budgeted_domain_count=3`
   - grounded query count: `32`
5. `home_profile_v1` is currently validated on `.semanticfs/bench/home_profile_v18.db`:
   - relevance: recall `1.0000`, MRR `0.8542`, symbol-hit `1.0000`
   - bounded head-to-head: SemanticFS recall `1.0000`, MRR `0.8307`, p95 `0.002 ms`
   - bounded `rg`: recall `0.7500`, MRR `0.6563`, p95 `832.532 ms`
6. Home-scope validation now has four distinct proofs:
   - exact bounded slices (`home_pilot_v1`)
   - budgeted broader home sweep (`home_sweep_v1`)
   - uncapped single-root capability proof (`home_full_v1`)
   - practical production-shaped mixed profile (`home_profile_v1`)
7. Productization is now landed:
   - sample profiles: `config/profiles/single-repo.sample.toml`, `config/profiles/multi-root-dev-box.sample.toml`, `config/profiles/home-projects.sample.toml`
   - config renderer: `scripts/apply_config_profile.ps1`
   - local installer: `scripts/install_local.ps1`
   - release packager: `scripts/package_release.ps1`
   - release readiness wrapper: `scripts/run_release_readiness.ps1`
   - setup guide: `docs/setup_10_minute_agents.md`
8. Release-readiness is now validated in one pass:
   - artifact: `.semanticfs/bench/release_readiness_latest.json`
   - rendered configs: `.semanticfs/bench/release.single-repo.toml`, `.semanticfs/bench/release.home-projects.toml`, `.semanticfs/bench/release.multi-root-dev-box.toml`
   - release bundle: `.semanticfs/releases/semanticfs-windows-x64.zip`
   - release install target: `.semanticfs/local-bin-release/semanticfs.exe`
   - installed binary health passed on the `single-repo` profile
   - packaged binary health passed on the `home-projects` and `multi-root-dev-box` profiles
   - extracted temp-bundle health also passed in a distinct environment:
     `C:\Users\navneeth\AppData\Local\Temp\semanticfs-release-smoke\semanticfs.exe`

## Current Technical Goals
1. Keep `v9` through `v14` green after retrieval, indexing, or policy changes.
2. Keep `home_pilot_v1`, `home_sweep_v1`, and `home_profile_v1` usable as bounded home regression baselines.
3. Treat `home_full_v1` as a capability proof, not the preferred production path.
4. Broaden home coverage through bounded, policy-explicit slices instead of unbounded sweeps.
5. Keep packaging simple enough that another machine can adopt the current profiles quickly.
6. Use the release-readiness wrapper as the default packaging smoke check after install/profile changes.

## Acceptance Criteria
This phase is ready to close when:
1. The repo multi-root baseline stays fully green.
2. The home baselines stay green enough to preserve the broader home-directory claim.
3. The production-shaped home profile remains the primary recommended path.
4. The install and packaging path is validated end to end on the current machine.
5. The release-readiness wrapper continues to pass on the supported profile set.

## Current Work
1. Treat `v14` as the active repo baseline and `v9` through `v13` as frozen gates.
2. Treat `home_profile_v1` as the main home-directory regression baseline.
3. Keep `home_full_v1` for capability proof only.
4. Use the profile matrix as the fast cross-profile regression check after meaningful runtime changes.
5. Use `scripts/run_release_readiness.ps1` as the default packaging and install regression check.
6. Keep the older phase docs as archive only; do not use them as active planning docs.

## Next Steps
1. Treat the current `home_profile_v1` as the supported production-shaped home baseline for this release band.
2. Treat `scripts/run_release_readiness.ps1` as the required smoke check after future profile or packaging changes.
3. If portability confidence matters next, move from the current distinct local environment smoke to a second machine.
4. If cold first-query latency becomes a real issue again, treat it as a separate tuning track; do not block the current release band on warmed-path behavior.
