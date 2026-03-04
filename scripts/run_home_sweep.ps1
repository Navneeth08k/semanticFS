param(
    [string]$UserRoot = $env:USERPROFILE,
    [string]$OutputDir = ".semanticfs/bench",
    [string]$DbPath = ".semanticfs/bench/home_sweep.db",
    [string]$ConfigPath = "",
    [string]$FixturePath = "",
    [string]$BaselineRoot = "",
    [switch]$SkipBuild,
    [switch]$SkipRelevance,
    [switch]$SkipHeadToHead
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$resolvedConfigPath = if ([string]::IsNullOrWhiteSpace($ConfigPath)) {
    Join-Path $OutputDir "home_sweep.toml"
} else {
    $ConfigPath
}
$resolvedFixturePath = if ([string]::IsNullOrWhiteSpace($FixturePath)) {
    Join-Path $OutputDir "home_sweep_fixture.json"
} else {
    $FixturePath
}

if (-not $SkipBuild) {
    $buildScript = Join-Path $PSScriptRoot "build_home_sweep.ps1"
    $buildResult = & $buildScript -UserRoot $UserRoot -OutputDir $OutputDir
    $resolvedConfigPath = $buildResult.config_path
    $resolvedFixturePath = $buildResult.fixture_path
}

if (-not (Test-Path $resolvedConfigPath)) {
    throw "Config file not found: $resolvedConfigPath"
}
if (-not (Test-Path $resolvedFixturePath)) {
    throw "Fixture file not found: $resolvedFixturePath"
}

$effectiveBaselineRoot = if ([string]::IsNullOrWhiteSpace($BaselineRoot)) {
    $UserRoot
} else {
    $BaselineRoot
}

$env:SEMANTICFS_DB_PATH = $DbPath
try {
    & "target\release\semanticfs.exe" --config $resolvedConfigPath health
    if ($SkipRelevance -and -not $SkipHeadToHead) {
        & "target\release\semanticfs.exe" --config $resolvedConfigPath index build
    }
    if (-not $SkipRelevance) {
        & "target\release\semanticfs.exe" --config $resolvedConfigPath benchmark relevance --fixture-repo $UserRoot --golden $resolvedFixturePath
        if (Test-Path ".semanticfs\bench\relevance_latest.json") {
            Copy-Item ".semanticfs\bench\relevance_latest.json" ".semanticfs\bench\home_sweep_relevance_latest.json" -Force
        }
    }

    if (-not $SkipHeadToHead) {
        & "target\release\semanticfs.exe" --config $resolvedConfigPath benchmark head-to-head --fixture-repo $effectiveBaselineRoot --golden $resolvedFixturePath --skip-reindex
        if (Test-Path ".semanticfs\bench\head_to_head_latest.json") {
            Copy-Item ".semanticfs\bench\head_to_head_latest.json" ".semanticfs\bench\home_sweep_head_to_head_latest.json" -Force
        }
    }
}
finally {
    Remove-Item Env:SEMANTICFS_DB_PATH -ErrorAction SilentlyContinue
}
