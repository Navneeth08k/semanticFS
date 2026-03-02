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
Phase: `v1.2 reliability and quality hardening` with `Phase 3 bootstrap` running in parallel (as of March 1, 2026)

Current state:
1. Core architecture is implemented and operational.
2. Reliability/quality features are active (session pinning, queue planning, anti-shadowing priors).
3. Benchmark + gate tooling is in place, including head-to-head comparisons.
4. Representative nightly stability evidence is now closed at `7/7`, so Phase 2 no longer blocks daytime architecture work.
5. Phase 3 runtime has moved from config-only scaffolding into real explicit multi-root behavior (persisted domain metadata, domain-aware `/raw` and `/map`, tracked multi-root benchmarks).

Remaining phase focus:
1. Keep representative quality green on maintenance cadence instead of gating cadence.
2. Tighten remaining measured ranking inefficiencies rather than broad repo-level viability.
3. Continue Phase 3 runtime hardening from bootstrap behavior into broader multi-root/system-scope contracts.
4. Preserve deterministic verification boundaries while multi-root scope expands.

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
5. A tracked explicit multi-root benchmark suite is now in the repo and green on the current `code` + `docs` + `config` + `scripts` + `systemd` + `github` + `fixture_repo` contract fixture, with SemanticFS ahead on recall, MRR, symbol-hit, and p95 on the latest head-to-head run.

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
2. `docs/new-chat-handoff.md`
3. `docs/v1_2_execution_plan.md`
4. `docs/phase3_execution_plan.md`
5. `docs/future-steps-log.md`
6. `docs/benchmark.md`
