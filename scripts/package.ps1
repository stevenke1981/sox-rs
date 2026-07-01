#!/usr/bin/env pwsh
<#
.SYNOPSIS
    Builds sox release binary and creates platform installation packages.
.DESCRIPTION
    - Builds the project in release mode
    - Copies the binary to dist/
    - Creates platform-specific archive (zip for Windows, tar.gz for Unix)
    - Generates SHA256 checksums
#>

$ErrorActionPreference = "Stop"
$ProjectRoot = Split-Path -Parent $PSScriptRoot
$Version = & cargo metadata --format-version=1 --no-deps | ConvertFrom-Json | Select-Object -ExpandProperty packages | Where-Object name -eq "rust-sox" | Select-Object -ExpandProperty version

Write-Host "=== Packaging sox v$Version ===" -ForegroundColor Cyan

# Step 1: Build release
Write-Host "`n[1/4] Building release binary..." -ForegroundColor Yellow
Push-Location $ProjectRoot
cargo build --release
if ($LASTEXITCODE -ne 0) {
    throw "Release build failed"
}

# Step 2: Determine platform
$RustTarget = & rustc -vV | Select-String "host:" | ForEach-Object { $_ -replace "host: ", "" }
$IsWindows = [System.Environment]::OSVersion.Platform -eq [System.PlatformID]::Win32NT
$BinaryName = if ($IsWindows) { "sox.exe" } else { "sox" }
$ArchiveName = "sox-$Version-$RustTarget"

# Step 3: Prepare dist directory
Write-Host "[2/4] Preparing dist/$ArchiveName..." -ForegroundColor Yellow
$DistDir = Join-Path $ProjectRoot "dist"
$PackageDir = Join-Path $DistDir $ArchiveName
New-Item -ItemType Directory -Path $PackageDir -Force | Out-Null

# Copy binary
Copy-Item (Join-Path $ProjectRoot "target\release\$BinaryName") (Join-Path $PackageDir $BinaryName) -Force

# Copy README and license
Copy-Item (Join-Path $ProjectRoot "README.md") (Join-Path $PackageDir "README.md") -Force
if (Test-Path (Join-Path $ProjectRoot "LICENSE-MIT")) {
    Copy-Item (Join-Path $ProjectRoot "LICENSE-MIT") (Join-Path $PackageDir "LICENSE-MIT") -Force
}
if (Test-Path (Join-Path $ProjectRoot "LICENSE-LGPL")) {
    Copy-Item (Join-Path $ProjectRoot "LICENSE-LGPL") (Join-Path $PackageDir "LICENSE-LGPL") -Force
}

# Create examples directory
$ExamplesDir = Join-Path $PackageDir "examples"
New-Item -ItemType Directory -Path $ExamplesDir -Force | Out-Null
Copy-Item (Join-Path $ProjectRoot "examples\*") $ExamplesDir -Force

# Step 4: Create archive
Write-Host "[3/4] Creating archive..." -ForegroundColor Yellow
if ($IsWindows) {
    $ArchiveFile = "$ArchiveName.zip"
    Compress-Archive -Path "$PackageDir\*" -DestinationPath (Join-Path $DistDir $ArchiveFile) -Force
} else {
    $ArchiveFile = "$ArchiveName.tar.gz"
    & tar -czf (Join-Path $DistDir $ArchiveFile) -C $DistDir $ArchiveName
}

# Step 5: Generate checksums
Write-Host "[4/4] Generating checksums..." -ForegroundColor Yellow
$ChecksumFile = Join-Path $DistDir "SHA256SUMS.txt"
Get-ChildItem -Path $DistDir -Filter "sox-*" | ForEach-Object {
    $hash = Get-FileHash $_.FullName -Algorithm SHA256
    "$($hash.Hash.ToLower())  $($_.Name)" | Out-File -FilePath $ChecksumFile -Append -Encoding ascii
}

Write-Host "`n=== Package complete ===" -ForegroundColor Green
Write-Host "Binary: target/release/$BinaryName"
Write-Host "Package: dist/$ArchiveFile"
Write-Host "Checksums: dist/SHA256SUMS.txt"
