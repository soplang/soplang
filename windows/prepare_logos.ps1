# Soplang Logo Preparation Script for Windows
# This script helps convert logo files to Windows icon format

# Stop on any error
$ErrorActionPreference = "Stop"

# Function to prompt user to select a logo file
function Select-LogoFile {
    param (
        [string]$Title,
        [string]$Filter = "Image files (*.png;*.jpg;*.jpeg;*.bmp)|*.png;*.jpg;*.jpeg;*.bmp|All files (*.*)|*.*"
    )

    Add-Type -AssemblyName System.Windows.Forms
    $FileBrowser = New-Object System.Windows.Forms.OpenFileDialog
    $FileBrowser.Title = $Title
    $FileBrowser.Filter = $Filter
    $FileBrowser.Multiselect = $false

    $result = $FileBrowser.ShowDialog()

    if ($result -eq [System.Windows.Forms.DialogResult]::OK) {
        return $FileBrowser.FileName
    } else {
        return $null
    }
}

Write-Host "Soplang Logo Preparation for Windows" -ForegroundColor Cyan
Write-Host "=====================================" -ForegroundColor Cyan
Write-Host ""

# Check if we should regenerate the icon, even if it exists
$regenerate = $false
if (Test-Path "soplang_icon.ico") {
    $response = Read-Host "Icon file already exists. Do you want to regenerate it? (y/n)"
    if ($response -eq "y") {
        $regenerate = $true
        Write-Host "Will regenerate the icon." -ForegroundColor Cyan
    }
}

# Check if the ImageMagick convert command is available
$hasImageMagick = $false
try {
    if (Get-Command "magick" -ErrorAction SilentlyContinue) {
        $hasImageMagick = $true
    }
} catch {
    # ImageMagick not installed
}

if (-not $hasImageMagick) {
    Write-Host "NOTE: ImageMagick is not installed." -ForegroundColor Yellow
    Write-Host "For best results, install ImageMagick from: https://imagemagick.org/script/download.php" -ForegroundColor Yellow
    Write-Host "Or manually convert your logo to a Windows .ico file." -ForegroundColor Yellow
    Write-Host ""
    Write-Host "Icon Tips for Professional Look:" -ForegroundColor Cyan
    Write-Host "1. Use a clean, simple design that's recognizable even at small sizes" -ForegroundColor Cyan
    Write-Host "2. Use a transparent background" -ForegroundColor Cyan
    Write-Host "3. Ensure high contrast for visibility in taskbars" -ForegroundColor Cyan
    Write-Host "4. Create a square image before converting to icon format" -ForegroundColor Cyan
    Write-Host "5. Include multiple resolutions (16x16, 32x32, 48x48, 256x256)" -ForegroundColor Cyan
    Write-Host ""
}

# Get project root (parent of windows directory)
$projectRoot = Split-Path -Parent $PSScriptRoot

# Check if we should proceed with icon creation
if (Test-Path "soplang_icon.ico" -and -not $regenerate) {
    Write-Host "Using existing icon file: soplang_icon.ico" -ForegroundColor Green
    exit 0
}

# Try to locate logo files automatically
$logoFiles = Get-ChildItem -Path $projectRoot -Filter "*.png" -Recurse | Where-Object { $_.Name -like "*logo*" -or $_.Name -like "*icon*" }

if ($logoFiles.Count -gt 0) {
    Write-Host "Found potential logo files:" -ForegroundColor Green
    for ($i = 0; $i -lt $logoFiles.Count; $i++) {
        Write-Host "[$i] $($logoFiles[$i].FullName)"
    }
    Write-Host ""

    $selectedIndex = Read-Host "Enter the number of the logo file you want to use, or press Enter to browse"

    if ($selectedIndex -ne "") {
        try {
            $selectedLogo = $logoFiles[[int]$selectedIndex].FullName
        } catch {
            Write-Host "Invalid selection. Please browse for the logo file." -ForegroundColor Red
            $selectedLogo = Select-LogoFile -Title "Select the Soplang logo file"
        }
    } else {
        $selectedLogo = Select-LogoFile -Title "Select the Soplang logo file"
    }
} else {
    Write-Host "No logo files automatically detected."
    $selectedLogo = Select-LogoFile -Title "Select the Soplang logo file"
}

if ($selectedLogo -eq $null) {
    Write-Host "No logo selected. Exiting." -ForegroundColor Red
    exit 1
}

Write-Host "Selected logo: $selectedLogo" -ForegroundColor Green

# Path for the icon file (now in the current windows directory)
$iconPath = "soplang_icon.ico"

# Convert the logo to an icon if ImageMagick is available
if ($hasImageMagick) {
    Write-Host "Converting logo to icon using ImageMagick..." -ForegroundColor Cyan

    # Create a multi-resolution icon for best appearance
    magick convert "$selectedLogo" -background transparent -define icon:auto-resize=256,128,64,48,32,16 "$iconPath"

    if (Test-Path $iconPath) {
        Write-Host "Icon created successfully: $iconPath" -ForegroundColor Green
    } else {
        Write-Host "Failed to create icon. Please convert the logo manually." -ForegroundColor Red
    }
} else {
    # Copy the file without conversion if ImageMagick isn't available
    Write-Host "Copying logo file without conversion..." -ForegroundColor Yellow
    Copy-Item -Path $selectedLogo -Destination "soplang_logo.png" -Force
    Write-Host "Copied logo to: soplang_logo.png" -ForegroundColor Yellow
    Write-Host "NOTE: You will need to manually convert this to an .ico file for best results." -ForegroundColor Yellow
    Write-Host "Try using an online converter like convertio.co or icoconvert.com" -ForegroundColor Yellow
}

# Copy logo to other locations where it might be useful (now all within windows folder)
if (-not (Test-Path "soplang_logo.png")) {
    Copy-Item -Path $selectedLogo -Destination "soplang_logo.png" -Force
    Write-Host "Copied logo to: soplang_logo.png" -ForegroundColor Green
}

Write-Host ""
Write-Host "Logo preparation complete!" -ForegroundColor Green
Write-Host "The icons are now ready in the windows folder." -ForegroundColor Cyan
Write-Host "You can now run the build script from the project root to build the Windows installer." -ForegroundColor Cyan
