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

function Test-BlockedSegment {
    param(
        [string]$RelativePath,
        [string[]]$BlockedSegments
    )

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

function Get-MatchingRelativePaths {
    param(
        [string]$FullRoot,
        [string[]]$Patterns,
        [bool]$Recurse,
        [int]$MaxResults,
        [string[]]$BlockedSegments,
        [int]$MaxFileMb
    )

    if (-not (Test-Path $FullRoot)) {
        return @()
    }

    $items = if ($Recurse) {
        Get-ChildItem -LiteralPath $FullRoot -Recurse -File -ErrorAction SilentlyContinue
    } else {
        Get-ChildItem -LiteralPath $FullRoot -File -ErrorAction SilentlyContinue
    }

    $matches = New-Object System.Collections.Generic.List[string]
    foreach ($item in $items) {
        $relative = Get-RelativePath -RootPath $FullRoot -FullPath $item.FullName
        if (Test-BlockedSegment -RelativePath $relative -BlockedSegments $BlockedSegments) {
            continue
        }
        if (-not (Test-TextLikeFile -Item $item -MaxFileMb $MaxFileMb)) {
            continue
        }

        foreach ($pattern in $Patterns) {
            if (Test-HomePattern -RelativePath $relative -Pattern $pattern) {
                $matches.Add($relative) | Out-Null
                break
            }
        }

        if ($MaxResults -gt 0 -and $matches.Count -ge $MaxResults) {
            break
        }
    }

    return $matches | Sort-Object -Unique
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

function Get-ExplicitSamplePaths {
    param(
        [string]$FullRoot,
        [string[]]$RelativePaths,
        [string[]]$BlockedSegments,
        [int]$MaxFileMb
    )

    $selected = New-Object System.Collections.Generic.List[string]
    foreach ($relativePath in $RelativePaths) {
        $normalized = Convert-ToPosixPath $relativePath
        if (Test-BlockedSegment -RelativePath $normalized -BlockedSegments $BlockedSegments) {
            continue
        }

        $fullPath = Join-Path $FullRoot ($normalized.Replace("/", "\"))
        if (-not (Test-Path -LiteralPath $fullPath -PathType Leaf)) {
            continue
        }

        $item = Get-Item -LiteralPath $fullPath -ErrorAction SilentlyContinue
        if ($null -eq $item) {
            continue
        }

        if (-not (Test-TextLikeFile -Item $item -MaxFileMb $MaxFileMb)) {
            continue
        }

        $selected.Add($normalized) | Out-Null
    }

    return $selected
}

New-Item -ItemType Directory -Path $OutputDir -Force | Out-Null

$blockedSegments = @(
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

$heavyDenyGlobs = @(
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
    "**/build/**",
    "**/*.pyc",
    "**/*.pyo",
    "**/*.pdb"
)

$domainSpecs = @(
    [pscustomobject]@{
        id = "home_catchall"
        root = ""
        allow_hidden_paths = $false
        recurse = $false
        max_indexed_files = 0
        sample_limit = 4
        patterns = @("*.txt", "*.py", "*.sh", "*.toml", "*.json", "*.md")
        sample_paths = @(
            "import pygame.py",
            "install.sh",
            "requirements.txt"
        )
        deny_globs = @()
    },
    [pscustomobject]@{
        id = "home_windsurf_tooling"
        root = ".windsurf"
        allow_hidden_paths = $true
        recurse = $false
        max_indexed_files = 0
        sample_limit = 2
        patterns = @("argv.json", "extensions/extensions.json")
        sample_paths = @(
            "argv.json",
            "extensions/extensions.json"
        )
        deny_globs = @()
    },
    [pscustomobject]@{
        id = "home_desktop_shallow"
        root = "Desktop"
        allow_hidden_paths = $false
        recurse = $false
        max_indexed_files = 0
        sample_limit = 2
        patterns = @("*.url", "*.txt", "*.md")
        sample_paths = @(
            "Fall Guys.url"
        )
        deny_globs = @()
    },
    [pscustomobject]@{
        id = "home_downloads_shallow"
        root = "Downloads"
        allow_hidden_paths = $false
        recurse = $false
        max_indexed_files = 0
        sample_limit = 2
        patterns = @("*.txt", "*.md", "*.json")
        sample_paths = @(
            "math_exp_00_input.txt",
            "measurements.txt"
        )
        deny_globs = @()
    },
    [pscustomobject]@{
        id = "home_documents_shallow"
        root = "Documents"
        allow_hidden_paths = $false
        recurse = $false
        max_indexed_files = 0
        sample_limit = 3
        patterns = @("*.csv", "*.ini", "*.txt", "*.md", "*.json")
        sample_paths = @(
            "amyloid.csv",
            "ClownfishVoiceChanger.ini",
            "SoundboardVoiceChanger.ini"
        )
        deny_globs = @()
    },
    [pscustomobject]@{
        id = "home_vscode_tooling"
        root = ".vscode"
        allow_hidden_paths = $true
        recurse = $false
        max_indexed_files = 0
        sample_limit = 2
        patterns = @("argv.json", "extensions/extensions.json")
        sample_paths = @(
            "argv.json",
            "extensions/extensions.json"
        )
        deny_globs = @()
    },
    [pscustomobject]@{
        id = "home_cursor_tooling"
        root = ".cursor"
        allow_hidden_paths = $true
        recurse = $false
        max_indexed_files = 0
        sample_limit = 2
        patterns = @("argv.json", "extensions/extensions.json")
        sample_paths = @(
            "argv.json",
            "extensions/extensions.json"
        )
        deny_globs = @()
    },
    [pscustomobject]@{
        id = "home_lmstudio_tooling"
        root = ".lmstudio"
        allow_hidden_paths = $true
        recurse = $false
        max_indexed_files = 0
        sample_limit = 2
        patterns = @("mcp.json", ".internal/backend-preferences-v1.json")
        sample_paths = @(
            "mcp.json",
            ".internal/backend-preferences-v1.json"
        )
        deny_globs = @()
    },
    [pscustomobject]@{
        id = "home_codex"
        root = ".codex"
        allow_hidden_paths = $true
        recurse = $true
        max_indexed_files = 0
        sample_limit = 4
        patterns = @("config.toml", "rules/*.rules", "skills/**/*.md")
        sample_paths = @(
            "config.toml",
            "rules/default.rules",
            "skills/.system/skill-creator/SKILL.md",
            "skills/.system/skill-installer/SKILL.md"
        )
        deny_globs = @()
    },
    [pscustomobject]@{
        id = "home_projects"
        root = "Desktop/NavneethThings/Projects"
        allow_hidden_paths = $false
        recurse = $true
        max_indexed_files = 128
        sample_limit = 4
        patterns = @("BuckitMobile/**", "BuckitReactNative/**", "Catapult Project/**", "Euler-r9/**", "semanticFS/**")
        sample_paths = @(
            "BuckitMobile/package-lock.json",
            "BuckitMobile/app.json",
            "BuckitReactNative/package-lock.json",
            "Catapult Project/catapult_project/components/sidebar-provider.tsx"
        )
        deny_globs = $heavyDenyGlobs
    },
    [pscustomobject]@{
        id = "home_school"
        root = "Desktop/NavneethThings/Navneeth School"
        allow_hidden_paths = $false
        recurse = $true
        max_indexed_files = 64
        sample_limit = 4
        patterns = @("Comp sci 2020-2021/**")
        sample_paths = @(
            "Comp sci 2020-2021/Python/Average.py",
            "Comp sci 2020-2021/Python/BubbleSort.py",
            "Comp sci 2020-2021/Python/Balloon.py",
            "Comp sci 2020-2021/HTML/FInalProject/topic.txt"
        )
        deny_globs = $heavyDenyGlobs
    },
    [pscustomobject]@{
        id = "home_robot"
        root = "Desktop/NavneethThings/Projects/Robot"
        allow_hidden_paths = $false
        recurse = $true
        max_indexed_files = 16
        sample_limit = 4
        patterns = @(
            "newModelCreate/classifai-blogs/README.md",
            "newModelCreate/classifai-blogs/0_Complete_Guide_To_Custom_Object_Detection_Model_With_Yolov5/ModelTraining/README.md",
            "TFODCourse/Tensorflow/models/README.md",
            "TFODCourse/Tensorflow/models/official/core/train_utils.py"
        )
        sample_paths = @(
            "newModelCreate/classifai-blogs/README.md",
            "newModelCreate/classifai-blogs/0_Complete_Guide_To_Custom_Object_Detection_Model_With_Yolov5/ModelTraining/README.md",
            "TFODCourse/Tensorflow/models/README.md",
            "TFODCourse/Tensorflow/models/official/core/train_utils.py"
        )
        deny_globs = $heavyDenyGlobs
    }
)

$queryOverrides = @{
    "home_catchall|import pygame.py" = "Basketball Shot Practice"
    "home_catchall|install.sh" = "Cursor Agent Installer"
    "home_catchall|requirements.txt" = "absl-py==0.12.0"
    "home_windsurf_tooling|argv.json" = "963323f3-786c-4ab6-b003-8b613b3a6ad5"
    "home_windsurf_tooling|extensions/extensions.json" = "Codeium.windsurfpyright"
    "home_desktop_shallow|Fall Guys.url" = "Epic Games\\FallGuys\\RunFallGuys.exe"
    "home_downloads_shallow|math_exp_00_input.txt" = "2^3+1"
    "home_documents_shallow|amyloid.csv" = '"Protein","Fibril Origins"'
    "home_documents_shallow|ClownfishVoiceChanger.ini" = "VOICE_EFFECT=17"
    "home_documents_shallow|SoundboardVoiceChanger.ini" = "LOCAL_PLAY=0"
    "home_vscode_tooling|argv.json" = "disable-color-correct-rendering"
    "home_vscode_tooling|extensions/extensions.json" = "cschlosser.doxdocgen"
    "home_cursor_tooling|argv.json" = "58e9b8de-d546-49ae-b492-a744924843f6"
    "home_cursor_tooling|extensions/extensions.json" = "anysphere.cursorpyright"
    "home_lmstudio_tooling|mcp.json" = "mcpServers"
    "home_lmstudio_tooling|.internal/backend-preferences-v1.json" = "llama.cpp-win-x86_64-nvidia-cuda-avx2"
    "home_codex|config.toml" = 'model_reasoning_effort = "xhigh"'
    "home_codex|rules/default.rules" = 'Get-Content .semanticfs\bench\head_to_head_latest.json'
    "home_projects|BuckitMobile/package-lock.json" = '"name": "BuckitMobile"'
    "home_projects|BuckitMobile/app.json" = '"slug": "BuckitMobile"'
    "home_projects|BuckitReactNative/package-lock.json" = '"name": "BuckitReactNative"'
    "home_projects|Catapult Project/catapult_project/components/sidebar-provider.tsx" = "SidebarProvider"
    "home_projects|semanticFS/README.md" = "# SemanticFS"
    "home_school|Comp sci 2020-2021/Python/BubbleSort.py" = "My List Sorted with the Bubble Sort"
    "home_robot|newModelCreate/classifai-blogs/README.md" = "Source code centered around classifai blog posts"
    "home_robot|newModelCreate/classifai-blogs/0_Complete_Guide_To_Custom_Object_Detection_Model_With_Yolov5/ModelTraining/README.md" = "train your own custom dataset with YOLOv5"
    "home_robot|TFODCourse/Tensorflow/models/README.md" = "Welcome to the Model Garden for TensorFlow"
    "home_robot|TFODCourse/Tensorflow/models/official/core/train_utils.py" = "convert_variables_to_constants_v2_as_graph"
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

    $selected = @(
        if ($spec.sample_paths.Count -gt 0) {
            Get-ExplicitSamplePaths -FullRoot $fullRoot -RelativePaths $spec.sample_paths -BlockedSegments $blockedSegments -MaxFileMb 5
        } else {
            Get-MatchingRelativePaths -FullRoot $fullRoot -Patterns $spec.patterns -Recurse $spec.recurse -MaxResults $spec.sample_limit -BlockedSegments $blockedSegments -MaxFileMb 5
        }
    )
    if ($selected.Count -eq 0) {
        continue
    }

    $allowRoots = ($spec.patterns | ForEach-Object { '  "{0}"' -f (Convert-ToPosixPath $_) }) -join ",`n"
    $denyGlobsText = if ($spec.deny_globs.Count -gt 0) {
        ($spec.deny_globs | ForEach-Object { '  "{0}"' -f (Convert-ToPosixPath $_) }) -join ",`n"
    } else {
        ""
    }
    $denyBlock = if ($spec.deny_globs.Count -gt 0) {
@"
deny_globs = [
$denyGlobsText
]
"@
    } else {
        "deny_globs = []"
    }

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
$denyBlock

"@) | Out-Null

    foreach ($relativePath in $selected) {
        $fullPath = Join-Path $fullRoot ($relativePath.Replace("/", "\"))
        $overrideKey = "{0}|{1}" -f $spec.id, $relativePath
        $queryText = if ($queryOverrides.ContainsKey($overrideKey)) {
            $queryOverrides[$overrideKey]
        } else {
            Resolve-DefaultQuery -FullPath $fullPath
        }

        $queryName = "hp{0:d2}" -f $queryId
        $queryId += 1
        $queries.Add((New-Query -Id $queryName -Query $queryText -ExpectedPaths @("{0}/{1}" -f $spec.id, $relativePath))) | Out-Null
    }

    $manifest.Add([pscustomobject]@{
        id = $spec.id
        root = Convert-ToPosixPath $fullRoot
        max_indexed_files = $spec.max_indexed_files
        allow_hidden_paths = $spec.allow_hidden_paths
        allow_roots = @($spec.patterns | ForEach-Object { Convert-ToPosixPath $_ })
        selected_paths = $selected
    }) | Out-Null
}

if ($queries.Count -eq 0) {
    throw "No supported files were discovered for the hybrid home profile under $UserRoot."
}

$configText = @"
[workspace]
repo_root = "$($UserRoot.Replace("\", "/"))"
mount_point = "/mnt/ai"

[workspace.scheduler]
max_watch_targets = 0
max_scan_targets = 32

$(($domainBlocks -join ""))
[filter]
mode = "repo_first"
allow_roots = ["**"]
deny_globs = [
  "**/.git/**", "**/node_modules/**", "**/target/**", "**/.cache/**",
  "**/*.db", "**/*.db-*", "**/*.sqlite", "**/*.sqlite-*",
  "**/*.jpg", "**/*.png", "**/*.mp4", "**/*.zip", "**/*.exe",
  "**/*.pyc", "**/*.pyo", "**/*.pdb"
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
metrics_bind = "127.0.0.1:9474"
health_bind = "127.0.0.1:9475"
log_level = "info"

[mcp]
enabled = true
mode = "minimal"
"@

$fixtureObject = [pscustomobject]@{
    schema_version = 1
    dataset_name = "home_profile_v1"
    queries = $queries
}

$manifestObject = [pscustomobject]@{
    user_root = $UserRoot
    domains = $manifest
}

$configPath = Join-Path $OutputDir "home_profile.toml"
$fixturePath = Join-Path $OutputDir "home_profile_fixture.json"
$manifestPath = Join-Path $OutputDir "home_profile_manifest.json"

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
