# Future Steps Log

Last updated: March 4, 2026

## Purpose
This is the active next-steps queue.
Keep it short.

## Current Queue
1. `active` Keep the repo-side multi-root baseline green
   - `v9` through `v13` are frozen gates
   - `v14` is the active broadened repo baseline
   - current repo state is fully green again at `1.0000 / 1.0000 / 1.0000`
2. `active` Keep the home baselines healthy
   - `home_pilot_v1` is the exact bounded home gate
   - `home_sweep_v1` is the budgeted broader home sweep
   - `home_profile_v1` is the production-shaped home baseline
   - `home_full_v1` remains capability proof only
3. `active` Broaden `home_profile_v1` one bounded slice at a time
   - current supported baseline now spans home root, `.windsurf`, Desktop, Downloads, Documents, `.vscode`, `.cursor`, `.lmstudio`, `.codex`, school files, project files, and a bounded `Robot` slice
   - additional broadening is no longer required for the current release band
4. `active` Preserve the adoption path
   - sample profiles, local installer, package script, `scripts/run_release_readiness.ps1`, and `docs/setup_10_minute_agents.md` are now landed
   - the one-command release-readiness check is now the default packaging smoke
   - a distinct local extracted-bundle smoke is now validated
   - next portability confidence should come from trying the packaged flow on another machine

## Deferred
1. `deferred` Cold first-query latency tuning if it becomes a real user bottleneck again
2. `deferred` Broader cross-machine validation beyond the current machine
3. `deferred` Per-commit vector snapshots at repository scale
4. `deferred` Full multimodal retrieval default (code + design/image)

## Recent Baseline
1. The repo-side scan-target cap was removed from `config/relevance-multiroot.toml`, which fixed the stale `v14` misses caused by scheduler pruning.
2. `tests/retrieval_golden/semanticfs_multiroot_explicit_v14.json` was repaired and is now fully green again at `43/43` rank `1`.
3. `.semanticfs/bench/profile_matrix_latest.json` now validates the current cross-profile state:
   - `repo_multiroot`: `1.0000 / 1.0000 / 1.0000`
   - `home_pilot`: `1.0000 / 1.0000 / 1.0000`
   - `home_sweep`: `1.0000 / 0.9545 / 1.0000`
   - `home_profile`: `1.0000 / 0.8393 / 1.0000`
4. `home_profile_v1` now includes the bounded `.windsurf` tooling slice, bringing the production-shaped home profile to `12` domains and `32` grounded queries.
   - new bounded hidden-root slice: `home_windsurf_tooling`
   - allowlist: `argv.json`, `extensions/extensions.json`
   - grounded queries: `963323f3-786c-4ab6-b003-8b613b3a6ad5`, `Codeium.windsurfpyright`
   - `workspace_scan_target_limit` was raised to `32` so the broader home profile stays unpruned
   - new bounded hidden-root slice: `home_vscode_tooling`
   - allowlist: `argv.json`, `extensions/extensions.json`
   - hidden-root opt-in remains explicit via `allow_hidden_paths = true`
   - additional bounded hidden-root slice: `home_cursor_tooling`
   - allowlist: `argv.json`, `extensions/extensions.json`
   - additional bounded hidden-root slice: `home_lmstudio_tooling`
   - allowlist: `mcp.json`, `.internal/backend-preferences-v1.json`
5. `.semanticfs/bench/profile_matrix_latest.json` now reflects the current cross-profile state on `active_version=6`:
   - `repo_multiroot`: `1.0000 / 1.0000 / 1.0000`
   - `home_pilot`: `1.0000 / 1.0000 / 1.0000`
   - `home_sweep`: `1.0000 / 0.9545 / 1.0000`
   - `home_profile`: `1.0000 / 0.8542 / 1.0000`
6. The release-readiness flow is now validated end to end:
   - artifact: `.semanticfs/bench/release_readiness_latest.json`
   - rendered configs: `.semanticfs/bench/release.single-repo.toml`, `.semanticfs/bench/release.home-projects.toml`, `.semanticfs/bench/release.multi-root-dev-box.toml`
   - packaged bundle: `.semanticfs/releases/semanticfs-windows-x64.zip`
   - installed binary: `.semanticfs/local-bin-release/semanticfs.exe`
   - health checks passed on `single-repo`, `home-projects`, `multi-root-dev-box`, and the extracted temp-bundle distinct environment
