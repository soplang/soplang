#!/bin/bash
# Soplang Logo Preparation Script for Linux
# This script helps convert logo files to Linux formats

# Colors for better output
GREEN='\033[0;32m'
CYAN='\033[0;36m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${CYAN}Soplang Logo Preparation for Linux${NC}"
echo -e "${CYAN}=====================================${NC}"
echo ""

# Check if we should regenerate the icon, even if it exists
REGENERATE=false
if [ -f "soplang_icon.png" ]; then
    read -p "Icon file already exists. Do you want to regenerate it? (y/n) " -n 1 -r
    echo    # Move to a new line
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        REGENERATE=true
        echo -e "${CYAN}Will regenerate the icon.${NC}"
    fi
fi

# Check if the ImageMagick convert command is available
if command -v convert &> /dev/null; then
    HAS_IMAGEMAGICK=true
else
    HAS_IMAGEMAGICK=false
    echo -e "${YELLOW}NOTE: ImageMagick is not installed.${NC}"
    echo -e "${YELLOW}For best results, install ImageMagick with: sudo apt-get install imagemagick${NC}"
    echo -e "${YELLOW}Or for Fedora/RHEL: sudo dnf install imagemagick${NC}"
    echo -e "${YELLOW}Or manually convert your logo to PNG format.${NC}"
    echo ""
    echo -e "${CYAN}Icon Tips for Professional Look:${NC}"
    echo -e "${CYAN}1. Use a clean, simple design that's recognizable even at small sizes${NC}"
    echo -e "${CYAN}2. Use a transparent background${NC}"
    echo -e "${CYAN}3. Ensure high contrast for visibility in menus and desktop environments${NC}"
    echo -e "${CYAN}4. Create a square image before converting${NC}"
    echo -e "${CYAN}5. Prepare multiple sizes: 16x16, 32x32, 48x48, 128x128, 256x256${NC}"
    echo ""
fi

# Get project root (parent of linux directory)
PROJECT_ROOT=$(dirname "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)")

# Check if we should proceed with icon creation
if [ -f "soplang_icon.png" ] && [ "$REGENERATE" = false ]; then
    echo -e "${GREEN}Using existing icon file: soplang_icon.png${NC}"
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
    echo "Please enter the full path to your logo file:"
    read -r SELECTED_LOGO

    if [ ! -f "$SELECTED_LOGO" ]; then
        echo -e "${RED}File not found: $SELECTED_LOGO${NC}"
        exit 1
    fi
fi

echo -e "${GREEN}Selected logo: $SELECTED_LOGO${NC}"

# Path for the icon file (in the current linux directory)
ICON_PATH="soplang_icon.png"

# Convert the logo to different icon formats if ImageMagick is available
if [ "$HAS_IMAGEMAGICK" = true ]; then
    echo -e "${CYAN}Converting logo to icons using ImageMagick...${NC}"

    # Create icons of different sizes for Linux
    for SIZE in 16 32 48 64 128 256; do
        convert "$SELECTED_LOGO" -resize ${SIZE}x${SIZE} -background transparent "soplang_icon_${SIZE}.png"
        echo -e "${GREEN}Created icon: soplang_icon_${SIZE}.png${NC}"
    done

    # Create a main icon (use the 256x256 version as the default)
    cp "soplang_icon_256.png" "$ICON_PATH"

    # Create an XPM version for desktop environments that require it
    if command -v convert &> /dev/null; then
        convert "$ICON_PATH" "soplang_icon.xpm"
        echo -e "${GREEN}Created XPM icon: soplang_icon.xpm${NC}"
    fi

    if [ -f "$ICON_PATH" ]; then
        echo -e "${GREEN}Icons created successfully!${NC}"
    else
        echo -e "${RED}Failed to create icons. Please convert the logo manually.${NC}"
    fi
else
    # Copy the file without conversion if ImageMagick isn't available
    echo -e "${YELLOW}Copying logo file without conversion...${NC}"
    cp "$SELECTED_LOGO" "$ICON_PATH"
    echo -e "${YELLOW}Copied logo to: $ICON_PATH${NC}"
    echo -e "${YELLOW}NOTE: You may need to manually create different icon sizes for best results.${NC}"
    echo -e "${YELLOW}Try using GIMP or another image editor to create 16x16, 32x32, 48x48, 128x128, and 256x256 versions.${NC}"
fi

echo ""
echo -e "${GREEN}Logo preparation complete!${NC}"
echo -e "${CYAN}The icons are now ready in the linux folder.${NC}"
echo -e "${CYAN}You can now run the build script to build the Linux installer.${NC}"
