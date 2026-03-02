# SemanticFS Phase 3 Status

Last updated: March 2, 2026

## Purpose
This document is the Phase 3 operational source of truth.
Use it to understand:
1. the overall goal of Phase 3
2. what is already done
3. what is still open
4. the exact next steps

This is the status companion to `docs/phase3_execution_plan.md`.
The plan explains the workstream shape.
This file explains where that workstream currently stands.
The explicit remaining completion ladder now lives in the `Phase 3 Completion Plan` section of `docs/phase3_execution_plan.md`.

## Phase 3 Goal
Phase 3 is the transition from repo-first validation to filesystem-scope runtime behavior.

The goal is not "index the whole machine immediately."
The goal is to make SemanticFS behave correctly and deterministically across multiple roots with:
1. explicit domain boundaries
2. explicit trust boundaries
3. predictable retrieval behavior
4. deterministic `/raw` verification
5. measurable multi-root quality and latency

In practical terms, Phase 3 succeeds when SemanticFS can treat multiple roots as one coherent, policy-safe intelligence layer without breaking the v1.x single-root contract.

## Current Phase 3 Position
Phase 3 is no longer planning-only.
It is operationally complete.

That means:
1. root promotion/bootstrap work is complete for the current discovered queue
2. multi-root runtime behavior is live and signed off
3. the remaining work is no longer Phase 3 closeout; it is the next expansion phase

Phase 2 remains in maintenance mode in parallel:
1. representative nightlies are closed at `7/7`
2. repo-level hardening is no longer the main blocker
3. reruns now happen only after meaningful retrieval/indexing changes or when drift needs reconfirmation

## What Is Already Done
### 1. Root Promotion And Coverage
The current discovered filesystem-scope queue is fully covered.

Current monitor counts:
1. `uncovered=0`
2. `covered_gap=0`
3. `covered_partial=0`
4. `covered_representative=0`
5. `covered_ok=21`

That means the current backlog is now a monitor artifact, not an active promotion queue.

### 2. Multi-Root Config And Contract Layer
The explicit domain model is landed.

Delivered:
1. `workspace.domains` config support
2. single-root fallback when no explicit domains are configured
3. domain contract validation
4. unique domain id enforcement
5. trust-label validation
6. normalized root collision checks
7. root-overlap warnings
8. deterministic scheduler ordering
9. CLI and benchmark fail-fast behavior on invalid domain configs
10. `/health/domains` visibility

### 3. Runtime Multi-Root Behavior
Multi-root behavior is live in the runtime path.

Delivered:
1. `policy-guard` resolves owning domains for disk paths and virtual paths
2. `indexer` walks domain roots and applies per-domain allow/deny rules
3. `retrieval-core` derives trust and recency from the owning domain
4. `/raw` is domain-aware
5. `/map` is domain-aware
6. map lookup/readdir validate real indexed directories
7. full index builds now honor domain schedule rank instead of falling back to plain cross-root path order

### 4. Persisted Ownership Metadata
Domain ownership is stored in the indexed snapshot itself.

Delivered:
1. `files` stores `domain_id` and exact `trust_label`
2. `files` now also stores `modified_unix_ms` for snapshot-timestamped recency inputs
3. `chunks_meta` stores `domain_id` and exact `trust_label`
4. retrieval reads stored ownership metadata directly
5. optional LanceDB sync writes the same metadata
6. optional LanceDB retrieval reads those columns directly when present

### 5. Multi-Root Retrieval Hardening
Several Phase 3 quality improvements are already landed:
1. repeated same-file hits are collapsed before final search output
2. config-like literals get targeted config-path priors
3. narrative docs queries prefer docs over scripts
4. command-like queries prefer scripts over docs/config/code
5. per-search prior work is cached by path to reduce repeated work inside a single query
6. benchmark `rg` baseline now uses `--` for literal safety
7. benchmark `rg` baseline now drops out-of-domain paths in explicit multi-root mode
8. narrative-heavy docs queries now trim vector fanout when top BM25 hits already carry docs signal
9. `semanticfs-cli` now has a regression test that locks top-level `.` baseline normalization to the configured `workspace_meta` allow-roots
10. exact symbol-like queries with exact hits now use an exact-symbol fast path instead of paying the full generic fusion path

