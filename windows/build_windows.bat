@echo off
REM Soplang Windows Build — build Rust binary and optional Inno Setup installer.
REM Prerequisites: Rust (rustup.rs). Optional: Inno Setup 6 for installer.

cd /d "%~dp0\.."
set PROJECT_ROOT=%CD%
for /f "tokens=3" %%V in ('findstr /B /C:"version = " Cargo.toml') do set "VERSION=%%~V"
if not defined VERSION (
    echo Could not read the package version from Cargo.toml.
    exit /b 1
)

echo Soplang Windows Build
echo =====================

where cargo >nul 2>&1
if %ERRORLEVEL% neq 0 (
    echo Error: cargo not found. Install Rust from https://rustup.rs
    exit /b 1
)

echo Building release binary...
cargo build --release
if %ERRORLEVEL% neq 0 (
    echo Build failed.
    exit /b 1
)
if not exist "target\release\soplang.exe" (
    echo Build failed: soplang.exe not found.
    exit /b 1
)
echo Binary: %PROJECT_ROOT%\target\release\soplang.exe

if not exist "dist\soplang" mkdir "dist\soplang"
copy /y "target\release\soplang.exe" "dist\soplang\soplang.exe" >nul
echo Dist: %PROJECT_ROOT%\dist\soplang

set ISCC=
if exist "%ProgramFiles(x86)%\Inno Setup 6\ISCC.exe" set "ISCC=%ProgramFiles(x86)%\Inno Setup 6\ISCC.exe"
if exist "%ProgramFiles%\Inno Setup 6\ISCC.exe" set "ISCC=%ProgramFiles%\Inno Setup 6\ISCC.exe"

if defined ISCC (
    echo Creating installer with Inno Setup...
    "%ISCC%" /DMyAppVersion=%VERSION% "windows\soplang_setup.iss"
    echo Installer: %PROJECT_ROOT%\windows\Output\soplang-setup.exe
) else (
    echo Inno Setup not found. Skipping installer. Install from https://jrsoftware.org/isdl.php
)

cd windows
echo Done.
