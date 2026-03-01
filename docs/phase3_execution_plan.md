# SemanticFS Phase 3 Execution Plan

Last updated: March 1, 2026

## Intent
Phase 3 is the controlled transition from repo-first validation to filesystem-scope architecture.
This is not a flip from "one repo" to "whole machine" in one step.
It is a staged bootstrap of multi-root semantics, trust boundaries, and scheduling.

Phase 3 starts now as a parallel workstream.
Phase 2 remains active until its stability and hardening criteria are closed.

## Why Start Now
1. Repo-level concept validation is strong enough to justify expansion:
   - representative head-to-head trend is favorable
   - date-separated night coverage is now closed at `7/7`
   - external strict holdout coverage spans multiple repos
2. Filesystem-scope prep artifacts now exist:
   - candidate discovery
   - cleaned candidate discovery (workspace-mirror suppression + clone dedupe)
   - backlog prioritization (`uncovered`, `covered_gap`, `covered_partial`, `covered_ok`)
3. The remaining repo-level gaps are now concentrated enough that they should not block architecture bootstrap.

## Scope Boundary
Phase 3 bootstrap is in scope:
1. Multi-root domain model and config scaffolding.
2. Domain-aware planning artifacts and queueing.
3. Trust and policy boundary design for heterogeneous roots.
4. Early non-code inclusion policy and content classes.
5. Scheduler and resource model definition for multiple roots.

Still out of scope for the first Phase 3 slice:
1. Full whole-machine indexing by default.
2. Background indexing of all discovered roots automatically.
3. Write-enabled cross-root operations.
4. Broad multimodal indexing as a default path.

## Parallel Operating Mode
Two workstreams now run in parallel:

1. `Phase 2 closeout`
   - keep representative nightlies on maintenance cadence now that `7/7` is closed
   - keep release-gate quality signal stable
   - preserve green holdout quality while Phase 3 scaffolding expands

2. `Phase 3 bootstrap`
   - land non-breaking multi-root config/domain scaffolding
   - build root/domain planning artifacts
   - define the first system-scope scheduling and policy contracts

## Bootstrap Progress (March 1, 2026)
1. Non-breaking multi-root config scaffolding is landed:
   - `workspace.domains` now exists in shared config.
   - effective-domain resolution preserves current single-root behavior when no explicit domains are configured.
   - CLI `init` and `health` now expose Phase 3 domain shape without changing runtime indexing behavior.
2. Domain planning artifact now exists:
   - `.semanticfs/bench/filesystem_domain_plan_latest.json`
3. Current planning counts:
   - backlog: `uncovered=0`, `covered_gap=0`, `covered_partial=0`, `covered_representative=0`, `covered_ok=21`
   - domain plan: `promote_candidate=0`, `harden_existing=0`, `expand_parent_root=0`, `add_strict_holdout=0`, `monitor=21`
4. Query-level hardening tooling is now in place:
   - `scripts/build_query_gap_report.ps1`
   - current gap reports also exist for `buckit_curated` and `yolov5`
5. Backlog-driven uncovered repo promotion is now established as the daytime loop:
   - `WilcoxRobotics`, `catapult_project`, `BoilerMakeXII`, `labelImg`, `yolov5`, `Euler-r9`, `mathGame`, and `navs-apple-folio` have all completed bootstrap, deterministic split, and strict holdout.
   - the uncovered queue is now fully cleared.
6. Bounded full-root validation is now proven for large roots:
   - `flutter_v2` completed successfully once the run was constrained to the exact package roots referenced by the dataset.
7. Scoped strict-suite generation is now aligned with benchmark filters:
   - `scripts/bootstrap_golden_from_repo.py` now supports `--config`, so scoped repos can generate strict suites that respect their actual benchmark allow/deny rules.
   - `ai_testgen_strict_*` was regenerated and direct holdout validation now passes.
8. Parent-root expansion is now underway:
   - `classifai-blogs` and the bounded `Robot` parent-root validation have both completed.
   - the current discovered-root bootstrap queue is fully closed; all current domains are now in monitor mode.

## Phase 3 Acceptance Criteria (Bootstrap Slice)
1. Config model can represent more than one root without breaking v1.x single-root behavior.
2. A deterministic root/domain plan can be generated from filesystem candidates.
3. Each root can carry explicit trust/policy metadata.
4. There is a documented promotion rule from discovery candidate -> indexed domain.
5. Phase 2 hardening remains green while Phase 3 scaffolding is added.

## Workstreams
## Workstream A: Domain Model
Goal:
1. Add non-breaking config support for multiple roots/domains.

Near-term deliverables:
1. Domain config type in shared config.
2. Effective-domain resolution that preserves current single-root behavior.
3. CLI visibility for configured/effective domains.
4. Sample config shape for future multi-root use.

## Workstream B: Domain Planning
Goal:
1. Turn discovered repos/roots into an explicit queue and domain plan.

Near-term deliverables:
1. Continue using `.semanticfs/bench/filesystem_scope_backlog_latest.json` as the queue source.
2. Add a draft domain-plan artifact that maps roots to trust classes and indexing intent.
3. Separate:
   - `promote_now`
   - `triage_quality_gap`
   - `expand_partial_root`
   - `defer`

## Workstream C: Phase 2 Hardening (Parallel)
Goal:
1. Keep retrieval quality and operational confidence increasing while Phase 3 starts.

Near-term deliverables:
1. Move from root promotion to architecture work; the current discovered-root queue is now fully covered.
2. `covered_representative` is now cleared; keep those roots in monitor mode.
3. Triage the residual `semanticfs_repo_v1` representative rank lag (`s20`) only if it can be done without destabilizing the now-green nightly baseline.
4. Keep representative nightlies on maintenance cadence rather than gating cadence.

## First Execution Order
1. Land multi-root config scaffolding without changing runtime indexing behavior.
2. Add reusable gap-analysis tooling so hardening remains fast.
3. Add/refresh domain-planning artifacts from filesystem backlog.
4. Use the backlog as a monitor artifact, not an active promotion queue, until new roots are discovered or a covered domain regresses.
   - completed examples: `WilcoxRobotics`, `catapult_project`, `BoilerMakeXII`, `labelImg`, `yolov5`, `Euler-r9`, `mathGame`, `navs-apple-folio`
   - the current discovered-root queue is fully covered; next work is scheduler/policy design, not more promotion
5. Keep representative nightlies on maintenance cadence while Phase 3 daytime work continues.

## Guardrails
1. Keep `/raw` as the deterministic read/verify boundary.
2. Do not let Phase 3 introduce silent cross-root ambiguity.
3. Do not treat "discovered root" as "trusted root".
4. Preserve single-root compatibility while adding multi-root structure.
5. Keep release decisions based on measured artifacts, not architectural optimism.

## Primary Artifacts
1. `.semanticfs/bench/filesystem_repo_candidates_latest.json`
2. `.semanticfs/bench/filesystem_scope_backlog_latest.json`
3. `.semanticfs/bench/filesystem_domain_plan_latest.json`
4. `docs/v1_2_execution_plan.md`
5. `docs/new-chat-handoff.md`
6. `docs/future-steps-log.md`