### 6. Symbol Coverage Hardening For Multi-Root Suites
Recent symbol extraction hardening now covers:
1. `pub(crate)` and other scoped Rust functions
2. `pub struct`
3. scoped `pub(...) struct`
4. `pub enum`
5. scoped `pub(...) enum`
6. `pub trait`
7. scoped `pub(...) trait`

This specifically recovered rank-1 behavior for queries like `ResolvedPath` and `map_dir_entries`.

### 7. Tracked Explicit Multi-Root Contract Fixture
There is now a stable tracked benchmark fixture for Phase 3.

Tracked config:
1. `config/relevance-multiroot.toml`

Tracked fixture:
1. `tests/retrieval_golden/semanticfs_multiroot_explicit.json`

Current tracked domain mix:
1. `workspace_meta`
2. `code`
3. `docs`
4. `config`
5. `scripts`
6. `systemd`
7. `github`
8. `fixture_repo`

Latest measured sign-off result on the current tracked fixture:
1. Relevance:
   - current active snapshot is `active_version=184`
   - recall `1.0000`
   - MRR `1.0000`
   - symbol-hit `1.0000`
2. Head-to-head:
   - correctness stayed green across three consecutive warmed median-of-3 reruns on the current active snapshot (`active_version=184`): SemanticFS recall `1.0000`, MRR `1.0000`, symbol-hit `1.0000`
   - the latest saved head-to-head artifact is SemanticFS p95 `42.989 ms` vs baseline `rg` p95 `28.468 ms`
   - the three consecutive warmed reruns now stay in a much tighter band: SemanticFS p95 `42.989-53.384 ms`, baseline `rg` p95 `28.468-37.609 ms`
3. E2E benchmark run:
   - `4/4` checks passed

Interpretation:
1. SemanticFS is ahead on recall
2. SemanticFS is ahead on MRR
3. SemanticFS is ahead on symbol-hit
4. the runtime contract is correct and the tracked p95 signal is now stable enough to close Phase 3
5. Structured-literal vector gating and symbol-hit vector gating are now both landed, which removes avoidable vector work on the current tracked suite
6. Narrative-heavy docs queries now also trim vector fanout when BM25 already shows docs signal, which cut the broader-fixture p95 cost without changing tracked correctness
7. The tracked contract now includes a real top-level workspace-metadata domain (`Cargo.toml`, `Cargo.lock`, `README.md`)
8. The previous top-level baseline leak (`workspace_meta/tests/...`) is now fixed in benchmark normalization by enforcing per-domain allow/deny policy on baseline paths
9. The top-level `.` normalization rule is now pinned by a `semanticfs-cli` regression test
10. The previously weak tracked literals (`m08`, `m09`, `m10`) are still rank `1`
11. The `.github` workflow domain is included in the tracked contract and validates at rank `1`
12. The `fixture_repo` domain adds a real multi-file mixed-content subtree and all tracked queries are now rank `1`
13. Workflow-style and systemd-unit-style queries are explicit tracked cases and validate at rank `1`
14. The `systemd` domain covers the full multi-file service set, not just one curated unit
15. `docs/runbook.md` is an explicit tracked operational-doc case and validates at rank `1`
16. `files.modified_unix_ms` is now persisted in the index, but the runtime currently only consumes it on exact-symbol hits, where it is already free from the existing `files` join
17. the exact-symbol path now probes the indexed `symbols(symbol_name, index_version)` path first and only falls back to case-folded matching if the indexed probe misses
18. BM25 now removes case-only duplicate query variants before querying FTS
19. BM25 now also pushes the existing workflow/systemd/script intent classes into SQL-side path filtering so those literal searches do less cross-domain work before ranking
20. head-to-head now performs one untimed warm-up plus median-of-3 timed samples per query for both SemanticFS and `rg`, which removed the earlier single-sample swings that were blocking sign-off

