# Soplang Windows Build Script
# This script builds the Soplang executable and installer for Windows

# Stop on any error
$ErrorActionPreference = "Stop"

# Navigate to the project root
$projectRoot = Split-Path -Parent $PSScriptRoot
Set-Location $projectRoot

Write-Host "Building Soplang for Windows from directory: $projectRoot" -ForegroundColor Cyan
Write-Host "=====================================================" -ForegroundColor Cyan

# Function to check if a command exists
function Test-CommandExists {
    param ($command)
    $oldPreference = $ErrorActionPreference
    $ErrorActionPreference = "stop"
    try {
        if (Get-Command $command) { return $true }
    } catch { return $false }
    finally { $ErrorActionPreference = $oldPreference }
}

# Check for required tools
if (-not (Test-CommandExists "python")) {
    Write-Host "Error: Python not found. Please install Python 3.6 or higher." -ForegroundColor Red
    exit 1
}

if (-not (Test-CommandExists "pip")) {
    Write-Host "Error: pip not found. Please install pip." -ForegroundColor Red
    exit 1
}

# Check for icon file in the windows directory
if (-not (Test-Path "windows\soplang_icon.ico")) {
    Write-Host "Note: Icon file not found in windows directory." -ForegroundColor Yellow
    Write-Host "You should prepare a logo before building:" -ForegroundColor Yellow
    Write-Host "  Option 1: Run 'cd windows && .\prepare_logos.ps1'" -ForegroundColor Yellow
    Write-Host "  Option 2: Manually place an icon file at 'windows\soplang_icon.ico'" -ForegroundColor Yellow

    $proceed = Read-Host "Do you want to continue without an icon? (y/n)"
    if ($proceed -ne "y") {
        Write-Host "Build aborted. Please prepare a logo file first." -ForegroundColor Red
        exit 1
    }

    # Create a placeholder icon if continuing
    Write-Host "Creating a placeholder icon..." -ForegroundColor Yellow
    New-Item -Path "windows\soplang_icon.ico" -ItemType File -Force | Out-Null
}

# Create a virtual environment
Write-Host "Creating virtual environment..." -ForegroundColor Cyan
python -m venv venv
. .\venv\Scripts\Activate.ps1

# Install dependencies
Write-Host "Installing dependencies..." -ForegroundColor Cyan
pip install -r requirements.txt
pip install pyinstaller
pip install pywin32
pip install setuptools wheel

# Install development package
Write-Host "Installing Soplang in development mode..." -ForegroundColor Cyan
pip install -e .

# Build with PyInstaller
Write-Host "Building executable with PyInstaller..." -ForegroundColor Cyan
pyinstaller soplang.spec

# Check if Inno Setup is installed
if (Test-Path "${env:ProgramFiles(x86)}\Inno Setup 6\ISCC.exe") {
    $iscc = "${env:ProgramFiles(x86)}\Inno Setup 6\ISCC.exe"
} elseif (Test-Path "${env:ProgramFiles}\Inno Setup 6\ISCC.exe") {
    $iscc = "${env:ProgramFiles}\Inno Setup 6\ISCC.exe"
} else {
    Write-Host "Note: Inno Setup not found. Skipping installer creation." -ForegroundColor Yellow
    Write-Host "Please install Inno Setup from https://jrsoftware.org/isdl.php" -ForegroundColor Yellow
    Write-Host "and then run: '& `"C:\Program Files (x86)\Inno Setup 6\ISCC.exe`" windows\soplang_setup.iss'" -ForegroundColor Yellow
    exit 0
}

# Create installer with Inno Setup
Write-Host "Creating installer with Inno Setup..." -ForegroundColor Cyan
& $iscc "windows\soplang_setup.iss"

Write-Host "Build completed successfully!" -ForegroundColor Green
Write-Host "Executable: $projectRoot\dist\soplang\soplang.exe" -ForegroundColor Green
Write-Host "Installer: $projectRoot\windows\Output\soplang-setup.exe" -ForegroundColor Green

# Return to the windows directory
Set-Location $PSScriptRoot
