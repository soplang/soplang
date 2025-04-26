# Soplang Scripts Directory

This directory contains utility scripts for Soplang development, testing, and benchmarking. The main build scripts have been moved to platform-specific directories (`windows/`, `linux/`, `macos/`).

## Directory Structure

- **benchmark/** - Scripts for performance testing and comparison
  - `benchmark.sh` - Basic performance benchmarking
  - `compare_all_implementations.sh` - Compare C, Python, and interpreted implementations
  - `compare_performance.sh` - Detailed performance metrics

- **test/** - Test runner scripts
  - `run_all_tests.sh` - Run the full test suite
  - `run_examples.sh` - Run and validate the example programs

- **build/** - Legacy build scripts
  - Note: These are kept for reference but the recommended build system is now in the platform-specific folders

## Platform-Specific Build Systems

For building Soplang, please use the dedicated platform-specific build systems:

- **Windows**: Use `windows/build_windows.ps1` or `windows/build_windows.bat`
- **Linux**: Use `linux/build_linux.sh`
- **macOS**: Use `macos/build_macos.sh`

Or simply use the universal build script in the root directory:

```
./build.sh
```

This script will automatically detect your platform and run the appropriate build script.

## Legacy Build Scripts

The following legacy build scripts are kept for reference:

- `build_soplang.sh` - Builds Soplang with Nuitka for both Windows and Unix platforms
- `compile_c_version.sh` - Compiles the C implementation
- `compile_python_version.sh` - Compiles the Python implementation with PyInstaller

These scripts are not actively maintained and may not be compatible with the latest Soplang structure.
