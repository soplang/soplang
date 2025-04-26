# Legacy Soplang Scripts

This directory contains scripts that were used in earlier versions of Soplang but have been replaced by the new platform-specific build system.

These scripts are kept for reference but are no longer maintained. Please use the platform-specific build scripts instead:

- **Windows**: `windows/build_windows.ps1` or `windows/build_windows.bat`
- **Linux**: `linux/build_linux.sh`
- **macOS**: `macos/build_macos.sh`

Or use the universal build script:

```
./build.sh
```

## Script Descriptions

- `soplang_c.sh` - Runner script for the C implementation
- `soplang_py.sh` - Runner script for the Python implementation
- `soplang_py_optimized` - Runner script for the optimized Python implementation
- `soplang_shell.sh` - Runner script for the interactive shell
