param(
  [string]$CandidatesPath = ".semanticfs/bench/filesystem_repo_candidates_latest.json",
  [string]$TuneHoldoutGlob = ".semanticfs/bench/tune_holdout_*_latest.json",
  [string]$OutputPath = ".semanticfs/bench/filesystem_scope_backlog_latest.json",
  [int]$TopN = 0
)

$ErrorActionPreference = "Stop"

function Normalize-PathKey([string]$PathValue) {
  if ([string]::IsNullOrWhiteSpace($PathValue)) {
    return ""
  }
  return ([IO.Path]::GetFullPath($PathValue)).ToLowerInvariant()
}

function Parse-Utc([string]$Value, [datetime]$FallbackUtc) {
  if ([string]::IsNullOrWhiteSpace($Value)) {
    return $FallbackUtc
  }
  try {
    return [datetime]::Parse($Value).ToUniversalTime()
  }
  catch {
    return $FallbackUtc
  }
}

if (-not (Test-Path $CandidatesPath)) {
  throw "Candidates artifact not found: $CandidatesPath"
}

$candidatePayload = Get-Content -Path $CandidatesPath | ConvertFrom-Json
if ($null -eq $candidatePayload -or $null -eq $candidatePayload.repos) {
  throw "Invalid candidates artifact: $CandidatesPath"
}

$globDir = Split-Path $TuneHoldoutGlob -Parent
if ([string]::IsNullOrWhiteSpace($globDir)) {
  $globDir = "."
}
$globLeaf = Split-Path $TuneHoldoutGlob -Leaf
$tuneFiles = @(
  Get-ChildItem -Path $globDir -File -Filter $globLeaf -ErrorAction SilentlyContinue
)

$latestCoverageByRepo = @{}
foreach ($f in $tuneFiles) {
  $doc = $null
  try {
    $doc = Get-Content -Path $f.FullName | ConvertFrom-Json
  }
  catch {
    continue
  }
  if ($null -eq $doc -or [string]::IsNullOrWhiteSpace($doc.repo_root) -or $null -eq $doc.holdout) {
    continue
  }
  if ($null -eq $doc.holdout.semantic -or $null -eq $doc.holdout.baseline) {
    continue
  }

  $repoKey = Normalize-PathKey $doc.repo_root
  if ([string]::IsNullOrWhiteSpace($repoKey)) {
    continue
  }

  $entry = [pscustomobject]@{
    label = $doc.label
    repo_root = $doc.repo_root
    artifact_path = $f.FullName
    generated_at_utc = $doc.generated_at_utc
    query_count = [int]$doc.holdout.query_count
    semantic = [pscustomobject]@{
      recall = [double]$doc.holdout.semantic.recall_at_topn
      mrr = [double]$doc.holdout.semantic.mrr
      symbol = [double]$doc.holdout.semantic.symbol_hit_rate
      p95_ms = [double]$doc.holdout.semantic.p95_latency_ms
    }
    baseline = [pscustomobject]@{
      recall = [double]$doc.holdout.baseline.recall_at_topn
      mrr = [double]$doc.holdout.baseline.mrr
      symbol = [double]$doc.holdout.baseline.symbol_hit_rate
      p95_ms = [double]$doc.holdout.baseline.p95_latency_ms
    }
  }

  if (-not $latestCoverageByRepo.ContainsKey($repoKey)) {
    $latestCoverageByRepo[$repoKey] = $entry
    continue
  }

  $existing = $latestCoverageByRepo[$repoKey]
  $existingTs = Parse-Utc -Value $existing.generated_at_utc -FallbackUtc (Get-Item $existing.artifact_path).LastWriteTimeUtc
  $incomingTs = Parse-Utc -Value $entry.generated_at_utc -FallbackUtc $f.LastWriteTimeUtc
  if ($incomingTs -gt $existingTs) {
    $latestCoverageByRepo[$repoKey] = $entry
  }
}

