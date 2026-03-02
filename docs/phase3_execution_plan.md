# SemanticFS Phase 3 Execution Plan

Last updated: March 2, 2026

## Purpose
This file is the forward-looking Phase 3 plan.

Use it for:
1. the Phase 3 objective
2. the active workstreams
3. acceptance criteria
4. execution order

Do not use this file as the running completion log.
Use `docs/phase3_execution_status.md` for the current measured state, completed work, and immediate status.

## Phase 3 Objective
Phase 3 moves SemanticFS from repo-first validation into filesystem-scope runtime behavior.

The goal is:
1. explicit multi-root domain behavior
2. explicit trust and policy boundaries
3. deterministic cross-root verification through `/raw`
4. measurable multi-root retrieval quality and latency
5. stable single-root behavior as the default contract

Phase 3 is not:
1. automatic whole-machine indexing by default
2. unconstrained background indexing of every discovered path
3. write-enabled cross-root behavior

## Operating Mode
Two tracks continue in parallel:

1. `Phase 2 maintenance`
   - keep representative nightlies green on maintenance cadence
   - rerun after meaningful retrieval/indexing changes
   - preserve the repo-level baseline while Phase 3 evolves

2. `Phase 3 closeout complete`
   - the explicit multi-root runtime is now signed off
   - the tracked multi-root contract is now the frozen Phase 3 baseline
   - additional broadening now moves to the next expansion phase

## Current Planning Baseline
Detailed status lives in `docs/phase3_execution_status.md`.

Planning assumptions:
1. the current discovered-root promotion queue is already closed
2. the filesystem backlog is now a monitor artifact
3. the tracked explicit multi-root benchmark is the primary Phase 3 regression signal
4. runtime multi-root behavior is already live and signed off, so future work now focuses on broader expansion from this baseline

## Acceptance Criteria
Phase 3 is considered operationally successful when:
1. explicit multi-root configs remain non-breaking for single-root users
2. domain ownership is deterministic across indexing, retrieval, `/raw`, and `/map`
3. trust/policy boundaries remain explicit and enforceable per domain
4. the tracked explicit multi-root benchmark remains green after runtime changes
5. broader domain expansion does not introduce silent cross-root ambiguity

## Workstreams
### Workstream A: Runtime Retrieval Quality
Goal:
1. keep improving the tracked multi-root contract quality without destabilizing single-root behavior

Near-term deliverables:
1. keep the current `25`-query tracked suite at rank `1`
2. keep same-file de-duplication stable
3. keep intent-aware ranking (config, docs, scripts, workflow, systemd) precise rather than overly broad
4. preserve the current `1.0000` tracked relevance result while continuing to narrow the remaining p95 gap on the broader tracked suite

### Workstream B: Domain Expansion
Goal:
1. expand the tracked multi-root fixture beyond code-adjacent roots in a controlled way

Near-term deliverables:
1. hold the current eight-domain tracked set stable after adding the top-level `workspace_meta` root
2. keep new domains deterministic and low-risk
3. ensure each added domain has clear expected-path queries
4. avoid broadening the fixture faster than it remains interpretable

### Workstream C: Runtime Contract Stability
Goal:
1. keep the multi-root runtime contract coherent as complexity increases

Near-term deliverables:
1. preserve deterministic ownership resolution
2. preserve `/raw` as the final verification boundary
3. prevent out-of-domain leakage in benchmarks and serving
4. keep `/map` aligned with the same domain model as `/raw`
5. keep top-level `.` domain normalization pinned by regression coverage
6. keep runtime indexing order aligned with the domain scheduler instead of relying on incidental cross-root path order

### Workstream D: Monitor-Only Coverage Operations
Goal:
1. keep prior Phase 2 and root-promotion work from regressing without turning coverage into the main workstream again

Near-term deliverables:
1. keep filesystem backlog/domain plan in monitor mode
2. rerun representative suites only after meaningful retrieval/indexing changes
3. rerun promotion flows only if new roots are discovered or a covered root regresses

## Execution Order
1. Make one narrow retrieval/indexing change at a time.
2. Re-run the tracked explicit multi-root suite only.
3. If the tracked suite stays green, then update docs.
4. Only after that, consider broadening the tracked domain mix.
5. Keep broad sweeps and representative nightlies secondary unless a regression demands them.

