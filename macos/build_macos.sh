#!/bin/bash
# Soplang macOS Build — build Rust binary and optional .app bundle / DMG.
# Prerequisites: Rust (rustup.rs), optionally create-dmg (brew install create-dmg).

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$PROJECT_ROOT"

GREEN='\033[0;32m'
CYAN='\033[0;36m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'
VERSION="$(sed -n 's/^version = "\([^"]*\)"/\1/p' Cargo.toml | head -n 1)"
if [[ -z "$VERSION" ]]; then
    echo -e "${RED}Error: could not read the package version from Cargo.toml.${NC}"
    exit 1
fi

echo -e "${CYAN}Soplang macOS Build${NC}"
echo -e "${CYAN}===================${NC}"

# Check Rust
if ! command -v cargo &>/dev/null; then
    echo -e "${RED}Error: cargo not found. Install Rust from https://rustup.rs${NC}"
    exit 1
fi

# Build release binary
echo -e "${CYAN}Building release binary...${NC}"
cargo build --release
BINARY="$PROJECT_ROOT/target/release/soplang"
if [[ ! -f "$BINARY" ]]; then
    echo -e "${RED}Build failed: binary not found.${NC}"
    exit 1
fi
echo -e "${GREEN}Binary: $BINARY${NC}"

# Optional: create .app bundle
mkdir -p dist/Soplang.app/Contents/MacOS
cp "$BINARY" dist/Soplang.app/Contents/MacOS/soplang
chmod +x dist/Soplang.app/Contents/MacOS/soplang

# Minimal Info.plist
cat > dist/Soplang.app/Contents/Info.plist << PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleExecutable</key>
    <string>soplang</string>
    <key>CFBundleIdentifier</key>
    <string>org.soplang</string>
    <key>CFBundleName</key>
    <string>Soplang</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleShortVersionString</key>
    <string>$VERSION</string>
    <key>CFBundleVersion</key>
    <string>1</string>
    <key>LSMinimumSystemVersion</key>
    <string>10.13</string>
    <key>NSHighResolutionCapable</key>
    <true/>
</dict>
</plist>
PLIST

echo -e "${GREEN}App bundle: dist/Soplang.app${NC}"

# Optional: create DMG if create-dmg is available
if command -v create-dmg &>/dev/null; then
    DMG_NAME="Soplang-$VERSION.dmg"
    echo -e "${CYAN}Creating disk image...${NC}"
    rm -f "macos/$DMG_NAME"
    create-dmg --volname "Soplang" --window-size 500 300 --app-drop-link 380 150 \
        "macos/$DMG_NAME" dist/
    echo -e "${GREEN}DMG: macos/$DMG_NAME${NC}"
else
    echo -e "${YELLOW}Tip: install create-dmg for DMG: brew install create-dmg${NC}"
fi

echo -e "${GREEN}Done.${NC}"
