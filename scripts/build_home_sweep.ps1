param(
    [string]$UserRoot = $env:USERPROFILE,
    [string]$OutputDir = ".semanticfs/bench"
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

function Get-RelativePath {
    param(
        [string]$RootPath,
        [string]$FullPath
    )

    $root = $RootPath.TrimEnd('\', '/')
    if ($FullPath.Length -le $root.Length) {
        return ""
    }

    $relative = $FullPath.Substring($root.Length).TrimStart('\', '/')
    return Convert-ToPosixPath $relative
}

function Test-HomePattern {
    param(
        [string]$RelativePath,
        [string]$Pattern
    )

    $path = Convert-ToPosixPath $RelativePath
    $normalizedPattern = Convert-ToPosixPath $Pattern

    if (-not $normalizedPattern.Contains("*")) {
        return $path -ieq $normalizedPattern
    }

    if ($normalizedPattern.StartsWith("**/")) {
        $suffix = $normalizedPattern.Substring(3)
        if ($suffix.StartsWith("*.")) {
            return $path.EndsWith($suffix.Substring(1), [System.StringComparison]::OrdinalIgnoreCase)
        }
        $wildcard = New-Object System.Management.Automation.WildcardPattern($suffix, ([System.Management.Automation.WildcardOptions]::IgnoreCase))
        $segments = $path.Split('/')
        foreach ($idx in 0..($segments.Length - 1)) {
            $candidate = ($segments[$idx..($segments.Length - 1)] -join "/")
            if ($wildcard.IsMatch($candidate)) {
                return $true
            }
        }
        return $false
    }

    if ($normalizedPattern.Contains("/**/")) {
        $parts = $normalizedPattern.Split(@("/**/"), 2, [System.StringSplitOptions]::None)
        $prefix = $parts[0].TrimEnd('/')
        $suffix = $parts[1]
        if (-not $path.StartsWith("$prefix/", [System.StringComparison]::OrdinalIgnoreCase)) {
            return $false
        }
        $remainder = $path.Substring($prefix.Length + 1)
        if ($suffix.StartsWith("*.")) {
            return $remainder.EndsWith($suffix.Substring(1), [System.StringComparison]::OrdinalIgnoreCase)
        }
        $wildcard = New-Object System.Management.Automation.WildcardPattern($suffix, ([System.Management.Automation.WildcardOptions]::IgnoreCase))
        return $wildcard.IsMatch($remainder)
    }

    if (-not $normalizedPattern.Contains("/")) {
        if ($path.Contains("/")) {
            return $false
        }
        $plainTopLevel = New-Object System.Management.Automation.WildcardPattern($normalizedPattern, ([System.Management.Automation.WildcardOptions]::IgnoreCase))
        return $plainTopLevel.IsMatch($path)
    }

    $plain = New-Object System.Management.Automation.WildcardPattern($normalizedPattern, ([System.Management.Automation.WildcardOptions]::IgnoreCase))
    return $plain.IsMatch($path)
}

function Get-MatchingRelativePaths {
    param(
        [string]$FullRoot,
        [string[]]$Patterns,
        [bool]$Recurse
    )

    if (-not (Test-Path $FullRoot)) {
        return @()
    }

    $items = if ($Recurse) {
        Get-ChildItem -LiteralPath $FullRoot -Recurse -File
    } else {
        Get-ChildItem -LiteralPath $FullRoot -File
    }

    $matches = New-Object System.Collections.Generic.List[string]
    foreach ($item in $items) {
        $relative = Get-RelativePath -RootPath $FullRoot -FullPath $item.FullName
        foreach ($pattern in $Patterns) {
            if (Test-HomePattern -RelativePath $relative -Pattern $pattern) {
                $matches.Add($relative)
                break
            }
        }
    }

    return $matches |
        Sort-Object -Unique
}

function Resolve-DefaultQuery {
    param([string]$FullPath)

    $lines = Get-Content -LiteralPath $FullPath -ErrorAction SilentlyContinue | Select-Object -First 25
    foreach ($line in $lines) {
        $trimmed = $line.Trim()
        if ([string]::IsNullOrWhiteSpace($trimmed)) {
            continue
        }
        if ($trimmed.Length -lt 5) {
            continue
        }
        if ($trimmed -match '^[\{\}\[\];,#/ ]+$') {
            continue
        }
        return $trimmed
    }

    return (Split-Path -Leaf $FullPath)
}

New-Item -ItemType Directory -Path $OutputDir -Force | Out-Null

$domainSpecs = @(
    [pscustomobject]@{
        id = "sweep_codex"
        root = ".codex"
        allow_hidden_paths = $true
        recurse = $true
        max_indexed_files = 5
        patterns = @("config.toml", "rules/*.rules", "skills/**/*.md")
    },
    [pscustomobject]@{
        id = "sweep_lmstudio"
        root = ".lmstudio"
        allow_hidden_paths = $true
        recurse = $true
        max_indexed_files = 4
        patterns = @("mcp.json", ".internal/*.json")
    },
    [pscustomobject]@{
        id = "sweep_cursor"
        root = ".cursor"
        allow_hidden_paths = $true
        recurse = $true
        max_indexed_files = 4
        patterns = @("*.json", "extensions/.obsolete", "extensions/extensions.json")
    },
    [pscustomobject]@{
        id = "sweep_vscode"
        root = ".vscode"
        allow_hidden_paths = $true
        recurse = $true
        max_indexed_files = 2
        patterns = @("*.json", "extensions/extensions.json")
    },
    [pscustomobject]@{
        id = "sweep_home_root"
        root = ""
        allow_hidden_paths = $false
        recurse = $false
        max_indexed_files = 0
        patterns = @("LabelObj.txt", "import pygame.py", "install.sh", "requirements.txt")
    },
    [pscustomobject]@{
        id = "sweep_desktop"
        root = "Desktop"
        allow_hidden_paths = $false
        recurse = $false
        max_indexed_files = 1
        patterns = @("*.url")
    },
    [pscustomobject]@{
        id = "sweep_downloads"
        root = "Downloads"
        allow_hidden_paths = $false
        recurse = $false
        max_indexed_files = 2
        patterns = @("math_exp*_input.txt", "math_exp*_expected.txt")
    }
)

$queryOverrides = @{
    "sweep_codex|config.toml" = 'model_reasoning_effort = "xhigh"'
    "sweep_codex|rules/default.rules" = 'Get-Content .semanticfs\bench\head_to_head_latest.json'
    "sweep_codex|skills/.system/skill-creator/SKILL.md" = 'This skill provides guidance for creating effective skills.'
    "sweep_codex|skills/.system/skill-creator/references/openai_yaml.md" = 'the skill is not injected into the model context by default'
    "sweep_codex|skills/.system/skill-installer/SKILL.md" = 'Helps install skills. By default these are from https://github.com/openai/skills/tree/main/skills/.curated'
    "sweep_lmstudio|.internal/app-install-location.json" = '--run-as-service'
    "sweep_lmstudio|.internal/artifact-permissions-list.json" = 'canMakeIndividualPrivate'
    "sweep_lmstudio|.internal/backend-preferences-v1.json" = 'llama.cpp-win-x86_64-nvidia-cuda-avx2'
    "sweep_lmstudio|.internal/conversation-config.json" = 'selectedPredictionProcessorIdentifier'
    "sweep_lmstudio|mcp.json" = '"mcpServers": {}'
    "sweep_cursor|argv.json" = '58e9b8de-d546-49ae-b492-a744924843f6'
    "sweep_cursor|extensions/.obsolete" = 'openai.chatgpt-0.4.76-universal'
    "sweep_cursor|extensions/extensions.json" = 'anysphere.pyright'
    "sweep_cursor|ide_state.json" = 'recentlyViewedFiles'
    "sweep_vscode|argv.json" = 'disable-color-correct-rendering'
    "sweep_vscode|extensions/extensions.json" = 'cschlosser.doxdocgen'
    "sweep_home_root|LabelObj.txt" = 'NoMask'
    "sweep_home_root|import pygame.py" = 'pygame.K_SPACE'
    "sweep_home_root|install.sh" = 'Cursor Agent Installer'
    "sweep_home_root|requirements.txt" = 'absl-py==0.12.0'
    "sweep_desktop|Fall Guys.url" = 'Epic Games\FallGuys\RunFallGuys.exe'
    "sweep_downloads|math_exp_00_input.txt" = '2^3+1'
    "sweep_downloads|math_exp_big_val_2_expected.txt" = '854616735'
}

$queries = New-Object System.Collections.Generic.List[object]
$domainBlocks = New-Object System.Collections.Generic.List[string]
$manifest = New-Object System.Collections.Generic.List[object]
$queryId = 1

foreach ($spec in $domainSpecs) {
    $fullRoot = if ([string]::IsNullOrWhiteSpace($spec.root)) {
        $UserRoot
    } else {
        Join-Path $UserRoot $spec.root
    }

    $allMatches = @(Get-MatchingRelativePaths -FullRoot $fullRoot -Patterns $spec.patterns -Recurse $spec.recurse)
    if ($allMatches.Count -eq 0) {
        continue
    }

    $selected = if ($spec.max_indexed_files -gt 0) {
        @($allMatches | Select-Object -First $spec.max_indexed_files)
    } else {
        $allMatches
    }
    $excluded = @()
    if ($spec.max_indexed_files -gt 0 -and $allMatches.Count -gt $spec.max_indexed_files) {
        $excluded = @($allMatches | Select-Object -Skip $spec.max_indexed_files)
    }

    $allowRoots = ($spec.patterns | ForEach-Object { '  "{0}"' -f (Convert-ToPosixPath $_) }) -join ",`n"
    $domainBlocks.Add(@"
[[workspace.domains]]
id = "$($spec.id)"
root = "$(Convert-ToPosixPath $fullRoot)"
trust_label = "untrusted"
watch_enabled = false
watch_priority = 1
max_indexed_files = $($spec.max_indexed_files)
allow_hidden_paths = $(if ($spec.allow_hidden_paths) { "true" } else { "false" })
allow_roots = [
$allowRoots
]
deny_globs = []

"@) | Out-Null

    foreach ($relativePath in $selected) {
        $fullPath = Join-Path $fullRoot ($relativePath.Replace("/", "\"))
        $overrideKey = "{0}|{1}" -f $spec.id, $relativePath
        $queryText = if ($queryOverrides.ContainsKey($overrideKey)) {
            $queryOverrides[$overrideKey]
        } else {
            Resolve-DefaultQuery -FullPath $fullPath
        }

        $queryName = "hs{0:d2}" -f $queryId
        $queryId += 1
        $queries.Add((New-Query -Id $queryName -Query $queryText -ExpectedPaths @("{0}/{1}" -f $spec.id, $relativePath)))
    }

    $manifest.Add([pscustomobject]@{
        id = $spec.id
        root = Convert-ToPosixPath $fullRoot
        max_indexed_files = $spec.max_indexed_files
        allow_hidden_paths = $spec.allow_hidden_paths
        allow_roots = @($spec.patterns | ForEach-Object { Convert-ToPosixPath $_ })
        selected_paths = $selected
        excluded_paths = $excluded
    }) | Out-Null
}

if ($queries.Count -eq 0) {
    throw "No supported files were discovered for the budgeted home sweep under $UserRoot."
}

$configText = @"
[workspace]
repo_root = "$($UserRoot.Replace("\", "/"))"
mount_point = "/mnt/ai"

[workspace.scheduler]
max_watch_targets = 0

$(($domainBlocks -join ""))
[filter]
mode = "repo_first"
allow_roots = ["**"]
deny_globs = [
  "**/.git/**", "**/node_modules/**", "**/target/**", "**/.cache/**",
  "**/*.db", "**/*.db-*", "**/*.sqlite", "**/*.sqlite-*",
  "**/*.jpg", "**/*.png", "**/*.mp4", "**/*.zip", "**/*.exe"
]
max_file_mb = 5

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
metrics_bind = "127.0.0.1:9484"
health_bind = "127.0.0.1:9485"
log_level = "info"

[mcp]
enabled = true
mode = "minimal"
"@

$fixtureObject = [pscustomobject]@{
    schema_version = 1
    dataset_name = "home_sweep_v1"
    queries = $queries
}

$manifestObject = [pscustomobject]@{
    user_root = $UserRoot
    domains = $manifest
}

$configPath = Join-Path $OutputDir "home_sweep.toml"
$fixturePath = Join-Path $OutputDir "home_sweep_fixture.json"
$manifestPath = Join-Path $OutputDir "home_sweep_manifest.json"

Set-Content -Path $configPath -Value $configText -NoNewline
$fixtureObject | ConvertTo-Json -Depth 6 | Set-Content -Path $fixturePath
$manifestObject | ConvertTo-Json -Depth 8 | Set-Content -Path $manifestPath

[pscustomobject]@{
    user_root = $UserRoot
    config_path = $configPath
    fixture_path = $fixturePath
    manifest_path = $manifestPath
    domain_count = $manifest.Count
    query_count = $queries.Count
}
