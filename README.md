# SemanticFS

> **Stop paying for your AI agent to wander around your codebase.**

SemanticFS is a persistent, local semantic index for your filesystem. Instead of burning tokens on `ls`, `find`, `grep`, and `cat` chains, your agent asks *"where is X?"* and gets back the exact file and line range — instantly.

**29% cheaper. 63% fewer context tokens. Same accuracy.**

Works with **Claude Code**, **Cline**, **Cursor**, **Continue.dev**, **OpenClaw**, and any HTTP-capable agent.

---

## 💸 The money problem

Every time your AI agent doesn't know where something is, it does this:

```
ls src/
find . -name "*.py" | head -40
grep -r "authentication" . | head -20   ← 800 tokens of noise
cat handlers/auth.py                    ← another 300 tokens
cat middleware/jwt.py                   ← another 200 tokens
# ... tries 4 more files before finding it
```

**Every one of those lines costs you money.** On a complex codebase exploration task, a naive Claude Code session burns **21,536 context tokens** just on file navigation. That same task with SemanticFS: **7,799 tokens**.

```
search("JWT authentication middleware")
→ middleware/jwt.py:15-82  (JWTMiddleware.validate)   ← 5 tokens
```

### Measured savings (real Claude API calls, not estimates)

| | Without SemanticFS | With SemanticFS | Saved |
|---|---|---|---|
| **API cost** (6 complex tasks) | **$0.2064** | **$0.1466** | **29% 💰** |
| Context tokens | 21,536 | 7,799 | **64%** |
| Tasks solved correctly | 6/6 | 6/6 | same accuracy |

**Projected annual savings:**
- Solo dev on Claude Pro ($20/mo): **~$70/year**
- 10-person team: **~$700/year**
- 100-person org: **~$7,000+/year**

Savings are largest on **complex, multi-file exploration tasks** (tracing APIs, locating integrated subsystems, refactoring across services). Simple single-file lookups break even.

---

## ⚡ Zero to running in 5 minutes

### Step 1 — Install (30 seconds)

**Linux / macOS:**
```bash
curl -fsSL https://raw.githubusercontent.com/Navneeth08k/semanticFS/main/scripts/install.sh | bash
```

**Windows (PowerShell):**
```powershell
irm https://raw.githubusercontent.com/Navneeth08k/semanticFS/main/scripts/install.ps1 | iex
```

**From source:**
```bash
cargo build --release -p semanticfs-cli
```

Verify install:
```bash
semanticfs --version
# semanticfs 0.1.0
```

---

### Step 2 — Index your repo (1–2 minutes, one-time)

```bash
cd /path/to/your/repo

# Auto-detects project type (Python, Node, Rust, Go, Java) and sets deny_globs
# Skips node_modules, .venv, target/, vendor/ automatically
semanticfs --config semanticfs.toml init

# Build the index — takes 5–30 seconds depending on repo size
# You'll see: "Indexed 247 files, 3,412 chunks, 0 errors"
semanticfs --config semanticfs.toml index build
```

> **What does indexing do?** It walks your source files, chunks them, extracts symbols (functions, classes, types), and builds a hybrid BM25 + vector index in a local SQLite file. The index persists — you only rebuild when files change (`index update` for incremental).

> **How long does it take?** Measured on real repos (deps excluded): Tiny (< 10 source files): ~5 sec. Medium (100+ source files, ~5k chunks): ~40 sec. Large (300–500 source files, ~12k chunks): ~90 sec (1.5 min). This is a **one-time cost** — `index update` for incremental changes takes a few seconds.

---

### Step 3 — Connect your agent

#### Claude Code / Cline / Cursor / Continue.dev

Create a file called `claude_mcp.json` in your repo root:

```json
{
  "mcpServers": {
    "semanticfs": {
      "command": "semanticfs",
      "args": ["--config", "/ABSOLUTE/PATH/TO/semanticfs.toml", "serve", "mcp-stdio"]
    }
  }
}
```

> ⚠️ Use an **absolute path** to `semanticfs.toml`. Replace `/ABSOLUTE/PATH/TO/` with the actual path on your machine (e.g. `/home/you/myrepo/semanticfs.toml` or `C:/Users/you/myrepo/semanticfs.toml`).

Then launch Claude Code with it:
```bash
claude --mcp-config claude_mcp.json
```

**That's it.** SemanticFS starts as a subprocess — no separate server, no background process to manage. The same `claude_mcp.json` works in Cline (paste into MCP settings), Cursor (MCP config), and Continue.dev.

