# Soplang Windows Test Script
# This script tests the Soplang executable built for Windows

# Parameters
param (
    [string]$SoplangExePath = ".\dist\soplang\soplang.exe"
)

# Function to display test results
function Show-TestResult {
    param (
        [string]$TestName,
        [bool]$Success,
        [string]$Output
    )

    if ($Success) {
        Write-Host "✓ $TestName" -ForegroundColor Green
    } else {
        Write-Host "✗ $TestName" -ForegroundColor Red
        Write-Host "  Output: $Output" -ForegroundColor Red
    }
}

# Check if executable exists
if (-not (Test-Path $SoplangExePath)) {
    Write-Host "Error: Soplang executable not found at $SoplangExePath" -ForegroundColor Red
    Write-Host "Please build Soplang using build_windows.ps1 or build_windows.bat first." -ForegroundColor Yellow
    exit 1
}

Write-Host "Testing Soplang Windows build..." -ForegroundColor Cyan
Write-Host "Executable: $SoplangExePath" -ForegroundColor Cyan
Write-Host ""

# Test 1: Check version
$output = & $SoplangExePath -v 2>&1
$success = $output -match "Soplang: The Somali Programming Language"
Show-TestResult -TestName "Version Check" -Success $success -Output $output

# Test 2: Execute a simple code snippet
$output = & $SoplangExePath -c 'qor("Hello from Soplang on Windows!")' 2>&1
$success = $output -match "Hello from Soplang on Windows!"
Show-TestResult -TestName "Code Execution" -Success $success -Output $output

# Test 3: Create and run a test file with .sop extension (primary)
$testFilePath1 = ".\windows\test_file.sop"
"qor(`"This is a test .sop file for Soplang on Windows`");" | Out-File -FilePath $testFilePath1

$output = & $SoplangExePath $testFilePath1 2>&1
$success = $output -match "This is a test .sop file for Soplang on Windows"
Show-TestResult -TestName "Primary (.sop) File Execution" -Success $success -Output $output

# Test 4: Create and run a test file with .so extension (secondary)
$testFilePath2 = ".\windows\test_file.so"
"qor(`"This is a test .so file for Soplang on Windows`");" | Out-File -FilePath $testFilePath2

$output = & $SoplangExePath $testFilePath2 2>&1
$success = $output -match "This is a test .so file for Soplang on Windows"
Show-TestResult -TestName "Secondary (.so) File Execution" -Success $success -Output $output

# Clean up
if (Test-Path $testFilePath1) {
    Remove-Item $testFilePath1
}
if (Test-Path $testFilePath2) {
    Remove-Item $testFilePath2
}

Write-Host ""
Write-Host "Testing completed." -ForegroundColor Cyan
