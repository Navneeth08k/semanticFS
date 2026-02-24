param(
  [string[]]$Roots,
  [int]$MinTrackedFiles = 200,
  [int]$TopN = 20,
  [string]$OutputPath = ".semanticfs/bench/filesystem_repo_candidates_latest.json",
  [switch]$IncludeWorkspaceMirrors,
  [switch]$DisableRemoteDedupe
)

$ErrorActionPreference = "Stop"

function Normalize-PathKey([string]$PathValue) {
  if ([string]::IsNullOrWhiteSpace($PathValue)) {
    return ""
  }
  return ([IO.Path]::GetFullPath($PathValue)).ToLowerInvariant()
}

function Is-WorkspaceMirrorPath([string]$PathValue) {
  if ([string]::IsNullOrWhiteSpace($PathValue)) {
    return $false
  }
  $needle = "\appdata\roaming\code\user\workspacestorage\"
  $norm = (Normalize-PathKey $PathValue)
  return $norm.Contains($needle)
}

function Resolve-DefaultRoots {
  if ($Roots -and $Roots.Count -gt 0) {
    return $Roots
  }
  $cwd = (Get-Location).Path
  $parent = Split-Path $cwd -Parent
  if ([string]::IsNullOrWhiteSpace($parent)) {
    return @($cwd)
  }
  return @($parent)
}

function Get-RepoStats([string]$RepoRoot) {
  $tracked = @()
  try {
    $tracked = git -C $RepoRoot ls-files 2>$null
  }
  catch {
    return $null
  }
  if ($LASTEXITCODE -ne 0) {
    return $null
  }

  $trackedCount = @($tracked).Count
  $codeRegex = '\.(rs|py|ts|tsx|js|jsx|go|java|c|cpp|h|hpp|cs|kt|swift|rb|php|scala|dart)$'
  $codeCount = @($tracked | Where-Object { $_ -imatch $codeRegex }).Count

  $identityKey = "path:{0}" -f (Normalize-PathKey $RepoRoot)
  if (-not $DisableRemoteDedupe.IsPresent) {
    $remote = ""
    try {
      $remote = (git -C $RepoRoot config --get remote.origin.url 2>$null | Select-Object -First 1)
    }
    catch {
      $remote = ""
    }
    if (-not [string]::IsNullOrWhiteSpace($remote)) {
      $identityKey = "remote:{0}" -f ($remote.Trim().ToLowerInvariant())
    }
  }

  return [pscustomobject]@{
    repo_root = $RepoRoot
    repo_key = Normalize-PathKey $RepoRoot
    identity_key = $identityKey
    is_workspace_mirror = (Is-WorkspaceMirrorPath $RepoRoot)
    tracked_files = $trackedCount
    code_files = $codeCount
  }
}

function Prefer-RepoEntry([object]$A, [object]$B) {
  if ($null -eq $A) { return $B }
  if ($null -eq $B) { return $A }

  if ($A.is_workspace_mirror -and -not $B.is_workspace_mirror) { return $B }
  if ($B.is_workspace_mirror -and -not $A.is_workspace_mirror) { return $A }
  if ($A.tracked_files -lt $B.tracked_files) { return $B }
  if ($B.tracked_files -lt $A.tracked_files) { return $A }
  if ($A.code_files -lt $B.code_files) { return $B }
  if ($B.code_files -lt $A.code_files) { return $A }
  if ($A.repo_root.Length -le $B.repo_root.Length) { return $A }
  return $B
}

$resolvedRoots = Resolve-DefaultRoots
$existingRoots = @(
  $resolvedRoots | Where-Object { -not [string]::IsNullOrWhiteSpace($_) -and (Test-Path $_) }
)

if ($existingRoots.Count -eq 0) {
  throw "No valid roots provided."
}

