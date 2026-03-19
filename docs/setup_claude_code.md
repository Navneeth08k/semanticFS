# Setting Up SemanticFS with Claude Code

SemanticFS integrates with Claude Code via the native stdio MCP protocol — no Python wrapper or separate server process required.

## Prerequisites

- SemanticFS binary installed (`semanticfs --help` works)
- Claude Code installed (`claude --help` works)
- A repo to index

If you haven't installed SemanticFS yet:
```bash
# Linux / macOS
curl -sSfL https://raw.githubusercontent.com/Navneeth08k/semanticFS/main/scripts/install.sh | bash

# Windows
irm https://raw.githubusercontent.com/Navneeth08k/semanticFS/main/scripts/install.ps1 | iex
```

## Step 1: Create a config

```bash
cd /path/to/your/repo
semanticfs --config semanticfs.toml init
```

This auto-detects your project type (Rust, Node, Python, Go, Java) and generates a `semanticfs.toml` with appropriate deny_globs (excludes `target/`, `node_modules/`, `.venv/`, etc.).

## Step 2: (Optional) Enable real semantic embeddings

```bash
semanticfs model setup
```

Downloads `bge-small-en-v1.5` (~33 MB) to `~/.semanticfs/models/`. Without this, SemanticFS uses a hash backend that gives perfect recall on symbol and keyword queries but no semantic understanding of natural-language queries.

## Step 3: Build the index

```bash
semanticfs --config semanticfs.toml index build
```

## Step 4: Create the MCP config for Claude Code

Copy `config/claude_mcp_example.json` from this repo and edit the config path:

```json
{
  "mcpServers": {
    "semanticfs": {
      "command": "semanticfs",
      "args": [
        "--config", "/absolute/path/to/your/semanticfs.toml",
        "serve", "mcp-stdio"
      ],
      "env": {}
    }
  }
}
```

Save this as `claude_mcp.json` (anywhere on your machine).

**Important:** Use an absolute path to your `semanticfs.toml`. Relative paths are resolved relative to where Claude Code is launched.

## Step 5: Start Claude Code with SemanticFS

```bash
claude --mcp-config /path/to/claude_mcp.json
```

Claude Code will launch the `semanticfs serve mcp-stdio` subprocess automatically. No separate server process to manage.

## Verifying it works

In Claude Code, ask:

> "Use SemanticFS to find where authentication is implemented"

Or:

> "Search the codebase for the main CLI entry point"

You should see Claude Code call `mcp__semanticfs__search_codebase` and get back file paths with line numbers.

## Troubleshooting

**SemanticFS tools don't appear in Claude Code:**
- Check that `claude_mcp.json` is valid JSON: `python -c "import json; json.load(open('claude_mcp.json'))"`
- Check the config path is absolute and the file exists

**Search returns no results:**
- Run `semanticfs --config semanticfs.toml doctor` to diagnose
- Ensure the index was built: `semanticfs --config semanticfs.toml index build`

**Stale results after code changes:**
```bash
semanticfs --config semanticfs.toml index update   # fast, re-indexes changed files only
# or
semanticfs --config semanticfs.toml index build    # full rebuild
```

## Keeping the index fresh

For active development, run `index watch` to auto-update the index as files change:

```bash
semanticfs --config semanticfs.toml index watch
```

Or just run `index update` before starting Claude Code sessions.
