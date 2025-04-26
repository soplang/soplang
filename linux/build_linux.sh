#!/bin/bash
# Soplang Linux Build Script
# This script builds the Soplang executable and installer for Linux systems

set -e  # Exit on any error

# Colors for better output
GREEN='\033[0;32m'
CYAN='\033[0;36m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Navigate to project root (parent of linux directory)
cd "$(dirname "$0")/.."
PROJECT_ROOT="$(pwd)"

echo -e "${CYAN}Building Soplang for Linux from directory: ${PROJECT_ROOT}${NC}"
echo -e "${CYAN}=====================================================${NC}"

# Check for required tools
if ! command -v python3 &> /dev/null; then
    echo -e "${RED}Error: Python 3 not found. Please install Python 3.6 or higher.${NC}"
    exit 1
fi

if ! command -v pip3 &> /dev/null; then
    echo -e "${RED}Error: pip3 not found. Please install pip for Python 3.${NC}"
    exit 1
fi

# Check for icon file in the linux directory
if [ ! -f "linux/soplang_icon.png" ]; then
    echo -e "${YELLOW}Note: Icon file not found in linux directory.${NC}"
    echo -e "${YELLOW}You should prepare a logo before building:${NC}"
    echo -e "${YELLOW}  Option 1: Run 'cd linux && ./prepare_logos.sh'${NC}"
    echo -e "${YELLOW}  Option 2: Manually place an icon file at 'linux/soplang_icon.png'${NC}"

    read -p "Do you want to continue without an icon? (y/n) " -n 1 -r
    echo    # Move to a new line
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo -e "${RED}Build aborted. Please prepare a logo file first.${NC}"
        exit 1
    fi

    # Create a placeholder icon if continuing
    echo -e "${YELLOW}Creating a placeholder icon...${NC}"
    touch "linux/soplang_icon.png"
fi

# Remove any existing venv to start fresh
if [ -d "venv" ]; then
    echo -e "${CYAN}Removing existing virtual environment...${NC}"
    rm -rf venv
fi

# Create a virtual environment
echo -e "${CYAN}Creating virtual environment...${NC}"
python3 -m venv venv

# Activate the virtual environment
echo -e "${CYAN}Activating virtual environment...${NC}"
source venv/bin/activate

# Install dependencies
echo -e "${CYAN}Installing Linux-specific dependencies...${NC}"
pip install -r linux/requirements_linux.txt

# Install development package
echo -e "${CYAN}Installing Soplang in development mode...${NC}"
pip install -e .

# Build with PyInstaller
echo -e "${CYAN}Building executable with PyInstaller...${NC}"
pyinstaller soplang.spec

