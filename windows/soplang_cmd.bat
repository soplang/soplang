@echo off
REM Forward command-line arguments and preserve the compiler exit status.
"%~dp0soplang.exe" %*
exit /b %ERRORLEVEL%