$coverageEntries = @($latestCoverageByRepo.Values)
$items = New-Object System.Collections.Generic.List[object]
foreach ($repo in $candidatePayload.repos) {
  $repoKey = Normalize-PathKey $repo.repo_root
  $coverage = $null
  $partialLabels = @()
  $partialCount = 0
  if ($latestCoverageByRepo.ContainsKey($repoKey)) {
    $coverage = $latestCoverageByRepo[$repoKey]
  } else {
    $prefix = "$repoKey\"
    $partialMatches = @(
      $coverageEntries | Where-Object {
        $candidateCoverageRoot = Normalize-PathKey $_.repo_root
        $candidateCoverageRoot.StartsWith($prefix)
      }
    )
    if ($partialMatches.Count -gt 0) {
      $partialCount = $partialMatches.Count
      $partialLabels = @($partialMatches | Select-Object -ExpandProperty label | Sort-Object -Unique)
    }
  }

  $status = "uncovered"
  $priorityBucket = 0
  $nextAction = "bootstrap_and_strict_holdout"
  $deltaRecall = $null
  $deltaMrr = $null
  $deltaSymbol = $null
  $deltaP95 = $null
  $qualityGap = $false

  if ($null -eq $coverage -and $partialCount -gt 0) {
    $status = "covered_partial"
    $priorityBucket = 2
    $nextAction = "expand_from_subrepo_coverage"
  } elseif ($null -ne $coverage) {
    $deltaRecall = $coverage.semantic.recall - $coverage.baseline.recall
    $deltaMrr = $coverage.semantic.mrr - $coverage.baseline.mrr
    $deltaSymbol = $coverage.semantic.symbol - $coverage.baseline.symbol
    $deltaP95 = $coverage.semantic.p95_ms - $coverage.baseline.p95_ms
    $qualityGap = ($deltaRecall -lt 0.0) -or ($deltaMrr -lt 0.0) -or ($deltaSymbol -lt 0.0)

    if ($qualityGap) {
      $status = "covered_gap"
      $priorityBucket = 1
      $nextAction = "query_level_triage"
    } else {
      $status = "covered_ok"
      $priorityBucket = 3
      $nextAction = "monitor_or_expand_queries"
    }
  }

  $items.Add([pscustomobject]@{
    repo_root = $repo.repo_root
    tracked_files = [int]$repo.tracked_files
    code_files = [int]$repo.code_files
    status = $status
    priority_bucket = $priorityBucket
    next_action = $nextAction
    latest_label = if ($null -ne $coverage) { $coverage.label } else { $null }
    latest_generated_at_utc = if ($null -ne $coverage) { $coverage.generated_at_utc } else { $null }
    latest_query_count = if ($null -ne $coverage) { $coverage.query_count } else { $null }
    partial_coverage_count = $partialCount
    partial_coverage_labels = $partialLabels
    delta_recall = $deltaRecall
    delta_mrr = $deltaMrr
    delta_symbol_hit = $deltaSymbol
    delta_p95_ms = $deltaP95
    semantic_p95_ms = if ($null -ne $coverage) { $coverage.semantic.p95_ms } else { $null }
    baseline_p95_ms = if ($null -ne $coverage) { $coverage.baseline.p95_ms } else { $null }
  })
}

$ordered = @(
  $items |
    Sort-Object -Property @{Expression = "priority_bucket"; Descending = $false}, @{Expression = "tracked_files"; Descending = $true}, @{Expression = "code_files"; Descending = $true}
)
if ($TopN -gt 0) {
  $ordered = @($ordered | Select-Object -First $TopN)
}

$uncoveredCount = @($items | Where-Object { $_.status -eq "uncovered" }).Count
$coveredGapCount = @($items | Where-Object { $_.status -eq "covered_gap" }).Count
$coveredPartialCount = @($items | Where-Object { $_.status -eq "covered_partial" }).Count
$coveredOkCount = @($items | Where-Object { $_.status -eq "covered_ok" }).Count

$payload = [pscustomobject]@{
  scenario = "filesystem_scope_backlog"
  generated_at_utc = (Get-Date).ToUniversalTime().ToString("o")
  candidates_path = $CandidatesPath
  tune_holdout_glob = $TuneHoldoutGlob
  candidate_repo_count = @($candidatePayload.repos).Count
  tune_holdout_artifact_count = $tuneFiles.Count
  counts = [pscustomobject]@{
    uncovered = $uncoveredCount
    covered_gap = $coveredGapCount
    covered_partial = $coveredPartialCount
    covered_ok = $coveredOkCount
  }
  items = $ordered
}

$outDir = Split-Path $OutputPath -Parent
if (-not [string]::IsNullOrWhiteSpace($outDir)) {
  New-Item -ItemType Directory -Force -Path $outDir | Out-Null
}
$json = $payload | ConvertTo-Json -Depth 8
Set-Content -Path $OutputPath -Value $json

Write-Host ""
Write-Host "Filesystem scope backlog complete."
Write-Host "candidates: $(@($candidatePayload.repos).Count) | tune/holdout artifacts: $($tuneFiles.Count)"
Write-Host "uncovered: $uncoveredCount | covered_gap: $coveredGapCount | covered_partial: $coveredPartialCount | covered_ok: $coveredOkCount"
$ordered |
  Select-Object -First 12 repo_root, status, tracked_files, code_files, latest_label, partial_coverage_count, next_action |
  Format-Table -AutoSize
Write-Host ""
Write-Host "Artifact: $OutputPath"
