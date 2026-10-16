@echo off
REM Keep Command Prompt and PowerShell packaging behavior identical.
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0build_windows.ps1"
exit /b %ERRORLEVEL%
