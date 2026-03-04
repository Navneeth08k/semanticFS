param(
    [string]$UserRoot = $env:USERPROFILE,
    [string]$OutputDir = ".semanticfs/bench",
    [int]$MaxIndexedFiles = 32,
    [int]$MaxFixtureQueries = 32,
    [int]$MaxFileMb = 5
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

function New-Query {
    param(
        [string]$Id,
        [string]$Query,
        [string[]]$ExpectedPaths
    )

    [pscustomobject]@{
        id = $Id
        query = $Query
        expected_paths = $ExpectedPaths
        symbol_query = $false
    }
}

function Convert-ToPosixPath {
    param([string]$PathText)

    return $PathText.Replace("\", "/")
}

function Test-HiddenRelativePath {
    param([string]$RelativePath)

    foreach ($segment in (Convert-ToPosixPath $RelativePath).Split('/')) {
        if ([string]::IsNullOrWhiteSpace($segment)) {
            continue
        }
        if ($segment.StartsWith(".")) {
            return $true
        }
    }
    return $false
}

function Test-TopLevelExcluded {
    param(
        [string]$RelativePath,
        [string[]]$Patterns
    )

    $normalized = Convert-ToPosixPath $RelativePath
    $firstSegment = ($normalized.Split('/')[0]).Trim()
    if ([string]::IsNullOrWhiteSpace($firstSegment)) {
        return $false
    }

    foreach ($pattern in $Patterns) {
        $wildcard = New-Object System.Management.Automation.WildcardPattern($pattern, ([System.Management.Automation.WildcardOptions]::IgnoreCase))
        if ($wildcard.IsMatch($firstSegment)) {
            return $true
        }
    }

    return $false
}

function Test-ExcludedByPathPolicy {
    param(
        [string]$RelativePath,
        [string[]]$TopLevelPatterns,
        [string[]]$BlockedSegments
    )

    if (Test-TopLevelExcluded -RelativePath $RelativePath -Patterns $TopLevelPatterns) {
        return $true
    }

    $normalized = Convert-ToPosixPath $RelativePath
    foreach ($segment in $normalized.Split('/')) {
        if ([string]::IsNullOrWhiteSpace($segment)) {
            continue
        }
        foreach ($blocked in $BlockedSegments) {
            if ($segment -ieq $blocked) {
                return $true
            }
        }
    }

    return $false
}

function Test-TextLikeFile {
    param(
        [System.IO.FileInfo]$Item,
        [int]$MaxFileMb
    )

    $allowedExtensions = @(
        ".txt", ".md", ".markdown", ".json", ".toml", ".yaml", ".yml",
        ".ini", ".cfg", ".conf", ".rules", ".url", ".py", ".ps1",
        ".sh", ".csv", ".log", ".xml"
    )

    $extension = [System.IO.Path]::GetExtension($Item.Name).ToLowerInvariant()
    if ($Item.Name -ieq "desktop.ini") {
        return $false
    }
    if ([string]::IsNullOrWhiteSpace($extension)) {
        return $false
    }
    if ($allowedExtensions -notcontains $extension) {
        return $false
    }

    $sizeMb = [math]::Floor($Item.Length / 1MB)
    return $sizeMb -le $MaxFileMb
}

function Add-FullHomeCandidates {
    param(
        [string]$CurrentDir,
        [string]$RelativeBase,
        [System.Collections.Generic.List[string]]$Selected,
        [int]$Limit,
        [string[]]$ExcludedTopLevel,
        [int]$MaxFileMb
    )

    if ($Selected.Count -ge $Limit) {
        return
    }
    if (-not (Test-Path -LiteralPath $CurrentDir)) {
        return
    }

    $entries = Get-ChildItem -LiteralPath $CurrentDir -Force -ErrorAction SilentlyContinue |
        Sort-Object Name -CaseSensitive

    foreach ($entry in $entries) {
        if ($Selected.Count -ge $Limit) {
            break
        }

        $relativePath = if ([string]::IsNullOrWhiteSpace($RelativeBase)) {
            $entry.Name
        } else {
            "{0}/{1}" -f (Convert-ToPosixPath $RelativeBase), $entry.Name
        }
        $relativePath = Convert-ToPosixPath $relativePath

        if (Test-HiddenRelativePath -RelativePath $relativePath) {
            continue
        }
        if (Test-ExcludedByPathPolicy -RelativePath $relativePath -TopLevelPatterns $ExcludedTopLevel -BlockedSegments $blockedPathSegments) {
            continue
        }

        if ($entry.PSIsContainer) {
            Add-FullHomeCandidates -CurrentDir $entry.FullName -RelativeBase $relativePath -Selected $Selected -Limit $Limit -ExcludedTopLevel $ExcludedTopLevel -MaxFileMb $MaxFileMb
            continue
        }

        if (Test-TextLikeFile -Item $entry -MaxFileMb $MaxFileMb) {
            $Selected.Add($relativePath) | Out-Null
        }
    }
}

function Convert-ToQueryText {
    param(
        [string]$RelativePath,
        [hashtable]$LeafCounts
    )

    $normalized = Convert-ToPosixPath $RelativePath
    $leaf = Split-Path -Path $normalized -Leaf
    $querySource = if (($LeafCounts[$leaf] | ForEach-Object { $_ }) -gt 1) {
        $normalized
    } else {
        $leaf
    }

    $queryText = ($querySource -replace "[\\/._-]+", " ").Trim()
    if ([string]::IsNullOrWhiteSpace($queryText)) {
        return $normalized
    }
    return $queryText
}

New-Item -ItemType Directory -Path $OutputDir -Force | Out-Null

$selectionLimit = if ($MaxIndexedFiles -gt 0) {
    [Math]::Min($MaxIndexedFiles, [Math]::Max($MaxFixtureQueries, 1))
} else {
    [Math]::Max($MaxFixtureQueries, 1)
}

$excludedTopLevel = @(
    "3D Objects",
    "anaconda3",
    "AndroidStudioProjects",
    "ansel",
    "AppData",
    "Apple",
    "Application Data",
    "BrawlhallaReplays",
    "Contacts",
    "Cookies",
    "Desktop_SHORTCUT",
    "Favorites",
    "Google Drive",
    "github-classroom",
    "IntelGraphicsProfiles",
    "LabelObj.txt",
    "labelImg",
    "Links",
    "Local Settings",
    "miktex-console.lock",
    "Music",
    "My Documents",
    "NetHood",
    "NTUSER.DAT*",
    "ntuser.dat.LOG*",
    "NTUSER.DAT{*}.TM*",
    "ntuser.ini",
    "OneDrive",
    "Pictures",
    "py",
    "practApp",
    "PrintHood",
    "Recent",
    "Saved Games",
    "Searches",
    "SendTo",
    "source",
    "Start Menu",
    "Templates",
    "_viminfo",
    "vimfiles"
)

$blockedPathSegments = @(
    "__pycache__",
    ".venv",
    "venv",
    "env",
    "myenv",
    "site-packages",
    ".pytest_cache",
    ".mypy_cache",
    "large_protein_dataset",
    "dist",
    "build"
)

$selectedPaths = New-Object System.Collections.Generic.List[string]
$topLevelEntries = Get-ChildItem -LiteralPath $UserRoot -Force -ErrorAction SilentlyContinue |
    Sort-Object Name -CaseSensitive

foreach ($entry in ($topLevelEntries | Where-Object { -not $_.PSIsContainer })) {
    if ($selectedPaths.Count -ge $selectionLimit) {
        break
    }

    $relativePath = Convert-ToPosixPath $entry.Name
    if (Test-HiddenRelativePath -RelativePath $relativePath) {
        continue
    }
    if (Test-ExcludedByPathPolicy -RelativePath $relativePath -TopLevelPatterns $excludedTopLevel -BlockedSegments $blockedPathSegments) {
        continue
    }
    if (Test-TextLikeFile -Item $entry -MaxFileMb $MaxFileMb) {
        $selectedPaths.Add($relativePath) | Out-Null
    }
}

foreach ($entry in ($topLevelEntries | Where-Object { $_.PSIsContainer })) {
    if ($selectedPaths.Count -ge $selectionLimit) {
        break
    }

    $relativePath = Convert-ToPosixPath $entry.Name
    if (Test-HiddenRelativePath -RelativePath $relativePath) {
        continue
    }
    if (Test-ExcludedByPathPolicy -RelativePath $relativePath -TopLevelPatterns $excludedTopLevel -BlockedSegments $blockedPathSegments) {
        continue
    }

    Add-FullHomeCandidates -CurrentDir $entry.FullName -RelativeBase $relativePath -Selected $selectedPaths -Limit $selectionLimit -ExcludedTopLevel $excludedTopLevel -MaxFileMb $MaxFileMb
}

if ($selectedPaths.Count -eq 0) {
    throw "No supported files were discovered for the full-home pilot under $UserRoot."
}

$leafCounts = @{}
foreach ($relativePath in $selectedPaths) {
    $leaf = Split-Path -Path $relativePath -Leaf
    if ($leafCounts.ContainsKey($leaf)) {
        $leafCounts[$leaf] += 1
    } else {
        $leafCounts[$leaf] = 1
    }
}

$queries = New-Object System.Collections.Generic.List[object]
$queryId = 1
foreach ($relativePath in $selectedPaths) {
    $queryText = Convert-ToQueryText -RelativePath $relativePath -LeafCounts $leafCounts
    $queries.Add((New-Query -Id ("hf{0:d2}" -f $queryId) -Query $queryText -ExpectedPaths @("home_full/$relativePath"))) | Out-Null
    $queryId += 1
}

$denyGlobs = @(
    "3D Objects/**",
    "anaconda3/**",
    "AndroidStudioProjects/**",
    "ansel/**",
    "AppData/**",
    "Apple/**",
    "Application Data/**",
    "BrawlhallaReplays/**",
    "Contacts/**",
    "Cookies/**",
    "Desktop_SHORTCUT",
    "Favorites/**",
    "Google Drive/**",
    "github-classroom/**",
    "IntelGraphicsProfiles/**",
    "LabelObj.txt",
    "labelImg/**",
    "Links/**",
    "Local Settings/**",
    "miktex-console.lock",
    "Music/**",
    "My Documents/**",
    "NetHood/**",
    "NTUSER.DAT*",
    "ntuser.dat.LOG*",
    "NTUSER.DAT{*}.TM*",
    "ntuser.ini",
    "OneDrive/**",
    "Pictures/**",
    "py",
    "practApp/**",
    "PrintHood/**",
    "Recent/**",
    "Saved Games/**",
    "Searches/**",
    "SendTo/**",
    "source/**",
    "Start Menu/**",
    "Templates/**",
    "_viminfo",
    "vimfiles/**",
    "**/__pycache__",
    "**/__pycache__/**",
    "**/.venv",
    "**/.venv/**",
    "**/venv",
    "**/venv/**",
    "**/env",
    "**/env/**",
    "**/myenv",
    "**/myenv/**",
    "**/site-packages",
    "**/site-packages/**",
    "**/.pytest_cache",
    "**/.pytest_cache/**",
    "**/.mypy_cache",
    "**/.mypy_cache/**",
    "**/large_protein_dataset",
    "**/large_protein_dataset/**",
    "**/dist",
    "**/dist/**",
    "**/build",
    "**/build/**"
)

$denyGlobsText = ($denyGlobs | ForEach-Object { '  "{0}"' -f (Convert-ToPosixPath $_) }) -join ",`n"

$configText = @"
[workspace]
repo_root = "$($UserRoot.Replace("\", "/"))"
mount_point = "/mnt/ai"

[workspace.scheduler]
max_watch_targets = 0

[[workspace.domains]]
id = "home_full"
root = "$($UserRoot.Replace("\", "/"))"
trust_label = "untrusted"
watch_enabled = false
watch_priority = 1
max_indexed_files = $MaxIndexedFiles
allow_hidden_paths = false
allow_roots = ["**"]
deny_globs = [
$denyGlobsText
]

[filter]
mode = "repo_first"
allow_roots = ["**"]
deny_globs = [
  "**/.git/**", "**/node_modules/**", "**/target/**", "**/.cache/**",
  "**/*.db", "**/*.db-*", "**/*.sqlite", "**/*.sqlite-*",
  "**/*.jpg", "**/*.png", "**/*.mp4", "**/*.zip", "**/*.exe",
  "**/*.pyc", "**/*.pyo", "**/*.pdb"
]
max_file_mb = $MaxFileMb

[index]
debounce_ms = 500
publish_mode = "two_phase"
chunk_max_lines = 120
chunk_overlap_lines = 20
bulk_event_threshold = 80
hotset_max_paths = 16
pending_path_report_limit = 20

[embedding]
model = "bge-small-en-v1.5"
runtime = "hash"
quantization = "int8"
dimension = 384
batch_size = 64

[retrieval]
rrf_mode = "plain"
rrf_k = 60
topn_symbol = 5
topn_bm25 = 10
topn_vector = 8
topn_final = 5
symbol_exact_boost = 2.0
symbol_prefix_boost = 1.2
allow_stale = false
code_path_boost = 1.10
docs_path_penalty = 0.90
test_path_penalty = 0.95
asset_path_penalty = 0.45
recency_half_life_hours = 24.0
recency_min_boost = 0.85
recency_max_boost = 1.20

[fuse_cache]
max_virtual_inodes = 5000
max_cached_mb = 64
entry_ttl_ms = 300
attr_ttl_ms = 300

[fuse_session]
mode = "pinned"
max_entries = 128

[map]
base_summary_mode = "deterministic_precompute"
llm_enrichment = "async_optional"
cache_ttl_sec = 3600

[policy]
read_only = true
deny_secret_paths = true
search_result_redaction = true
trust_labels = ["trusted", "untrusted"]

[observability]
metrics_bind = "127.0.0.1:9494"
health_bind = "127.0.0.1:9495"
log_level = "info"

[mcp]
enabled = true
mode = "minimal"
"@

$fixtureObject = [pscustomobject]@{
    schema_version = 1
    dataset_name = "home_full_v1"
    queries = $queries
}

$manifestObject = [pscustomobject]@{
    user_root = $UserRoot
    max_indexed_files = $MaxIndexedFiles
    sample_query_limit = $MaxFixtureQueries
    excluded_top_level = $excludedTopLevel
    selected_paths = $selectedPaths
}

$configPath = Join-Path $OutputDir "home_full.toml"
$fixturePath = Join-Path $OutputDir "home_full_fixture.json"
$manifestPath = Join-Path $OutputDir "home_full_manifest.json"

Set-Content -Path $configPath -Value $configText -NoNewline
$fixtureObject | ConvertTo-Json -Depth 6 | Set-Content -Path $fixturePath
$manifestObject | ConvertTo-Json -Depth 6 | Set-Content -Path $manifestPath

[pscustomobject]@{
    user_root = $UserRoot
    config_path = $configPath
    fixture_path = $fixturePath
    manifest_path = $manifestPath
    query_count = $queries.Count
    selected_count = $selectedPaths.Count
    max_indexed_files = $MaxIndexedFiles
}
