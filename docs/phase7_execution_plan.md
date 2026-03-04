# SemanticFS Phase 7 Execution Plan

Last updated: March 3, 2026

## Purpose
This file defines the next active phase after the Phase 6 sign-off.

Use it for:
1. the Phase 7 objective
2. the required workstreams
3. acceptance criteria
4. the signed-off execution order

Phase 6 proved that one bounded host-scale control can land cleanly.
Phase 7 takes the next step up: expand several bounded roots in one slice while keeping the existing regression gates stable.

## Current Phase 7 Position
Phase 7 is operationally complete.

The Phase 7 acceptance bar is now met because:
1. a multi-domain batch was added instead of a single-domain slice
2. the batch stayed bounded through explicit `allow_roots` and `max_indexed_files`
3. the broadened Phase 7 suite (`semanticfs_multiroot_explicit_v13`) is green while the frozen `v9`, `v10`, `v11`, and `v12` gates remain green
4. capped third files for the new domains remain out of the live index
5. the runtime stayed deterministic, policy-bounded, and opt-in under a broader batch

## Phase 7 Objective
Phase 7 moves SemanticFS from one-by-one bounded expansion into batched bounded expansion.

The goal is:
1. widen the indexed surface faster than one-domain-at-a-time
2. keep each new root individually bounded and explicitly benchmarked
3. preserve the prior signed-off phases as frozen regression gates
4. use the broader batch to expose the next scaling bottleneck clearly
5. keep deterministic `/raw` verification unchanged while broadening

## Entry Conditions
Phase 7 starts from:
1. Phase 3 operationally complete
2. Phase 4 operationally complete
3. Phase 5 operationally complete
4. Phase 6 operationally complete
5. frozen Phase 3 gate: `semanticfs_multiroot_explicit_v9`
6. frozen Phase 4 broadened baseline: `semanticfs_multiroot_explicit_v10`
7. frozen Phase 5 broadened baseline: `semanticfs_multiroot_explicit_v11`
8. frozen Phase 6 broadened baseline: `semanticfs_multiroot_explicit_v12`

## Non-Goals
Phase 7 is not:
1. uncontrolled whole-machine indexing
2. automatic admission of unbounded roots
3. dropping per-domain caps while broadening
4. changing the frozen `v9` through `v12` suites
5. claiming final full-filesystem coverage

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
4. Phase 6 frozen broadened baseline:
   - fixture: `tests/retrieval_golden/semanticfs_multiroot_explicit_v12.json`
   - suite: `semanticfs_multiroot_explicit_v12`

All four must stay green while Phase 7 lands and after sign-off remain frozen as regression gates.

## Workstreams
### Workstream A: Batched Bounded Expansion
Goal:
1. add multiple low-risk roots in one slice without losing control

Required direction:
1. add `3-5` bounded, text-heavy roots together
2. keep each new root explicitly queryable
3. preserve trust and policy boundaries for every root in the batch

### Workstream B: Coverage Discipline
Goal:
1. widen faster without letting breadth turn ambiguous

Required direction:
1. keep exact `allow_roots`
2. keep `max_indexed_files` active on the new roots
3. only benchmark files intentionally inside the configured caps
4. verify the deferred third file in each capped root stays out of the live index

### Workstream C: Regression Gates
Goal:
1. keep expansion measurable and reversible

Required direction:
1. keep `v9` green
2. keep `v10` green
3. keep `v11` green
4. keep `v12` green
5. use one broadened Phase 7 suite for the batched slice

### Workstream D: Bottleneck Identification
Goal:
1. let the broader batch reveal what is actually next

Required direction:
1. keep correctness green first
2. measure the latency cost of the broader batch explicitly
3. treat widened p95 as a post-Phase-7 scaling target, not as a hidden regression

## Acceptance Criteria
Phase 7 is considered successful when:
1. at least three additional bounded domains beyond Phase 6 are added cleanly in one slice
2. the frozen `v9`, `v10`, `v11`, and `v12` suites remain green under the broadened config
3. the broadened Phase 7 suite is green and interpretable
4. direct probes confirm the capped third file in each new domain remains out of the live index
5. the runtime stays deterministic, policy-bounded, and opt-in

## Execution Order
1. Keep `v9`, `v10`, `v11`, and `v12` frozen.
2. Add a bounded batch of low-risk roots.
3. Keep caps active on every new root in that batch.
4. Add one broadened Phase 7 fixture for the full batch.
5. Run narrow validation on the frozen gates and the new broadened suite.
6. Use the measured latency result to define the next post-Phase-7 scaling phase.

## Phase 7 Sign-Off
Final measured state:
1. New domains:
   - `profiles`
   - `operations`
   - `intake`
2. New signed-off Phase 7 suite: `semanticfs_multiroot_explicit_v13`
   - `active_version=195`
   - query count `37`
   - relevance: recall `1.0000`, MRR `1.0000`, symbol-hit `1.0000`
   - head-to-head SemanticFS: recall `1.0000`, MRR `1.0000`, symbol-hit `1.0000`, p95 `84.076 ms`
   - head-to-head baseline `rg`: recall `0.9459`, MRR `0.8604`, symbol-hit `0.4000`, p95 `50.329 ms`
3. Frozen gates on the same active snapshot:
   - `semanticfs_multiroot_explicit_v9`: recall `1.0000`, MRR `1.0000`, symbol-hit `1.0000`
   - `semanticfs_multiroot_explicit_v10`: recall `1.0000`, MRR `1.0000`, symbol-hit `1.0000`
   - `semanticfs_multiroot_explicit_v11`: recall `1.0000`, MRR `1.0000`, symbol-hit `1.0000`
   - `semanticfs_multiroot_explicit_v12`: recall `1.0000`, MRR `1.0000`, symbol-hit `1.0000`
4. Direct cap probes:
   - `profiles/z_future_aggressive_modes.md` stays unmatched on `active_version=195`
   - `operations/z_unbounded_host_sweeps.md` stays unmatched on `active_version=195`
   - `intake/z_sensitive_root_backlog.md` stays unmatched on `active_version=195`
5. Explicit multi-root `benchmark run --skip-reindex --soak-seconds 1` still passes `4/4` E2E checks on `active_version=195` with runtime RSS `42 MB`

Phase 7 therefore closes as:
1. broader than Phase 6
2. still deterministic
3. still benchmarked
4. proving that bounded batches can land without breaking the prior gates
5. clearly surfacing the next scaling bottleneck: broader-batch latency and scheduler pressure

## Primary Files
1. `docs/phase7_execution_plan.md`
2. `docs/phase6_execution_plan.md`
3. `docs/new-chat-handoff.md`
4. `docs/big-picture-roadmap.md`
5. `config/relevance-multiroot.toml`
6. `profiles/a_root_selection_heuristics.md`
7. `profiles/b_latency_budget_targets.md`
8. `operations/a_agent_lookup_playbook.md`
9. `operations/b_cache_trim_policy.md`
10. `intake/a_low_risk_root_batch.md`
11. `intake/b_domain_promotion_checklist.md`
12. `tests/retrieval_golden/semanticfs_multiroot_explicit_v13.json`
