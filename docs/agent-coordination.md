## SemanticFS Agent Coordination

This document explains how multiple agents should work in parallel on this repo, which branches they use, and what is in or out of scope for each workflow.

It is intentionally detailed so you (and any agents) can avoid stepping on each other’s work.

---

## Branches and Workstreams

- **Branch `v1.2/testing-grounding`**  
  - **Workstream name**: v1.2 Testing & Grounding  
  - **Purpose**: Improve confidence in retrieval quality, benchmarks, and release safety for the current repo-first product.

- **Branch `phase3/filesystem-scope`**  
  - **Workstream name**: Filesystem-Scope Exploration (Phase 3 prep)  
  - **Purpose**: Explore and prepare for multi-root / filesystem-level expansion, without blocking or destabilizing v1.2.

Each agent should stay on its assigned branch unless explicitly instructed otherwise.

---

## Agent A: v1.2 Testing & Grounding (`v1.2/testing-grounding`)

### Goal

Make the **current repo-first SemanticFS**:

- Measurably accurate (recall, MRR, symbol hit rate).  
- Honest and well-grounded (no “cheating” benchmarks).  
- Operationally safe (release gate, soak, drift checks) for real repos.

### Primary Responsibilities

- **Golden suite quality**
  - Refine and extend `tests/retrieval_golden/*.json` for:
    - `semanticfs_repo_v1`
    - `ai_testgen_repo_v1`
    - Other curated suites (buckit, tensorflow_models, etc.).
  - Add **rg-friendly / neutral** queries where appropriate so comparisons are not one-sided.
  - Keep symbol vs non-symbol coverage explicit (`symbol_query` flag, mixed suites).

- **Benchmark harness & metrics**
  - Maintain and improve:
    - `crates/semanticfs-cli/src/benchmark.rs`
    - `docs/benchmark.md`
    - `scripts/daytime_smoke.ps1`
    - `scripts/nightly_representative.ps1`
    - `scripts/daytime_tune_holdout.ps1`
    - `scripts/daytime_action_items.ps1`
  - Ensure metrics are:
    - Clearly defined (recall, MRR, symbol-hit, p50/p95/max).  
    - Correctly implemented (fair top-N, consistent path normalization).  
    - Reported per suite and per query-type (symbol vs non-symbol where relevant).

- **Release gate and drift**
  - Own the configuration and documentation for:
    - `benchmark release-gate` behavior and thresholds.  
    - Drift summary tooling (e.g. `scripts/drift_summary.ps1`).  
  - Ensure:
    - Strict thresholds are realistic and meaningful for representative suites.  
    - Release gate artifacts are interpretable (`.semanticfs/bench/release_gate.json`).  
    - Drift summaries capture relevance/head-to-head/release-gate history in a way that is easy to read.

- **Documentation & honesty**
  - Update:
    - `README.md` (benchmark/metrics sections only).  
    - `docs/v1_2_execution_plan.md` (progress snapshots, metrics explanation).  
    - `docs/benchmark.md` (what each command checks, what each metric means).  
  - Make the **comparison story explicit**:
    - What tasks are being measured (definition-finding, semantic queries, literal queries).  
    - What baseline we compare to (plain `rg -F` with default ordering).  
    - Where SemanticFS wins, where baseline can win, and known gaps.

### Allowed Areas to Edit

- `tests/retrieval_golden/*.json`
- `crates/semanticfs-cli/src/benchmark.rs`
- `scripts/daytime_*.ps1`
- `scripts/nightly_*.ps1`
- `scripts/drift_summary.ps1`
- `docs/benchmark.md`
- Benchmark-related sections of:
  - `docs/v1_2_execution_plan.md`
  - `docs/future-steps-log.md`
  - `README.md` (metrics/validation sections only)

### Out of Scope for Agent A

- Implementing multi-root or filesystem-wide indexing.  
- Changing core indexer behavior for multi-root scheduling.  
- Adding new Phase 3 architecture (policy domains, multi-root schedulers, etc.).  
- Large refactors unrelated to testing/grounding.

If a change is required that clearly belongs to filesystem-scope or Phase 3, Agent A should leave a note in `docs/future-steps-log.md` and not implement it.

---

## Agent B: Filesystem Scope / Phase 3 Prep (`phase3/filesystem-scope`)

### Goal

Prepare SemanticFS for **system-scope expansion**:

- Explore and validate behavior on many diverse repos.  
- Build discovery and orchestration tools to support multiple roots.  
- Identify architectural and performance gaps that matter at filesystem level.

### Primary Responsibilities

