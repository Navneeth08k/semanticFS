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

function Add-HomeFile {
    param(
        [System.Collections.Generic.List[object]]$Queries,
        [System.Collections.Generic.Dictionary[string, object]]$Domains,
        [string]$DomainId,
        [string]$Root,
        [string]$AllowRoot,
        [string]$QueryId,
        [string]$QueryText,
        [bool]$AllowHiddenPaths = $false
    )

    $fullRoot = if ([string]::IsNullOrWhiteSpace($Root)) {
        $UserRoot
    } else {
        Join-Path $UserRoot $Root
    }
    $fullPath = Join-Path $fullRoot $AllowRoot
    if (-not (Test-Path $fullPath)) {
        return $false
    }

    if (-not $Domains.ContainsKey($DomainId)) {
        $Domains[$DomainId] = [pscustomobject]@{
            id = $DomainId
            root = $fullRoot.Replace("\", "/")
            trust_label = "untrusted"
            watch_enabled = $false
            watch_priority = 1
            allow_hidden_paths = $AllowHiddenPaths
            allow_roots = New-Object System.Collections.Generic.List[string]
        }
    }

    $Domains[$DomainId].allow_roots.Add($AllowRoot.Replace("\", "/"))
    $Queries.Add((New-Query -Id $QueryId -Query $QueryText -ExpectedPaths @("{0}/{1}" -f $DomainId, $AllowRoot.Replace("\", "/"))))
    return $true
}

New-Item -ItemType Directory -Path $OutputDir -Force | Out-Null

$queries = New-Object System.Collections.Generic.List[object]
$domains = New-Object 'System.Collections.Generic.Dictionary[string, object]'

[void](Add-HomeFile -Queries $queries -Domains $domains -DomainId "home_meta" -Root ".codex" -AllowRoot "config.toml" -QueryId "h01" -QueryText 'model_reasoning_effort = "xhigh"' -AllowHiddenPaths $true)
[void](Add-HomeFile -Queries $queries -Domains $domains -DomainId "home_rules" -Root ".codex\rules" -AllowRoot "default.rules" -QueryId "h02" -QueryText 'Get-Content .semanticfs\bench\head_to_head_latest.json' -AllowHiddenPaths $true)
[void](Add-HomeFile -Queries $queries -Domains $domains -DomainId "home_skills" -Root ".codex\skills" -AllowRoot ".system\skill-installer\SKILL.md" -QueryId "h03" -QueryText 'Helps install skills. By default these are from https://github.com/openai/skills/tree/main/skills/.curated' -AllowHiddenPaths $true)
[void](Add-HomeFile -Queries $queries -Domains $domains -DomainId "home_skills" -Root ".codex\skills" -AllowRoot ".system\skill-creator\SKILL.md" -QueryId "h04" -QueryText 'This skill provides guidance for creating effective skills.' -AllowHiddenPaths $true)
[void](Add-HomeFile -Queries $queries -Domains $domains -DomainId "home_skills" -Root ".codex\skills" -AllowRoot ".system\skill-creator\references\openai_yaml.md" -QueryId "h05" -QueryText 'the skill is not injected into the model context by default' -AllowHiddenPaths $true)
[void](Add-HomeFile -Queries $queries -Domains $domains -DomainId "home_root_text" -Root "" -AllowRoot ".condarc" -QueryId "h06" -QueryText 'defaults' -AllowHiddenPaths $true)
[void](Add-HomeFile -Queries $queries -Domains $domains -DomainId "home_root_text" -Root "" -AllowRoot "requirements.txt" -QueryId "h07" -QueryText 'absl-py==0.12.0' -AllowHiddenPaths $true)
[void](Add-HomeFile -Queries $queries -Domains $domains -DomainId "home_root_text" -Root "" -AllowRoot "LabelObj.txt" -QueryId "h08" -QueryText 'NoMask' -AllowHiddenPaths $true)
[void](Add-HomeFile -Queries $queries -Domains $domains -DomainId "home_root_text" -Root "" -AllowRoot "install.sh" -QueryId "h09" -QueryText 'Cursor Agent Installer' -AllowHiddenPaths $true)
[void](Add-HomeFile -Queries $queries -Domains $domains -DomainId "home_root_text" -Root "" -AllowRoot "import pygame.py" -QueryId "h10" -QueryText 'pygame.K_SPACE' -AllowHiddenPaths $true)
[void](Add-HomeFile -Queries $queries -Domains $domains -DomainId "home_desktop" -Root "Desktop" -AllowRoot "Fall Guys.url" -QueryId "h12" -QueryText 'Epic Games\FallGuys\RunFallGuys.exe')
[void](Add-HomeFile -Queries $queries -Domains $domains -DomainId "home_lmstudio" -Root ".lmstudio" -AllowRoot "mcp.json" -QueryId "h13" -QueryText '"mcpServers": {}' -AllowHiddenPaths $true)
[void](Add-HomeFile -Queries $queries -Domains $domains -DomainId "home_lmstudio" -Root ".lmstudio" -AllowRoot ".internal\server-logs-state.json" -QueryId "h14" -QueryText 'lastWrittenFileSizeBytes' -AllowHiddenPaths $true)
[void](Add-HomeFile -Queries $queries -Domains $domains -DomainId "home_cursor" -Root ".cursor" -AllowRoot "extensions\.obsolete" -QueryId "h15" -QueryText 'openai.chatgpt-0.4.76-universal' -AllowHiddenPaths $true)
[void](Add-HomeFile -Queries $queries -Domains $domains -DomainId "home_vscode" -Root ".vscode" -AllowRoot "argv.json" -QueryId "h16" -QueryText '"disable-color-correct-rendering": true' -AllowHiddenPaths $true)
[void](Add-HomeFile -Queries $queries -Domains $domains -DomainId "home_downloads" -Root "Downloads" -AllowRoot "math_exp_00_input.txt" -QueryId "h17" -QueryText '2^3+1')
[void](Add-HomeFile -Queries $queries -Domains $domains -DomainId "home_downloads" -Root "Downloads" -AllowRoot "measurements.txt" -QueryId "h18" -QueryText 'Y multiplier: 0.00017')

if ($queries.Count -eq 0) {
    throw "No supported home-pilot files were found under $UserRoot."
}

$domainBlocks = foreach ($domain in $domains.Values) {
    $allowRoots = ($domain.allow_roots | ForEach-Object { '  "{0}"' -f $_ }) -join ",`n"
@"
[[workspace.domains]]
id = "$($domain.id)"
root = "$($domain.root)"
trust_label = "$($domain.trust_label)"
watch_enabled = false
watch_priority = $($domain.watch_priority)
allow_hidden_paths = $(if ($domain.allow_hidden_paths) { "true" } else { "false" })
allow_roots = [
$allowRoots
]
deny_globs = []

"@
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
metrics_bind = "127.0.0.1:9474"
health_bind = "127.0.0.1:9475"
log_level = "info"

[mcp]
enabled = true
mode = "minimal"
"@

$fixtureObject = [pscustomobject]@{
    schema_version = 1
    dataset_name = "home_pilot_v1"
    queries = $queries
}

$configPath = Join-Path $OutputDir "home_pilot.toml"
$fixturePath = Join-Path $OutputDir "home_pilot_fixture.json"

Set-Content -Path $configPath -Value $configText -NoNewline
$fixtureObject | ConvertTo-Json -Depth 6 | Set-Content -Path $fixturePath

[pscustomobject]@{
    user_root = $UserRoot
    config_path = $configPath
    fixture_path = $fixturePath
    domain_count = $domains.Count
    query_count = $queries.Count
}
