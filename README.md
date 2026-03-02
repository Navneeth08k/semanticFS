# SemanticFS

SemanticFS is a filesystem-wide intelligence layer for AI agents.
Currently (v1.x), SemanticFS is optimized for software repositories as we build toward full system-scope capabilities.
It keeps deterministic file truth (`/raw`) while adding semantic discovery (`/search`) and orientation summaries (`/map`).

## Purpose
Traditional agent tooling wastes turns finding context.
Agents like OpenClaw can burn time and tokens repeatedly `cd`-ing, `ls`-ing, and `grep`-ing through the filesystem just to locate the right files.
SemanticFS moves retrieval into a filesystem-shaped interface so agents can use normal file operations to get grounded results faster.
That is why `/search` exists: instead of wandering the tree manually, an agent can ask for the relevant files directly and then verify the final target through `/raw`.

Core design goal:
1. Probabilistic discovery.
2. Deterministic verification before edits.

## Product Shape (v1.x)
SemanticFS is currently optimized for software repositories, not whole-machine indexing.

Provided surfaces:
1. `/raw/<path>`: byte-accurate passthrough.
2. `/search/<query>.md`: hybrid retrieval results with grounded paths/lines.
3. `/map/<dir>/directory_overview.md`: deterministic summaries with optional async enrichment.
4. MCP minimal server for agent discovery/use.

## Architecture

Request flow:
1. Files change in repo.
2. `indexer` updates metadata/symbols/vectors and publishes a new index version via two-phase publish.
3. `fuse-bridge` serves virtual paths against a snapshot.
4. `retrieval-core` executes symbol-first + BM25 + vector fusion and applies policy + ranking priors.
5. Agent verifies final edit targets through `/raw`.

Main crates:
1. `semanticfs-common`: shared config/types.
2. `policy-guard`: filtering, secret heuristics, redaction.
3. `indexer`: chunking/symbol extraction/embedding/map precompute/watch.
4. `retrieval-core`: hybrid planner + RRF + priors.
5. `map-engine`: map summary read path + enrichment merge.
6. `fuse-bridge`: virtual FS rendering + caches + stats.
7. `mcp`: minimal MCP-compatible tool/resource server.
8. `semanticfs-cli`: operational commands and benchmarks.

## How retrieval works

Search runs **symbol lookup**, **BM25**, and **vector search** in parallel, then fuses and re-ranks results. The diagrams below render on GitHub and other Mermaid-capable viewers.

### Retrieval pipeline

```mermaid
graph TB
    subgraph Input
        Q[Query string]
    end

    subgraph Pipelines
        SE[Symbol exact]
        SP[Symbol prefix]
        BM[BM25 chunk text]
        V[Vector search]
    end

    subgraph Merge
        RRF[RRF fuse]
        Prior[Path and recency priors]
        Top[Take top N]
    end

    Q --> SE
    Q --> SP
    Q --> BM
    Q --> V
    SE --> RRF
    SP --> RRF
    BM --> RRF
    V --> RRF
    RRF --> Prior
    Prior --> Top
    Top --> Hits[path, start_line, end_line]
```

### Symbol search (e.g. function or class name)

```mermaid
sequenceDiagram
    participant U as User
    participant S as Search
    participant Sym as Symbol index
    participant RRF as RRF plus priors

    U->>S: query parse_config
    S->>Sym: exact symbol lookup
    Sym-->>S: path, line_start, line_end
    S->>S: BM25 and vector search
    S->>RRF: fuse all lists
    RRF-->>S: fused ranking
    S->>S: apply priors, take top N
    S-->>U: path and line ranges
```

### Semantic search (e.g. natural-language intent)

```mermaid
sequenceDiagram
    participant U as User
    participant S as Search
    participant BM25 as BM25
    participant Emb as Embedding and vector DB
    participant RRF as RRF plus priors

    U->>S: where do we validate user input
    S->>BM25: full-text on chunk text
    BM25-->>S: chunks containing validate, user, input
    S->>Emb: embed query
    Emb->>Emb: nearest chunk embeddings
    Emb-->>S: semantically similar chunks
    S->>RRF: fuse lists
    RRF-->>S: fused ranking
    S->>S: path prior, recency prior
    S-->>U: path and line ranges
```

### Literal search (e.g. string in file)

```mermaid
sequenceDiagram
    participant U as User
    participant S as Search
    participant BM25 as BM25
    participant V as Vector
    participant RRF as RRF plus priors

    U->>S: TODO fix this
    S->>BM25: full-text TODO fix this
    BM25-->>S: chunks with those terms
    S->>V: vector search
    V-->>S: vector hits
    S->>RRF: fuse lists
    RRF-->>S: fused ranking
    S->>S: priors, top N
    S-->>U: path and line ranges
```

### All query types use the same process

