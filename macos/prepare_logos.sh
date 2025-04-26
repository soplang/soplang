#!/bin/bash
# Soplang Logo Preparation Script for macOS
# This script helps convert logo files to macOS .icns format

# Colors for better output
GREEN='\033[0;32m'
CYAN='\033[0;36m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${CYAN}Soplang Logo Preparation for macOS${NC}"
echo -e "${CYAN}=====================================${NC}"
echo ""

# Check if we should regenerate the icon, even if it exists
REGENERATE=false
if [ -f "soplang_icon.icns" ]; then
    read -p "Icon file already exists. Do you want to regenerate it? (y/n) " -n 1 -r
    echo    # Move to a new line
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        REGENERATE=true
        echo -e "${CYAN}Will regenerate the icon.${NC}"
    fi
fi

# Check if the required tools are available
HAS_ICONUTIL=false
if command -v iconutil &> /dev/null; then
    HAS_ICONUTIL=true
else
    echo -e "${YELLOW}NOTE: iconutil not found. This tool is required on macOS.${NC}"
    echo -e "${YELLOW}If you're not on macOS, you'll need to create the .icns file manually.${NC}"
fi

HAS_SIPS=false
if command -v sips &> /dev/null; then
    HAS_SIPS=true
else
    echo -e "${YELLOW}NOTE: sips not found. This tool is normally available on macOS.${NC}"
fi

if [ "$HAS_ICONUTIL" = false ] || [ "$HAS_SIPS" = false ]; then
    echo -e "${YELLOW}macOS icon creation requires iconutil and sips, which are built into macOS.${NC}"
    echo -e "${YELLOW}If you're running this script on Linux or Windows, you'll need to:${NC}"
    echo -e "${YELLOW}1. Use a macOS machine to create the icon${NC}"
    echo -e "${YELLOW}2. Use a third-party tool to create the .icns file${NC}"
    echo -e "${YELLOW}3. Manually place the .icns file in this directory${NC}"
    echo ""
    echo -e "${CYAN}Icon Tips for Professional Look:${NC}"
    echo -e "${CYAN}1. Use a clean, simple design that's recognizable even at small sizes${NC}"
    echo -e "${CYAN}2. Use a transparent background${NC}"
    echo -e "${CYAN}3. Ensure high contrast for visibility in Dock and Finder${NC}"
    echo -e "${CYAN}4. Create a square image before converting${NC}"
    echo -e "${CYAN}5. Include multiple resolutions (16x16 to 1024x1024)${NC}"
    echo ""
fi

# Get project root (parent of macos directory)
PROJECT_ROOT=$(dirname "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)")

# Check if we should proceed with icon creation
if [ -f "soplang_icon.icns" ] && [ "$REGENERATE" = false ]; then
    echo -e "${GREEN}Using existing icon file: soplang_icon.icns${NC}"
    exit 0
fi

# Try to locate logo files automatically
LOGO_FILES=$(find "$PROJECT_ROOT" -name "*.png" -o -name "*.jpg" -o -name "*.jpeg" | grep -i -E 'logo|icon')

if [ -n "$LOGO_FILES" ]; then
    echo -e "${GREEN}Found potential logo files:${NC}"
    readarray -t LOGOS <<< "$LOGO_FILES"
    for i in "${!LOGOS[@]}"; do
        echo "[$i] ${LOGOS[$i]}"
    done
    echo ""

    read -p "Enter the number of the logo file you want to use, or press Enter to browse: " SELECTED_INDEX

    if [ -n "$SELECTED_INDEX" ]; then
        if [ "$SELECTED_INDEX" -ge 0 ] && [ "$SELECTED_INDEX" -lt "${#LOGOS[@]}" ]; then
            SELECTED_LOGO="${LOGOS[$SELECTED_INDEX]}"
        else
            echo -e "${RED}Invalid selection.${NC}"
            SELECTED_LOGO=""
        fi
    else
        # No automatic selection, prompt for a file
        SELECTED_LOGO=""
    fi
else
    echo "No logo files automatically detected."
    SELECTED_LOGO=""
fi

# If no logo selected from the automatic search, ask for a path
if [ -z "$SELECTED_LOGO" ]; then
    echo "Please enter the full path to your logo file (PNG format recommended):"
    read -r SELECTED_LOGO

    if [ ! -f "$SELECTED_LOGO" ]; then
        echo -e "${RED}File not found: $SELECTED_LOGO${NC}"
        exit 1
    fi
fi

echo -e "${GREEN}Selected logo: $SELECTED_LOGO${NC}"

# Create icon in macOS format if tools are available
if [ "$HAS_ICONUTIL" = true ] && [ "$HAS_SIPS" = true ]; then
    echo -e "${CYAN}Creating macOS .icns icon...${NC}"

    # Create temporary iconset directory
    ICONSET_DIR="soplang.iconset"
    rm -rf "$ICONSET_DIR"
    mkdir -p "$ICONSET_DIR"

    # Generate the various icon sizes
    for SIZE in 16 32 64 128 256 512 1024; do
        # Standard resolution
        sips -z $SIZE $SIZE "$SELECTED_LOGO" --out "$ICONSET_DIR/icon_${SIZE}x${SIZE}.png"

        # Retina resolution (except for the 1024 size)
        if [ $SIZE -lt 512 ]; then
            DOUBLE_SIZE=$((SIZE * 2))
            sips -z $DOUBLE_SIZE $DOUBLE_SIZE "$SELECTED_LOGO" --out "$ICONSET_DIR/icon_${SIZE}x${SIZE}@2x.png"
        fi
    done

    # Convert iconset to icns
    iconutil -c icns "$ICONSET_DIR" -o "soplang_icon.icns"

    # Clean up
    rm -rf "$ICONSET_DIR"

    if [ -f "soplang_icon.icns" ]; then
        echo -e "${GREEN}Icon created successfully: soplang_icon.icns${NC}"
    else
        echo -e "${RED}Failed to create icon. Please create the .icns file manually.${NC}"
    fi
else
    # Copy the file without conversion if tools aren't available
    echo -e "${YELLOW}Copying logo file without conversion...${NC}"
    cp "$SELECTED_LOGO" "soplang_logo.png"
    echo -e "${YELLOW}Copied logo to: soplang_logo.png${NC}"
    echo -e "${YELLOW}NOTE: You'll need to manually convert this to a .icns file.${NC}"
    echo -e "${YELLOW}To convert on macOS, you can use:${NC}"
    echo -e "${YELLOW}  1. Create an iconset directory: mkdir soplang.iconset${NC}"
    echo -e "${YELLOW}  2. Generate different sizes${NC}"
    echo -e "${YELLOW}  3. Run: iconutil -c icns soplang.iconset -o soplang_icon.icns${NC}"
    echo -e "${YELLOW}There are also online converters like:${NC}"
    echo -e "${YELLOW}  - https://cloudconvert.com/png-to-icns${NC}"
    echo -e "${YELLOW}  - https://img2icnsapp.com/${NC}"
fi

echo ""
echo -e "${GREEN}Logo preparation complete!${NC}"
echo -e "${CYAN}The icons are now ready in the macos folder.${NC}"
echo -e "${CYAN}You can now run the build script to build the macOS application.${NC}"
