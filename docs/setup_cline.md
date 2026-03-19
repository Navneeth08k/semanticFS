# Setting Up SemanticFS with Cline (VS Code)

Cline is an AI coding agent for VS Code that supports MCP tools. SemanticFS integrates via the same stdio MCP protocol as Claude Code.

## Prerequisites

- SemanticFS binary installed
- VS Code with the [Cline extension](https://marketplace.visualstudio.com/items?itemName=saoudrizwan.claude-dev) installed
- A repo indexed by SemanticFS (see Steps 1–3 in [setup_claude_code.md](setup_claude_code.md))

## Step 1: Index your repo

```bash
cd /path/to/your/repo
semanticfs --config semanticfs.toml init
semanticfs --config semanticfs.toml index build
```

## Step 2: Configure Cline

Open VS Code settings (`Ctrl+,` / `Cmd+,`) and search for `cline.mcpServers`. Click "Edit in settings.json" and add:

```json
{
  "cline.mcpServers": {
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

Or copy `config/cline_mcp_example.json` from this repo and adjust the config path.

## Step 3: Verify in Cline

1. Open Cline in VS Code (click the Cline icon in the sidebar)
2. Open the MCP tools panel — you should see `semanticfs` listed as a connected server
3. The tools `search_codebase` and `get_directory_map` should appear as available

## Troubleshooting

```bash
semanticfs --config semanticfs.toml doctor
```

Check that the index DB exists and the binary is in PATH.
