# 10-Minute Setup for Claude, Cursor, and OpenClaw

## Goal
Get SemanticFS running locally, indexed, and exposed over MCP in one short setup pass.

## 1. Install the binary
Fast local install on Windows:

```powershell
powershell -ExecutionPolicy Bypass -File scripts/install_local.ps1 -AddToUserPath
```

If you prefer a release bundle instead of a direct install:

```powershell
powershell -ExecutionPolicy Bypass -File scripts/package_release.ps1
```

That stages a portable zip under `.semanticfs/releases` containing:
- `semanticfs.exe`
- sample config profiles
- this setup guide
- the profile-instantiation script

## 2. Generate a config from a recommended profile
Single repo:

```powershell
powershell -ExecutionPolicy Bypass -File scripts/apply_config_profile.ps1 -Profile single-repo -OutputPath local.toml -RepoRoot C:\path\to\repo
```

Multi-root dev box (same workspace, split by code/docs/config/scripts):

```powershell
powershell -ExecutionPolicy Bypass -File scripts/apply_config_profile.ps1 -Profile multi-root-dev-box -OutputPath local.toml -RepoRoot C:\path\to\workspace
```

Home plus projects:

```powershell
powershell -ExecutionPolicy Bypass -File scripts/apply_config_profile.ps1 -Profile home-projects -OutputPath local.toml -HomeRoot $env:USERPROFILE -ProjectsRoot C:\path\to\projects
```

## 3. Build the first index

```powershell
semanticfs --config local.toml index build
```

If `semanticfs` is not yet on `PATH`, use:

```powershell
target\release\semanticfs.exe --config local.toml index build
```

## 4. Start the MCP server

```powershell
semanticfs --config local.toml serve mcp
```

Keep that process running.

## 5. Register SemanticFS in your agent
Use a command-based MCP entry that launches:

```text
semanticfs --config C:\path\to\local.toml serve mcp
```

Use the same command in:
- Claude Code
- Cursor
- OpenClaw

The exact UI differs, but the working shape is the same: add an MCP server entry that runs the command above and leave it enabled for the workspace you want indexed.

## 6. First-use workflow
Once connected, use the same pattern every time:
1. Ask SemanticFS to search for the relevant file or symbol.
2. Read the exact file through `/raw` before editing.
3. Keep edits grounded to the returned path.

Examples:
- `search for SidebarProvider`
- `search for training utils`
- `open /raw/path/to/file`

## 7. Recommended defaults
- Use `single-repo` for one codebase.
- Use `multi-root-dev-box` when one workspace has clear code/docs/config/script boundaries.
- Use `home-projects` when you want a bounded home root plus one larger projects root.

## 8. Validate quickly
Run a quick health check:

```powershell
semanticfs --config local.toml health
```

Run one relevance check if you have a fixture:

```powershell
semanticfs --config local.toml benchmark relevance --fixture-repo C:\path\to\repo --golden tests\retrieval_golden\semanticfs_multiroot_explicit_v14.json
```