#### OpenClaw

```bash
clawhub install semanticfs
```

One command. OpenClaw picks it up automatically for all file-related tasks.

---

### Step 4 — Verify it's working

```bash
# Health check — confirms index exists, embedding backend, MCP available
semanticfs --config semanticfs.toml doctor
# [OK] Config valid
# [OK] Index DB: 247 files, 3,412 chunks
# [OK] Embedding backend: hash (fast, keyword/symbol)
# [OK] MCP stdio: available
```

Inside your Claude Code session, you can also ask Claude: *"use the search_codebase tool to find the authentication middleware"* — it should return a result in one call.

---

### Step 5 — (Optional) Upgrade to full semantic search

By default, SemanticFS uses a fast hash-based embedding that gives **100% recall on symbol and keyword queries** with zero setup.

For full natural-language semantic search (e.g. *"find where we handle rate limiting errors"*):

```bash
# Downloads bge-small-en-v1.5 ONNX model (~33 MB, one-time)
semanticfs model setup
# Auto-detected on next startup — no config change needed
```

---

## 📊 Full benchmark results

### Benchmark 1 — ai-testgen repo (complex multi-file exploration)

6 tasks on a 4,638-file repo (24 real source files + `.venv`). Tasks include: tracing CLI entry points, locating test harness integration, finding API pattern implementations.

![Summary](docs/charts/chart_summary.svg)

![Context tokens per task](docs/charts/chart_ctx_tokens.svg)

![Token savings per task](docs/charts/chart_savings.svg)

| Metric | Naive (Bash only) | + SemanticFS | Δ |
|---|---|---|---|
| **API cost** | **$0.2064** | **$0.1466** | **−29% 💰** |
| Context tokens | 21,536 | 7,799 | −64% |
| Avg agent turns | 3.8 | 3.5 | −8% |
| Accuracy | 6/6 ✅ | 6/6 ✅ | same |

**The extreme case:** Finding the CLI entry point naively cost **4,265 context tokens** (12+ tool calls: directory listings, multiple wrong files, retries). With SemanticFS: **5 tokens** — one search, immediate answer.

---

### Benchmark 2 — 4 repos × 4 tasks × 2 modes (32 Claude API calls)

![Scorecard](docs/charts/multi_repo_scorecard.svg)

![Cost comparison by repo](docs/charts/multi_repo_cost.svg)

![Savings vs codebase size](docs/charts/multi_repo_savings.svg)

| Repo | Real source files | Cost Naive | Cost + SFS | Δ |
|---|---|---|---|---|
| prizePicksAI (tiny) | 5 | 8.1¢ | 8.4¢ | −3% (break even) |
| KalshiTradingAlgo (small) | 17 | 13.6¢ | 13.5¢ | +1% (neutral) |
| syntaxless (medium) | 95+ TS | 8.8¢ | 9.7¢ | −10% (small overhead) |
| **buckit (large)** | **70+ JS** | **13.1¢** | **11.7¢** | **+11% 💰** |

**Accuracy:** 16/16 correct in both modes across all repos.

#### When does SemanticFS help most?

| Scenario | Savings |
|---|---|
| Complex multi-file exploration (tracing APIs, refactoring) | **~29%** |
| Large repos (70+ real source files) | **~11%** |
| Persistent agent session (one MCP process, many tasks) | **highest** |
| Simple single-file lookup on tiny repo | ~0% (break even) |
| Pattern search (`grep "literal_string"`) | 0% (use grep) |

---

## How it works

### What happens when your agent calls `search_codebase`

```
Agent: search("JWT authentication middleware")
         │
         ▼
   Symbol lookup ─────────┐
   BM25 full-text ─────────┤──→ RRF fusion → path priors → top 5 results
   Vector search ─────────┘
         │
         ▼
   middleware/jwt.py:15-82  (JWTMiddleware.validate)
   handlers/auth.py:40-65   (require_auth decorator)
```

Every query runs symbol lookup, BM25, and vector search in parallel, fused with Reciprocal Rank Fusion, then re-ranked by path priors and recency. The agent verifies any result through `/raw` for byte-accurate file reads.

**Core invariant:** discovery is probabilistic (semantic search), verification is deterministic (`/raw` always returns the real bytes).

### Architecture (8 Rust crates)

