<#
.SYNOPSIS
    Head-to-head: Naive Claude Code vs Claude Code + SemanticFS MCP.

.DESCRIPTION
    1. Builds semanticfs-cli in --release (skips if binary is recent)
    2. Starts SemanticFS MCP server pointing at ai-testgen repo
    3. Waits for the index to be ready (polls /health/ready)
    4. Writes the MCP config JSON for Claude Code
    5. Runs the Python comparison harness
    6. Stops the SemanticFS server
    7. Prints where the results artifact was saved

.PARAMETER SkipBuild
    Skip cargo build (use existing binary)

.PARAMETER TasksFilter
    Space-separated list of task IDs to run (e.g. "T1 T3"). Default: all.

.PARAMETER NaiveOnly
    Run only the naive mode (useful for debugging)

.PARAMETER SfsOnly
    Run only the SemanticFS mode (useful when naive results already exist)

.EXAMPLE
    .\scripts\run_head_to_head_comparison.ps1
    .\scripts\run_head_to_head_comparison.ps1 -SkipBuild -TasksFilter "T1 T2"
#>

param(
    [switch]$SkipBuild,
    [string]$TasksFilter = "",
    [switch]$NaiveOnly,
    [switch]$SfsOnly
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$SfsRoot  = $PSScriptRoot | Split-Path -Parent
$AiTestgenRoot = "C:\Users\navneeth\Desktop\NavneethThings\Projects\ai-testgen"
$PythonExe = "C:\Users\navneeth\anaconda3\python.exe"
$BinaryPath = Join-Path $SfsRoot ".semanticfs\local-bin-release\semanticfs.exe"
$ConfigPath = Join-Path $SfsRoot "config\head_to_head_ai_testgen.toml"
$McpWrapperPath = Join-Path $SfsRoot "scripts\semanticfs_mcp_stdio.py"
$McpConfigPath = Join-Path $SfsRoot "config\semanticfs_for_claude_mcp.json"
$HarnessPath = Join-Path $SfsRoot "scripts\head_to_head_claude.py"
$DbPath = Join-Path $env:TEMP "semanticfs_h2h\ai_testgen.db"

# Ports for this run (different from default 9464/9465 to avoid conflicts)
$McpPort     = 9466
$HealthPort  = 9467
$McpBaseUrl  = "http://127.0.0.1:$McpPort"
$HealthBaseUrl = "http://127.0.0.1:$HealthPort"

Write-Host ""
Write-Host "================================================================" -ForegroundColor Cyan
Write-Host "  SemanticFS Head-to-Head Comparison" -ForegroundColor Cyan
Write-Host "  Naive Claude Code  vs  Claude Code + SemanticFS MCP" -ForegroundColor Cyan
Write-Host "================================================================" -ForegroundColor Cyan
Write-Host ""

# ── Step 1: Build (optional) ──────────────────────────────────────────────────
if (-not $SkipBuild) {
    Write-Host "[1/5] Building semanticfs-cli --release ..." -ForegroundColor Yellow
    Push-Location $SfsRoot
    try {
        cargo build --release -p semanticfs-cli 2>&1 | Tee-Object -Variable buildOut | Select-Object -Last 5
        if ($LASTEXITCODE -ne 0) { throw "cargo build failed" }
        # Copy to local-bin-release
        $releaseBin = Join-Path $SfsRoot "target\release\semanticfs.exe"
        $localBin   = Join-Path $SfsRoot ".semanticfs\local-bin-release"
        if (-not (Test-Path $localBin)) { New-Item -ItemType Directory -Path $localBin | Out-Null }
        Copy-Item $releaseBin (Join-Path $localBin "semanticfs.exe") -Force
        Write-Host "  Binary ready: $BinaryPath" -ForegroundColor Green
    } finally {
        Pop-Location
    }
} else {
    Write-Host "[1/5] Skipping build (using existing binary)" -ForegroundColor DarkGray
    if (-not (Test-Path $BinaryPath)) {
        Write-Error "Binary not found at $BinaryPath — run without -SkipBuild first"
    }
}

# ── Step 2: Write MCP config JSON ─────────────────────────────────────────────
Write-Host "[2/5] Writing MCP config for Claude Code ..." -ForegroundColor Yellow

$mcpConfig = @{
    mcpServers = @{
        semanticfs = @{
            command = $PythonExe
            args    = @($McpWrapperPath, "--url", $McpBaseUrl)
            env     = @{}
        }
    }
}
$mcpConfig | ConvertTo-Json -Depth 5 | Set-Content -Path $McpConfigPath -Encoding UTF8
Write-Host "  MCP config: $McpConfigPath" -ForegroundColor Green

# ── Step 3: Start SemanticFS MCP server ───────────────────────────────────────
Write-Host "[3/5] Starting SemanticFS MCP server (port $McpPort) ..." -ForegroundColor Yellow

# Ensure DB temp dir exists
if (-not (Test-Path (Split-Path $DbPath -Parent))) {
    New-Item -ItemType Directory -Path (Split-Path $DbPath -Parent) | Out-Null
}

# First: run indexer to build the DB (non-blocking start, will index in background)
$env:SEMANTICFS_DB_PATH = $DbPath

$sfsProcess = Start-Process `
    -FilePath $BinaryPath `
    -ArgumentList "--config `"$ConfigPath`" serve mcp" `
    -PassThru `
    -NoNewWindow `
    -RedirectStandardError (Join-Path $env:TEMP "semanticfs_h2h_stderr.txt")

Write-Host "  SemanticFS PID: $($sfsProcess.Id)" -ForegroundColor Green

# ── Step 4: Wait for ready ────────────────────────────────────────────────────
Write-Host "[4/5] Waiting for SemanticFS to index and become ready ..." -ForegroundColor Yellow

$maxWaitSecs = 120
$pollIntervalSecs = 3
$waited = 0
$ready = $false

while ($waited -lt $maxWaitSecs) {
    Start-Sleep -Seconds $pollIntervalSecs
    $waited += $pollIntervalSecs

    try {
        $resp = Invoke-WebRequest -Uri "$HealthBaseUrl/health/ready" -TimeoutSec 2 -ErrorAction SilentlyContinue
        if ($resp.StatusCode -eq 200) {
            $body = $resp.Content | ConvertFrom-Json -ErrorAction SilentlyContinue
            if ($body.ready -eq $true -or $resp.StatusCode -eq 200) {
                $ready = $true
                break
            }
        }
    } catch {
        # Not ready yet — keep polling
    }

    Write-Host "  ...waiting ($waited/$maxWaitSecs s)" -ForegroundColor DarkGray
}

if (-not $ready) {
    Write-Host "  WARNING: SemanticFS did not report ready in $maxWaitSecs s — trying anyway" -ForegroundColor Yellow
    # Give it a bit more time for indexing
    Start-Sleep -Seconds 5
} else {
    Write-Host "  SemanticFS is ready after ${waited}s" -ForegroundColor Green
}

# Also verify the MCP HTTP endpoint responds
try {
    $mcpResp = Invoke-WebRequest -Uri "$McpBaseUrl/resources/health" -TimeoutSec 5 -ErrorAction SilentlyContinue
    Write-Host "  MCP HTTP endpoint OK: $McpBaseUrl" -ForegroundColor Green
} catch {
    Write-Host "  WARNING: MCP HTTP endpoint not responding at $McpBaseUrl" -ForegroundColor Yellow
}

# ── Step 5: Run comparison harness ───────────────────────────────────────────
Write-Host "[5/5] Running comparison harness ..." -ForegroundColor Yellow
Write-Host ""

$harnessArgs = @($HarnessPath, "--mcp-config", $McpConfigPath)
if ($TasksFilter -ne "") {
    $taskIds = $TasksFilter -split "\s+"
    $harnessArgs += "--tasks"
    $harnessArgs += $taskIds
}
if ($NaiveOnly) { $harnessArgs += "--naive-only" }
if ($SfsOnly)   { $harnessArgs += "--sfs-only" }

try {
    & $PythonExe @harnessArgs
    $harnessExit = $LASTEXITCODE
} finally {
    # ── Cleanup: stop SemanticFS ──────────────────────────────────────────────
    Write-Host ""
    Write-Host "Stopping SemanticFS (PID $($sfsProcess.Id)) ..." -ForegroundColor DarkGray
    if (-not $sfsProcess.HasExited) {
        $sfsProcess.Kill()
    }
    $env:SEMANTICFS_DB_PATH = ""
}

Write-Host ""
Write-Host "================================================================" -ForegroundColor Cyan
Write-Host "  Results: $SfsRoot\.semanticfs\bench\head_to_head_claude_latest.json" -ForegroundColor Green
Write-Host "================================================================" -ForegroundColor Cyan
Write-Host ""

if ($harnessExit -ne 0) {
    Write-Error "Harness exited with code $harnessExit"
}
