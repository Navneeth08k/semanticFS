# SemanticFS Big-Picture Roadmap

## Purpose
This is the long-lived product direction anchor.
Use it to keep implementation decisions connected to the broader goal.

## Vision
1. Short term: repo-first intelligence layer for coding agents.
2. Mid term: highly reliable, measurable, secure agent context substrate.
3. Long term: multi-root, system-scope intelligence interface with strong governance.

Core invariant:
1. Discovery is semantic.
2. Edits are grounded through deterministic source reads.

## Current Phase
Phase: `v1.2 maintenance` with `Phase 8 scaling and pressure hardening` active (as of March 3, 2026)

Current state:
1. Core architecture is implemented and operational.
2. Reliability/quality features are active (session pinning, queue planning, anti-shadowing priors).
3. Benchmark + gate tooling is in place, including head-to-head comparisons.
4. Representative nightly stability evidence is now closed at `7/7`, so Phase 2 no longer blocks daytime architecture work.
5. Phase 3 runtime is now operationally complete as a signed-off explicit multi-root foundation (persisted domain metadata, domain-aware `/raw` and `/map`, tracked multi-root benchmarks).
6. Phase 4 is now operationally complete: one additional domain class was added beyond the Phase 3 baseline, the broadened suite is green, and first-pass resource-aware watch scheduling is landed.
7. Phase 5 is now operationally complete: per-domain watch participation and watch priority are live, indexing is decoupled from watch participation, and the new `governance` domain is signed off under the `v11` broadened suite.
8. Phase 6 is now operationally complete: per-domain indexing budgets are live, a new bounded `inventory` domain is signed off under the `v12` broadened suite, and host-scale index breadth is now explicitly budgeted.
9. Phase 7 is now operationally complete: a three-domain bounded batch (`profiles`, `operations`, `intake`) is signed off under the `v13` broadened suite, proving that expansion can widen faster than one domain at a time without breaking the frozen gates.
10. The active work is the post-Phase-7 scaling phase: hold the frozen gates green, reduce broader-batch latency, and improve scheduler visibility before expanding again.
11. The first Phase 8 slice is now landed: CLI health exposes aggregate scan/watch pressure, and broader-batch narrative queries use tighter BM25/vector fanout when BM25 already has signal.

Remaining phase focus:
1. Keep representative quality green on maintenance cadence instead of gating cadence.
2. Treat the signed-off Phase 3 (`v9`), Phase 4 (`v10`), Phase 5 (`v11`), Phase 6 (`v12`), and Phase 7 (`v13`) suites as frozen regression gates.
3. Deepen scheduler/resource behavior beyond the new per-domain watch controls and index-breadth budgets, specifically for broader batched expansion.
4. Reduce broader-batch latency without regressing the frozen multi-root suites.
5. Preserve deterministic verification boundaries while system scope expands.

## Phases
## Phase 1: v1.1 Repo-First Foundation
Goal:
1. Ship core architecture with measurable ops baseline.

Delivered:
1. `/raw`, `/search`, `/map`.
2. Two-phase publish and snapshot reads.
3. Policy guard, MCP minimal surface, observability.
4. Benchmark/runbook/release-gate scaffolding.

## Phase 2: v1.2 Reliability and Quality
Goal:
1. Improve retrieval quality and real-world operational confidence.

Delivered so far:
1. Expanded golden suites and relevance harness.
2. Head-to-head benchmark (`SemanticFS` vs `rg` baseline).
3. Breadcrumb grounding contract.
4. MCP session pinning + refresh control.
5. Branch-swap queue planning with in-progress signaling.
6. Anti-shadowing priors.

Still open in this phase:
1. Keep representative nightlies on maintenance cadence and watch for drift after retrieval/ranking changes.
2. Keep release-gate thresholds stable while Phase 3 runtime changes continue landing.
3. Re-run FUSE long-lived session validation only when the session/mount path changes.

## Phase 3: System-Scope Expansion (Major Re-scope)
Goal:
1. Move beyond repo-first into policy-safe multi-root intelligence interface.

Required capabilities:
1. Multi-root indexing domains and scheduling.
2. Rich non-code content handling at scale.
3. Stronger identity and data-governance boundaries.
4. Predictable memory/latency behavior across heterogeneous workloads.

Bootstrap status:
1. Started in parallel with late Phase 2 hardening.
2. Initial discovery and backlog artifacts are now in place.
3. Non-breaking multi-root config/domain scaffolding is landed.
4. Explicit multi-root runtime ownership is now active across indexing, retrieval, `/raw`, and `/map`.
5. A tracked explicit multi-root benchmark suite is now in the repo and green on the current `workspace_meta` + `code` + `docs` + `config` + `scripts` + `systemd` + `github` + `fixture_repo` contract fixture.
6. Phase 3 is now operationally complete as the signed-off multi-root runtime baseline.

## Phase 4: Controlled Domain Expansion
Goal:
1. Expand beyond the signed-off Phase 3 contract into broader, more realistic system-scope coverage.

