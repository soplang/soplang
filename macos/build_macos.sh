#!/bin/bash
# Soplang macOS Build Script
# This script builds the Soplang executable and disk image for macOS

set -e  # Exit on any error

# Colors for better output
GREEN='\033[0;32m'
CYAN='\033[0;36m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Navigate to project root (parent of macos directory)
cd "$(dirname "$0")/.."
PROJECT_ROOT="$(pwd)"

echo -e "${CYAN}Building Soplang for macOS from directory: ${PROJECT_ROOT}${NC}"
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

# Check for icon file in the macos directory
if [ ! -f "macos/soplang_icon.icns" ]; then
    echo -e "${YELLOW}Note: Icon file not found in macos directory.${NC}"
    echo -e "${YELLOW}You should prepare a logo before building:${NC}"
    echo -e "${YELLOW}  Option 1: Run 'cd macos && ./prepare_logos.sh'${NC}"
    echo -e "${YELLOW}  Option 2: Manually place an icon file at 'macos/soplang_icon.icns'${NC}"

    read -p "Do you want to continue without an icon? (y/n) " -n 1 -r
    echo    # Move to a new line
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo -e "${RED}Build aborted. Please prepare a logo file first.${NC}"
        exit 1
    fi

    # Create a placeholder icon if continuing
    echo -e "${YELLOW}Creating a placeholder icon...${NC}"
    touch "macos/soplang_icon.icns"
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
echo -e "${CYAN}Installing macOS-specific dependencies...${NC}"
pip install -r macos/requirements_macos.txt

# Install development package
echo -e "${CYAN}Installing Soplang in development mode...${NC}"
pip install -e .

# Modify the spec file to use macOS icon if needed
if grep -q "icon_file = os.path.join('windows', 'soplang_icon.ico')" soplang.spec; then
    echo -e "${CYAN}Updating spec file for macOS...${NC}"
    # Create a backup of the original spec file
    cp soplang.spec soplang.spec.bak
    # Replace the icon path based on platform
    sed -i.bak "s|icon_file = os.path.join('windows', 'soplang_icon.ico')|icon_file = os.path.join('macos', 'soplang_icon.icns') if platform.system() == 'Darwin' else os.path.join('windows', 'soplang_icon.ico')|g" soplang.spec
    # Add platform import if needed
    sed -i.bak "1s|^|import platform\n|" soplang.spec
fi

# Build with PyInstaller
echo -e "${CYAN}Building executable with PyInstaller...${NC}"
pyinstaller soplang.spec

# Create a macOS app bundle
echo -e "${CYAN}Creating macOS app bundle...${NC}"
APP_PATH="dist/Soplang.app"
mkdir -p "${APP_PATH}/Contents/MacOS"
mkdir -p "${APP_PATH}/Contents/Resources"

# Copy the executable and supporting files
cp -r dist/soplang/* "${APP_PATH}/Contents/MacOS/"

# Create Info.plist
APP_VERSION=$(grep "version" setup.py | head -1 | cut -d '"' -f 2)
cat > "${APP_PATH}/Contents/Info.plist" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleDevelopmentRegion</key>
    <string>English</string>
    <key>CFBundleDisplayName</key>
    <string>Soplang</string>
    <key>CFBundleExecutable</key>
    <string>soplang</string>
    <key>CFBundleIconFile</key>
    <string>soplang_icon</string>
    <key>CFBundleIdentifier</key>
    <string>org.soplang</string>
    <key>CFBundleInfoDictionaryVersion</key>
    <string>6.0</string>
    <key>CFBundleName</key>
    <string>Soplang</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleShortVersionString</key>
    <string>${APP_VERSION}</string>
    <key>CFBundleVersion</key>
    <string>${APP_VERSION}</string>
    <key>NSHumanReadableCopyright</key>
    <string>Copyright © 2024 Soplang Software Foundation</string>
    <key>NSPrincipalClass</key>
    <string>NSApplication</string>
    <key>NSHighResolutionCapable</key>
    <true/>
    <key>CFBundleDocumentTypes</key>
    <array>
        <dict>
            <key>CFBundleTypeExtensions</key>
            <array>
                <string>sop</string>
                <string>so</string>
            </array>
            <key>CFBundleTypeIconFile</key>
            <string>soplang_icon</string>
            <key>CFBundleTypeName</key>
            <string>Soplang Source Code</string>
            <key>CFBundleTypeRole</key>
            <string>Editor</string>
            <key>LSIsAppleDefaultForType</key>
            <true/>
        </dict>
    </array>
</dict>
</plist>
EOF

# Copy icon to Resources
cp "macos/soplang_icon.icns" "${APP_PATH}/Contents/Resources/"

# Create launcher script
cat > "${APP_PATH}/Contents/MacOS/SoplangLauncher" << 'EOF'
#!/bin/bash
cd "$(dirname "$0")"
./soplang "$@"
EOF
chmod +x "${APP_PATH}/Contents/MacOS/SoplangLauncher"

# Create DMG if create-dmg is available
if command -v create-dmg &> /dev/null; then
    echo -e "${CYAN}Creating DMG image...${NC}"
    DMG_PATH="macos/Soplang-${APP_VERSION}.dmg"

    # Remove any existing DMG
    if [ -f "$DMG_PATH" ]; then
        rm "$DMG_PATH"
    fi

    # Create the DMG
    create-dmg \
        --volname "Soplang Installer" \
        --volicon "macos/soplang_icon.icns" \
        --window-pos 200 120 \
        --window-size 800 400 \
        --icon-size 100 \
        --icon "Soplang.app" 200 190 \
        --app-drop-link 600 185 \
        --no-internet-enable \
        "$DMG_PATH" \
        "dist/Soplang.app"

    echo -e "${GREEN}DMG created at: $DMG_PATH${NC}"
else
    echo -e "${YELLOW}create-dmg not found. Skipping DMG creation.${NC}"
    echo -e "${YELLOW}To create a DMG, install create-dmg:${NC}"
    echo -e "${YELLOW}  brew install create-dmg${NC}"
    echo -e "${YELLOW}Then run: create-dmg --volname 'Soplang Installer' --volicon 'macos/soplang_icon.icns' --window-pos 200 120 --window-size 800 400 --icon-size 100 --icon 'Soplang.app' 200 190 --app-drop-link 600 185 --no-internet-enable 'macos/Soplang-${APP_VERSION}.dmg' 'dist/Soplang.app'${NC}"
fi

echo -e "${GREEN}Build completed successfully!${NC}"
echo -e "${GREEN}App bundle: ${PROJECT_ROOT}/dist/Soplang.app${NC}"
if command -v create-dmg &> /dev/null; then
    echo -e "${GREEN}DMG installer: ${PROJECT_ROOT}/macos/Soplang-${APP_VERSION}.dmg${NC}"
fi

# Return to the macos directory
cd "$(dirname "$0")"
