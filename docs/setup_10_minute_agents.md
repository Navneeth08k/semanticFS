# SemanticFS: 10-Minute Agent Setup

Connect Claude Code (or any MCP-capable agent) to SemanticFS in ~10 minutes.

---

## Prerequisites

- SemanticFS binary — build from source or download a pre-built release from GitHub Releases
- The repo you want to index

---

## Step 1: Install

### Linux / macOS (pre-built binary)

```bash
curl -sSfL https://raw.githubusercontent.com/YOUR_ORG/semanticfs/main/scripts/install.sh | bash
```

### Build from source

```bash
cargo build --release -p semanticfs-cli
# Binary: target/release/semanticfs (or target/release/semanticfs.exe on Windows)
```

### Windows (local install)

```powershell
powershell -ExecutionPolicy Bypass -File scripts/install_local.ps1 -AddToUserPath
```

---

## Step 2: Generate a config

### Fastest: auto-detect (recommended)

```bash
cd /path/to/your/repo
semanticfs --config semanticfs.toml init
# Detects git root + project type, writes ready-to-use config
```

### Manual profile selection

**Linux / macOS:**
```bash
bash scripts/apply_config_profile.sh \
  --profile single-repo \
  --output semanticfs.toml \
  --repo-root "$(pwd)"
```

**Windows:**
```powershell
powershell -ExecutionPolicy Bypass -File scripts/apply_config_profile.ps1 `
  -Profile single-repo `
  -OutputPath semanticfs.toml `
  -RepoRoot (Get-Location).Path
```

Available profiles: `single-repo`, `multi-root-dev-box`, `home-projects`

---

## Step 3: (Optional) Set up real embeddings

The default `hash` backend gives **100% recall** on symbol and keyword queries. For full semantic search quality on natural language queries, run:

```bash
semanticfs model setup
```

This downloads `bge-small-en-v1.5` (~33 MB ONNX) to `~/.semanticfs/models/`. SemanticFS auto-detects it on the next startup — no config change needed.

---

## Step 4: Build the index

```bash
semanticfs --config semanticfs.toml index build
```

---

## Step 5: Connect to Claude Code

### Primary path: native stdio MCP (recommended — no Python required)

Add to your Claude Code MCP config (`claude_mcp.json` or `.mcp.json`):

```json
{
  "mcpServers": {
    "semanticfs": {
      "command": "/absolute/path/to/semanticfs",
      "args": ["--config", "/absolute/path/to/semanticfs.toml", "serve", "mcp-stdio"]
    }
  }
}
```

Start Claude Code:
```bash
claude --mcp-config claude_mcp.json
```

Claude Code manages the `semanticfs serve mcp-stdio` subprocess automatically. No separate server process needed.

### Alternative: HTTP MCP server + Python wrapper

If you need the HTTP server (for debugging, or sharing between multiple agents):

```bash
# Start HTTP MCP server (binds to 127.0.0.1:9464 by default)
semanticfs --config semanticfs.toml serve mcp
```

```json
{
  "mcpServers": {
    "semanticfs": {
      "command": "python",
      "args": ["/path/to/scripts/semanticfs_mcp_stdio.py", "--url", "http://127.0.0.1:9464"]
    }
  }
}
```

---

## Step 6: (Optional) Keep the index fresh

Run the file watcher in the background:

```bash
semanticfs --config semanticfs.toml index watch
```

---

## Validation

```bash
# Health check
semanticfs --config semanticfs.toml health

# Retrieval quality benchmark (needs a fixture repo)
semanticfs --config semanticfs.toml benchmark relevance \
  --fixture-repo /path/to/repo \
  --golden tests/retrieval_golden/semanticfs_multiroot_explicit_v14.json
```

---

## Troubleshooting

**MCP subprocess not starting**

Ensure paths in the MCP JSON are **absolute** (not relative). On Windows use forward slashes:
```json
"command": "C:/Users/you/.local/bin/semanticfs"
```

**HTTP server not responding**

Check the bind address in `[observability]`:
- MCP HTTP: `metrics_bind` — default `127.0.0.1:9464`
- Health/observability: `health_bind` — default `127.0.0.1:9465`

```bash
curl http://127.0.0.1:9464/resources/health
```

**Index is empty / no results**

Run `semanticfs index build` before starting the server. Use `index watch` to auto-update on file changes.

**"using hash embeddings" warning**

Run `semanticfs model setup` to download the ONNX model. Hash embeddings still give full recall on symbol and keyword queries — this warning is informational only.

---

## Platform notes

| Platform | FUSE mount | MCP (stdio) | MCP (HTTP) |
|---|---|---|---|
| Linux | Yes (`serve fuse`) | Yes | Yes |
| macOS | No (macFUSE separately) | Yes | Yes |
| Windows | No | Yes | Yes |

All retrieval and MCP features work on all platforms. FUSE virtual filesystem is Linux-only.

---

## Agent workflow

Once connected, the typical agent pattern is:

```
search_codebase("Python signature extraction")
  → ai_testgen_core/diff_parser.py:40-95  (extract_signatures_python)

Read tool: ai_testgen_core/diff_parser.py (lines 40-95)
  → exact bytes, grounded context

Agent makes edit with full confidence
```

SemanticFS replaces the `ls` / `find` / `grep` / `cat` exploration loop with a single targeted call.
