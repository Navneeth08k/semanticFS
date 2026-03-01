param(
  [string]$BacklogPath = ".semanticfs/bench/filesystem_scope_backlog_latest.json",
  [string]$OutputPath = ".semanticfs/bench/filesystem_domain_plan_latest.json",
  [int]$TopN = 0
)

$ErrorActionPreference = "Stop"

function Normalize-Slug([string]$Value) {
  if ([string]::IsNullOrWhiteSpace($Value)) {
    return "domain"
  }
  $slug = $Value.ToLowerInvariant() -replace '[^a-z0-9]+', '_'
  $slug = $slug.Trim('_')
  if ([string]::IsNullOrWhiteSpace($slug)) {
    return "domain"
  }
  return $slug
}

function Domain-IdFromPath([string]$PathValue) {
  if ([string]::IsNullOrWhiteSpace($PathValue)) {
    return "domain"
  }
  $leaf = Split-Path $PathValue -Leaf
  if ([string]::IsNullOrWhiteSpace($leaf)) {
    $leaf = $PathValue
  }
  return Normalize-Slug $leaf
}

function Guess-TrustClass([string]$PathValue) {
  if ([string]::IsNullOrWhiteSpace($PathValue)) {
    return "user_workspace"
  }
  $norm = $PathValue.ToLowerInvariant()
  if ($norm.Contains('\desktop\navneeththings\projects\')) {
    return "user_workspace"
  }
  if ($norm.Contains('\documents\')) {
    return "user_workspace"
  }
  if ($norm.Contains('\downloads\')) {
    return "downloaded_clone"
  }
  return "user_local"
}

function Map-PromotionState([string]$Status, [string]$NextAction) {
  switch ($Status) {
    "uncovered" { return "promote_candidate" }
    "covered_gap" { return "harden_existing" }
    "covered_partial" { return "expand_parent_root" }
    "covered_representative" { return "add_strict_holdout" }
    "covered_ok" { return "monitor" }
    default {
      if ($NextAction -eq "query_level_triage") { return "harden_existing" }
      return "review"
    }
  }
}

if (-not (Test-Path $BacklogPath)) {
  throw "Backlog artifact not found: $BacklogPath"
}

$backlog = Get-Content -Path $BacklogPath | ConvertFrom-Json
if ($null -eq $backlog -or $null -eq $backlog.items) {
  throw "Invalid backlog artifact: $BacklogPath"
}

$items = @($backlog.items)
if ($TopN -gt 0) {
  $items = @($items | Select-Object -First $TopN)
}

$domains = New-Object System.Collections.Generic.List[object]
foreach ($item in $items) {
  $domainId = Domain-IdFromPath $item.repo_root
  $trustClass = Guess-TrustClass $item.repo_root
  $promotionState = Map-PromotionState -Status $item.status -NextAction $item.next_action
  $enabled = $false
  $allowRoots = @("**")
  $denyGlobs = @()

  if ($item.status -eq "covered_gap") {
    $enabled = $true
  }

  $domains.Add([pscustomobject]@{
    domain_id = $domainId
    root = $item.repo_root
    trust_class = $trustClass
    source_status = $item.status
    next_action = $item.next_action
    promotion_state = $promotionState
    enabled = $enabled
    allow_roots = $allowRoots
    deny_globs = $denyGlobs
    notes = @(
      "Derived from filesystem backlog",
      "Single-root runtime remains authoritative until multi-root indexing is wired"
    )
  })
}

$counts = [pscustomobject]@{
  promote_candidate = @($domains | Where-Object { $_.promotion_state -eq "promote_candidate" }).Count
  harden_existing = @($domains | Where-Object { $_.promotion_state -eq "harden_existing" }).Count
  expand_parent_root = @($domains | Where-Object { $_.promotion_state -eq "expand_parent_root" }).Count
  add_strict_holdout = @($domains | Where-Object { $_.promotion_state -eq "add_strict_holdout" }).Count
  monitor = @($domains | Where-Object { $_.promotion_state -eq "monitor" }).Count
  review = @($domains | Where-Object { $_.promotion_state -eq "review" }).Count
}

$payload = [pscustomobject]@{
  scenario = "filesystem_domain_plan"
  generated_at_utc = (Get-Date).ToUniversalTime().ToString("o")
  backlog_path = $BacklogPath
  domain_count = $domains.Count
  counts = $counts
  domains = $domains
}

$outDir = Split-Path $OutputPath -Parent
if (-not [string]::IsNullOrWhiteSpace($outDir)) {
  New-Item -ItemType Directory -Force -Path $outDir | Out-Null
}
$json = $payload | ConvertTo-Json -Depth 8
Set-Content -Path $OutputPath -Value $json

Write-Host ""
Write-Host "Phase 3 domain plan complete."
Write-Host "domains: $($domains.Count)"
Write-Host "promote_candidate=$($counts.promote_candidate) harden_existing=$($counts.harden_existing) expand_parent_root=$($counts.expand_parent_root) add_strict_holdout=$($counts.add_strict_holdout) monitor=$($counts.monitor) review=$($counts.review)"
$domains |
  Select-Object -First 12 domain_id, source_status, promotion_state, trust_class, enabled, root |
  Format-Table -AutoSize
Write-Host ""
Write-Host "Artifact: $OutputPath"
