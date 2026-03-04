param(
    [string]$OutputDir = ".semanticfs/releases",
    [string]$BundleName = "semanticfs-windows-x64"
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

New-Item -ItemType Directory -Force -Path $OutputDir | Out-Null
& cargo build --release -p semanticfs-cli | Out-Host

$stageDir = Join-Path $OutputDir $BundleName
$zipPath = Join-Path $OutputDir ("{0}.zip" -f $BundleName)
if (Test-Path $stageDir) { Remove-Item $stageDir -Recurse -Force }
if (Test-Path $zipPath) { Remove-Item $zipPath -Force }

New-Item -ItemType Directory -Force -Path $stageDir | Out-Null
New-Item -ItemType Directory -Force -Path (Join-Path $stageDir "config\profiles") | Out-Null
New-Item -ItemType Directory -Force -Path (Join-Path $stageDir "docs") | Out-Null
New-Item -ItemType Directory -Force -Path (Join-Path $stageDir "scripts") | Out-Null

Copy-Item target\release\semanticfs.exe (Join-Path $stageDir "semanticfs.exe") -Force
Copy-Item LICENSE (Join-Path $stageDir "LICENSE") -Force
Copy-Item README.md (Join-Path $stageDir "README.md") -Force
Copy-Item docs\setup_10_minute_agents.md (Join-Path $stageDir "docs\setup_10_minute_agents.md") -Force
Copy-Item config\semanticfs.sample.toml (Join-Path $stageDir "config\semanticfs.sample.toml") -Force
Copy-Item config\profiles\*.sample.toml (Join-Path $stageDir "config\profiles") -Force
Copy-Item scripts\apply_config_profile.ps1 (Join-Path $stageDir "scripts\apply_config_profile.ps1") -Force
Copy-Item scripts\install_local.ps1 (Join-Path $stageDir "scripts\install_local.ps1") -Force

Compress-Archive -Path (Join-Path $stageDir "*") -DestinationPath $zipPath

[pscustomobject]@{
    stage_dir = $stageDir
    zip_path = $zipPath
    binary = (Join-Path $stageDir "semanticfs.exe")
}
