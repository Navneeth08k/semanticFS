param(
  [string]$DatasetName,
  [string]$HistoryDir = ".semanticfs/bench/history",
  [string]$ArtifactPath = "",
  [string]$OutputPath = ""
)

$ErrorActionPreference = "Stop"

function Normalize-Label([string]$Value) {
  if ([string]::IsNullOrWhiteSpace($Value)) {
    return "dataset"
  }
  $slug = $Value.ToLowerInvariant() -replace '[^a-z0-9]+', '_'
  $slug = $slug.Trim('_')
  if ([string]::IsNullOrWhiteSpace($slug)) {
    return "dataset"
  }
  return $slug
}

function Resolve-ArtifactPath([string]$HistoryDir, [string]$DatasetName) {
  if (-not (Test-Path $HistoryDir)) {
    throw "History dir not found: $HistoryDir"
  }
  $files = @(
    Get-ChildItem -Path $HistoryDir -File -Filter "head_to_head_latest_*.json" |
      Sort-Object LastWriteTimeUtc -Descending
  )
  foreach ($f in $files) {
    $doc = $null
    try {
      $doc = Get-Content -Path $f.FullName | ConvertFrom-Json
    }
    catch {
      continue
    }
    if ($null -eq $doc -or $null -eq $doc.datasets) {
      continue
    }
    $match = @($doc.datasets | Where-Object { $_.dataset_name -eq $DatasetName } | Select-Object -First 1)
    if ($match.Count -gt 0) {
      return $f.FullName
    }
  }
  throw "No head-to-head artifact found for dataset: $DatasetName"
}

function New-DetailSummary([object]$Detail) {
  $semRank = $Detail.semanticfs.first_relevant_rank
  $baseRank = $Detail.baseline_rg.first_relevant_rank
  $semTop = if ($Detail.semanticfs.retrieved_paths_topn.Count -gt 0) { $Detail.semanticfs.retrieved_paths_topn[0] } else { $null }
  $baseTop = if ($Detail.baseline_rg.retrieved_paths_topn.Count -gt 0) { $Detail.baseline_rg.retrieved_paths_topn[0] } else { $null }
  [pscustomobject]@{
    id = $Detail.id
    query = $Detail.query
    expected_paths = $Detail.expected_paths
    semantic_rank = $semRank
    baseline_rank = $baseRank
    semantic_top1 = $semTop
    baseline_top1 = $baseTop
    semantic_latency_ms = $Detail.semanticfs.latency_ms
    baseline_latency_ms = $Detail.baseline_rg.latency_ms
  }
}

if ([string]::IsNullOrWhiteSpace($ArtifactPath)) {
  if ([string]::IsNullOrWhiteSpace($DatasetName)) {
    throw "Either -DatasetName or -ArtifactPath is required."
  }
  $ArtifactPath = Resolve-ArtifactPath -HistoryDir $HistoryDir -DatasetName $DatasetName
}

if (-not (Test-Path $ArtifactPath)) {
  throw "Artifact not found: $ArtifactPath"
}

$doc = Get-Content -Path $ArtifactPath | ConvertFrom-Json
if ($null -eq $doc -or $null -eq $doc.datasets) {
  throw "Invalid head-to-head artifact: $ArtifactPath"
}

$dataset = $null
if (-not [string]::IsNullOrWhiteSpace($DatasetName)) {
  $dataset = @($doc.datasets | Where-Object { $_.dataset_name -eq $DatasetName } | Select-Object -First 1)
} else {
  $dataset = @($doc.datasets | Select-Object -First 1)
}
if ($dataset.Count -eq 0) {
  throw "Dataset not found in artifact: $DatasetName"
}
$dataset = $dataset[0]

if ([string]::IsNullOrWhiteSpace($OutputPath)) {
  $slug = Normalize-Label $dataset.dataset_name
  $OutputPath = ".semanticfs/bench/query_gap_{0}_latest.json" -f $slug
}

$details = @($dataset.details)
$semanticMisses = @($details | Where-Object { $null -eq $_.semanticfs.first_relevant_rank } | ForEach-Object { New-DetailSummary $_ })
$baselineMisses = @($details | Where-Object { $null -eq $_.baseline_rg.first_relevant_rank } | ForEach-Object { New-DetailSummary $_ })
$rankLag = @(
  $details |
    Where-Object {
      $null -ne $_.semanticfs.first_relevant_rank -and
      $null -ne $_.baseline_rg.first_relevant_rank -and
      $_.semanticfs.first_relevant_rank -gt $_.baseline_rg.first_relevant_rank
    } |
    ForEach-Object { New-DetailSummary $_ }
)
$rankGain = @(
  $details |
    Where-Object {
      $null -ne $_.semanticfs.first_relevant_rank -and
      $null -ne $_.baseline_rg.first_relevant_rank -and
      $_.semanticfs.first_relevant_rank -lt $_.baseline_rg.first_relevant_rank
    } |
    ForEach-Object { New-DetailSummary $_ }
)
$equalHits = @(
  $details |
    Where-Object {
      $null -ne $_.semanticfs.first_relevant_rank -and
      $null -ne $_.baseline_rg.first_relevant_rank -and
      $_.semanticfs.first_relevant_rank -eq $_.baseline_rg.first_relevant_rank
    } |
    ForEach-Object { New-DetailSummary $_ }
)

$payload = [pscustomobject]@{
  scenario = "query_gap_report"
  generated_at_utc = (Get-Date).ToUniversalTime().ToString("o")
  source_artifact = $ArtifactPath
  dataset_name = $dataset.dataset_name
  query_count = @($details).Count
  summary = [pscustomobject]@{
    semantic_miss_count = $semanticMisses.Count
    baseline_miss_count = $baselineMisses.Count
    semantic_rank_lag_count = $rankLag.Count
    semantic_rank_gain_count = $rankGain.Count
    equal_hit_rank_count = $equalHits.Count
  }
  semantic_misses = $semanticMisses
  baseline_misses = $baselineMisses
  semantic_rank_lag = $rankLag
  semantic_rank_gain = $rankGain
  equal_hit_rank = $equalHits
}

$outDir = Split-Path $OutputPath -Parent
if (-not [string]::IsNullOrWhiteSpace($outDir)) {
  New-Item -ItemType Directory -Force -Path $outDir | Out-Null
}
$json = $payload | ConvertTo-Json -Depth 8
Set-Content -Path $OutputPath -Value $json

Write-Host ""
Write-Host "Query gap report complete."
Write-Host "dataset=$($dataset.dataset_name) queries=$(@($details).Count)"
Write-Host "semantic_miss=$($semanticMisses.Count) baseline_miss=$($baselineMisses.Count) rank_lag=$($rankLag.Count) rank_gain=$($rankGain.Count) equal_rank=$($equalHits.Count)"
if ($semanticMisses.Count -gt 0) {
  Write-Host "Semantic misses:"
  $semanticMisses | Select-Object id, query, semantic_top1, baseline_rank | Format-Table -AutoSize
}
if ($rankLag.Count -gt 0) {
  Write-Host "Semantic rank lag:"
  $rankLag | Select-Object id, query, semantic_rank, baseline_rank, semantic_top1 | Format-Table -AutoSize
}
Write-Host ""
Write-Host "Artifact: $OutputPath"
