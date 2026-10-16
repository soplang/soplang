# Soplang build guide

Step-by-step instructions for building Soplang from source on **Windows**, **macOS**, and **Linux**. The project is implemented in Rust; all platforms use the same codebase and Cargo.

---

## Prerequisites (all platforms)

- **Rust** — Install the stable toolchain from [rustup.rs](https://rustup.rs).  
  After installation, verify:
  ```bash
  rustc --version
  cargo --version
  ```

- **Git** — To clone the repository.

---

## Quick build (any platform)

From the project root:

```bash
git clone https://github.com/soplang/soplang.git
cd soplang
cargo build --release
```

- **Windows:** binary at `target\release\soplang.exe`
- **macOS / Linux:** binary at `target/release/soplang`

Run:

```bash
# Run a script
./target/release/soplang examples/01_hello.sop    # macOS/Linux
target\release\soplang.exe examples\01_hello.sop  # Windows

# Interactive REPL
./target/release/soplang -i
```

---

## Windows

### Prerequisites

- **Rust** — [rustup.rs](https://rustup.rs) (use the “Visual Studio C++ Build Tools” option if prompted).
- **Inno Setup 6** (optional) — For building the installer: [jrsoftware.org/isdl.php](https://jrsoftware.org/isdl.php).

### Build steps

1. Open **PowerShell** or **Command Prompt** and go to the project root:
   ```cmd
   cd path\to\soplang
   ```

2. Run the Windows build script:
   - **PowerShell:**
     ```powershell
     .\windows\build_windows.ps1
     ```
   - **Command Prompt:**
     ```cmd
     windows\build_windows.bat
     ```

3. The script will:
   - Run `cargo build --release`
   - Copy `soplang.exe` to `dist\soplang\`
   - If Inno Setup 6 is installed, create `windows\Output\soplang-setup.exe`

### Output

| Item        | Path |
|------------|------|
| Binary     | `target\release\soplang.exe` |
| Dist copy  | `dist\soplang\soplang.exe` |
| Installer  | `windows\Output\soplang-setup.exe` (if Inno Setup used) |

### Custom installer icon

Place a 256×256 (or larger) **soplang_icon.ico** in the `windows\` folder before building. The Inno Setup script uses it for the setup and app icon.

### Test the build

From the project root (PowerShell):

```powershell
.\windows\test_windows_build.ps1
```

### Troubleshooting

- **“cargo not found”** — Install Rust with rustup and restart the terminal. Ensure “Add to PATH” was selected.
- **Linker errors** — On Windows, rustup may prompt for “Visual Studio C++ Build Tools”; install them and retry.
- **Inno Setup not found** — Build still produces the binary and `dist\soplang\`; only the installer is skipped.

---

## macOS

### Prerequisites

- **Rust** — [rustup.rs](https://rustup.rs).
- **create-dmg** (optional) — For building a disk image: `brew install create-dmg`.

### Build steps

1. Open Terminal and go to the project root:
   ```bash
   cd /path/to/soplang
   ```

2. Run the build (either way):
   ```bash
   ./build.sh
   ```
   or:
   ```bash
   chmod +x macos/build_macos.sh
   ./macos/build_macos.sh
   ```

3. The script will:
   - Run `cargo build --release`
   - Create **dist/Soplang.app** (macOS application bundle)
   - If `create-dmg` is installed, create **macos/Soplang-2.0.0.dmg**

### Output

| Item      | Path |
|-----------|------|
| Binary    | `target/release/soplang` |
| App bundle| `dist/Soplang.app` |
| DMG      | `macos/Soplang-2.0.0.dmg` (if create-dmg installed) |

### Run

```bash
./target/release/soplang examples/01_hello.sop
./target/release/soplang -i
open dist/Soplang.app
```

### Distribution (optional)

For distribution you may want to sign and notarize:

```bash
codesign --force --sign "Developer ID Application: Your Name" dist/Soplang.app
# Then notarize the DMG with Apple (see Apple’s documentation).
```

### Troubleshooting

- **“cargo not found”** — Install Rust via rustup; restart the terminal.
- **“create-dmg not found”** — Optional; install with `brew install create-dmg` if you need a DMG.

---

## Linux

### Prerequisites

- **Rust** — [rustup.rs](https://rustup.rs).
- **dpkg-dev**, **fakeroot** (optional) — For building a .deb package:
  - Debian/Ubuntu: `sudo apt install dpkg-dev fakeroot`
  - Fedora: `sudo dnf install dpkg fakeroot` (if you need .deb on non-Debian systems).

### Build steps

1. Open a terminal and go to the project root:
   ```bash
   cd /path/to/soplang
   ```

2. Run the build (either way):
   ```bash
   ./build.sh
   ```
   or:
   ```bash
   chmod +x linux/build_linux.sh
   ./linux/build_linux.sh
   ```

3. The script will:
   - Run `cargo build --release`
   - Copy the binary to **dist/soplang/soplang**
   - Create **linux/soplang-2.0.0-linux-&lt;arch&gt;.tar.gz**
   - If `dpkg-deb` is available, create **linux/soplang_2.0.0_amd64.deb** (or your architecture)

### Output

| Item     | Path |
|----------|------|
| Binary   | `target/release/soplang` |
| Dist     | `dist/soplang/soplang` |
| Tarball  | `linux/soplang-2.0.0-linux-<arch>.tar.gz` |
| .deb     | `linux/soplang_2.0.0_amd64.deb` (if dpkg-deb available) |

### Install from .deb

```bash
sudo dpkg -i linux/soplang_2.0.0_amd64.deb
soplang examples/01_hello.sop
```

### Run without installing

```bash
./target/release/soplang examples/01_hello.sop
./target/release/soplang -i
```

### Troubleshooting

- **“cargo not found”** — Install Rust via rustup; restart the terminal.
- **No .deb produced** — Install `dpkg-dev` and `fakeroot` (see Prerequisites).

---

## Summary

| Platform | Build command (from repo root)     | Optional packaging        |
|----------|------------------------------------|---------------------------|
| Windows  | `windows\build_windows.bat` or `.\windows\build_windows.ps1` | Inno Setup → `.exe` installer |
| macOS    | `./build.sh` or `./macos/build_macos.sh` | create-dmg → `.dmg` |
| Linux    | `./build.sh` or `./linux/build_linux.sh` | dpkg-deb → `.deb`, tarball |

For a minimal build on any platform, `cargo build --release` is enough; platform scripts add packaging and installer creation.
