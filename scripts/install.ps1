# SemanticFS installer for Windows (PowerShell).
#
# Downloads the latest pre-built release binary from GitHub Releases and
# installs it to $env:LOCALAPPDATA\semanticfs\ (or a path you choose with -Prefix).
#
# Usage:
#   irm https://raw.githubusercontent.com/Navneeth08k/semanticFS/main/scripts/install.ps1 | iex
#   .\scripts\install.ps1 [-Prefix "C:\Tools\semanticfs"] [-Version "v1.0.0"]

param(
    [string]$Prefix = "$env:LOCALAPPDATA\semanticfs",
    [string]$Version = ""
)

$ErrorActionPreference = "Stop"
$REPO = "Navneeth08k/semanticFS"   # <- set to your GitHub org/repo before publishing
$BINARY_NAME = "semanticfs.exe"
$TARGET = "x86_64-pc-windows-msvc"

# ── Resolve version ────────────────────────────────────────────────────────────
if ($Version -eq "") {
    Write-Host "Fetching latest release version..."
    try {
        $release = Invoke-RestMethod "https://api.github.com/repos/$REPO/releases/latest"
        $Version = $release.tag_name
    } catch {
        Write-Error "Could not determine latest version. Pass -Version vX.Y.Z explicitly."
        exit 1
    }
}

if ($Version -eq "") {
    Write-Error "Could not determine version. Pass -Version vX.Y.Z explicitly."
    exit 1
}

Write-Host "Installing semanticfs $Version for $TARGET..."

# ── Download ───────────────────────────────────────────────────────────────────
$Artifact = "semanticfs-$Version-$TARGET.zip"
$Url = "https://github.com/$REPO/releases/download/$Version/$Artifact"
$TmpDir = Join-Path $env:TEMP "semanticfs-install-$([System.IO.Path]::GetRandomFileName())"
New-Item -ItemType Directory -Path $TmpDir | Out-Null

$ArchivePath = Join-Path $TmpDir $Artifact

try {
    Write-Host "Downloading $Url..."
    Invoke-WebRequest -Uri $Url -OutFile $ArchivePath -UseBasicParsing

    # ── Extract ────────────────────────────────────────────────────────────────
    Write-Host "Extracting..."
    Expand-Archive -Path $ArchivePath -DestinationPath $TmpDir -Force

    $ExtractedDir = Join-Path $TmpDir "semanticfs-$Version-$TARGET"

    # ── Install ────────────────────────────────────────────────────────────────
    if (-not (Test-Path $Prefix)) {
        New-Item -ItemType Directory -Path $Prefix | Out-Null
    }

    $BinaryDest = Join-Path $Prefix $BINARY_NAME
    Copy-Item (Join-Path $ExtractedDir $BINARY_NAME) -Destination $BinaryDest -Force

    # Also install the Python stdio wrapper alongside the binary (fallback for HTTP mode)
    $PySrc = Join-Path $ExtractedDir "semanticfs_mcp_stdio.py"
    if (Test-Path $PySrc) {
        Copy-Item $PySrc -Destination (Join-Path $Prefix "semanticfs_mcp_stdio.py") -Force
    }

} finally {
    # Clean up temp directory
    Remove-Item -Recurse -Force $TmpDir -ErrorAction SilentlyContinue
}

# ── Add to PATH ────────────────────────────────────────────────────────────────
$UserPath = [Environment]::GetEnvironmentVariable("PATH", "User")
if ($UserPath -notlike "*$Prefix*") {
    [Environment]::SetEnvironmentVariable("PATH", "$Prefix;$UserPath", "User")
    Write-Host ""
    Write-Host "  Added $Prefix to your user PATH."
    Write-Host "  Restart your terminal (or run: `$env:PATH = `"$Prefix;`$env:PATH`") to use semanticfs immediately."
}

# ── Done ───────────────────────────────────────────────────────────────────────
Write-Host ""
Write-Host "semanticfs $Version installed to $BinaryDest"
Write-Host ""
Write-Host "Next steps:"
Write-Host "  1. Create a config:  semanticfs init --root C:\path\to\your\repo"
Write-Host "  2. Build the index:  semanticfs --config semanticfs.toml index build"
Write-Host "  3. Start MCP server: semanticfs --config semanticfs.toml serve mcp-stdio"
Write-Host ""
Write-Host "For Claude Code integration, see: docs/setup_claude_code.md"
Write-Host "For OpenClaw integration, see:    docs/setup_openclaw.md"
