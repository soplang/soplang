# Soplang for Windows

Build Soplang (Rust) on Windows and optionally create an installer with Inno Setup.

**Full build guide (all platforms):** [../docs/BUILD_GUIDE.md](../docs/BUILD_GUIDE.md)

## Prerequisites

- **Rust** — [rustup.rs](https://rustup.rs)
- **Inno Setup 6** (optional, for installer) — [jrsoftware.org/isdl.php](https://jrsoftware.org/isdl.php)

## Build

From the **project root** (PowerShell or Command Prompt):

**PowerShell (recommended):**
```powershell
.\windows\build_windows.ps1
```

**Command Prompt:**
```cmd
windows\build_windows.bat
```

The script will:

1. Run `cargo build --release`
2. Copy **soplang.exe** to **dist\soplang\**
3. If Inno Setup 6 is installed, create **windows\Output\soplang-setup.exe**

## Output

- **Binary:** `target\release\soplang.exe`
- **Dist:** `dist\soplang\soplang.exe`
- **Installer:** `windows\Output\soplang-setup.exe` (if Inno Setup installed)

## Run

```cmd
target\release\soplang.exe examples\01_hello.sop
target\release\soplang.exe -i
```

## Custom icon

Place **soplang_icon.ico** in the `windows\` folder before building the installer; the script uses it for the setup and app icon.
