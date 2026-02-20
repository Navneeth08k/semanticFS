param(
  [string]$SemanticFsRepo = (Get-Location).Path,
  [string]$AiTestgenRepo = (Join-Path (Resolve-Path "..").Path "ai-testgen"),
  [int]$SoakSeconds = 2,
  [switch]$IncludeReleaseGate
)

$ErrorActionPreference = "Stop"

function Run-Step([string]$Label, [string]$Cmd) {
  Write-Host ""
  Write-Host "== $Label =="
  Write-Host $Cmd
  Invoke-Expression $Cmd
  if ($LASTEXITCODE -ne 0) {
    throw "step failed ($Label) with exit code $LASTEXITCODE"
  }
}

Write-Host "== SemanticFS Daytime Smoke =="
Write-Host "semanticfs repo: $SemanticFsRepo"
Write-Host "ai-testgen repo: $AiTestgenRepo"
Write-Host "soak seconds:   $SoakSeconds"

if (-not (Test-Path $AiTestgenRepo)) {
  throw "ai-testgen repo path not found: $AiTestgenRepo"
}

$runId = (Get-Date -Format "yyyyMMddTHHmmss")

$cliBin = Join-Path (Get-Location).Path "target\release\semanticfs.exe"
if (-not (Test-Path $cliBin)) {
  Run-Step "Build semanticfs-cli (release)" "cargo build --release -p semanticfs-cli"
}

$env:SEMANTICFS_DB_PATH = "semanticfs.daytime.semanticfs.$runId.db"
Run-Step "Relevance - semanticFS suite" "& `"$cliBin`" --config config/relevance-real.toml benchmark relevance --fixture-repo `"$SemanticFsRepo`" --golden tests/retrieval_golden/semanticfs_repo.json --history"

$env:SEMANTICFS_DB_PATH = "semanticfs.daytime.ai_testgen.$runId.db"
Run-Step "Relevance - ai-testgen suite" "& `"$cliBin`" --config config/relevance-ai-testgen.toml benchmark relevance --fixture-repo `"$AiTestgenRepo`" --golden tests/retrieval_golden/ai_testgen_repo.json --history"

if ($IncludeReleaseGate) {
  $env:SEMANTICFS_DB_PATH = "semanticfs.daytime.release_gate.$runId.db"
  Run-Step "Release gate (semanticFS with relevance thresholds)" "& `"$cliBin`" --config config/relevance-real.toml benchmark release-gate --refresh --fixture-repo `"$SemanticFsRepo`" --soak-seconds $SoakSeconds --enforce-relevance --min-relevance-queries 20 --min-recall-at-5 0.90 --min-symbol-hit-rate 0.99 --min-mrr 0.80"
}
Remove-Item Env:SEMANTICFS_DB_PATH -ErrorAction SilentlyContinue

Write-Host ""
Write-Host "Daytime smoke complete."
Write-Host "Artifacts:"
Write-Host "  .semanticfs/bench/relevance_latest.json"
Write-Host "  .semanticfs/bench/release_gate.json"
Write-Host "  .semanticfs/bench/history/*"