$gitDirs = New-Object System.Collections.Generic.List[string]
foreach ($root in $existingRoots) {
  Write-Host "Scanning for repos under: $root"
  $dirs = Get-ChildItem -Path $root -Directory -Recurse -Force -ErrorAction SilentlyContinue |
    Where-Object { $_.Name -eq ".git" }
  foreach ($d in $dirs) {
    $gitDirs.Add($d.FullName)
  }
}

$repoRoots = @(
  $gitDirs |
    ForEach-Object { Split-Path $_ -Parent } |
    Sort-Object -Unique
)

$results = New-Object System.Collections.Generic.List[object]
$excludedWorkspaceMirrors = New-Object System.Collections.Generic.List[string]
foreach ($repo in $repoRoots) {
  if (-not $IncludeWorkspaceMirrors.IsPresent -and (Is-WorkspaceMirrorPath $repo)) {
    $excludedWorkspaceMirrors.Add($repo)
    continue
  }
  $stats = Get-RepoStats -RepoRoot $repo
  if ($null -eq $stats) {
    continue
  }
  if ($stats.tracked_files -lt $MinTrackedFiles) {
    continue
  }
  $results.Add($stats)
}

$preDedupeCount = $results.Count
$dedupedMap = @{}
$dedupedAway = New-Object System.Collections.Generic.List[object]
foreach ($r in $results) {
  $k = $r.identity_key
  if ([string]::IsNullOrWhiteSpace($k)) {
    $k = "path:{0}" -f $r.repo_key
  }
  if (-not $dedupedMap.ContainsKey($k)) {
    $dedupedMap[$k] = $r
    continue
  }
  $existing = $dedupedMap[$k]
  $winner = Prefer-RepoEntry -A $existing -B $r
  if ($winner.repo_root -eq $existing.repo_root) {
    $dedupedAway.Add([pscustomobject]@{
      identity_key = $k
      kept_repo_root = $existing.repo_root
      removed_repo_root = $r.repo_root
      reason = "identity_dedupe"
    })
  } else {
    $dedupedAway.Add([pscustomobject]@{
      identity_key = $k
      kept_repo_root = $r.repo_root
      removed_repo_root = $existing.repo_root
      reason = "identity_dedupe"
    })
    $dedupedMap[$k] = $r
  }
}

$deduped = @($dedupedMap.Values)
$sorted = @(
  $deduped |
    Sort-Object -Property @{Expression = "tracked_files"; Descending = $true}, @{Expression = "code_files"; Descending = $true}
)
if ($TopN -gt 0) {
  $sorted = @($sorted | Select-Object -First $TopN)
}

$payload = [pscustomobject]@{
  scenario = "filesystem_repo_discovery"
  generated_at_utc = (Get-Date).ToUniversalTime().ToString("o")
  roots = $existingRoots
  min_tracked_files = $MinTrackedFiles
  top_n = $TopN
  include_workspace_mirrors = $IncludeWorkspaceMirrors.IsPresent
  remote_dedupe_enabled = (-not $DisableRemoteDedupe.IsPresent)
  repo_count_before_dedupe = $preDedupeCount
  repo_count = $sorted.Count
  excluded_workspace_mirror_count = $excludedWorkspaceMirrors.Count
  deduped_away_count = $dedupedAway.Count
  excluded_workspace_mirrors = $excludedWorkspaceMirrors
  deduped_away = $dedupedAway
  repos = $sorted
}

$outDir = Split-Path $OutputPath -Parent
if (-not [string]::IsNullOrWhiteSpace($outDir)) {
  New-Item -ItemType Directory -Force -Path $outDir | Out-Null
}
$json = $payload | ConvertTo-Json -Depth 8
Set-Content -Path $OutputPath -Value $json

Write-Host ""
Write-Host "Repo discovery complete. Candidates: $($sorted.Count)"
Write-Host "Excluded workspace mirrors: $($excludedWorkspaceMirrors.Count)"
Write-Host "Identity deduped away: $($dedupedAway.Count)"
$sorted | Format-Table -AutoSize
Write-Host ""
Write-Host "Artifact: $OutputPath"
