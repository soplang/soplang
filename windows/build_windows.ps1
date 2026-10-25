# Build a complete portable package and optional Inno Setup installer.
$ErrorActionPreference = "Stop"
$projectRoot = Split-Path -Parent $PSScriptRoot
Push-Location $projectRoot
try {
    $cargoToml = Get-Content "Cargo.toml" -Raw
    if ($cargoToml -notmatch '(?m)^version = "([^"]+)"') { throw "Missing Cargo version" }
    $version = $Matches[1]
    python scripts/package_native_windows_gnu.py --output dist/soplang
    if ($LASTEXITCODE -ne 0) { throw "Package build failed" }
    $archive = "windows/soplang-$version-windows-$env:PROCESSOR_ARCHITECTURE.zip"
    if (Test-Path $archive) { Remove-Item $archive }
    Compress-Archive -Path dist/soplang -DestinationPath $archive
    $assets = @($archive)
    $iscc = Get-Command ISCC.exe -ErrorAction SilentlyContinue
    $isccPath = if ($iscc) { $iscc.Source } else { $null }
    if (-not $isccPath) {
        foreach ($base in @(${env:ProgramFiles(x86)}, $env:ProgramFiles)) {
            if ($base -and (Test-Path "$base\Inno Setup 6\ISCC.exe")) {
                $isccPath = "$base\Inno Setup 6\ISCC.exe"
                break
            }
        }
    }
    if ($isccPath) {
        $numericVersion = ($version -split '-')[0]
        & $isccPath "/DMyAppVersion=$version" "/DMyNumericVersion=$numericVersion" "windows/soplang_setup.iss"
        if ($LASTEXITCODE -ne 0) { throw "Inno Setup failed" }
        $assets += "windows/Output/soplang-setup.exe"
    } else {
        Write-Host "Inno Setup unavailable; installer verification remains pending."
    }
    python scripts/checksums.py @assets --output windows/SHA256SUMS
    if ($LASTEXITCODE -ne 0) { throw "Checksum generation failed" }
} finally {
    Pop-Location
}
