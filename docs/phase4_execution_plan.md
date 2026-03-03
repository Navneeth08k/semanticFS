# SemanticFS Phase 4 Execution Plan

Last updated: March 2, 2026

## Purpose
This file defines the next phase after the Phase 3 multi-root runtime sign-off.

Use it for:
1. the Phase 4 objective
2. the expansion workstreams
3. acceptance criteria
4. execution order

Phase 3 proved the multi-root runtime contract.
Phase 4 uses that signed-off baseline to broaden scope in a controlled way.

## Current Phase 4 Position
Phase 4 is operationally complete.

The Phase 4 acceptance bar is now met because:
1. a new domain class (`playbooks`) was added beyond the Phase 3 eight-domain baseline
2. the broadened tracked suite is now green at `27/27` rank `1`
3. resource-aware watch scheduling now exists beyond static domain ordering
4. the original frozen `v9` suite remains green under the broadened config
5. the runtime stayed policy-bounded and deterministic while the domain set expanded

## Phase 4 Objective
Phase 4 expands SemanticFS from a signed-off multi-root foundation into broader, more realistic system-scope coverage.

The goal is:
1. add new domain classes beyond the frozen Phase 3 eight-domain contract
2. keep domain ownership, trust, and `/raw` verification deterministic
3. improve scheduler behavior from static ordering into resource-aware orchestration
4. preserve the current quality lead while broadening heterogeneous content
5. move closer to real filesystem-scope coverage without turning on unconstrained whole-machine indexing

In practical terms:
1. Phase 3 made multi-root runtime correct
2. Phase 4 makes broader system-scope expansion controlled, scalable, and repeatable

## Entry Conditions
Phase 4 starts from these conditions:
1. Phase 3 is operationally complete
2. `semanticfs_multiroot_explicit_v9` is the frozen regression baseline
3. the current Phase 3 tracked suite is fully green (`25/25` rank `1`)
4. representative repo-first validation is in maintenance mode, not gating mode

## Non-Goals
Phase 4 is not:
1. automatic whole-machine indexing by default
2. blindly indexing every discovered directory
3. weakening per-domain trust boundaries to grow faster
4. introducing write-enabled cross-root behavior
5. adding multiple new domain classes at once without measurement

## Frozen Baseline
The signed-off Phase 3 baseline remains:
1. tracked config: `config/relevance-multiroot.toml`
2. tracked fixture: `tests/retrieval_golden/semanticfs_multiroot_explicit.json`
3. frozen suite: `semanticfs_multiroot_explicit_v9`
4. current domain set:
   - `workspace_meta`
   - `code`
   - `docs`
   - `config`
   - `scripts`
   - `systemd`
   - `github`
   - `fixture_repo`

Phase 4 should treat this as the primary regression gate while broadening.

## Workstreams
### Workstream A: Domain-Class Expansion
Goal:
1. add new root classes beyond the Phase 3 contract without losing determinism

Immediate direction:
1. add one new low-risk domain class at a time
2. prefer bounded, text-heavy, interpretable roots first
3. require explicit expected-path queries for every new domain
4. do not broaden multiple domain classes in one step

Examples of acceptable early Phase 4 additions:
1. an additional bounded operational-doc or note-like root
2. a bounded config-heavy root outside the current set
3. a safe mixed text subtree that is not just another code root

### Workstream B: Scheduler And Budgeting
Goal:
1. move from deterministic domain order to resource-aware multi-root orchestration

Required direction:
1. per-domain indexing budgets
2. per-domain watch/update prioritization
3. bounded background work across many roots
4. predictable behavior when multiple roots change at once

Phase 3 solved ordering.
Phase 4 needs to solve load management.

### Workstream C: Heterogeneous Content Handling
Goal:
1. make broader non-code content classes first-class without collapsing retrieval quality

