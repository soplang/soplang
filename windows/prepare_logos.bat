@echo off
REM Soplang Logo Preparation Script for Windows
REM This script helps convert logo files to Windows icon format

echo Soplang Logo Preparation for Windows
echo =====================================
echo.

REM Check if the ImageMagick convert command is available
where magick >nul 2>&1
if %ERRORLEVEL% neq 0 (
    echo NOTE: ImageMagick is not installed.
    echo For best results, install ImageMagick from: https://imagemagick.org/script/download.php
    echo Or manually convert your logo to a Windows .ico file.
    echo.
    set IMAGEMAGICK_AVAILABLE=0
) else (
    set IMAGEMAGICK_AVAILABLE=1
)

echo This script will create a Windows icon from your Soplang logo.
echo.
echo Please copy your Soplang logo file to the windows directory and name it:
echo   - soplang_logo.png  (for automatic icon creation)
echo   - soplang_icon.ico  (if you already have an icon file)
echo.
echo Press any key when you've copied the logo file...
pause >nul

REM Check for logo files
if exist soplang_icon.ico (
    echo Found existing icon file: soplang_icon.ico
    echo Icon is ready for use in the Windows build.
    goto :end
)

if exist soplang_logo.png (
    echo Found logo file: soplang_logo.png

    if %IMAGEMAGICK_AVAILABLE%==1 (
        echo Converting logo to icon using ImageMagick...
        magick convert "soplang_logo.png" -define icon:auto-resize=256,128,64,48,32,16 "soplang_icon.ico"

        if exist soplang_icon.ico (
            echo Icon created successfully: soplang_icon.ico
        ) else (
            echo Failed to create icon. Please convert the logo manually.
        )
    ) else (
        echo ImageMagick not available for automatic conversion.
        echo Please manually convert soplang_logo.png to an .ico file.
    )
) else (
    echo No logo file found in the windows directory.
    echo Please manually add a logo image named soplang_logo.png or an icon file named soplang_icon.ico.
)

:end
echo.
echo Logo preparation complete!
echo The icons are now ready in the windows folder.
echo You can now run the build script from the project root to build the Windows installer.
echo.
