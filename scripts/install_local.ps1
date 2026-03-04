param(
    [string]$InstallDir = (Join-Path $env:USERPROFILE "bin"),
    [string]$BinaryPath = "",
    [switch]$AddToUserPath
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$resolvedBinary = $BinaryPath
if ([string]::IsNullOrWhiteSpace($resolvedBinary)) {
    & cargo build --release -p semanticfs-cli | Out-Host
    $resolvedBinary = "target\release\semanticfs.exe"
}
if (-not (Test-Path $resolvedBinary)) {
    throw "Binary not found: $resolvedBinary"
}

New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
$dest = Join-Path $InstallDir "semanticfs.exe"
Copy-Item $resolvedBinary $dest -Force

if ($AddToUserPath) {
    $current = [Environment]::GetEnvironmentVariable("Path", "User")
    $parts = @()
    if (-not [string]::IsNullOrWhiteSpace($current)) {
        $parts = $current.Split(';') | Where-Object { -not [string]::IsNullOrWhiteSpace($_) }
    }
    if ($parts -notcontains $InstallDir) {
        $newPath = (($parts + $InstallDir) -join ';')
        [Environment]::SetEnvironmentVariable("Path", $newPath, "User")
    }
}

[pscustomobject]@{
    install_dir = $InstallDir
    installed_binary = $dest
    add_to_user_path = [bool]$AddToUserPath
}
