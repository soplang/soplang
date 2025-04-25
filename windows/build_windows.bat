@echo off
REM Soplang Windows Build Script
REM This batch file builds the Soplang executable and installer for Windows

REM Navigate to the project root (parent of windows directory)
cd ..
set PROJECT_ROOT=%CD%

echo Building Soplang for Windows from directory: %PROJECT_ROOT%
echo =====================================================

REM Check if Python is installed
where python >nul 2>&1
if %ERRORLEVEL% neq 0 (
    echo Error: Python not found. Please install Python 3.6 or higher.
    exit /b 1
)

REM Check if pip is installed
where pip >nul 2>&1
if %ERRORLEVEL% neq 0 (
    echo Error: pip not found. Please install pip.
    exit /b 1
)

REM Check for icon file in the windows directory
if not exist windows\soplang_icon.ico (
    echo Note: Icon file not found in windows directory.
    echo You should prepare a logo before building:
    echo   Option 1: Run "cd windows && prepare_logos.bat"
    echo   Option 2: Manually place an icon file at windows\soplang_icon.ico

    set /p proceed=Do you want to continue without an icon? (y/n):
    if /i not "%proceed%"=="y" (
        echo Build aborted. Please prepare a logo file first.
        cd windows
        exit /b 1
    )

    echo Creating a placeholder icon...
    echo. > windows\soplang_icon.ico
)

REM Remove existing venv if it exists
if exist venv (
    echo Removing existing virtual environment...
    rmdir /s /q venv
)

REM Create a virtual environment with elevated permissions if needed
echo Creating virtual environment...
python -m venv venv 2>nul
if %ERRORLEVEL% neq 0 (
    echo Error creating virtual environment. Trying with admin rights...
    powershell Start-Process python -ArgumentList "-m venv venv" -Verb RunAs -Wait
)

REM Activate the virtual environment
echo Activating virtual environment...
call venv\Scripts\activate.bat
if %ERRORLEVEL% neq 0 (
    echo Error activating environment. Please run this script with administrator privileges.
    exit /b 1
)

REM Install Windows-specific dependencies
echo Installing Windows-specific dependencies...
pip install -r windows\requirements_windows.txt

REM Install development package
echo Installing Soplang in development mode...
pip install -e .

REM Build with PyInstaller
echo Building executable with PyInstaller...
pyinstaller soplang.spec

REM Check if Inno Setup is installed
if exist "%ProgramFiles(x86)%\Inno Setup 6\ISCC.exe" (
    set ISCC="%ProgramFiles(x86)%\Inno Setup 6\ISCC.exe"
) else if exist "%ProgramFiles%\Inno Setup 6\ISCC.exe" (
    set ISCC="%ProgramFiles%\Inno Setup 6\ISCC.exe"
) else (
    echo Note: Inno Setup not found. Skipping installer creation.
    echo Please install Inno Setup from https://jrsoftware.org/isdl.php
    echo and then run: "C:\Program Files (x86)\Inno Setup 6\ISCC.exe" windows\soplang_setup.iss
    goto :end
)

REM Create installer with Inno Setup
echo Creating installer with Inno Setup...
%ISCC% windows\soplang_setup.iss

:end
echo Build completed successfully!
echo Executable: %PROJECT_ROOT%\dist\soplang\soplang.exe
if exist "%ISCC%" echo Installer: %PROJECT_ROOT%\windows\Output\soplang-setup.exe

REM Deactivate the virtual environment
call venv\Scripts\deactivate.bat

REM Return to the windows directory
cd windows
echo Done.
