# Soplang Installation Guide

This guide provides detailed instructions for installing Soplang on Windows, Linux, and macOS platforms.

## Quick Start

For the fastest way to get started:

1. **Download Soplang**: Get the latest version from our [official Soplang releases page](https://github.com/sharafdin/soplang/releases)
2. **Choose your installer**: Select the appropriate installer for your platform (.exe for Windows, .deb/.rpm for Linux, .dmg for macOS)
3. **Follow platform-specific instructions**: See the relevant section below for your operating system

## Table of Contents

- [Soplang Installation Guide](#soplang-installation-guide)
  - [Quick Start](#quick-start)
  - [Table of Contents](#table-of-contents)
  - [Universal Installation Method](#universal-installation-method)
  - [Windows Installation](#windows-installation)
    - [Using the Installer](#using-the-installer)
    - [Building from Source on Windows](#building-from-source-on-windows)
      - [Prerequisites](#prerequisites)
      - [Build Steps](#build-steps)
    - [Troubleshooting Windows Installation](#troubleshooting-windows-installation)
  - [Linux Installation](#linux-installation)
    - [Using Package Manager](#using-package-manager)
      - [Debian/Ubuntu and Derivatives](#debianubuntu-and-derivatives)
      - [Fedora/RHEL and Derivatives](#fedorarhel-and-derivatives)
    - [Building from Source on Linux](#building-from-source-on-linux)
      - [Prerequisites](#prerequisites-1)
      - [Build Steps](#build-steps-1)
    - [Troubleshooting Linux Installation](#troubleshooting-linux-installation)
  - [macOS Installation](#macos-installation)
    - [Using the DMG Image](#using-the-dmg-image)
    - [Building from Source on macOS](#building-from-source-on-macos)
      - [Prerequisites](#prerequisites-2)
      - [Build Steps](#build-steps-2)
    - [Troubleshooting macOS Installation](#troubleshooting-macos-installation)
  - [Verifying Your Installation](#verifying-your-installation)
  - [Next Steps](#next-steps)

## Universal Installation Method

The easiest way to build Soplang on any platform is to use the universal build script:

```bash
# Clone the repository
git clone https://github.com/sharafdin/soplang.git
cd soplang

# Make the script executable (Unix systems)
chmod +x build.sh

# Run the script
./build.sh
```

This script automatically detects your platform (Windows, Linux, or macOS) and runs the appropriate build script.

## Windows Installation

### Using the Installer

1. Download the Soplang installer (`soplang-setup.exe`) from the [official Soplang releases page](https://github.com/sharafdin/soplang/releases).
2. Run the installer and follow the on-screen instructions.
3. During installation, you can choose to:
   - Add Soplang to the PATH environment variable
   - Create desktop and Start menu shortcuts
   - Associate `.sop` and `.so` files with Soplang

After installation, you can:
- Run Soplang from the Start menu
- Double-click `.sop` or `.so` files to run them with Soplang
- Open a Command Prompt and type `soplang` to start the interpreter

### Building from Source on Windows

#### Prerequisites

- Python 3.6 or higher - [Download Python](https://www.python.org/downloads/windows/)
- Inno Setup 6 (for creating the installer) - [Download Inno Setup](https://jrsoftware.org/isdl.php)
- Git - [Download Git](https://git-scm.com/download/win)

#### Build Steps

1. Clone the repository:
   ```powershell
   git clone https://github.com/sharafdin/soplang.git
   cd soplang/windows
   ```

2. Run the build script (PowerShell recommended):
   ```powershell
   .\build_windows.ps1
   ```

   Or using Command Prompt:
   ```cmd
   build_windows.bat
   ```

3. After the build completes, you'll find:
   - Standalone executable: `dist\soplang\soplang.exe`
   - Windows installer: `windows\Output\soplang-setup.exe`

4. Run the installer to install Soplang on your system.

### Troubleshooting Windows Installation

- **Missing Python**: Ensure Python is installed and added to PATH
- **Missing dependencies**: Run `pip install -r windows\requirements_windows.txt`
- **Inno Setup errors**: Ensure Inno Setup 6 is installed properly
- **PyInstaller errors**: Try `pip install --upgrade pyinstaller`
- **"Publisher: Unknown" warning**: This is normal if the installer isn't digitally signed
- **File association issues**: Ensure you selected the file association option during installation

## Linux Installation

### Using Package Manager

#### Debian/Ubuntu and Derivatives

1. Download the `.deb` package from the [official Soplang releases page](https://github.com/sharafdin/soplang/releases).
2. Install using:
```bash
sudo dpkg -i soplang_<version>_amd64.deb
```

If there are dependency issues:
```bash
sudo apt install -f
```

#### Fedora/RHEL and Derivatives

1. Download the `.rpm` package from the [official Soplang releases page](https://github.com/sharafdin/soplang/releases).
2. Install using:
```bash
sudo rpm -i soplang-<version>.x86_64.rpm
```

### Building from Source on Linux

#### Prerequisites

1. Install Python 3.6 or higher:
   ```bash
   # Debian/Ubuntu
   sudo apt install python3 python3-pip python3-venv

   # Fedora/RHEL
   sudo dnf install python3 python3-pip
   ```

2. Install ImageMagick (for icon conversion):
   ```bash
   # Debian/Ubuntu
   sudo apt install imagemagick

   # Fedora/RHEL
   sudo dnf install imagemagick
   ```

3. Install packaging tools (optional):
   ```bash
   # Debian/Ubuntu
   sudo apt install dpkg-dev

   # Fedora/RHEL
   sudo dnf install rpm-build
   ```

#### Build Steps

1. Clone the repository:
   ```bash
   git clone https://github.com/sharafdin/soplang.git
   cd soplang/linux
   ```

2. Make the build script executable:
   ```bash
   chmod +x build_linux.sh
   ```

3. Run the build script:
   ```bash
   ./build_linux.sh
   ```

4. After the build completes, you'll find:
   - Standalone executable: `dist/soplang/soplang`
   - Debian package (Debian-based systems): `linux/soplang_<version>_amd64.deb`
   - RPM package (Red Hat-based systems): `linux/soplang-<version>.x86_64.rpm`

5. Install the appropriate package for your system (see [Using Package Manager](#using-package-manager)).

### Troubleshooting Linux Installation

- **Missing Python**: Ensure Python 3.6+ is installed with `python3 --version`
- **Missing dependencies**: Run `pip3 install -r linux/requirements_linux.txt`
- **Permission issues**: Make sure scripts are executable with `chmod +x *.sh`
- **Package conflicts**: On Debian/Ubuntu, try `sudo apt --fix-broken install`
- **Executable not found**: Ensure Soplang is in your PATH or try running `/usr/bin/soplang`

## macOS Installation

### Using the DMG Image

1. Download the Soplang disk image (`Soplang-<version>.dmg`) from the [official Soplang releases page](https://github.com/sharafdin/soplang/releases).
2. Open the `.dmg` file.
3. Drag `Soplang.app` to your Applications folder.
4. Eject the disk image.

After installation, you can:
- Launch Soplang from the Applications folder or Dock
- Double-click `.sop` or `.so` files to open them with Soplang
- Open Terminal and run `/Applications/Soplang.app/Contents/MacOS/soplang`

### Building from Source on macOS

#### Prerequisites

1. Install Python 3.6 or higher:
   ```bash
   # Using Homebrew (recommended)
   brew install python

   # Or download from Python.org
   # https://www.python.org/downloads/mac-osx/
   ```

2. Install create-dmg (optional, for creating disk image):
   ```bash
   brew install create-dmg
   ```

#### Build Steps

1. Clone the repository:
   ```bash
   git clone https://github.com/sharafdin/soplang.git
   cd soplang/macos
   ```

2. Make the build script executable:
   ```bash
   chmod +x build_macos.sh
   ```

3. Run the build script:
   ```bash
   ./build_macos.sh
   ```

4. After the build completes, you'll find:
   - App Bundle: `dist/Soplang.app`
   - Disk Image (if create-dmg is installed): `macos/Soplang-<version>.dmg`

5. Drag `Soplang.app` to your Applications folder or run the DMG installer.

### Troubleshooting macOS Installation

- **Build errors**: Make sure Python 3.6+ and pip are properly installed
- **Icon creation fails**: Ensure you're using a high-quality source image
- **App won't open**: Check Console.app for logs about why the app was prevented from opening
- **"App is damaged"**: This is a security feature of macOS. Open System Preferences → Security & Privacy and click "Open Anyway" for Soplang
- **"Unknown developer" warning**: Control-click (or right-click) the app and select "Open" from the shortcut menu

## Verifying Your Installation

After installing Soplang, verify that it's working properly:

1. Open a terminal or command prompt and run:
   ```bash
   soplang -v
   ```
   You should see the Soplang version information.

2. Run a simple Soplang program:
   ```bash
   soplang -c 'qor("Hello from Soplang!")'
   ```
   You should see the output: `Hello from Soplang!`

3. Try the interactive shell:
   ```bash
   soplang
   ```
   You should see the Soplang welcome message and prompt.

## Next Steps

Now that you have Soplang installed, you can:

1. [Learn the basics of Soplang](language/keywords.md)
2. [Explore example programs](examples/)
3. [Create your first Soplang program](language/tutorial.md)
4. [Contribute to Soplang development](CONTRIBUTING.md)

For more information, visit the [Soplang website](https://www.soplang.org/).