Required direction:
1. preserve clear ranking intent across docs, configs, scripts, workflow, and service units
2. add new non-code content classes with explicit, narrow benchmarks
3. keep semantic retrieval useful when content is less code-shaped and less symbol-heavy

### Workstream D: Governance And Trust Hardening
Goal:
1. keep broader system-scope expansion safe as the domain set grows

Required direction:
1. maintain explicit per-domain trust labels
2. keep deny-by-default behavior for out-of-domain paths
3. avoid treating newly discovered content as trusted automatically
4. make future identity-sensitive or user-sensitive roots opt-in and measurable

### Workstream E: Performance And Regression Guardrails
Goal:
1. broaden scope without losing the signed-off runtime contract

Required direction:
1. keep `semanticfs_multiroot_explicit_v9` green at all times
2. continue shrinking the remaining absolute p95 gap vs `rg`
3. instrument performance by query class instead of relying on aggregate intuition
4. require narrow reruns after each retrieval/indexing change

## Acceptance Criteria
Phase 4 is considered successful when:
1. at least one new domain class beyond the Phase 3 baseline is added cleanly
2. the broadened suite stays green without introducing cross-root ambiguity
3. resource-aware scheduling exists beyond static domain ordering
4. the original Phase 3 `v9` contract remains green while the broadened suite also stays interpretable
5. the system remains opt-in, policy-bounded, and deterministic rather than drifting into uncontrolled crawl behavior

## Execution Order
1. Keep `semanticfs_multiroot_explicit_v9` as the fixed regression gate.
2. Add exactly one new low-risk domain class.
3. Build explicit expected-path queries for that new class.
4. Run only the narrow multi-root suite plus the new broadened suite.
5. If the broadened suite is green, then add or refine scheduler-budget behavior.
6. Only after that, consider the next domain class.

## Guardrails
1. Do not weaken `/raw` as the deterministic final verification path.
2. Do not let domain growth outpace benchmark clarity.
3. Do not add hidden trust upgrades.
4. Prefer explicit bounded roots over broad discovery-by-default.
5. Preserve single-root behavior as the default contract.

## Immediate First Targets
1. Freeze `semanticfs_multiroot_explicit_v9` as the original Phase 3 regression gate.
2. Freeze `semanticfs_multiroot_explicit_v10` as the Phase 4 broadened baseline.
3. Treat the new watch-target planner as the minimum scheduler-budget layer going forward.
4. Keep representative Phase 2 maintenance checks secondary, and rerun them only after meaningful retrieval/indexing changes.

## Phase 4 Sign-Off
Final measured state:
1. Frozen Phase 3 gate (`semanticfs_multiroot_explicit_v9`):
   - `active_version=185`
   - recall `1.0000`
   - MRR `1.0000`
   - symbol-hit `1.0000`
2. Broadened Phase 4 suite (`semanticfs_multiroot_explicit_v10`):
   - `active_version=186`
   - query count `27`
   - relevance: recall `1.0000`, MRR `1.0000`, symbol-hit `1.0000`
   - head-to-head SemanticFS: recall `1.0000`, MRR `1.0000`, symbol-hit `1.0000`, p95 `69.421 ms`
   - head-to-head baseline `rg`: recall `0.9259`, MRR `0.7778`, symbol-hit `0.2000`, p95 `45.671 ms`
3. Explicit multi-root `benchmark run --skip-reindex --soak-seconds 1` still passes `4/4` E2E checks on `active_version=186`

Phase 4 therefore closes as:
1. broader than Phase 3
2. still deterministic
3. still benchmarked
4. still using the Phase 3 gate as the frozen regression baseline

## Primary Files
1. `docs/phase4_execution_plan.md`
2. `docs/phase3_execution_plan.md`
3. `docs/phase3_execution_status.md`
4. `docs/new-chat-handoff.md`
5. `docs/big-picture-roadmap.md`
6. `config/relevance-multiroot.toml`
7. `tests/retrieval_golden/semanticfs_multiroot_explicit.json`
