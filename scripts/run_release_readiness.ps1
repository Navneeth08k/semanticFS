param(
    [string]$OutputDir = ".semanticfs/bench",
    [string]$ReleaseDir = ".semanticfs/releases",
    [string]$InstallDir = ".semanticfs/local-bin-release",
    [string]$SmokeDir = "",
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

if ([string]::IsNullOrWhiteSpace($SmokeDir)) {
    $SmokeDir = Join-Path $env:TEMP "semanticfs-release-smoke"
}

if (-not $SkipBuild) {
    & cargo build --release -p semanticfs-cli | Out-Host
}

& .\scripts\run_profile_matrix.ps1 -OutputDir $OutputDir -UserRoot $UserRoot -SkipBuild:$true | Out-Host

$singleRepoConfig = Join-Path $OutputDir "release.single-repo.toml"
$homeProjectsConfig = Join-Path $OutputDir "release.home-projects.toml"
$multiRootConfig = Join-Path $OutputDir "release.multi-root-dev-box.toml"
$projectsRoot = Join-Path $UserRoot "Desktop\NavneethThings\Projects"

& .\scripts\apply_config_profile.ps1 -Profile single-repo -OutputPath $singleRepoConfig -RepoRoot (Get-Location).Path | Out-Null
& .\scripts\apply_config_profile.ps1 -Profile home-projects -OutputPath $homeProjectsConfig -HomeRoot $UserRoot -ProjectsRoot $projectsRoot | Out-Null
& .\scripts\apply_config_profile.ps1 -Profile multi-root-dev-box -OutputPath $multiRootConfig -RepoRoot (Get-Location).Path | Out-Null

$bundle = & .\scripts\package_release.ps1 -OutputDir $ReleaseDir
$install = & .\scripts\install_local.ps1 -InstallDir $InstallDir -BinaryPath "target\release\semanticfs.exe"

$installedBinary = $install.installed_binary
$packagedBinary = $bundle.binary

if (Test-Path $SmokeDir) {
    Remove-Item $SmokeDir -Recurse -Force
}
New-Item -ItemType Directory -Force -Path $SmokeDir | Out-Null
Expand-Archive -Path $bundle.zip_path -DestinationPath $SmokeDir -Force
$smokeBinary = Join-Path $SmokeDir "semanticfs.exe"

$singleRepoHealth = & $installedBinary --config $singleRepoConfig health
$homeProjectsHealth = & $packagedBinary --config $homeProjectsConfig health
$multiRootHealth = & $packagedBinary --config $multiRootConfig health
$smokeHomeProjectsHealth = & $smokeBinary --config $homeProjectsConfig health

$matrix = Get-Content (Join-Path $OutputDir "profile_matrix_latest.json") -Raw | ConvertFrom-Json
$summary = [pscustomobject]@{
    generated_at = (Get-Date).ToString("o")
    profile_matrix = [pscustomobject]@{
        repo_multiroot = ($matrix.profiles | Where-Object { $_.name -eq "repo_multiroot" } | Select-Object -First 1)
        home_pilot = ($matrix.profiles | Where-Object { $_.name -eq "home_pilot" } | Select-Object -First 1)
        home_sweep = ($matrix.profiles | Where-Object { $_.name -eq "home_sweep" } | Select-Object -First 1)
        home_profile = ($matrix.profiles | Where-Object { $_.name -eq "home_profile" } | Select-Object -First 1)
    }
    rendered_profiles = @(
        $singleRepoConfig,
        $homeProjectsConfig,
        $multiRootConfig
    )
    install = $install
    bundle = $bundle
    health_checks = [pscustomobject]@{
        single_repo = [pscustomobject]@{
            binary = $installedBinary
            workspace_domain_count = [int](Get-HealthValue -Lines $singleRepoHealth -Key "workspace_domain_count")
            workspace_domain_plan_mode = (Get-HealthValue -Lines $singleRepoHealth -Key "workspace_domain_plan_mode")
        }
        home_projects = [pscustomobject]@{
            binary = $packagedBinary
            workspace_domain_count = [int](Get-HealthValue -Lines $homeProjectsHealth -Key "workspace_domain_count")
            workspace_domain_plan_mode = (Get-HealthValue -Lines $homeProjectsHealth -Key "workspace_domain_plan_mode")
        }
        multi_root_dev_box = [pscustomobject]@{
            binary = $packagedBinary
            workspace_domain_count = [int](Get-HealthValue -Lines $multiRootHealth -Key "workspace_domain_count")
            workspace_domain_plan_mode = (Get-HealthValue -Lines $multiRootHealth -Key "workspace_domain_plan_mode")
        }
        distinct_environment = [pscustomobject]@{
            binary = $smokeBinary
            workspace_domain_count = [int](Get-HealthValue -Lines $smokeHomeProjectsHealth -Key "workspace_domain_count")
            workspace_domain_plan_mode = (Get-HealthValue -Lines $smokeHomeProjectsHealth -Key "workspace_domain_plan_mode")
        }
    }
}

$outPath = Join-Path $OutputDir "release_readiness_latest.json"
$summary | ConvertTo-Json -Depth 8 | Set-Content $outPath
Get-Item $outPath
