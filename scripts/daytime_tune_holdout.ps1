param(
  [Parameter(Mandatory = $true)][string]$Label,
  [Parameter(Mandatory = $true)][string]$RepoRoot,
  [Parameter(Mandatory = $true)][string]$BaseConfig,
  [Parameter(Mandatory = $true)][string]$TuneGolden,
  [Parameter(Mandatory = $true)][string]$HoldoutGolden,
  [switch]$History
)

$ErrorActionPreference = "Stop"

function Assert-PathExists([string]$PathValue, [string]$Kind) {
  if (-not (Test-Path $PathValue)) {
    throw "$Kind not found: $PathValue"
  }
}

function Write-VariantConfig(
  [string]$BaseConfigPath,
  [string]$OutPath,
  [hashtable]$Overrides
) {
  $raw = Get-Content $BaseConfigPath -Raw
  foreach ($key in $Overrides.Keys) {
    $value = $Overrides[$key]
    $escaped = [Regex]::Escape($key)
    $pattern = "(?m)^\s*$escaped\s*=\s*.*$"
    if ($raw -notmatch $pattern) {
      throw "key not found in config for override: $key ($BaseConfigPath)"
    }
    $raw = [Regex]::Replace($raw, $pattern, "$key = $value")
  }
  Set-Content -Path $OutPath -Value $raw -NoNewline
}

function Run-HeadToHead(
  [string]$CliBin,
  [string]$ConfigPath,
  [string]$Repo,
  [string]$GoldenPath,
  [string]$DbName,
  [bool]$UseHistory
) {
  $env:SEMANTICFS_DB_PATH = $DbName
  $args = @(
    "--config", $ConfigPath,
    "benchmark", "head-to-head",
    "--fixture-repo", $Repo,
    "--golden", $GoldenPath
  )
  if ($UseHistory) {
    $args += "--history"
  }
  & $CliBin @args
  if ($LASTEXITCODE -ne 0) {
    throw "head-to-head failed for config=$ConfigPath golden=$GoldenPath exit=$LASTEXITCODE"
  }

  $reportPath = ".semanticfs/bench/head_to_head_latest.json"
  if (-not (Test-Path $reportPath)) {
    throw "missing head-to-head artifact: $reportPath"
  }
  return (Get-Content $reportPath -Raw | ConvertFrom-Json)
}

function As-MetricObject([object]$Report) {
  return [pscustomobject]@{
    query_count = [int]$Report.query_count
    semantic = [pscustomobject]@{
      recall_at_topn = [double]$Report.engines.semanticfs.recall_at_topn
      mrr = [double]$Report.engines.semanticfs.mrr
      symbol_hit_rate = [double]$Report.engines.semanticfs.symbol_hit_rate
      p95_latency_ms = [double]$Report.engines.semanticfs.latency_ms.p95
    }
    baseline = [pscustomobject]@{
      recall_at_topn = [double]$Report.engines.baseline_rg.recall_at_topn
      mrr = [double]$Report.engines.baseline_rg.mrr
      symbol_hit_rate = [double]$Report.engines.baseline_rg.symbol_hit_rate
      p95_latency_ms = [double]$Report.engines.baseline_rg.latency_ms.p95
    }
    delta_semantic_minus_baseline = [pscustomobject]@{
      recall_at_topn = [double]$Report.delta_semanticfs_minus_baseline.recall_at_topn
      mrr = [double]$Report.delta_semanticfs_minus_baseline.mrr
      symbol_hit_rate = [double]$Report.delta_semanticfs_minus_baseline.symbol_hit_rate
      p95_latency_ms = [double]$Report.delta_semanticfs_minus_baseline.p95_latency_ms
    }
  }
}

function Candidate-Score([object]$Metrics) {
  $s = $Metrics.semantic
  # Favor ranking quality first, then latency as a light tie-breaker.
  return ($s.mrr * 1000.0) + ($s.recall_at_topn * 100.0) + ($s.symbol_hit_rate * 100.0) - ($s.p95_latency_ms * 0.01)
}

Assert-PathExists $RepoRoot "repo_root"
Assert-PathExists $BaseConfig "base config"
Assert-PathExists $TuneGolden "tune golden"
Assert-PathExists $HoldoutGolden "holdout golden"

$cliBin = Join-Path (Get-Location).Path "target\release\semanticfs.exe"
if (-not (Test-Path $cliBin)) {
  Write-Host "Building semanticfs-cli release binary..."
  cargo build --release -p semanticfs-cli
  if ($LASTEXITCODE -ne 0) {
    throw "cargo build failed with exit code $LASTEXITCODE"
  }
}

$safeLabel = ($Label -replace "[^a-zA-Z0-9_-]", "_").ToLowerInvariant()
$runId = (Get-Date -Format "yyyyMMddTHHmmss")
$tmpDir = ".semanticfs/bench/tune_holdout_tmp"
New-Item -ItemType Directory -Force -Path $tmpDir | Out-Null

$candidates = @(
  @{
    id = "base"
    overrides = @{}
  },
  @{
    id = "symbol_focus"
    overrides = @{
      "symbol_exact_boost" = "3.0"
      "symbol_prefix_boost" = "1.5"
      "code_path_boost" = "1.30"
      "docs_path_penalty" = "0.80"
      "test_path_penalty" = "0.90"
    }
  },
  @{
    id = "code_focus"
    overrides = @{
      "symbol_exact_boost" = "2.8"
      "symbol_prefix_boost" = "1.4"
      "code_path_boost" = "1.40"
      "docs_path_penalty" = "0.70"
      "test_path_penalty" = "0.85"
      "topn_bm25" = "30"
      "topn_vector" = "30"
    }
  },
  @{
    id = "balanced_wide"
    overrides = @{
      "symbol_exact_boost" = "2.6"
      "symbol_prefix_boost" = "1.35"
      "code_path_boost" = "1.25"
      "docs_path_penalty" = "0.75"
      "test_path_penalty" = "0.90"
      "topn_symbol" = "15"
      "topn_vector" = "40"
    }
  }
)