## Guardrails
1. Keep `/raw` as the deterministic truth path.
2. Do not let ranking heuristics blur domain ownership.
3. Do not treat discovered roots as trusted by default.
4. Keep benchmark comparisons fair and domain-aligned.
5. Prefer narrow measurable improvements over broad architectural churn.

## Phase 3 Primary Files
1. `docs/phase3_execution_plan.md`
2. `docs/phase3_execution_status.md`
3. `config/relevance-multiroot.toml`
4. `tests/retrieval_golden/semanticfs_multiroot_explicit.json`
5. `docs/benchmark.md`
6. `docs/new-chat-handoff.md`

## Immediate Planning Focus
1. hold `semanticfs_multiroot_explicit_v9` as the Phase 3 sign-off suite
2. use the signed-off active-version-`184` result as the baseline for the next expansion phase (`25/25` rank `1`, with three consecutive warmed median-of-3 reruns at SemanticFS p95 `42.989-53.384 ms` vs baseline `28.468-37.609 ms`)
3. move further domain-class broadening into the next expansion phase instead of keeping Phase 3 open

## Phase 3 Completion Plan
Phase 3 is complete only when the runtime is both:
1. correct on the tracked multi-root contract
2. stable enough that broadening scope is no longer likely to collapse quality or latency

### Milestone 1: Stabilize The Current Eight-Domain Contract
Exit criteria:
1. the tracked `semanticfs_multiroot_explicit_v9` suite stays fully green (`25/25` rank `1`)
2. the current p95 gap is materially reduced and the warmed narrow reruns no longer swing badly between runs
3. recent runtime changes stop reopening the same doc/symbol latency regressions

Concrete work:
1. remove avoidable runtime I/O in scoring paths
2. target the highest-latency tracked queries first; the current noisy set is the narrative/workflow/systemd cluster (`m05`, `m11`, `m12`, `m13`, `m19`, `m20`, `m21`, `m22`)
3. keep all reruns narrow and use the tracked suite as the only primary regression gate

### Milestone 2: Remove Remaining Runtime Cost From Search-Time Metadata Lookups
Exit criteria:
1. `files.modified_unix_ms` is persisted in the snapshot and available for a future recency path
2. retrieval-side use of persisted recency data either matches or beats the current tracked p95 before it replaces the live-fs fallback
3. the tracked suite stays fully green after any retrieval-side recency change

Concrete work:
1. keep recency inputs persisted in indexed metadata, but do not leave a retrieval-side metadata change enabled if it widens the tracked p95 gap
2. keep the current exact-symbol optimization (`indexed symbol probe first, case-fold fallback second`) because it removes an indexed-path full scan without changing the search contract
3. keep SQLite/LanceDB ownership metadata parity intact
4. rerun only the tracked suite after each retrieval/indexing change

### Milestone 3: Lock The Current Eight-Domain Contract As The Phase 3 Broadening Baseline
Exit criteria:
1. the current eight-domain contract is stable enough that further broadening is no longer required for Phase 3 sign-off
2. the top-level `workspace_meta` root and the mixed-content `fixture_repo` subtree remain green and domain-correct
3. additional domain classes are explicitly deferred to the next expansion phase

Concrete work:
1. keep the fixture interpretable and avoid vague queries
2. preserve the current eight-domain suite as the signed-off contract
3. do not reopen Phase 3 just to add more domains once the runtime contract is already stable

### Milestone 4: Final Runtime Contract Sign-Off
Exit criteria:
1. indexing, retrieval, `/raw`, and `/map` all use the same domain model without known contract gaps
2. benchmark baseline normalization remains domain-correct across top-level and nested roots
3. monitor-mode reruns do not reveal new cross-root ambiguity

Concrete work:
1. run one final narrow tracked rerun after the last runtime change
2. run one final explicit multi-root `benchmark run --skip-reindex --soak-seconds 1`
3. confirm docs, handoff, and benchmark references all match the final canonical artifacts

### Milestone 5: Declare Phase 3 Operationally Complete
Phase 3 is now marked complete because:
1. the tracked contract is green and stable
2. the runtime contract is fully wired and documented
3. the remaining work is no longer "make multi-root runtime correct," but "future broader system-scope expansion"
4. the final active-version-`184` sign-off held the tracked suite at `25/25` rank `1`, with three consecutive warmed median-of-3 reruns staying inside a materially tighter p95 band than the earlier single-sample runs

At this point:
1. Phase 3 status moves from `runtime hardening` to `operationally complete`
2. future work shifts into the next expansion phase instead of Phase 3 closeout
