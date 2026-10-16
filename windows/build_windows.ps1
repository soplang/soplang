# Soplang Windows Build — build Rust binary and optional Inno Setup installer.
# Prerequisites: Rust (rustup.rs). Optional: Inno Setup 6 for installer.

$ErrorActionPreference = "Stop"

$projectRoot = Split-Path -Parent $PSScriptRoot
Set-Location $projectRoot

Write-Host "Soplang Windows Build" -ForegroundColor Cyan
Write-Host "=====================" -ForegroundColor Cyan

# Check for cargo
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Host "Error: cargo not found. Install Rust from https://rustup.rs" -ForegroundColor Red
    exit 1
}

$cargoToml = Get-Content (Join-Path $projectRoot "Cargo.toml") -Raw
if ($cargoToml -notmatch '(?m)^version = "([^"]+)"') {
    Write-Host "Could not read the package version from Cargo.toml." -ForegroundColor Red
    exit 1
}
$version = $Matches[1]

# Build release binary
Write-Host "Building release binary..." -ForegroundColor Cyan
cargo build --release
$binary = Join-Path $projectRoot "target\release\soplang.exe"
if (-not (Test-Path $binary)) {
    Write-Host "Build failed: soplang.exe not found." -ForegroundColor Red
    exit 1
}
Write-Host "Binary: $binary" -ForegroundColor Green

# Prepare dist for installer (Inno Setup expects ..\dist\soplang\*)
$distDir = Join-Path $projectRoot "dist\soplang"
New-Item -ItemType Directory -Force -Path $distDir | Out-Null
Copy-Item -Path $binary -Destination (Join-Path $distDir "soplang.exe") -Force
Write-Host "Dist: $distDir" -ForegroundColor Green

# Optional: Inno Setup installer
$iscc = $null
if (Test-Path "${env:ProgramFiles(x86)}\Inno Setup 6\ISCC.exe") {
    $iscc = "${env:ProgramFiles(x86)}\Inno Setup 6\ISCC.exe"
} elseif (Test-Path "${env:ProgramFiles}\Inno Setup 6\ISCC.exe") {
    $iscc = "${env:ProgramFiles}\Inno Setup 6\ISCC.exe"
}

if ($iscc) {
    Write-Host "Creating installer with Inno Setup..." -ForegroundColor Cyan
    & $iscc "/DMyAppVersion=$version" "windows\soplang_setup.iss"
    Write-Host "Installer: $projectRoot\windows\Output\soplang-setup.exe" -ForegroundColor Green
} else {
    Write-Host "Inno Setup not found. Skipping installer. Install from https://jrsoftware.org/isdl.php to create soplang-setup.exe" -ForegroundColor Yellow
}

Write-Host "Done." -ForegroundColor Green
Set-Location $PSScriptRoot