```mermaid
graph LR
    subgraph Situations
        S1[Symbol search]
        S2[Semantic search]
        S3[Literal search]
    end

    subgraph SameProcess
        A[Symbol exact and prefix]
        B[BM25]
        C[Vector search]
        D[RRF fuse]
        E[Path prior]
        F[Recency prior]
        G[Top N]
        A --> D
        B --> D
        C --> D
        D --> E
        E --> F
        F --> G
    end

    S1 --> SameProcess
    S2 --> SameProcess
    S3 --> SameProcess
```

## Current State (As of March 1, 2026)
Implemented:
1. Core `/raw` + `/search` + `/map` behavior.
2. Snapshot versioning and two-phase publish.
3. Symbol-first hybrid retrieval (symbol + BM25 + vector).
4. Policy guard at indexing and retrieval/render.
5. Async `/map` enrichment worker.
6. MCP session pinning (`session_id`, `refresh_session`).
7. Branch-swap queue planning with in-progress status signaling.
8. Anti-shadowing ranking priors (file-type + recency).
9. FUSE long-lived session pinning with explicit refresh/status control files.
10. Benchmark suite: run/soak/relevance/release-gate/head-to-head.
11. Mounted Linux FUSE workflow validation for `/.well-known/session.json` and `/.well-known/session.refresh` in WSL long-lived session.
12. Strict daytime tune-vs-holdout workflow with deterministic suite splitting and holdout-only final reporting.
13. Phase 3 bootstrap has started in parallel: non-breaking multi-root domain scaffolding, filesystem domain-plan artifacts, and explicit multi-root runtime ownership are now landed while single-root remains the default mode.
14. Phase 3 runtime now persists domain ownership into indexed file/chunk metadata, serves `/map` through domain-aware directory summaries and enrichment, and includes a tracked explicit multi-root benchmark config + fixture (`code` + `docs` + `config` + `scripts` + `systemd` + `github` + `fixture_repo`).

Known constraints:
1. Default embedding runtime is `hash` unless ONNX is configured.
2. FUSE mount path is Linux-only; Windows/macOS can still use indexing/retrieval/MCP/benchmarks.

## Quickstart
1. Build:
```bash
cargo build
```
2. Copy config and set repo path:
```bash
cp config/semanticfs.sample.toml local.toml
```
3. Build index:
```bash
cargo run -p semanticfs-cli -- --config local.toml index build
```
4. Start MCP server:
```bash
cargo run -p semanticfs-cli -- --config local.toml serve mcp
```
5. Run benchmark in release mode:
```bash
cargo run --release -p semanticfs-cli -- --config local.toml benchmark run --soak-seconds 30
```