# Create Debian package if dpkg-deb is available
if command -v dpkg-deb &> /dev/null; then
    echo -e "${CYAN}Creating Debian package...${NC}"

    # Create the package structure
    PACKAGE_DIR="linux/soplang_package"
    DEB_VERSION=$(grep "version" setup.py | head -1 | cut -d '"' -f 2)

    # Clean up any previous build
    rm -rf "$PACKAGE_DIR"

    # Create directory structure
    mkdir -p "$PACKAGE_DIR/DEBIAN"
    mkdir -p "$PACKAGE_DIR/usr/local/bin"
    mkdir -p "$PACKAGE_DIR/usr/local/share/soplang"
    mkdir -p "$PACKAGE_DIR/usr/local/share/applications"
    mkdir -p "$PACKAGE_DIR/usr/local/share/pixmaps"

    # Copy executable and support files
    cp -r dist/soplang/* "$PACKAGE_DIR/usr/local/share/soplang/"

    # Create launcher script
    cat > "$PACKAGE_DIR/usr/local/bin/soplang" << 'EOF'
#!/bin/bash
/usr/local/share/soplang/soplang "$@"
EOF
    chmod +x "$PACKAGE_DIR/usr/local/bin/soplang"

    # Create desktop entry
    cat > "$PACKAGE_DIR/usr/local/share/applications/soplang.desktop" << EOF
[Desktop Entry]
Name=Soplang Interpreter
Comment=The Somali Programming Language
Exec=/usr/local/bin/soplang %f
Icon=soplang
Terminal=true
Type=Application
Categories=Development;Education;
MimeType=text/x-soplang;
Keywords=soplang;programming;development;somali;
StartupNotify=true
StartupWMClass=soplang
EOF

    # Copy icon
    cp "linux/soplang_icon.png" "$PACKAGE_DIR/usr/local/share/pixmaps/soplang.png"

    # Create control file
    cat > "$PACKAGE_DIR/DEBIAN/control" << EOF
Package: soplang
Version: $DEB_VERSION
Section: development
Priority: optional
Architecture: amd64
Depends: libc6, python3
Maintainer: Sharafdin <info@soplang.org>
Description: The Somali Programming Language
 Soplang is a programming language with syntax inspired by Somali language,
 making programming more accessible to Somali speakers. It combines static
 and dynamic typing systems in one elegant language with a focus on clarity
 and ease of use.
EOF

    # Create postinst script for file associations
    cat > "$PACKAGE_DIR/DEBIAN/postinst" << 'EOF'
#!/bin/bash
set -e

# Update desktop database and icon cache
if [ -x "$(command -v update-desktop-database)" ]; then
    update-desktop-database -q
fi

if [ -x "$(command -v gtk-update-icon-cache)" ]; then
    gtk-update-icon-cache -f -t -q /usr/share/icons/hicolor
fi

# Create file associations for .sop and .so files
if [ -x "$(command -v xdg-mime)" ]; then
    # Register MIME types
    xdg-mime install --novendor /usr/local/share/soplang/soplang-mime.xml
    # Set default application
    xdg-mime default soplang.desktop text/x-soplang
fi

exit 0
EOF
    chmod 755 "$PACKAGE_DIR/DEBIAN/postinst"

    # Create postrm script
    cat > "$PACKAGE_DIR/DEBIAN/postrm" << 'EOF'
#!/bin/bash
set -e

if [ "$1" = "remove" ] || [ "$1" = "purge" ]; then
    # Remove file associations
    if [ -x "$(command -v xdg-mime)" ]; then
        xdg-mime uninstall --novendor /usr/local/share/soplang/soplang-mime.xml
    fi
fi

exit 0
EOF
    chmod 755 "$PACKAGE_DIR/DEBIAN/postrm"

    # Create MIME type definition
    mkdir -p "$PACKAGE_DIR/usr/local/share/soplang"
    cat > "$PACKAGE_DIR/usr/local/share/soplang/soplang-mime.xml" << 'EOF'
<?xml version="1.0" encoding="UTF-8"?>
<mime-info xmlns="http://www.freedesktop.org/standards/shared-mime-info">
  <mime-type type="text/x-soplang">
    <comment>Soplang source code</comment>
    <glob pattern="*.sop"/>
    <glob pattern="*.so"/>
    <icon name="soplang"/>
  </mime-type>
</mime-info>
EOF

    # Build the package
    dpkg-deb --build "$PACKAGE_DIR" "linux/soplang_${DEB_VERSION}_amd64.deb"

    echo -e "${GREEN}Debian package created at linux/soplang_${DEB_VERSION}_amd64.deb${NC}"
fi

# Create RPM package if rpmbuild is available
if command -v rpmbuild &> /dev/null; then
    echo -e "${CYAN}Creating RPM package...${NC}"
    echo -e "${YELLOW}RPM packaging will be implemented in a future version${NC}"
    # This would require creating spec files and using rpmbuild
fi

echo -e "${GREEN}Build completed successfully!${NC}"
echo -e "${GREEN}Executable: ${PROJECT_ROOT}/dist/soplang/soplang${NC}"
if command -v dpkg-deb &> /dev/null; then
    echo -e "${GREEN}Debian package: ${PROJECT_ROOT}/linux/soplang_${DEB_VERSION}_amd64.deb${NC}"
fi

# Return to the linux directory
cd "$(dirname "$0")"