$tuneResults = @()
foreach ($candidate in $candidates) {
  $candidateId = [string]$candidate.id
  $variantConfig = Join-Path $tmpDir "cfg_${safeLabel}_${candidateId}_$runId.toml"
  Write-VariantConfig -BaseConfigPath $BaseConfig -OutPath $variantConfig -Overrides $candidate.overrides

  Write-Host ""
  Write-Host "== Tune run ($Label / $candidateId) =="
  $dbName = "semanticfs.tuneholdout.$safeLabel.tune.$candidateId.$runId.db"
  $tuneReport = Run-HeadToHead -CliBin $cliBin -ConfigPath $variantConfig -Repo $RepoRoot -GoldenPath $TuneGolden -DbName $dbName -UseHistory $History.IsPresent
  $metrics = As-MetricObject -Report $tuneReport
  $score = Candidate-Score -Metrics $metrics

  $tuneResults += [pscustomobject]@{
    id = $candidateId
    config_path = $variantConfig
    overrides = [pscustomobject]$candidate.overrides
    score = $score
    tune = $metrics
  }
}

$best = $tuneResults | Sort-Object @{Expression = "score"; Descending = $true}, @{Expression = { $_.tune.semantic.mrr }; Descending = $true}, @{Expression = { $_.tune.semantic.recall_at_topn }; Descending = $true}, @{Expression = { $_.tune.semantic.symbol_hit_rate }; Descending = $true}, @{Expression = { $_.tune.semantic.p95_latency_ms }; Descending = $false} | Select-Object -First 1
if ($null -eq $best) {
  throw "no tune candidates evaluated"
}

Write-Host ""
Write-Host "== Holdout run ($Label / selected=$($best.id)) =="
$holdoutDb = "semanticfs.tuneholdout.$safeLabel.holdout.$($best.id).$runId.db"
$holdoutReport = Run-HeadToHead -CliBin $cliBin -ConfigPath $best.config_path -Repo $RepoRoot -GoldenPath $HoldoutGolden -DbName $holdoutDb -UseHistory $History.IsPresent
$holdoutMetrics = As-MetricObject -Report $holdoutReport

$result = [pscustomobject]@{
  scenario = "tune_holdout"
  label = $Label
  repo_root = $RepoRoot
  base_config = $BaseConfig
  tune_golden = $TuneGolden
  holdout_golden = $HoldoutGolden
  selected_candidate = [pscustomobject]@{
    id = $best.id
    config_path = $best.config_path
    overrides = $best.overrides
    score = $best.score
  }
  tune_candidates = $tuneResults
  holdout = $holdoutMetrics
  run_id = $runId
  generated_at_utc = (Get-Date).ToUniversalTime().ToString("o")
}

$benchDir = ".semanticfs/bench"
$historyDir = Join-Path $benchDir "history"
New-Item -ItemType Directory -Force -Path $benchDir | Out-Null
New-Item -ItemType Directory -Force -Path $historyDir | Out-Null

$latestPath = Join-Path $benchDir "tune_holdout_${safeLabel}_latest.json"
$payload = $result | ConvertTo-Json -Depth 12
Set-Content -Path $latestPath -Value $payload

if ($History.IsPresent) {
  $ts = (Get-Date).ToUniversalTime().ToString("yyyyMMddTHHmmssZ")
  $histPath = Join-Path $historyDir "tune_holdout_${safeLabel}_${ts}.json"
  Set-Content -Path $histPath -Value $payload
}

Remove-Item Env:SEMANTICFS_DB_PATH -ErrorAction SilentlyContinue

Write-Host ""
Write-Host "Tune/Holdout complete for $Label"
Write-Host "Selected candidate: $($best.id)"
Write-Host ("Holdout semanticfs metrics: recall={0:N4} mrr={1:N4} symbol_hit={2:N4} p95_ms={3:N3}" -f $holdoutMetrics.semantic.recall_at_topn, $holdoutMetrics.semantic.mrr, $holdoutMetrics.semantic.symbol_hit_rate, $holdoutMetrics.semantic.p95_latency_ms)
Write-Host ("Holdout baseline metrics:  recall={0:N4} mrr={1:N4} symbol_hit={2:N4} p95_ms={3:N3}" -f $holdoutMetrics.baseline.recall_at_topn, $holdoutMetrics.baseline.mrr, $holdoutMetrics.baseline.symbol_hit_rate, $holdoutMetrics.baseline.p95_latency_ms)
Write-Host ("Holdout delta (sf-rg):     recall={0:N4} mrr={1:N4} symbol_hit={2:N4} p95_ms={3:N3}" -f $holdoutMetrics.delta_semantic_minus_baseline.recall_at_topn, $holdoutMetrics.delta_semantic_minus_baseline.mrr, $holdoutMetrics.delta_semantic_minus_baseline.symbol_hit_rate, $holdoutMetrics.delta_semantic_minus_baseline.p95_latency_ms)
Write-Host "Artifact: $latestPath"
