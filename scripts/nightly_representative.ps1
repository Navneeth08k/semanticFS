param(
  [string]$SemanticFsRepo = (Get-Location).Path,
  [string]$AiTestgenRepo = (Join-Path (Resolve-Path "..").Path "ai-testgen"),
  [int]$SoakSeconds = 30
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

Write-Host "== SemanticFS Representative Nightly =="
Write-Host "semanticfs repo: $SemanticFsRepo"
Write-Host "ai-testgen repo: $AiTestgenRepo"
Write-Host "soak seconds:   $SoakSeconds"

if (-not (Test-Path $SemanticFsRepo)) {
  throw "semanticFS repo path not found: $SemanticFsRepo"
}
if (-not (Test-Path $AiTestgenRepo)) {
  throw "ai-testgen repo path not found: $AiTestgenRepo"
}

$runId = (Get-Date -Format "yyyyMMddTHHmmss")
$cliBin = Join-Path (Get-Location).Path "target\release\semanticfs.exe"
if (-not (Test-Path $cliBin)) {
  Run-Step "Build semanticfs-cli (release)" "cargo build --release -p semanticfs-cli"
}

$relevanceLatestPath = ".semanticfs/bench/relevance_latest.json"
$semanticFsRelevanceSnapshot = ".semanticfs/bench/relevance_semanticfs_pre_release_gate_$runId.json"

$env:SEMANTICFS_DB_PATH = "semanticfs.nightly.semanticfs.relevance.$runId.db"
Run-Step "Relevance - semanticFS suite" "& `"$cliBin`" --config config/relevance-real.toml benchmark relevance --fixture-repo `"$SemanticFsRepo`" --golden tests/retrieval_golden/semanticfs_repo.json --history"
if (Test-Path $relevanceLatestPath) {
  Copy-Item -Path $relevanceLatestPath -Destination $semanticFsRelevanceSnapshot -Force
}

$env:SEMANTICFS_DB_PATH = "semanticfs.nightly.semanticfs.h2h.$runId.db"
Run-Step "Head-to-head - semanticFS suite" "& `"$cliBin`" --config config/relevance-real.toml benchmark head-to-head --fixture-repo `"$SemanticFsRepo`" --golden tests/retrieval_golden/semanticfs_repo.json --history"

$env:SEMANTICFS_DB_PATH = "semanticfs.nightly.ai_testgen.relevance.$runId.db"
Run-Step "Relevance - ai-testgen suite" "& `"$cliBin`" --config config/relevance-ai-testgen.toml benchmark relevance --fixture-repo `"$AiTestgenRepo`" --golden tests/retrieval_golden/ai_testgen_repo.json --history"

$env:SEMANTICFS_DB_PATH = "semanticfs.nightly.ai_testgen.h2h.$runId.db"
Run-Step "Head-to-head - ai-testgen suite" "& `"$cliBin`" --config config/relevance-ai-testgen.toml benchmark head-to-head --fixture-repo `"$AiTestgenRepo`" --golden tests/retrieval_golden/ai_testgen_repo.json --history"

if (Test-Path $semanticFsRelevanceSnapshot) {
  Copy-Item -Path $semanticFsRelevanceSnapshot -Destination $relevanceLatestPath -Force
}

$env:SEMANTICFS_DB_PATH = "semanticfs.nightly.semanticfs.release_gate.$runId.db"
Run-Step "Release gate (strict relevance thresholds)" "& `"$cliBin`" --config config/relevance-real.toml benchmark release-gate --refresh --fixture-repo `"$SemanticFsRepo`" --soak-seconds $SoakSeconds --enforce-relevance --min-relevance-queries 20 --min-recall-at-5 0.90 --min-symbol-hit-rate 0.99 --min-mrr 0.80"

Remove-Item Env:SEMANTICFS_DB_PATH -ErrorAction SilentlyContinue

Write-Host ""
Write-Host "Representative nightly complete."
Write-Host "Artifacts:"
Write-Host "  .semanticfs/bench/relevance_latest.json"
Write-Host "  .semanticfs/bench/head_to_head_latest.json"
Write-Host "  .semanticfs/bench/release_gate.json"
Write-Host "  .semanticfs/bench/history/*"
