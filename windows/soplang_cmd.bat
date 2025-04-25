@echo off
REM Soplang Command Line Launcher
REM This batch file helps run the Soplang executable from the command line
REM It supports both .sop (primary) and .so (secondary) file extensions

setlocal enabledelayedexpansion

REM Get the directory of this batch file
set "SOPLANG_HOME=%~dp0"

REM Remove trailing backslash
set "SOPLANG_HOME=%SOPLANG_HOME:~0,-1%"

REM Check if there are arguments
if "%1"=="" (
    REM No arguments, start interactive shell
    "%SOPLANG_HOME%\soplang.exe"
) else (
    REM Check if the first argument is a file
    if exist "%1" (
        REM If it's a file, pass it to Soplang
        "%SOPLANG_HOME%\soplang.exe" "%~1"
    ) else (
        REM Otherwise, pass all arguments directly
        "%SOPLANG_HOME%\soplang.exe" %*
    )
)

endlocal
exit /b %ERRORLEVEL%
