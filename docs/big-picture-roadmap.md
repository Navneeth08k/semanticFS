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
Phase: `v1.2 reliability and quality hardening` with `Phase 3 bootstrap` running in parallel (as of February 24, 2026)

Current state:
1. Core architecture is implemented and operational.
2. Reliability/quality features are active (session pinning, queue planning, anti-shadowing priors).
3. Benchmark + gate tooling is in place, including head-to-head comparisons.

Remaining phase focus:
1. Nightly trend stability over representative suites.
2. Threshold hardening from measured data.
3. FUSE long-lived session semantics parity with MCP session behavior.
4. Start non-breaking Phase 3 domain-model and system-scope planning scaffolding.

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
1. 7-night trend stability sequence.
2. Release-gate threshold tightening based on representative trend data.
3. FUSE long-lived session pin semantics.

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
3. First implementation target is non-breaking multi-root config/domain scaffolding.

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
