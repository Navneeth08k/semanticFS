param(
    [string]$UserRoot = $env:USERPROFILE,
    [string]$OutputDir = ".semanticfs/bench",
    [string]$DbPath = ".semanticfs/bench/home_pilot.db",
    [string]$BaselineRoot = "",
    [switch]$SkipHeadToHead
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$buildScript = Join-Path $PSScriptRoot "build_home_pilot.ps1"
$buildResult = & $buildScript -UserRoot $UserRoot -OutputDir $OutputDir

$configPath = $buildResult.config_path
$fixturePath = $buildResult.fixture_path
$effectiveBaselineRoot = if ([string]::IsNullOrWhiteSpace($BaselineRoot)) {
    $UserRoot
} else {
    $BaselineRoot
}

$env:SEMANTICFS_DB_PATH = $DbPath
try {
    & "target\release\semanticfs.exe" --config $configPath health
    & "target\release\semanticfs.exe" --config $configPath benchmark relevance --fixture-repo $UserRoot --golden $fixturePath
    if (Test-Path ".semanticfs\bench\relevance_latest.json") {
        Copy-Item ".semanticfs\bench\relevance_latest.json" ".semanticfs\bench\home_pilot_relevance_latest.json" -Force
    }

    if (-not $SkipHeadToHead) {
        & "target\release\semanticfs.exe" --config $configPath benchmark head-to-head --fixture-repo $effectiveBaselineRoot --golden $fixturePath --skip-reindex
        if (Test-Path ".semanticfs\bench\head_to_head_latest.json") {
            Copy-Item ".semanticfs\bench\head_to_head_latest.json" ".semanticfs\bench\home_pilot_head_to_head_latest.json" -Force
        }
    }
}
finally {
    Remove-Item Env:SEMANTICFS_DB_PATH -ErrorAction SilentlyContinue
}