- **Discovery tooling**
  - Own and extend:
    - `scripts/discover_repo_candidates.ps1`  
    - Any new discovery scripts or helpers invoked by `scripts/daytime_action_items.ps1`
  - Ensure discovery artifacts:
    - `.semanticfs/bench/filesystem_repo_candidates_latest.json`  
    - `.semanticfs/bench/filesystem_repo_candidates_userroot.json`
    are well-structured and documented in `docs/benchmark.md` / `docs/v1_2_execution_plan.md`.

- **External repo coverage**
  - Add and maintain external golden suites and splits:
    - `tests/retrieval_golden/*_bootstrap_v*.json`  
    - `*_tune.json` / `*_holdout.json` for external repos (e.g. `rlbeta`, `stockguessr`, `repo8872pp`, etc.).
  - Use strict tune/holdout runs to:
    - Identify quality gaps (where baseline beats SemanticFS).  
    - Identify latency gaps (where baseline is significantly faster).  
    - Record these clearly in `docs/future-steps-log.md` as filesystem-scope prep signals.

- **Filesystem-scope preparation**
  - Design and document, but do **not yet fully implement**, Phase 3 capabilities:
    - Multi-root indexing domains (what a “root” is, how policies attach).  
    - Scheduling strategies (which roots to index when, resource limits).  
    - High-level policy/governance model across roots (what must be default-deny, what is configurable).  
  - Add design notes and early prototypes under:
    - `docs/` (e.g. separate Phase 3 design doc).  
    - Clearly separated experimental code paths guarded by config/env flags (no behavior changes to current v1.2 defaults).

- **Feedback into v1.2**
  - When external runs expose gaps that matter to repo-first product (e.g. slow latency, poor recall on some patterns), file concise notes in:
    - `docs/future-steps-log.md` (filesystem-scope prep section).  
  - Agent B can suggest changes for Agent A to consider, but **should not** modify v1.2 acceptance criteria or release-gate behavior directly.

### Allowed Areas to Edit

- `scripts/discover_repo_candidates.ps1`
- `scripts/daytime_action_items.ps1` (discovery and external-suite orchestration sections)
- New external `tests/retrieval_golden/*.json` suites and their tune/holdout splits
- Phase 3 / filesystem-scope–related docs:
  - `docs/big-picture-roadmap.md` (Phase 3 sections only)
  - New design docs under `docs/` for system-scope
  - Filesystem prep sections of `docs/future-steps-log.md`

### Out of Scope for Agent B

- Changing v1.2 release-gate thresholds or definitions.  
- Modifying core golden suites for the representative v1.2 repos (`semanticfs_repo_v1`, `ai_testgen_repo_v1`) without coordination.  
- Altering `scripts/nightly_representative.ps1` semantics (that’s shared infra; changes should be coordinated and usually go via Agent A).

---

## Shared Guidelines and Coordination

### Communication via Docs

- Use `docs/future-steps-log.md` for:
  - New ideas, risks, and follow-ups that cross workstreams.  
  - Short notes like “Agent B: external suite X shows SF slower than rg; candidate issue: vector backend config.”

- Use `docs/v1_2_execution_plan.md` only for:
  - v1.2 status and acceptance criteria.  
  - Summaries of benchmark results that directly feed into v1.2 decisions.

### Avoiding Conflicts

- If both agents must touch the same file (e.g. `docs/v1_2_execution_plan.md`, `README.md`):
  - Limit changes to clearly separated sections (e.g. Agent A → metrics/status; Agent B → future/Phase 3 notes).  
  - Prefer appending new sections over rewriting shared ones.

- Prefer **adding new files** for Phase 3 design rather than overloading existing v1.2 docs.

### Merge Expectations

- `v1.2/testing-grounding`:
  - Merged into `main` when v1.2 acceptance criteria (including 7-night trend, release gate, and no P0/P1 issues) are met.

- `phase3/filesystem-scope`:
  - Merged when there is a coherent, tested slice of filesystem-scope prep or initial implementation that does not regress v1.2 guarantees.

Agents should assume that **v1.2/testing-grounding has priority** when there is a conflict that affects release stability.

---

## Quick Reference (For Agents)

- **Agent A (`v1.2/testing-grounding`)**
  - Focus: accuracy, benchmarks, release safety for repo-first v1.2.  
  - Owns: golden suites, benchmark harness, release gate, drift summaries, metrics docs.  
  - Avoids: multi-root/filesystem implementation work.

- **Agent B (`phase3/filesystem-scope`)**
  - Focus: filesystem-scope prep, external repos, discovery, Phase 3 design.  
  - Owns: repo discovery tooling, external golden suites/tune-holdout, system-scope design docs.  
  - Avoids: changing v1.2 acceptance criteria or core v1.2 benchmark semantics.

If a task doesn’t clearly belong to one agent, write a short note in `docs/future-steps-log.md` and defer until it’s explicitly assigned.

