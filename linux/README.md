# Soplang for Linux

This directory contains the files needed to build Soplang for Linux, including the executable and installer packages.

## Prerequisites

Before building Soplang for Linux, you need:

1. **Python 3.6 or higher** - Install from your distribution's package manager:
   ```
   # Debian/Ubuntu
   sudo apt install python3 python3-pip python3-venv

   # Fedora/RHEL
   sudo dnf install python3 python3-pip
   ```

2. **ImageMagick** (for icon conversion) - Install from your distribution's package manager:
   ```
   # Debian/Ubuntu
   sudo apt install imagemagick

   # Fedora/RHEL
   sudo dnf install imagemagick
   ```

3. **Packaging Tools** (optional, for creating .deb/.rpm packages):
   ```
   # Debian/Ubuntu
   sudo apt install dpkg-dev

   # Fedora/RHEL
   sudo dnf install rpm-build
   ```

## Building Soplang for Linux

Building Soplang on Linux is simple:

1. Navigate to the Soplang linux directory:
   ```bash
   cd soplang/linux
   ```

2. Make the build script executable (if needed):
   ```bash
   chmod +x build_linux.sh
   ```

3. Run the build script:
   ```bash
   ./build_linux.sh
   ```

The script will guide you through the build process and create:
- A standalone executable
- A .deb package (on Debian-based systems)
- An .rpm package (on Red Hat-based systems, if RPM tools are installed)

## Build Output

After the build process completes successfully, you will find:

- **Standalone executable**: `dist/soplang/soplang`
- **Debian package** (on Debian-based systems): `linux/soplang_<version>_amd64.deb`
- **RPM package** (on Red Hat-based systems): `linux/soplang-<version>.x86_64.rpm`

## Custom Icon

If you want to use a custom icon for Soplang on Linux:

1. Make the icon preparation script executable:
   ```bash
   chmod +x prepare_logos.sh
   ```

2. Run the icon preparation script:
   ```bash
   ./prepare_logos.sh
   ```

3. Follow the prompts to select or provide a path to your logo image
   - The script will create icons in various sizes required for Linux
   - These icons will be used in the application launcher and file associations

## Installation

### Debian/Ubuntu and Derivatives

If you built a .deb package:
```bash
sudo dpkg -i linux/soplang_<version>_amd64.deb
```

If there are dependency issues:
```bash
sudo apt install -f
```

### Fedora/RHEL and Derivatives

If you built an .rpm package:
```bash
sudo rpm -i linux/soplang-<version>.x86_64.rpm
```

### Manual Installation

You can also manually install the standalone executable:

1. Copy the entire `dist/soplang` directory to `/opt/soplang`:
   ```bash
   sudo mkdir -p /opt/soplang
   sudo cp -r dist/soplang/* /opt/soplang/
   ```

2. Create a symbolic link to make Soplang available in PATH:
   ```bash
   sudo ln -s /opt/soplang/soplang /usr/local/bin/soplang
   ```

3. Create desktop entry for application menus:
   ```bash
   echo "[Desktop Entry]
   Name=Soplang Interpreter
   Comment=The Somali Programming Language
   Exec=/usr/local/bin/soplang %f
   Icon=/opt/soplang/soplang_icon.png
   Terminal=true
   Type=Application
   Categories=Development;Education;
   MimeType=text/x-soplang;
   Keywords=soplang;programming;development;somali;" | sudo tee /usr/share/applications/soplang.desktop
   ```

4. Set up file associations:
   ```bash
   echo "text/x-soplang=soplang.desktop" | sudo tee -a /usr/share/applications/defaults.list
   ```

## After Installation

After installing Soplang, you can:
- Run Soplang from the application menu
- Open a terminal and type `soplang` to start the interpreter
- Double-click `.sop` or `.so` files to run them with Soplang

## Troubleshooting

- **Missing Python**: Ensure Python 3.6+ is installed with `python3 --version`
- **Missing dependencies**: Run `pip3 install -r requirements.txt`
- **Permission issues**: Make sure scripts are executable with `chmod +x *.sh`
- **Package conflicts**: On Debian/Ubuntu, try `sudo apt --fix-broken install`