## What Moves To The Next Phase
Phase 3 closeout is complete.
The remaining work is broader expansion, not Phase 3 repair.

### 1. Broaden Beyond The Current Phase 3 Contract
The current eight-domain contract is now the signed-off baseline.

The next phase should:
1. add new domain classes beyond `workspace_meta` + `code` + `docs` + `config` + `scripts` + `systemd` + `github` + `fixture_repo`
2. keep the current `m01`-`m25` suite as the regression gate while broadening
3. avoid adding multiple new domain classes at once

### 2. Continue Performance Work Without Reopening The Contract
The tracked suite is now stable enough for sign-off, but it is still not faster than `rg` on the broader mixed-domain contract.

The next phase can keep improving:
1. the remaining absolute p95 gap vs baseline
2. the narrative/workflow/systemd cluster (`m05`, `m11`, `m12`, `m13`, `m19`, `m20`, `m21`, `m22`)
3. search-time metadata usage only when it matches or beats the signed-off Phase 3 latency band

### 3. Keep The Runtime Contract Stable While Expanding
As more domains are added in the next phase, preserve:
1. deterministic ownership
2. no silent cross-root ambiguity
3. consistent `/raw` verification semantics
4. stable benchmark comparisons

### 4. Keep Phase 2 In Maintenance Mode
Phase 2 is still not the main workstream, but it keeps the repo-first baseline anchored:
1. representative nightlies stay on maintenance cadence
2. rerun after meaningful retrieval/indexing changes
3. preserve the green representative baseline while Phase 3 keeps evolving

## Current Risks
### 1. Over-Broad Intent Priors
Phase 3 now uses more intent-aware ranking.
That is useful, but it creates a risk of over-boosting one content class.

Watch for:
1. scripts over-ranking on documentation queries
2. docs over-ranking on code-symbol queries
3. config priors leaking into non-config literals

### 2. Fixture Growth Without Discipline
The tracked multi-root fixture is useful only if it stays intentional.

Avoid:
1. adding vague queries
2. adding redundant queries
3. adding domains faster than the benchmark still clearly measures

### 3. Phase 3 Scope Creep
The project is moving fast, but Phase 3 should still stay controlled.

Avoid:
1. flipping into whole-machine indexing by default
2. adding unbounded background work
3. blurring trust boundaries just to add more roots quickly

## Immediate Next Steps
### Next Step 1
Treat `semanticfs_multiroot_explicit_v9` as the frozen Phase 3 sign-off suite.

Keep using it to:
1. protect the signed-off runtime contract
2. catch regressions before broader expansion
3. keep single-root behavior stable while the next phase broadens scope

### Next Step 2
Start the next expansion phase from a stable baseline.

That means:
1. add one new low-risk domain class at a time
2. keep top-level `workspace_meta` normalization pinned
3. keep all new roots explicit, domain-bounded, and benchmarked

### Next Step 3
Keep reruns narrow by default.

Do not expand into broad sweeps unless:
1. retrieval/indexing behavior changed materially
2. or a new root/domain class is added

## Working Rules For Phase 3
1. Keep single-root behavior stable by default.
2. Keep `/raw` as the final deterministic truth path.
3. Treat the tracked explicit multi-root suite as the primary Phase 3 regression signal.
4. Treat filesystem backlog artifacts as monitor artifacts unless new roots are discovered.
5. Prefer narrow reruns over broad sweeps.

## Primary Files For This Phase
1. `docs/phase3_execution_plan.md`
2. `docs/phase3_execution_status.md`
3. `docs/new-chat-handoff.md`
4. `docs/v1_2_execution_plan.md`
5. `docs/benchmark.md`
6. `config/relevance-multiroot.toml`
7. `tests/retrieval_golden/semanticfs_multiroot_explicit.json`

## Short Summary
Phase 3 is implemented and signed off.

The main transition is already complete:
1. from repo-only proof
2. to real multi-root runtime behavior

The remaining work is:
1. future domain broadening
2. additional performance polish
3. broader system-scope expansion in the next phase
