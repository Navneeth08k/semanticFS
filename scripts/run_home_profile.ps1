param(
    [string]$UserRoot = $env:USERPROFILE,
    [string]$OutputDir = ".semanticfs/bench",
    [string]$DbPath = ".semanticfs/bench/home_profile.db",
    [string]$ConfigPath = "",
    [string]$FixturePath = "",
    [switch]$SkipBuild,
    [switch]$IncludeHeadToHead
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$resolvedConfigPath = if ([string]::IsNullOrWhiteSpace($ConfigPath)) {
    Join-Path $OutputDir "home_profile.toml"
} else {
    $ConfigPath
}
$resolvedFixturePath = if ([string]::IsNullOrWhiteSpace($FixturePath)) {
    Join-Path $OutputDir "home_profile_fixture.json"
} else {
    $FixturePath
}

if (-not $SkipBuild) {
    $buildScript = Join-Path $PSScriptRoot "build_home_profile.ps1"
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

$env:SEMANTICFS_DB_PATH = $DbPath
try {
    & "target\release\semanticfs.exe" --config $resolvedConfigPath health
    & "target\release\semanticfs.exe" --config $resolvedConfigPath benchmark relevance --fixture-repo $UserRoot --golden $resolvedFixturePath
    if (Test-Path ".semanticfs\bench\relevance_latest.json") {
        Copy-Item ".semanticfs\bench\relevance_latest.json" ".semanticfs\bench\home_profile_relevance_latest.json" -Force
    }

    if ($IncludeHeadToHead) {
        & "target\release\semanticfs.exe" --config $resolvedConfigPath benchmark head-to-head --fixture-repo $UserRoot --golden $resolvedFixturePath --skip-reindex
        if (Test-Path ".semanticfs\bench\head_to_head_latest.json") {
            Copy-Item ".semanticfs\bench\head_to_head_latest.json" ".semanticfs\bench\home_profile_head_to_head_latest.json" -Force
        }
    }
}
finally {
    Remove-Item Env:SEMANTICFS_DB_PATH -ErrorAction SilentlyContinue
}
