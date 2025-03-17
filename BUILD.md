# Building Soplang

This document explains how to build Soplang executables for different platforms using Nuitka.

## Prerequisites

Before building Soplang, you need to have the following installed:

- Python 3.6 or higher
- Nuitka (`pip install nuitka`)
- For Windows builds on a non-Windows system: MinGW-w64

### Installing Dependencies

```bash
# Install Python dependencies
pip install nuitka

# On Ubuntu/Debian for Windows cross-compilation
sudo apt-get install mingw-w64

# On macOS (with Homebrew) for Windows cross-compilation
brew install mingw-w64
```

## Building Soplang

We provide a build script that compiles all Soplang components for both Unix (Linux/macOS) and Windows platforms.

### Quick Build

To build Soplang for all supported platforms:

```bash
./build_soplang.sh
```

The script will:
1. Create output directories (`dist/unix` and `dist/win`)
2. Compile all Soplang components for Unix platforms
3. Compile all Soplang components for Windows (if MinGW is available)
4. Create launcher scripts for easy execution
5. Measure and report the build times

### Build Output

After running the build script, you'll find:

- **Unix binaries** in `dist/unix/`:
  - `soplang` - Main Soplang executable
  - `shell` - Interactive shell component
  - `interpreter`, `lexer`, `parser`, `tokens` - Individual components
  - `soplang.sh` - Launcher script

- **Windows binaries** in `dist/win/`:
  - `soplang.exe` - Main Soplang executable
  - `shell.exe` - Interactive shell component
  - `interpreter.exe`, `lexer.exe`, `parser.exe`, `tokens.exe` - Individual components
  - `soplang.bat` - Launcher script for Windows

## Running Soplang Binaries

### On Unix (Linux/macOS)

```bash
# Using the launcher script
./dist/unix/soplang.sh

# Direct execution
./dist/unix/soplang
```

### On Windows

```cmd
# Using the launcher script
dist\win\soplang.bat

# Direct execution
dist\win\soplang.exe
```

## Advanced Build Options

The build script uses Nuitka's `--onefile` and `--standalone` flags to create single executables that don't require Python to be installed on the target system.

If you need to customize the build process:

1. Edit the `build_soplang.sh` script
2. Add or modify Nuitka options for specific components

## Build Times

The build process can take several minutes, especially for Windows cross-compilation. The script reports build times for:

- Unix builds
- Windows builds 
- Total build time

These times are useful for benchmarking build performance across different systems.

## Troubleshooting

- **MinGW not found**: If you see a warning about MinGW not being found, Windows builds will be skipped. Install MinGW as described in the prerequisites.
- **Build errors**: Check the error messages for missing dependencies or configuration issues.
- **Run errors**: Make sure the executable has the proper permissions. You might need to run `chmod +x` on Unix binaries.

## Creating Installer Packages

For distribution to end users, you may want to create installers:

- For Windows: Consider using NSIS or Inno Setup
- For macOS: Create a DMG file or use `pkgbuild`
- For Linux: Consider `.deb` packages for Debian/Ubuntu or `.rpm` for Red Hat-based systems

These packaging steps are not covered by the build script. 