# SemanticFS

**A filesystem-wide intelligence layer for AI coding agents.**

SemanticFS replaces manual codebase navigation (endless `ls`, `grep`, `cat` chains) with a semantic search interface. Agents ask *where* relevant code is, get back precise file paths and line ranges, verify through a byte-accurate read, then act — spending no tokens on exploration.

---

## Why it exists

When an AI agent works on a large codebase without SemanticFS, it burns context doing this:

```
ls src/
find . -name "*.py" | head -40
grep -r "signature" . | head -20
cat adapters/java_adapter.py
cat ai_testgen_core/diff_parser.py
...
```

Every directory listing, every file read, every grep output costs input tokens. In large repos — or repos contaminated with `node_modules`, `.venv`, `target/` — this gets expensive fast.

SemanticFS replaces that with one call:

```
search("Python function signature extraction from git diff")
→ ai_testgen_core/diff_parser.py:40-95  (extract_signatures_python)
```

---

## Measured results

Real head-to-head run on the `ai-testgen` repo (4,638 total files including `.venv`, 24 real source files). Same 6 tasks. Same model (Claude Sonnet 4.6). Both modes got 6/6 correct.

![Summary](docs/charts/chart_summary.svg)

![Context tokens per task](docs/charts/chart_ctx_tokens.svg)

![Token savings per task](docs/charts/chart_savings.svg)

![Cost per task](docs/charts/chart_cost.svg)

| Metric | Naive (Bash only) | + SemanticFS MCP | Reduction |
|---|---|---|---|
| Context tokens | 21,536 | 7,799 | **63.8%** |
| API cost | $0.2064 | $0.1466 | **29.0%** |
| Avg turns | 3.8 | 3.5 | **8%** |
| Accuracy | 6/6 (100%) | 6/6 (100%) | same |

The standout case: finding the CLI entry point cost **4,265 context tokens** naively (the agent had to explore directories, read multiple files). With SemanticFS it cost **5 tokens** — the search result pointed directly to `cli.py` and the agent answered immediately.

> **Note on timing:** SemanticFS wall-clock was slower in this test because each `claude --print` invocation cold-starts a new MCP subprocess. In a real Claude Code session, the MCP server starts once and persists. Real-world latency for SemanticFS is lower than naive (fewer tool-call round trips).

---

## How it works

### The agent workflow

```
Agent asks question
    │
    ▼
search("where is X")          ← ONE call, returns file:line ranges
    │
    ▼
raw_read("path/to/file:40-95") ← byte-accurate verification
    │
    ▼
Agent acts with grounded context
```

**Core invariant:** discovery is probabilistic (semantic search), verification is deterministic (`/raw` always returns the real bytes).

### Retrieval pipeline

Every query runs the same unified pipeline — symbol lookup, BM25 full-text, and vector search in parallel, fused with RRF, then re-ranked by path priors and recency:

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

---

## Architecture

### Crates

| Crate | Role |
|---|---|
| `semanticfs-common` | Shared config types, health reporting, audit events |
| `policy-guard` | Trust boundaries, filtering, redaction, multi-root ownership |
| `indexer` | File watching, chunking, symbol extraction, embeddings, snapshot publish |
| `retrieval-core` | Hybrid retrieval planner, RRF fusion, ranking priors |
| `map-engine` | Directory summary generation, caching, LLM enrichment |
| `fuse-bridge` | Virtual filesystem rendering, inode/content LRU cache |
| `mcp` | MCP-compatible HTTP server (search tools + map resources) |
| `semanticfs-cli` | CLI entry point: index, serve, health, benchmark, recover |

### Request flow

```
File changes → Indexer (watch + chunk + symbol + embed)
             → Two-phase snapshot publish
             → FuseBridge (virtual FS render)
             → Retrieval-core (hybrid fusion)
             → Agent verifies through /raw
```

---

## Quickstart

### 1. Build

```bash
cargo build --release -p semanticfs-cli
```

### 2. Create config

```bash
# Auto-detect git root + project type
cd /path/to/your/repo
cargo run --release -p semanticfs-cli -- --config semanticfs.toml init
```

Or use a profile directly:

```bash
# Linux / macOS
bash scripts/apply_config_profile.sh --profile single-repo --output semanticfs.toml --repo-root "$(pwd)"

# Windows
powershell -ExecutionPolicy Bypass -File scripts/apply_config_profile.ps1 `
  -Profile single-repo -OutputPath semanticfs.toml -RepoRoot (Get-Location).Path