## Validation Commands
1. Relevance:
```bash
cargo run --release -p semanticfs-cli -- --config local.toml benchmark relevance --fixture-repo /abs/repo --golden-dir tests/retrieval_golden --history
```
2. Head-to-head (SemanticFS vs `rg` baseline):
```bash
cargo run --release -p semanticfs-cli -- --config local.toml benchmark head-to-head --fixture-repo /abs/repo --golden-dir tests/retrieval_golden --history
```
3. Daytime smoke:
```powershell
powershell -ExecutionPolicy Bypass -File scripts/daytime_smoke.ps1 -SoakSeconds 2
```
4. Nightly sequence:
```powershell
powershell -ExecutionPolicy Bypass -File scripts/nightly_bench.ps1 -ConfigPath config/semanticfs.sample.toml -FixtureRepo tests/fixtures/benchmark_repo -GoldenDir tests/retrieval_golden -SoakSeconds 30
```
5. Representative nightly (semanticFS + ai-testgen suites + strict release gate):
```powershell
powershell -ExecutionPolicy Bypass -File scripts/nightly_representative.ps1 -SoakSeconds 30
```
6. Mounted Linux FUSE session validation (WSL):
```powershell
wsl -d Ubuntu -- bash -lc 'cd /mnt/c/path/to/semanticFS && bash scripts/wsl_run_fuse_session_validation.sh'
```
7. Drift summary (history counts + deltas + date coverage):
```powershell
powershell -ExecutionPolicy Bypass -File scripts/drift_summary.ps1
```
8. Daytime full action runner (split + smoke + tune/holdout + drift summary):
```powershell
powershell -ExecutionPolicy Bypass -File scripts/daytime_action_items.ps1 -SoakSeconds 2
```
Optional strict gate variant:
```powershell
powershell -ExecutionPolicy Bypass -File scripts/daytime_action_items.ps1 -SoakSeconds 2 -IncludeReleaseGate
```
9. Tune-vs-holdout selection on a repo:
```powershell
powershell -ExecutionPolicy Bypass -File scripts/daytime_tune_holdout.ps1 -Label repo -RepoRoot C:\path\repo -BaseConfig config/relevance-real.toml -TuneGolden tests/retrieval_golden/repo_tune.json -HoldoutGolden tests/retrieval_golden/repo_holdout.json -History
```
10. Deterministic split from bootstrap suite:
```bash
python scripts/split_golden_suite.py --input tests/retrieval_golden/repo_bootstrap.json --tune-output tests/retrieval_golden/repo_tune.json --holdout-output tests/retrieval_golden/repo_holdout.json --tune-count 10
```
11. Curated mixed split build from expanded bootstrap suite:
```bash
python scripts/build_curated_mixed_suites.py --input tests/retrieval_golden/repo_bootstrap_v2_full.json --tune-output tests/retrieval_golden/repo_curated_tune.json --holdout-output tests/retrieval_golden/repo_curated_holdout.json --split-size 40 --non-symbol-per-split 10 --dataset-prefix repo
```
12. Config-aligned bootstrap generation for a scoped repo:
```bash
python scripts/bootstrap_golden_from_repo.py --repo-root C:\path\repo --config config/relevance-ai-testgen.toml --output tests/retrieval_golden/repo_bootstrap.json --dataset-name repo_bootstrap_v1 --max-queries 20
```
Optional faster mode for large git repos:
```bash
python scripts/bootstrap_golden_from_repo.py --repo-root C:\path\repo --git-tracked-only --output tests/retrieval_golden/repo_bootstrap.json --dataset-name repo_bootstrap_v1 --max-queries 20
```
12. Filesystem candidate discovery (workspace mirrors excluded + remote dedupe by default):
```powershell
powershell -ExecutionPolicy Bypass -File scripts/discover_repo_candidates.ps1 -Roots C:\Users\<user> -MinTrackedFiles 80 -TopN 80 -OutputPath .semanticfs/bench/filesystem_repo_candidates_min80.json
```
13. Filesystem backlog build (prioritized uncovered/gap/partial/representative/ok queue):
```powershell
powershell -ExecutionPolicy Bypass -File scripts/build_filesystem_scope_backlog.ps1 -CandidatesPath .semanticfs/bench/filesystem_repo_candidates_min80.json -OutputPath .semanticfs/bench/filesystem_scope_backlog_latest.json
```
14. Phase 3 domain-plan build from latest backlog:
```powershell
powershell -ExecutionPolicy Bypass -File scripts/build_phase3_domain_plan.ps1 -BacklogPath .semanticfs/bench/filesystem_scope_backlog_latest.json -OutputPath .semanticfs/bench/filesystem_domain_plan_latest.json
```
15. Query gap report for targeted hardening:
```powershell
powershell -ExecutionPolicy Bypass -File scripts/build_query_gap_report.ps1 -DatasetName repo8872pp_bootstrap_v1_holdout_v1
```
16. Explicit multi-root benchmark (tracked `code` + `docs` + `config` + `scripts` + `systemd` domains):
```bash
cargo run --release -p semanticfs-cli -- --config config/relevance-multiroot.toml benchmark relevance --golden tests/retrieval_golden/semanticfs_multiroot_explicit.json --history
cargo run --release -p semanticfs-cli -- --config config/relevance-multiroot.toml benchmark head-to-head --golden tests/retrieval_golden/semanticfs_multiroot_explicit.json --history
```
The tracked fixture is now `code` + `docs` + `config` + `scripts` + `systemd` + `github` + `fixture_repo`. Latest result on active version `1`: relevance recall `1.0000`, MRR `1.0000`, symbol-hit `1.0000`; head-to-head gives SemanticFS recall `1.0000`, MRR `1.0000`, symbol-hit `1.0000`, p95 `27.644 ms` vs `rg` recall `0.9375`, MRR `0.8021`, symbol-hit `0.4000`, p95 `30.716 ms`. All 16 tracked queries are currently rank `1`; the remaining Phase 3 work is holding this broader seven-domain contract green while expanding toward less synthetic mixed-content roots.

## Documentation Map
Use these docs by role:
1. `docs/new-chat-handoff.md`: current status, exact next steps, execution order.
2. `docs/big-picture-roadmap.md`: multi-phase product direction and guardrails.
3. `docs/v1_2_execution_plan.md`: v1.2 scope, acceptance criteria, active work items.
4. `docs/phase3_execution_plan.md`: parallel Phase 3 bootstrap scope and execution order.
5. `docs/phase3_execution_status.md`: current Phase 3 status, completed work, open work, and exact next steps.
6. `docs/future-steps-log.md`: running backlog/history of discussed future work.
7. `docs/benchmark.md`: command reference and artifact semantics.
8. `docs/implemented-v1_1.md`: v1.1 implementation baseline.
9. `docs/release-v1_1_0-rc1.md`: release gate checklist.
10. `docs/README.md`: documentation index and read order.

## New Chat Bootstrap
If starting a fresh assistant chat, read in this order:
1. `README.md`
2. `docs/new-chat-handoff.md`
3. `docs/v1_2_execution_plan.md`
4. `docs/phase3_execution_plan.md`
5. `docs/phase3_execution_status.md`
6. `docs/future-steps-log.md`
7. `docs/benchmark.md`

This sequence is the source of truth for current priorities.
