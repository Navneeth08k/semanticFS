param(
    [ValidateSet("single-repo", "multi-root-dev-box", "home-projects")]
    [string]$Profile = "single-repo",
    [string]$OutputPath = "local.toml",
    [string]$RepoRoot = (Get-Location).Path,
    [string]$HomeRoot = $env:USERPROFILE,
    [string]$ProjectsRoot = (Join-Path $env:USERPROFILE "Desktop\NavneethThings\Projects"),
    [string]$MountPoint = "/mnt/ai"
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

function Convert-ToPosixPath {
    param([string]$PathText)
    return $PathText.Replace("\", "/")
}

$profileDir = Join-Path $PSScriptRoot "..\config\profiles"
$templateName = switch ($Profile) {
    "single-repo" { "single-repo.sample.toml" }
    "multi-root-dev-box" { "multi-root-dev-box.sample.toml" }
    "home-projects" { "home-projects.sample.toml" }
}
$templatePath = Join-Path $profileDir $templateName
if (-not (Test-Path $templatePath)) {
    throw "Profile template not found: $templatePath"
}

$content = Get-Content $templatePath -Raw
$replacements = @{
    "__REPO_ROOT__" = (Convert-ToPosixPath $RepoRoot)
    "__HOME_ROOT__" = (Convert-ToPosixPath $HomeRoot)
    "__PROJECTS_ROOT__" = (Convert-ToPosixPath $ProjectsRoot)
    "__MOUNT_POINT__" = $MountPoint
}
foreach ($key in $replacements.Keys) {
    $content = $content.Replace($key, $replacements[$key])
}

$parent = Split-Path -Parent $OutputPath
if (-not [string]::IsNullOrWhiteSpace($parent)) {
    New-Item -ItemType Directory -Force -Path $parent | Out-Null
}
Set-Content -Path $OutputPath -Value $content -NoNewline

[pscustomobject]@{
    profile = $Profile
    output_path = $OutputPath
    repo_root = $RepoRoot
    home_root = $HomeRoot
    projects_root = $ProjectsRoot
    mount_point = $MountPoint
}
