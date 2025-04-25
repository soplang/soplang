# Soplang for Windows

This directory contains the files needed to build Soplang for Windows, including the executable and installer.

## Prerequisites

Before building Soplang for Windows, you need:

1. **Python 3.6 or higher** - [Download Python](https://www.python.org/downloads/windows/)
2. **Inno Setup 6** (for creating the installer) - [Download Inno Setup](https://jrsoftware.org/isdl.php)

## Building Soplang for Windows

There are two ways to build Soplang for Windows:

### Using PowerShell (Recommended)

1. Open PowerShell as Administrator
2. Navigate to the Soplang root directory
3. Run the build script:

```powershell
.\build_windows.ps1
```

### Using Command Prompt

1. Open Command Prompt as Administrator
2. Navigate to the Soplang root directory
3. Run the batch file:

```cmd
build_windows.bat
```

## Build Output

After the build process completes successfully, you will find:

- **Standalone executable**: `dist\soplang\soplang.exe`
- **Windows installer**: `windows\Output\soplang-setup.exe`

## Custom Icon

If you want to use a custom icon for Soplang, replace the `windows\soplang_icon.ico` file with your own 256x256 icon before building.

## Manual Installation

If you don't want to use the installer, you can manually:

1. Copy the entire `dist\soplang` directory to any location on your computer
2. Add that location to your PATH environment variable
3. Optionally, associate `.so` files with `soplang.exe`

## Troubleshooting

- **Missing Python**: Ensure Python is installed and added to PATH
- **Missing dependencies**: Run `pip install -r requirements.txt`
- **Inno Setup errors**: Ensure Inno Setup 6 is installed properly
- **PyInstaller errors**: Try `pip install --upgrade pyinstaller`

## File Associations

The installer automatically associates `.so` files with Soplang. After installation, you can double-click any `.so` file to run it with Soplang.
