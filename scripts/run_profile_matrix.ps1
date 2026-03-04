param(
    [string]$OutputDir = ".semanticfs/bench",
    [string]$UserRoot = $env:USERPROFILE,
    [switch]$SkipBuild
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

function Get-HealthValue {
    param(
        [string[]]$Lines,
        [string]$Key
    )

    $prefix = "$Key="
    foreach ($line in $Lines) {
        if ($line.StartsWith($prefix)) {
            return $line.Substring($prefix.Length)
        }
    }
    return $null
}

New-Item -ItemType Directory -Force -Path $OutputDir | Out-Null
if (-not $SkipBuild) {
    & cargo build --release -p semanticfs-cli | Out-Host
}

& powershell -ExecutionPolicy Bypass -File scripts\build_home_pilot.ps1 -OutputDir $OutputDir | Out-Null
& powershell -ExecutionPolicy Bypass -File scripts\build_home_sweep.ps1 -OutputDir $OutputDir | Out-Null
& powershell -ExecutionPolicy Bypass -File scripts\build_home_profile.ps1 -OutputDir $OutputDir | Out-Null

$profiles = @(
    [pscustomobject]@{
        name = "repo_multiroot"
        config = "config/relevance-multiroot.toml"
        fixture = "tests/retrieval_golden/semanticfs_multiroot_explicit_v14.json"
        fixture_repo = (Get-Location).Path
        db = (Join-Path $OutputDir "profile_matrix_repo_multiroot.db")
    },
    [pscustomobject]@{
        name = "home_pilot"
        config = (Join-Path $OutputDir "home_pilot.toml")
        fixture = (Join-Path $OutputDir "home_pilot_fixture.json")
        fixture_repo = $UserRoot
        db = (Join-Path $OutputDir "profile_matrix_home_pilot.db")
    },
    [pscustomobject]@{
        name = "home_sweep"
        config = (Join-Path $OutputDir "home_sweep.toml")
        fixture = (Join-Path $OutputDir "home_sweep_fixture.json")
        fixture_repo = $UserRoot
        db = (Join-Path $OutputDir "profile_matrix_home_sweep.db")
    },
    [pscustomobject]@{
        name = "home_profile"
        config = (Join-Path $OutputDir "home_profile.toml")
        fixture = (Join-Path $OutputDir "home_profile_fixture.json")
        fixture_repo = $UserRoot
        db = (Join-Path $OutputDir "profile_matrix_home_profile.db")
    }
)

$summary = New-Object System.Collections.Generic.List[object]
foreach ($profile in $profiles) {
    Set-Item Env:SEMANTICFS_DB_PATH $profile.db
    try {
        $healthLines = & target\release\semanticfs.exe --config $profile.config health
        & target\release\semanticfs.exe --config $profile.config benchmark relevance --fixture-repo $profile.fixture_repo --golden $profile.fixture | Out-Host
        $report = Get-Content .semanticfs\bench\relevance_latest.json -Raw | ConvertFrom-Json
        $copied = Join-Path $OutputDir ("{0}_relevance_latest.json" -f $profile.name)
        Copy-Item .semanticfs\bench\relevance_latest.json $copied -Force
        $summary.Add([pscustomobject]@{
            name = $profile.name
            config = $profile.config
            fixture = $profile.fixture
            fixture_repo = $profile.fixture_repo
            db = $profile.db
            workspace_domain_count = [int](Get-HealthValue -Lines $healthLines -Key "workspace_domain_count")
            workspace_scan_target_raw_count = [int](Get-HealthValue -Lines $healthLines -Key "workspace_scan_target_raw_count")
            workspace_scan_target_count = [int](Get-HealthValue -Lines $healthLines -Key "workspace_scan_target_count")
            workspace_scan_target_pruned_count = [int](Get-HealthValue -Lines $healthLines -Key "workspace_scan_target_pruned_count")
            workspace_scan_target_limit = [int](Get-HealthValue -Lines $healthLines -Key "workspace_scan_target_limit")
            metrics = $report.metrics
            query_count = $report.query_count
            active_version = $report.active_version
            report_path = $copied
        }) | Out-Null
    }
    finally {
        Remove-Item Env:SEMANTICFS_DB_PATH -ErrorAction SilentlyContinue
    }
}

$outPath = Join-Path $OutputDir "profile_matrix_latest.json"
[pscustomobject]@{
    generated_at = (Get-Date).ToString("o")
    profiles = $summary
} | ConvertTo-Json -Depth 6 | Set-Content $outPath

Get-Item $outPath
