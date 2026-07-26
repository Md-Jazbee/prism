# Prism installer (Windows) — P11 Stage A
# Contract: docs/architecture/RELEASE-ARTIFACTS.md
#
# Usage:
#   irm https://raw.githubusercontent.com/<owner>/<repo>/main/scripts/install.ps1 | iex
#   ./scripts/install.ps1 [-Version 0.0.1] [-DryRun] [-Uninstall] [-BinDir path]
#
# Env:
#   PRISM_GITHUB_REPO     owner/repo (default: example/prism)
#   PRISM_VERSION         override version (without leading v)
#   PRISM_DOWNLOAD_BASE   override asset base URL (mirror / local); requires -Version

[CmdletBinding()]
param(
    [string]$Version = $env:PRISM_VERSION,
    [string]$BinDir = $(if ($env:PRISM_BIN_DIR) { $env:PRISM_BIN_DIR } else { Join-Path $env:LOCALAPPDATA "Prism\bin" }),
    [string]$Repo = $(if ($env:PRISM_GITHUB_REPO) { $env:PRISM_GITHUB_REPO } else { "example/prism" }),
    [string]$DownloadBase = $env:PRISM_DOWNLOAD_BASE,
    [switch]$DryRun,
    [switch]$Uninstall
)

$ErrorActionPreference = "Stop"

function Get-Triple {
    $arch = $env:PROCESSOR_ARCHITECTURE
    switch ($arch) {
        "AMD64" { return "x86_64-pc-windows-msvc" }
        "ARM64" {
            Write-Error "ARM64 Windows is a stretch target for P11; use x86_64 or build from source."
        }
        default { Write-Error "unsupported Windows arch: $arch" }
    }
}

function Get-Sha256([string]$Path) {
    (Get-FileHash -Algorithm SHA256 -Path $Path).Hash.ToLowerInvariant()
}

if ($Uninstall) {
    $target = Join-Path $BinDir "prism.exe"
    if (Test-Path $target) {
        if ($DryRun) {
            Write-Host "+ Remove-Item $target"
        } else {
            Remove-Item -Force $target
            Write-Host "removed $target"
        }
    } else {
        Write-Host "nothing to uninstall at $target"
    }
    exit 0
}

$triple = Get-Triple
$api = "https://api.github.com/repos/$Repo/releases"

if ([string]::IsNullOrWhiteSpace($Version)) {
    if (-not [string]::IsNullOrWhiteSpace($DownloadBase)) {
        Write-Error "PRISM_DOWNLOAD_BASE requires an explicit -Version"
    }
    Write-Host "resolving latest release from $Repo…"
    $latest = Invoke-RestMethod -Uri "$api/latest"
    $tag = $latest.tag_name
    if (-not $tag) {
        Write-Error "could not resolve latest release for $Repo"
    }
    $Version = $tag.TrimStart("v")
} else {
    $Version = $Version.TrimStart("v")
}
$tag = "v$Version"

$asset = "prism-$Version-$triple.zip"
$base = if ([string]::IsNullOrWhiteSpace($DownloadBase)) { "https://github.com/$Repo/releases/download/$tag" } else { $DownloadBase.TrimEnd("/") }
$tmp = Join-Path ([System.IO.Path]::GetTempPath()) ("prism-install-" + [guid]::NewGuid().ToString("n"))
New-Item -ItemType Directory -Path $tmp | Out-Null

try {
    Write-Host "downloading $asset…"
    if ($DryRun) {
        Write-Host "+ Invoke-WebRequest $base/$asset"
        Write-Host "+ Invoke-WebRequest $base/SHA256SUMS"
        Write-Host "+ verify checksum + install to $(Join-Path $BinDir 'prism.exe')"
        Write-Host "dry-run complete (version=$Version triple=$triple)"
        exit 0
    }

    $zipPath = Join-Path $tmp $asset
    $sumPath = Join-Path $tmp "SHA256SUMS"
    Invoke-WebRequest -Uri "$base/$asset" -OutFile $zipPath
    Invoke-WebRequest -Uri "$base/SHA256SUMS" -OutFile $sumPath

    $expected = $null
    Get-Content $sumPath | ForEach-Object {
        if ($_ -match "^\s*([A-Fa-f0-9]+)\s+$([Regex]::Escape($asset))\s*$") {
            $expected = $Matches[1].ToLowerInvariant()
        }
    }
    if (-not $expected) {
        Write-Error "$asset not listed in SHA256SUMS"
    }
    $actual = Get-Sha256 $zipPath
    if ($expected -ne $actual) {
        Write-Error "checksum mismatch for $asset`n  expected: $expected`n  actual:   $actual"
    }

    Expand-Archive -Path $zipPath -DestinationPath $tmp -Force
    $exe = Join-Path $tmp "prism.exe"
    if (-not (Test-Path $exe)) {
        Write-Error "archive missing prism.exe"
    }

    New-Item -ItemType Directory -Force -Path $BinDir | Out-Null
    Copy-Item -Force $exe (Join-Path $BinDir "prism.exe")

    Write-Host "installed $(Join-Path $BinDir 'prism.exe') ($Version, $triple)"
    $pathEntries = $env:PATH -split ";"
    if ($pathEntries -notcontains $BinDir) {
        Write-Host "note: $BinDir is not on PATH — add it for this user, then re-open the shell"
        Write-Host "  [Environment]::SetEnvironmentVariable('Path', `"$BinDir;`" + [Environment]::GetEnvironmentVariable('Path','User'), 'User')"
    }
    Write-Host "next: cd <your-repo>; prism setup .; prism doctor --ready"
}
finally {
    Remove-Item -Recurse -Force $tmp -ErrorAction SilentlyContinue
}
