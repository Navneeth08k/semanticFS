# SemanticFS Phase 6 Execution Plan

Last updated: March 3, 2026

## Purpose
This file defines the next active phase after the Phase 5 sign-off.

Use it for:
1. the Phase 6 objective
2. the required workstreams
3. acceptance criteria
4. the signed-off execution order

Phase 5 proved that bounded domain expansion can be repeated safely.
Phase 6 adds the first host-scale indexing budget control so future broadening can stay predictable as root count grows.

## Current Phase 6 Position
Phase 6 is operationally complete.

The Phase 6 acceptance bar is now met because:
1. per-domain indexing budgets are now part of the runtime contract
2. full indexing now enforces those budgets deterministically per domain
3. a new bounded `inventory` domain is added on top of the Phase 5 baseline
4. the broadened Phase 6 suite (`semanticfs_multiroot_explicit_v12`) is green while the frozen `v9`, `v10`, and `v11` gates remain green
5. the runtime stayed deterministic, policy-bounded, and opt-in while the new budget control was exercised by a real domain

## Phase 6 Objective
Phase 6 moves SemanticFS from bounded domain expansion into host-scale budgeted expansion.

The goal is:
1. add the first runtime control that caps index breadth per domain
2. exercise that control with a real bounded text domain
3. keep the prior signed-off phases frozen as regression gates
4. preserve deterministic `/raw` verification while host-scale controls become more explicit
5. keep future broadening measurable instead of allowing silent coverage drift

## Entry Conditions
Phase 6 starts from:
1. Phase 3 operationally complete
2. Phase 4 operationally complete
3. Phase 5 operationally complete
4. frozen Phase 3 gate: `semanticfs_multiroot_explicit_v9`
5. frozen Phase 4 broadened baseline: `semanticfs_multiroot_explicit_v10`
6. frozen Phase 5 broadened baseline: `semanticfs_multiroot_explicit_v11`

## Non-Goals
Phase 6 is not:
1. automatic whole-machine indexing
2. automatic discovery of unbounded new roots
3. replacing policy review with heuristics
4. adding multiple new domains in one slice
5. changing the frozen `v9`, `v10`, or `v11` suites

## Frozen Baselines
1. Phase 3 frozen gate:
   - fixture: `tests/retrieval_golden/semanticfs_multiroot_explicit.json`
   - suite: `semanticfs_multiroot_explicit_v9`
2. Phase 4 frozen broadened baseline:
   - fixture: `tests/retrieval_golden/semanticfs_multiroot_explicit_v10.json`
   - suite: `semanticfs_multiroot_explicit_v10`
3. Phase 5 frozen broadened baseline:
   - fixture: `tests/retrieval_golden/semanticfs_multiroot_explicit_v11.json`
   - suite: `semanticfs_multiroot_explicit_v11`

All three must stay green while Phase 6 lands and after sign-off remain frozen as regression gates.

## Workstreams
### Workstream A: Index Breadth Budgeting
Goal:
1. cap per-domain indexing deterministically before broader host-scale roots are added

Required direction:
1. add `workspace.domains[].max_indexed_files`
2. expose the configured budget in health output
3. enforce the cap in full-index builds
4. keep uncapped domains unchanged when the value is `0`

### Workstream B: Budgeted Domain Expansion
Goal:
1. add one new bounded domain that proves the budget control is not theoretical

Required direction:
1. keep the new domain text-heavy and interpretable
2. use exact `allow_roots`
3. cap indexed files below the allowed file count
4. only benchmark files that are intentionally inside the budget

### Workstream C: Regression Gates
Goal:
1. keep expansion measurable and reversible

Required direction:
1. keep `v9` green
2. keep `v10` green
3. keep `v11` green
4. use one broadened Phase 6 suite for the current expansion slice
5. keep reruns narrow after each runtime change

## Acceptance Criteria
Phase 6 is considered successful when:
1. per-domain indexing budgets are active in the runtime
2. the health surface reports the new budget explicitly
3. at least one additional bounded domain beyond Phase 5 is added cleanly
4. the frozen `v9`, `v10`, and `v11` suites remain green under the broadened config
5. the broadened Phase 6 suite is green and interpretable
6. the runtime stays deterministic, policy-bounded, and opt-in

## Execution Order
1. Keep `v9`, `v10`, and `v11` frozen.
2. Add the new per-domain indexing budget control.
3. Exercise it through one bounded domain with more allowed files than indexed files.
4. Add one broadened Phase 6 fixture for the new domain.
5. Run narrow validation on the frozen gates and the new broadened suite.
6. Only after that, choose the next post-Phase-6 expansion slice.

## Phase 6 Sign-Off
Final measured state:
1. New domain: `inventory`
2. New runtime control:
   - `workspace.domains[].max_indexed_files`
   - full indexing now skips extra allowed files after the configured per-domain cap is reached
3. Health confirmation:
   - `workspace_domain_count=11`
   - `inventory` is reported with `watch_enabled=false` and `max_indexed_files=2`
4. New signed-off Phase 6 suite: `semanticfs_multiroot_explicit_v12`
   - `active_version=194`
   - query count `31`
   - relevance: recall `1.0000`, MRR `1.0000`, symbol-hit `1.0000`
   - head-to-head SemanticFS: recall `1.0000`, MRR `1.0000`, symbol-hit `1.0000`, p95 `50.747 ms`
   - head-to-head baseline `rg`: recall `0.9355`, MRR `0.8091`, symbol-hit `0.4000`, p95 `31.659 ms`
5. Frozen gates on the same active snapshot:
   - `semanticfs_multiroot_explicit_v9`: recall `1.0000`, MRR `1.0000`, symbol-hit `1.0000`
   - `semanticfs_multiroot_explicit_v10`: recall `1.0000`, MRR `1.0000`, symbol-hit `1.0000`
   - `semanticfs_multiroot_explicit_v11`: recall `1.0000`, MRR `1.0000`, symbol-hit `1.0000`
6. Explicit multi-root `benchmark run --skip-reindex --soak-seconds 1` still passes `4/4` E2E checks on `active_version=194` with runtime RSS `38 MB`
7. Direct budget probe:
   - an explicit one-query probe against `inventory/z_deferred_large_roots.md` stays unmatched on `active_version=194`, confirming the third allowed file remains out of the live index once the `max_indexed_files=2` cap is reached

Phase 6 therefore closes as:
1. broader than Phase 5
2. still deterministic
3. still benchmarked
4. now carrying the first host-scale index-breadth control in the signed-off runtime contract
5. still preserving the frozen Phase 3 / 4 / 5 gates

## Primary Files
1. `docs/phase6_execution_plan.md`
2. `docs/phase5_execution_plan.md`
3. `docs/new-chat-handoff.md`
4. `docs/big-picture-roadmap.md`
5. `config/relevance-multiroot.toml`
6. `inventory/a_root_class_matrix.md`
7. `inventory/b_index_budget_policy.md`
8. `inventory/z_deferred_large_roots.md`
9. `tests/retrieval_golden/semanticfs_multiroot_explicit.json`
10. `tests/retrieval_golden/semanticfs_multiroot_explicit_v10.json`
11. `tests/retrieval_golden/semanticfs_multiroot_explicit_v11.json`
12. `tests/retrieval_golden/semanticfs_multiroot_explicit_v12.json`
