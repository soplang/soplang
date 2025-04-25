@echo off
REM Soplang Interactive Launcher
REM This batch file launches Soplang in interactive mode and keeps the window open

setlocal enabledelayedexpansion

REM Get the directory of this batch file
set "SOPLANG_HOME=%~dp0"

REM Remove trailing backslash
set "SOPLANG_HOME=%SOPLANG_HOME:~0,-1%"

REM Display banner
echo.
echo =====================================================
echo    Soplang Interpreter - The Somali Programming Language
echo =====================================================
echo.

REM Launch Soplang in interactive mode
"%SOPLANG_HOME%\soplang.exe" -i

REM If Soplang exits unexpectedly, keep the window open
if %ERRORLEVEL% neq 0 (
    echo.
    echo Soplang exited with error code: %ERRORLEVEL%
    echo.
    pause
)

endlocal
exit /b 0
