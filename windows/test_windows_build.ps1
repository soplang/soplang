# Run the same complete installed-package checks used on Linux and macOS.
param ([string]$SoplangExePath = ".\dist\soplang\soplang.exe")
$ErrorActionPreference = "Stop"
$checker = Join-Path (Split-Path -Parent $PSScriptRoot) "scripts/verify_package.py"
python $checker $SoplangExePath
exit $LASTEXITCODE
