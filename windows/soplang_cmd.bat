@echo off
REM Soplang Command Line Launcher
REM This batch file helps run the Soplang executable from the command line
REM It supports both .sop (primary) and .so (secondary) file extensions

setlocal enabledelayedexpansion

REM Get the directory of this batch file
set "SOPLANG_HOME=%~dp0"

REM Remove trailing backslash
set "SOPLANG_HOME=%SOPLANG_HOME:~0,-1%"

REM Store the current command line mode (interactive or not)
set "IS_INTERACTIVE=0"
echo %CMDCMDLINE% | findstr /C:"cmd.exe" >nul 2>&1
if %ERRORLEVEL% equ 0 set "IS_INTERACTIVE=1"

REM Check if there are arguments
if "%1"=="" (
    REM No arguments, start interactive shell
    "%SOPLANG_HOME%\soplang.exe"
) else (
    REM Check if the first argument is a file
    if exist "%1" (
        REM If it's a file, pass it to Soplang
        "%SOPLANG_HOME%\soplang.exe" "%~1"
        set LAST_ERROR=%ERRORLEVEL%

        REM Only pause if launched from explorer (not command line) and there was an error
        if %IS_INTERACTIVE% equ 0 (
            if %LAST_ERROR% neq 0 (
                echo.
                echo Soplang execution finished with error code: %LAST_ERROR%
                echo.
                pause
            ) else (
                REM Brief pause so user can see the output
                timeout /t 3 >nul
            )
        )
    ) else (
        REM Otherwise, pass all arguments directly
        "%SOPLANG_HOME%\soplang.exe" %*
    )
)

endlocal
exit /b %ERRORLEVEL%
