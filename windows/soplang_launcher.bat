@echo off
REM Soplang Interactive Launcher
REM This batch file launches Soplang in interactive mode and keeps the window open

setlocal enabledelayedexpansion

REM Get the directory of this batch file
set "SOPLANG_HOME=%~dp0"

REM Remove trailing backslash
set "SOPLANG_HOME=%SOPLANG_HOME:~0,-1%"

REM Launch Soplang in interactive mode
cd /d "%SOPLANG_HOME%"
call "%SOPLANG_HOME%\soplang.exe" -i

REM If Soplang exits unexpectedly, keep the window open
if %ERRORLEVEL% neq 0 (
    echo.
    echo Soplang exited with error code: %ERRORLEVEL%
    echo.
    pause
) else (
    echo.
    echo Soplang session ended.
    timeout /t 3 >nul
)

endlocal
exit /b 0
