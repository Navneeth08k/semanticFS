# Setting Up SemanticFS with Cursor

Cursor supports MCP tools starting from version 0.40. SemanticFS integrates via the stdio MCP protocol.

## Prerequisites

- SemanticFS binary installed
- Cursor 0.40 or newer
- A repo indexed by SemanticFS (see Steps 1–3 in [setup_claude_code.md](setup_claude_code.md))

## Step 1: Index your repo

```bash
cd /path/to/your/repo
semanticfs --config semanticfs.toml init
semanticfs --config semanticfs.toml index build
```

## Step 2: Configure Cursor

Create or edit `.cursor/mcp.json` in your project root (project-specific) or `~/.cursor/mcp.json` (global):

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

You can also copy `config/cursor_mcp_example.json` from this repo.

## Step 3: Restart Cursor

After saving the config file, restart Cursor. MCP tools are loaded at startup.

## Step 4: Verify

In Cursor's AI chat, ask:

> "Use SemanticFS to find where X is implemented"

Cursor should call `search_codebase` and return file paths with line numbers.

## Troubleshooting

```bash
semanticfs --config semanticfs.toml doctor
```

Check the index exists and the binary is accessible.