```

### 3. (Optional) Set up real embeddings

```bash
cargo run --release -p semanticfs-cli -- model setup
# Downloads bge-small-en-v1.5 ONNX model to ~/.semanticfs/models/
# SemanticFS auto-detects it on next startup
```

### 4. Build the index

```bash
cargo run --release -p semanticfs-cli -- --config semanticfs.toml index build
```

### 5. Connect Claude Code (native stdio — no Python required)

Create `claude_mcp.json`:

```json
{
  "mcpServers": {
    "semanticfs": {
      "command": "/abs/path/to/semanticfs",
      "args": ["--config", "/abs/path/to/semanticfs.toml", "serve", "mcp-stdio"]
    }
  }
}
```

Then:
```bash
claude --mcp-config claude_mcp.json
```

Claude Code starts the `serve mcp-stdio` subprocess — no separate server process needed.

For a full walkthrough: [`docs/setup_10_minute_agents.md`](docs/setup_10_minute_agents.md)

---

## Recommended profiles

| Profile | Use case |
|---|---|
| `single-repo` | One project, clean root |
| `multi-root-dev-box` | Curated set of development repos + configs |
| `home-projects` | Bounded home-directory coverage (12 domains) |

Sample configs live in `config/profiles/`. The production-validated home profile (`home_profile_v1`) covers 12 domains with 25 scan targets at 1.0 recall / 0.854 MRR.

---

## Quality gates

All retrieval/indexing changes are guarded by frozen golden suites:

| Suite | Queries | Recall | MRR | Symbol-hit |
|---|---|---|---|---|
| v9 (Phase 3 — frozen) | 25 | 1.000 | 1.000 | 1.000 |
| v10 (Phase 4 — frozen) | 27 | 1.000 | 1.000 | 1.000 |
| v11 (Phase 5 — frozen) | 29 | 1.000 | 1.000 | 1.000 |
| v12 (Phase 6 — frozen) | 31 | 1.000 | 1.000 | 1.000 |
| v13 (Phase 7 — frozen) | 34 | 1.000 | 1.000 | 1.000 |
| v14 (active — broadened) | 43 | 1.000 | 1.000 | 1.000 |
| home_profile_v1 | 32 | 1.000 | 0.854 | 1.000 |

Head-to-head vs `rg` (ripgrep) on the Phase 7 suite: SemanticFS recall `1.000`, MRR `1.000` vs `rg` recall `0.946`, MRR `0.860`.

---

## Common commands

```bash
# Health check
cargo run -p semanticfs-cli -- --config config/local.toml health

# Full relevance benchmark
cargo run --release -p semanticfs-cli -- \
  --config config/local.toml benchmark relevance \
  --fixture-repo /abs/repo \
  --golden tests/retrieval_golden/semanticfs_multiroot_explicit_v14.json

# Head-to-head vs rg
cargo run --release -p semanticfs-cli -- \
  --config config/local.toml benchmark head-to-head \
  --fixture-repo /abs/repo \
  --golden tests/retrieval_golden/semanticfs_multiroot_explicit_v14.json

# Claude Code head-to-head (token comparison)
powershell -ExecutionPolicy Bypass -File scripts/run_head_to_head_comparison.ps1

# Release smoke check
powershell -ExecutionPolicy Bypass -File scripts/run_release_readiness.ps1 -SkipBuild
```

---

## Embeddings

| Backend | Quality | Setup |
|---|---|---|
| `hash` (default) | 100% recall on symbol/keyword queries | No setup |
| `onnx` | Full semantic recall on natural language queries | `semanticfs model setup` |

Run `semanticfs model setup` to download `bge-small-en-v1.5` (~33 MB) to `~/.semanticfs/models/`. SemanticFS auto-detects the model on the next startup — no config change needed.

To use a custom model:
```bash
export SEMANTICFS_ONNX_MODEL=/path/to/model.onnx
export SEMANTICFS_ONNX_TOKENIZER=/path/to/tokenizer.json
```

---

## Known constraints

- Default embedding runtime is `hash`. Run `semanticfs model setup` for full semantic search quality. Hash embeddings still give 100% recall on symbol and keyword queries.
- FUSE virtual filesystem mount is Linux-only. Windows and macOS use the MCP server path (fully functional for indexing, retrieval, and agent use — no FUSE needed).
- The recommended default is the bounded single-repo or home profile, not unbounded full-home crawling.
- The native `serve mcp-stdio` subcommand speaks JSON-RPC 2.0 stdio natively. A Python wrapper (`scripts/semanticfs_mcp_stdio.py`) is still available for the HTTP mode.

---

## Repo docs

| Doc | Purpose |
|---|---|
| [`docs/setup_10_minute_agents.md`](docs/setup_10_minute_agents.md) | Quick agent setup guide |
| [`docs/benchmark.md`](docs/benchmark.md) | Full benchmark command reference |
| [`CONTRIBUTING.md`](CONTRIBUTING.md) | How to contribute, run tests, add golden queries |
| [`SECURITY.md`](SECURITY.md) | Trust model, policy-guard boundary, vulnerability reporting |
| [`docs/current_execution_plan.md`](docs/current_execution_plan.md) | Active implementation baseline |
| [`docs/future-steps-log.md`](docs/future-steps-log.md) | Short active queue |
| [`docs/big-picture-roadmap.md`](docs/big-picture-roadmap.md) | Long-term product direction |
