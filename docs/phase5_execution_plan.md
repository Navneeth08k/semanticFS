# SemanticFS Phase 5 Execution Plan

Last updated: March 3, 2026

## Purpose
This file defines the next active phase after the Phase 4 controlled-domain sign-off.

Use it for:
1. the Phase 5 objective
2. the required workstreams
3. acceptance criteria
4. the current execution order

Phase 3 signed off the multi-root runtime.
Phase 4 proved that one controlled domain expansion can land cleanly.
Phase 5 turns that into a repeatable expansion model with stronger orchestration and governance.

## Current Phase 5 Position
Phase 5 is operationally complete.

The Phase 5 acceptance bar is now met because:
1. per-domain watch priority is now part of the runtime contract
2. domains can now opt out of watch subscriptions without leaving the index entirely
3. a new bounded `governance` domain is added on top of the Phase 4 baseline
4. the broadened Phase 5 suite (`semanticfs_multiroot_explicit_v11`) is green while the frozen `v9` and `v10` gates remain green
5. the runtime stayed deterministic, policy-bounded, and opt-in while indexing ownership was decoupled from watch participation

## Phase 5 Objective
Phase 5 moves SemanticFS from one successful controlled expansion to a durable, repeatable system-scope expansion model.

The goal is:
1. keep broadening beyond the Phase 4 baseline one domain class at a time
2. make per-domain orchestration explicit instead of relying on one global watch cap
3. harden governance rules so future sensitive roots can be introduced without trust drift
4. preserve the frozen Phase 3 and Phase 4 regression gates while the active expansion suite grows
5. stay deterministic and opt-in while moving closer to practical filesystem-scope coverage

## Entry Conditions
Phase 5 starts from:
1. Phase 3 operationally complete
2. Phase 4 operationally complete
3. frozen Phase 3 gate: `semanticfs_multiroot_explicit_v9`
4. frozen Phase 4 broadened baseline: `semanticfs_multiroot_explicit_v10`

## Non-Goals
Phase 5 is not:
1. automatic whole-machine indexing
2. broad discovery-by-default across the host
3. hidden trust promotion for newly added roots
4. adding multiple domain classes without explicit benchmark queries
5. replacing `/raw` as the deterministic final verification path

## Frozen Baselines
1. Phase 3 frozen gate:
   - config: `config/relevance-multiroot.toml`
   - fixture: `tests/retrieval_golden/semanticfs_multiroot_explicit.json`
   - suite: `semanticfs_multiroot_explicit_v9`
2. Phase 4 frozen broadened baseline:
   - fixture: `tests/retrieval_golden/semanticfs_multiroot_explicit_v10.json`
   - suite: `semanticfs_multiroot_explicit_v10`

Both must stay green while Phase 5 broadens and after sign-off remain frozen as regression gates.

## Workstreams
### Workstream A: Repeatable Domain Expansion
Goal:
1. add another bounded, low-risk domain beyond Phase 4 and keep the existing gates green

Required direction:
1. add exactly one domain class at a time
2. require explicit expected-path queries for the new class
3. prefer text-heavy, interpretable roots before higher-risk content classes

### Workstream B: Per-Domain Orchestration
Goal:
1. move from one global watch budget to domain-scoped orchestration controls

Required direction:
1. allow per-domain watch participation (`watch_enabled`)
2. allow per-domain watch priority (`watch_priority`)
3. preserve deterministic watch-target planning under a global budget cap
4. keep indexing ownership independent from watch participation

### Workstream C: Governance Hardening
Goal:
1. make trust and intake rules explicit before sensitive roots broaden

Required direction:
1. new governance text roots stay untrusted by default
2. root intake rules are documented in-repo and benchmarked
3. future sensitive roots must prove boundedness before expansion

### Workstream D: Regression Gates
Goal:
1. keep expansion measurable and reversible

Required direction:
1. keep `v9` green
2. keep `v10` green
3. use one broadened Phase 5 suite for the current expansion slice
4. keep reruns narrow after each runtime change

## Acceptance Criteria
Phase 5 is considered successful when:
1. at least one additional domain class beyond Phase 4 is added cleanly
2. per-domain watch participation and watch priority are active in the runtime
3. the frozen `v9` and `v10` suites remain green under the broadened config
4. the broadened Phase 5 suite is green and interpretable
5. the runtime stays deterministic, policy-bounded, and opt-in

## Execution Order
1. Keep `v9` and `v10` frozen.
2. Add one new bounded domain class.
3. Add one active candidate fixture for that domain.
4. Land one orchestration improvement that applies across all domains.
5. Run narrow relevance/head-to-head checks on the frozen gates and the active candidate.
6. Only after that, choose the next domain class.

## Immediate Targets
1. Keep `semanticfs_multiroot_explicit_v9` frozen as the Phase 3 gate.
2. Keep `semanticfs_multiroot_explicit_v10` frozen as the Phase 4 broadened baseline.
3. Freeze `semanticfs_multiroot_explicit_v11` as the signed-off Phase 5 broadened baseline.
4. Treat per-domain `watch_enabled` and `watch_priority` as the new minimum orchestration layer.

## Phase 5 Sign-Off
Final measured state:
1. New domain: `governance`
2. New signed-off Phase 5 suite: `semanticfs_multiroot_explicit_v11`
   - `active_version=192`
   - query count `29`
   - relevance: recall `1.0000`, MRR `1.0000`, symbol-hit `1.0000`
   - head-to-head SemanticFS: recall `1.0000`, MRR `1.0000`, symbol-hit `1.0000`, p95 `48.298 ms`
   - head-to-head baseline `rg`: recall `0.9310`, MRR `0.7655`, symbol-hit `0.4000`, p95 `28.439 ms`
3. Frozen Phase 3 and Phase 4 gates on the same active snapshot:
   - `semanticfs_multiroot_explicit_v9`: recall `1.0000`, MRR `1.0000`, symbol-hit `1.0000`
   - `semanticfs_multiroot_explicit_v10`: recall `1.0000`, MRR `1.0000`, symbol-hit `1.0000`
4. New runtime controls:
   - `workspace.domains[].watch_enabled`
   - `workspace.domains[].watch_priority`
   - indexing now walks `scan_targets` instead of `watch_roots`, so watch participation no longer controls index coverage
5. New governance texts:
   - `governance/trust_boundary_contract.md`
   - `governance/sensitive_root_intake.md`
6. Explicit multi-root `benchmark run --skip-reindex --soak-seconds 1` still passes `4/4` E2E checks on `active_version=192` with runtime RSS `38 MB`

Phase 5 therefore closes as:
1. broader than Phase 4
2. still deterministic
3. still benchmarked
4. still preserving the frozen Phase 3 and Phase 4 gates
5. now adding per-domain orchestration controls as part of the signed-off runtime contract

## Primary Files
1. `docs/phase5_execution_plan.md`
2. `docs/phase4_execution_plan.md`
3. `docs/new-chat-handoff.md`
4. `docs/big-picture-roadmap.md`
5. `config/relevance-multiroot.toml`
6. `tests/retrieval_golden/semanticfs_multiroot_explicit.json`
7. `tests/retrieval_golden/semanticfs_multiroot_explicit_v10.json`
8. `tests/retrieval_golden/semanticfs_multiroot_explicit_v11.json`
