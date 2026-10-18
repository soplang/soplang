# Soplang 3 for Windows

The [v3.0.1 release](https://github.com/soplang/soplang/releases/tag/v3.0.1)
provides a Windows x64 ZIP and installer. Native debug/release suites,
JIT/LLVM package checks, and installer PATH/file-association removal are
covered by the [release gates](../docs/testing/LOCALIZATION_3_0_1.md). The installer is
unsigned. See [Installing Soplang](../docs/installation.md) for installation.

## Build packages locally

Install Rust via rustup, Python 3.9+, Visual Studio C++ Build Tools / Windows SDK,
and LLVM clang. Open a developer shell with the linker available. Inno Setup 6
is optional for building the installer.

```powershell
./windows/build_windows.ps1
# Command Prompt: windows\build_windows.bat
```

The locked builder stages `dist/soplang` with runtime libraries and produces
`windows/soplang-<version>-windows-<arch>.zip`. With Inno Setup it also produces
`windows/Output/soplang-setup.exe`. Failures in Cargo, packaging, checksumming,
or Inno Setup stop the build. `windows/SHA256SUMS` records actual assets.

```powershell
python scripts/verify_package.py dist/soplang/soplang.exe
# Or: ./windows/test_windows_build.ps1
```

Repeat the checker on the extracted ZIP and use
`python scripts/verify_installation.py windows windows/Output/soplang-setup.exe`
on a clean, disposable Windows runner. This installs, exercises, and removes
the actual package, including PATH and file associations. See
[package requirements](../docs/INSTALLING_PACKAGES.md) and the
[verification record](../docs/testing/LOCALIZATION_3_0_1.md).
