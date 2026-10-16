# Soplang for Windows

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

Repeat the checker on the extracted ZIP. Test installer installation/removal,
PATH changes, and file associations on a clean Windows machine. Native JIT and
LLVM calling conventions must pass the Windows CI job before claiming support;
this Linux-hosted session cannot establish that result. See
[package requirements](../docs/INSTALLING_PACKAGES.md) and the
[verification record](../docs/testing/RELEASE_VERIFICATION.md).
