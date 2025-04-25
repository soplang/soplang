# Complete Windows Build and Distribution Guide for Soplang

This guide provides detailed, step-by-step instructions for building, testing, and distributing Soplang on Windows. Unlike the general README, this guide is specifically for Windows developers who want to create installable packages.

## Overview of the Windows Build Process

Building Soplang for Windows involves:

1. **Logo preparation**: Converting your logo to Windows icon format
2. **Executable creation**: Using PyInstaller to bundle Python into an executable
3. **Installer creation**: Using Inno Setup to create a professional installer
4. **Testing**: Verifying the build works correctly

## Prerequisites

Before you begin, install the following software on your Windows machine:

1. **Python 3.6 or higher**
   - Download from [python.org](https://www.python.org/downloads/windows/)
   - During installation, check "Add Python to PATH"

2. **Inno Setup 6**
   - Download from [jrsoftware.org/isdl.php](https://jrsoftware.org/isdl.php)
   - Install with default settings

3. **Git** (optional, for cloning the repository)
   - Download from [git-scm.com](https://git-scm.com/download/win)

4. **ImageMagick** (optional, for icon conversion)
   - Download from [imagemagick.org](https://imagemagick.org/script/download.php)
   - Check "Install legacy utilities" during installation

## Step 1: Prepare the Build Environment

1. Clone or download the Soplang repository:
   ```
   git clone https://github.com/sharafdin/soplang.git
   cd soplang
   ```

2. Verify Python is correctly installed:
   ```
   python --version
   ```
   You should see Python 3.6 or higher.

## Step 2: Navigate to the Windows Build Directory

All Windows-specific build tools are located in the `windows` directory:

```
cd windows
```

## Step 3: Prepare the Soplang Logo

You have two options for logo preparation:

### Option 1: Use the Preparation Scripts

Run the logo preparation script:
```
.\prepare_logos.ps1
```
or using Command Prompt:
```
prepare_logos.bat
```

Follow the prompts to select a logo file:
- The script will convert the logo to an icon file (.ico)
- The icon will be saved as `soplang_icon.ico` in the windows directory

### Option 2: Manual Preparation

1. Convert your logo to ICO format using any image editor or online converter
2. Name it `soplang_icon.ico`
3. Place it directly in the `windows` folder

The icon will be used for:
- Application icon in Start menu
- File association icons for .sop and .so files
- Installer icon

## Step 4: Build the Windows Executable and Installer

Run the build script from the windows directory:
```
.\build_windows.ps1
```
or using Command Prompt:
```
build_windows.bat
```

The script will:
- Navigate to the project root directory
- Create a Python virtual environment
- Install all dependencies
- Build the executable using PyInstaller (outputs multiple files)
- Create an installer using Inno Setup
- Return to the windows directory when complete

The build process produces:
- `dist\soplang\` directory with executable and support files
- `windows\Output\soplang-setup.exe` installer

## Output Files Explained

After building, you'll have:

1. **Standalone Directory** (`dist\soplang\`):
   - `soplang.exe` - Main executable
   - Multiple DLL and PYD files - Supporting libraries
   - Python bytecode files - Compiled Python code
   - These files must stay together; the .exe alone won't work

2. **Installer** (`windows\Output\soplang-setup.exe`):
   - Single file that will install all necessary components
   - Creates file associations for .sop and .so files
   - Adds Soplang to the Windows PATH (if selected)
   - Creates Start menu shortcuts

## Step 5: Test the Windows Build

Run the test script:
```
.\test_windows_build.ps1
```

The test script will verify:
- The executable runs correctly
- Version command works
- Code snippet execution works
- .sop file execution works (primary file extension)
- .so file execution works (secondary file extension)

## Step 6: Install and Use Soplang

You now have two options for using Soplang on Windows:

### Option 1: Run the Standalone Executable

1. Navigate to `..\dist\soplang` directory
2. Run `soplang.exe` to start the interactive shell
3. Run `soplang.exe your_file.sop` to execute a Soplang file
4. Note: You need the entire directory, not just the .exe file

### Option 2: Run the Installer

1. Navigate to `Output` directory
2. Run `soplang-setup.exe`
3. Follow the installation wizard
4. After installation, you can:
   - Run Soplang from the Start menu
   - Double-click `.sop` or `.so` files to run them with Soplang
   - Use the `soplang` command from any Command Prompt

## Key Components of the Windows Build

All Windows-specific files are now organized in the windows directory:

1. **windows/build_windows.ps1** and **windows/build_windows.bat**:
   - Scripts to build the Windows executable and installer
   - Automatically navigate to the project root and back

2. **windows/prepare_logos.ps1** and **windows/prepare_logos.bat**:
   - Scripts to help convert logo images to Windows icon format

3. **windows/soplang_setup.iss**:
   - Inno Setup script for creating the Windows installer
   - Handles file associations and registry entries

4. **windows/file_association.reg**:
   - Windows Registry entries for file associations
   - Associates .sop (primary) and .so (secondary) extensions

5. **windows/soplang_cmd.bat**:
   - Command-line wrapper for the Soplang executable

6. **windows/test_windows_build.ps1**:
   - Script to test the Windows build

## Troubleshooting

### Python Issues
- Ensure Python 3.6+ is installed and added to PATH
- Try running `python --version` to verify

### Virtual Environment Issues
- If you encounter `venv` errors, try:
  ```
  python -m pip install --upgrade pip
  python -m pip install virtualenv
  python -m virtualenv venv
  ```

### PyInstaller Issues
- If you get "No module named" errors, try:
  ```
  pip install pyinstaller
  pip install -r requirements.txt
  ```

### Inno Setup Issues
- Verify Inno Setup 6 is installed
- The default installation paths are:
  - `C:\Program Files (x86)\Inno Setup 6`
  - `C:\Program Files\Inno Setup 6`

### Icon Issues
- If the icon doesn't appear, ensure:
  - `soplang_icon.ico` exists in the windows directory
  - It's a valid .ico file (multiple resolutions)
  - Try converting it with an online tool if ImageMagick fails

### File Association Issues
- If the `.sop` or `.so` file associations aren't working:
  - Make sure you selected the file association option during installation
  - Try repairing the installation through Windows Control Panel
  - Verify there are no conflicting file associations

## Creating a Release

Once you've verified the build works, you can create a release:

1. Copy these files to a release directory:
   - `windows\Output\soplang-setup.exe` (installer)
   - Create a ZIP archive of the `..\dist\soplang\` directory (for users who prefer not to use the installer)
   - `..\README.md` (documentation)
   - `..\examples\` directory (example files)

2. Upload these files to your release platform (GitHub Releases, etc.)