Required capabilities:
1. Add new domain classes one at a time with explicit expected-path benchmarks.
2. Evolve scheduler behavior from deterministic ordering into resource-aware orchestration.
3. Preserve trust/policy boundaries as heterogeneous non-code roots are added.
4. Keep the Phase 3 `v9` tracked suite green while broader suites expand.
5. Move closer to practical filesystem-scope coverage without enabling uncontrolled whole-machine indexing by default.

Initial operating model:
1. Freeze the Phase 3 `v9` suite as the regression baseline.
2. Add one bounded, low-risk domain class at a time.
3. Keep reruns narrow and benchmark-driven.
4. Treat broader machine scope as opt-in, policy-bounded, and measurable.
5. Current status: this Phase 4 bar is now met on top of the new `playbooks` domain and the broadened `v10` suite.

## Phase 5: Adaptive Expansion And Governance Hardening
Goal:
1. Turn one successful controlled expansion into a repeatable system-scope expansion model.

Required capabilities:
1. Per-domain orchestration controls beyond one global watch cap.
2. Additional bounded domain classes beyond the Phase 4 baseline.
3. Stronger governance rules for how new roots enter the system.
4. Frozen Phase 3/4 regression gates plus one active expansion candidate suite.
5. Continued deterministic `/raw` verification while broader roots are introduced.

Current operating model:
1. `semanticfs_multiroot_explicit_v9` remains the frozen Phase 3 gate.
2. `semanticfs_multiroot_explicit_v10` remains the frozen Phase 4 broadened baseline.
3. `semanticfs_multiroot_explicit_v11` is the signed-off Phase 5 broadened baseline.
4. `workspace.domains[].watch_enabled` and `workspace.domains[].watch_priority` are now the minimum per-domain orchestration layer.
5. Current status: this Phase 5 bar is now met on top of the new `governance` domain and the broadened `v11` suite.

## Phase 6: Host-Scale Budgeted Expansion
Goal:
1. Add explicit per-domain index-breadth controls before broader host-scale roots are introduced.

Required capabilities:
1. Per-domain indexing budgets in the runtime contract.
2. Health visibility for those budgets.
3. Another bounded domain slice beyond the Phase 5 baseline.
4. Frozen Phase 3/4/5 regression gates plus one active broadened Phase 6 suite.
5. Continued deterministic `/raw` verification while index breadth becomes more explicitly bounded.

Current operating model:
1. `semanticfs_multiroot_explicit_v9` remains the frozen Phase 3 gate.
2. `semanticfs_multiroot_explicit_v10` remains the frozen Phase 4 broadened baseline.
3. `semanticfs_multiroot_explicit_v11` remains the frozen Phase 5 broadened baseline.
4. `semanticfs_multiroot_explicit_v12` is the signed-off Phase 6 broadened baseline.
5. `workspace.domains[].max_indexed_files` is now the minimum host-scale index-breadth control.
6. Current status: this Phase 6 bar is now met on top of the new `inventory` domain and the broadened `v12` suite.

## Phase 7: Batched Bounded Expansion
Goal:
1. Widen scope faster by landing several bounded roots in one slice instead of one-at-a-time.

Required capabilities:
1. Add `3-5` low-risk bounded domains in one batch.
2. Keep per-domain caps active for every new root in the batch.
3. Preserve the frozen Phase 3/4/5/6 regression gates while the batch lands.
4. Explicitly measure the latency impact of the broader batch.
5. Keep deterministic `/raw` verification unchanged.

Current operating model:
1. `semanticfs_multiroot_explicit_v9` remains the frozen Phase 3 gate.
2. `semanticfs_multiroot_explicit_v10` remains the frozen Phase 4 broadened baseline.
3. `semanticfs_multiroot_explicit_v11` remains the frozen Phase 5 broadened baseline.
4. `semanticfs_multiroot_explicit_v12` remains the frozen Phase 6 broadened baseline.
5. `semanticfs_multiroot_explicit_v13` is the signed-off Phase 7 broadened baseline.
6. Current status: this Phase 7 bar is now met on top of the new `profiles`, `operations`, and `intake` domains and the broadened `v13` suite.

## Decision Guardrails
1. Grounded edits over clever retrieval.
2. Measured performance over intuition.
3. Security controls at both index-time and retrieval-time.
4. Release decisions based on `--release` artifacts only.
5. Probabilistic retrieval must never be treated as authoritative execution truth.

## Misconceptions To Avoid
1. Current SemanticFS is not a full semantic OS for the whole machine.
2. It is a strong repo-first substrate that can evolve toward system scope.
3. Moving to system scope is a product/architecture phase change, not a config tweak.

## Source Of Truth Links
1. `README.md`
2. `docs/big-picture-roadmap.md`
3. `docs/current_execution_plan.md`
4. `docs/future-steps-log.md`
5. `docs/benchmark.md`
