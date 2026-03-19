# Setting Up SemanticFS with OpenClaw

SemanticFS integrates with OpenClaw via a ClawHub skill. Once installed, OpenClaw will use SemanticFS automatically whenever it needs to search files, find functions, or navigate your filesystem — instead of running slow grep/find/ls/cat chains.

## Why this matters for OpenClaw

OpenClaw, like any LLM-based agent, burns context tokens every time it runs `find`, `grep`, `ls`, or `cat` to explore files it doesn't know yet. SemanticFS replaces those chains with a single semantic search call — returning exact file paths and line numbers in one shot.

This is true for **coding tasks** and for **any general file-based task** (searching docs, configs, notes — anything in an indexed directory).

## Prerequisites

- OpenClaw installed and configured
- SemanticFS binary installed:
  ```bash
  # Linux / macOS
  curl -sSfL https://raw.githubusercontent.com/Navneeth08k/semanticFS/main/scripts/install.sh | bash

  # Windows
  irm https://raw.githubusercontent.com/Navneeth08k/semanticFS/main/scripts/install.ps1 | iex
  ```

## Step 1: Install the SemanticFS skill

### Via ClawHub (recommended)

```bash
clawhub install semanticfs
```

### Manually (from this repo)

```bash
# Linux / macOS
cp -r skills/semanticfs ~/.openclaw/skills/

# Windows
xcopy skills\semanticfs %USERPROFILE%\.openclaw\skills\semanticfs\ /E /I
```

## Step 2: Index your workspace

```bash
# Create a config for your workspace
semanticfs --config ~/semanticfs.toml init --root /path/to/your/workspace

# Build the index
semanticfs --config ~/semanticfs.toml index build
```

To index your entire home directory (or a curated set of directories), use the `home-projects` or `multi-root-dev-box` profile:

```bash
semanticfs --config ~/semanticfs.toml init --profile multi-root
```

## Step 3: Start the SemanticFS HTTP server

OpenClaw's skill calls the SemanticFS HTTP API, so the server must be running:

```bash
semanticfs --config ~/semanticfs.toml serve mcp &
```

To start it automatically on login, add this to your shell profile or use a system service.

**Verify it's running:**
```bash
curl -s http://localhost:9464/health/live && echo "SemanticFS is running"
```

## Step 4: Use with OpenClaw

Once the skill is installed and the server is running, OpenClaw will automatically use SemanticFS when:
- You ask it to find files or code
- It needs to navigate an unfamiliar directory
- It's doing coding tasks that require locating implementations

Example prompts to OpenClaw that will trigger the skill:

> "Find where the authentication logic is in my project"

> "What file handles database connections in ~/projects/myapp?"

> "Search my codebase for the CLI entry point"

## Keeping the index fresh

After making code changes, update the index:

```bash
semanticfs --config ~/semanticfs.toml index update   # fast incremental
# or
semanticfs --config ~/semanticfs.toml index build    # full rebuild
```

Or run in watch mode to auto-update:

```bash
semanticfs --config ~/semanticfs.toml index watch &
```

## Troubleshooting

**Run the doctor command:**
```bash
semanticfs --config ~/semanticfs.toml doctor
```

**Skill not found:** Check the skill is in `~/.openclaw/skills/semanticfs/SKILL.md`

**No results:** Ensure the index is built and the server is running on port 9464

**Stale results:** Re-run `semanticfs index update` or `semanticfs index build`
