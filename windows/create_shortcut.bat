@echo off
REM Soplang Shortcut Creator
REM This script creates proper shortcuts for Soplang

setlocal

echo Creating Soplang shortcuts...

REM Get Soplang installation directory
for /f "tokens=2*" %%a in ('reg query "HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\{F1C77F9E-F26A-4D23-9A8B-CF3D26AE5A18}_is1" /v "Inno Setup: App Path" 2^>nul') do (
    set "SOPLANG_DIR=%%b"
)

if not defined SOPLANG_DIR (
    echo Soplang installation not found.
    echo Please install Soplang first.
    goto :EOF
)

echo Soplang installed at: %SOPLANG_DIR%

REM Create Start Menu shortcuts
set "STARTMENU=%APPDATA%\Microsoft\Windows\Start Menu\Programs\Soplang"
if not exist "%STARTMENU%" mkdir "%STARTMENU%"

REM Create the Soplang Interpreter shortcut
echo Creating Soplang Interpreter shortcut...
powershell -Command "$ws = New-Object -ComObject WScript.Shell; $s = $ws.CreateShortcut('%STARTMENU%\Soplang Interpreter.lnk'); $s.TargetPath = '%SOPLANG_DIR%\soplang_launcher.bat'; $s.WorkingDirectory = '%SOPLANG_DIR%'; $s.IconLocation = '%SOPLANG_DIR%\soplang_icon.ico'; $s.Description = 'Run Soplang Interactive Shell'; $s.Save()"

REM Create the Soplang Command Prompt shortcut
echo Creating Soplang Command Prompt shortcut...
powershell -Command "$ws = New-Object -ComObject WScript.Shell; $s = $ws.CreateShortcut('%STARTMENU%\Soplang Command Prompt.lnk'); $s.TargetPath = 'cmd.exe'; $s.Arguments = '/k set ""PATH=%SOPLANG_DIR%;%PATH%"" && cd /d ""%SOPLANG_DIR%""'; $s.WorkingDirectory = '%SOPLANG_DIR%'; $s.IconLocation = '%SOPLANG_DIR%\soplang_icon.ico'; $s.Description = 'Open a command prompt with Soplang in the path'; $s.Save()"

echo Shortcuts created successfully!
echo Soplang Interpreter: %STARTMENU%\Soplang Interpreter.lnk
echo Soplang Command Prompt: %STARTMENU%\Soplang Command Prompt.lnk

endlocal