| Crate | Role |
|---|---|
| `semanticfs-common` | Shared config types, health reporting, audit events |
| `policy-guard` | Trust boundaries, filtering, redaction, multi-root ownership |
| `indexer` | File watching, chunking, symbol extraction, embeddings |
| `retrieval-core` | Hybrid retrieval planner, RRF fusion, ranking priors |
| `map-engine` | Directory summary generation, caching |
| `fuse-bridge` | Virtual filesystem rendering (Linux) |
| `mcp` | MCP JSON-RPC 2.0 stdio server |
| `semanticfs-cli` | CLI: `init`, `index`, `serve`, `doctor`, `benchmark` |

---

## Supported agents

| Agent | Integration | Setup time |
|---|---|---|
| **Claude Code** | MCP stdio — `serve mcp-stdio` | 2 min (one JSON file) |
| **OpenClaw** | ClawHub skill — `clawhub install semanticfs` | 30 sec |
| **Cline** (VS Code) | MCP stdio — same config as Claude Code | 2 min |
| **Cursor** | MCP stdio | 2 min |
| **Continue.dev** | MCP stdio | 2 min |
| **Custom agents** | HTTP API on `localhost:9464` | Direct `curl` |

---

## Keeping the index fresh

```bash
# Rebuild from scratch (after major refactor)
semanticfs --config semanticfs.toml index build

# Incremental update — only re-indexes files changed since last build
# Typically takes < 2 seconds
semanticfs --config semanticfs.toml index update

# Watch mode — auto-updates as files change (runs in background)
semanticfs --config semanticfs.toml index watch
```

In a real workflow, run `index update` once before starting a coding session. The index persists in a local SQLite file — no rebuild needed between sessions.

---

## Quality gates

Every retrieval change is guarded by frozen golden query suites:

| Suite | Queries | Recall | MRR |
|---|---|---|---|
| v14 (active) | 43 | **1.000** | **1.000** |
| home_profile_v1 | 32 | **1.000** | 0.854 |

Head-to-head vs `ripgrep` on the v14 suite: SemanticFS recall **1.000** / MRR **1.000** vs `rg` recall **0.946** / MRR **0.860**.

---

## Embeddings

| Backend | Quality | Setup |
|---|---|---|
| `hash` (default) | 100% recall on symbol/keyword queries | Zero — works out of the box |
| `onnx` | Full semantic recall on natural-language queries | `semanticfs model setup` (~33 MB download) |

---

## vs alternatives

| Tool | Local? | Any agent? | Persistent? | Multi-root? |
|---|---|---|---|---|
| `ripgrep` / `grep` | ✅ | ✅ | ✅ | ✅ |
| GitHub Copilot workspace | ❌ cloud | ❌ Copilot only | ✅ | ❌ |
| Sourcegraph Cody | ❌ SaaS | ❌ Cody only | ✅ | partial |
| Continue.dev `@codebase` | ✅ | ❌ Continue only | ❌ per-session | ❌ |
| Cursor codebase index | ❌ cloud | ❌ Cursor only | ✅ | ❌ |
| **SemanticFS** | **✅** | **✅** | **✅** | **✅** |

SemanticFS is the only local-first, agent-agnostic, persistent, multi-root option.

> `ripgrep` is fast for pattern search. SemanticFS wins on **semantic** queries ("where is the authentication logic?") and on **reducing total agent exploration cost** — the agent doesn't need to call grep 8 times before finding the right file.

---

## Known constraints

- Default embeddings: `hash` backend (100% recall on symbol/keyword). Run `semanticfs model setup` for full semantic quality.
- FUSE virtual filesystem: Linux only. Windows and macOS use MCP server path (fully functional).
- Best results on codebases with 50+ real source files. Small repos (< 50 files) see minimal savings.

---

## Docs

| Doc | |
|---|---|
| [`docs/setup_10_minute_agents.md`](docs/setup_10_minute_agents.md) | Full agent setup walkthrough |
| [`docs/setup_claude_code.md`](docs/setup_claude_code.md) | Claude Code specific guide |
| [`docs/setup_cline.md`](docs/setup_cline.md) | Cline specific guide |
| [`docs/setup_cursor.md`](docs/setup_cursor.md) | Cursor specific guide |
| [`docs/setup_openclaw.md`](docs/setup_openclaw.md) | OpenClaw specific guide |
| [`docs/benchmark.md`](docs/benchmark.md) | Benchmark methodology + commands |
| [`CONTRIBUTING.md`](CONTRIBUTING.md) | How to contribute |
| [`SECURITY.md`](SECURITY.md) | Trust model and vulnerability reporting |
