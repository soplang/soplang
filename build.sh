#!/bin/bash
# Soplang Universal Build Script
# This script detects your platform and runs the appropriate build script
#
# Note: This is part of the platform-specific build system. Legacy build scripts
# from earlier versions of Soplang have been moved to the scripts/build directory
# and are kept for reference only.

# Colors for better output
GREEN='\033[0;32m'
CYAN='\033[0;36m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${CYAN}Soplang Universal Build Script${NC}"
echo -e "${CYAN}=============================${NC}"
echo ""

# Detect platform
if [[ "$OSTYPE" == "darwin"* ]]; then
    # macOS
    PLATFORM="macOS"
    BUILD_SCRIPT="macos/build_macos.sh"
elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
    # Linux
    PLATFORM="Linux"
    BUILD_SCRIPT="linux/build_linux.sh"
elif [[ "$OSTYPE" == "msys" || "$OSTYPE" == "cygwin" || "$OSTYPE" == "win32" ]]; then
    # Windows with Git Bash, Cygwin, or similar
    PLATFORM="Windows"
    BUILD_SCRIPT="windows/build_windows.ps1"
else
    # Unknown platform
    echo -e "${RED}Unsupported platform: $OSTYPE${NC}"
    echo -e "${YELLOW}Please build manually using:${NC}"
    echo -e "  - ${YELLOW}Windows:${NC} windows/build_windows.ps1"
    echo -e "  - ${YELLOW}Linux:${NC} linux/build_linux.sh"
    echo -e "  - ${YELLOW}macOS:${NC} macos/build_macos.sh"
    echo -e "${YELLOW}For legacy build scripts, see scripts/build directory.${NC}"
    exit 1
fi

echo -e "${GREEN}Detected platform: ${PLATFORM}${NC}"
echo -e "Will use build script: ${BUILD_SCRIPT}"
echo ""

# Check if the build script exists
if [ ! -f "$BUILD_SCRIPT" ]; then
    echo -e "${RED}Build script not found: $BUILD_SCRIPT${NC}"
    echo -e "${YELLOW}Please make sure you're running this script from the root of the Soplang repository.${NC}"
    echo -e "${YELLOW}If you're experiencing issues with the new build system, you can try the legacy scripts:${NC}"
    echo -e "  - ${YELLOW}scripts/build/build_soplang.sh${NC} (for all platforms)"
    echo -e "  - ${YELLOW}scripts/build/compile_c_version.sh${NC} (for C implementation)"
    echo -e "  - ${YELLOW}scripts/build/compile_python_version.sh${NC} (for Python implementation)"
    exit 1
fi

# Check if the build script is executable (for Linux/macOS)
if [[ "$PLATFORM" == "Linux" || "$PLATFORM" == "macOS" ]]; then
    if [ ! -x "$BUILD_SCRIPT" ]; then
        echo -e "${YELLOW}Making build script executable...${NC}"
        chmod +x "$BUILD_SCRIPT"
    fi
fi

# Run the appropriate build script
echo -e "${CYAN}Starting build process for ${PLATFORM}...${NC}"
echo -e "${CYAN}==================================${NC}"
echo ""

if [[ "$PLATFORM" == "Windows" ]]; then
    # Windows - use PowerShell
    if command -v powershell.exe &> /dev/null; then
        powershell.exe -File "$BUILD_SCRIPT"
    else
        echo -e "${RED}PowerShell not found. Please run the build script manually:${NC}"
        echo -e "${YELLOW}  windows/build_windows.ps1${NC}"
        exit 1
    fi
else
    # Linux/macOS - use bash
    "./$BUILD_SCRIPT"
fi

# Check if build was successful
if [ $? -eq 0 ]; then
    echo -e "${GREEN}Build completed successfully for ${PLATFORM}!${NC}"

    # Display build artifacts location based on platform
    if [[ "$PLATFORM" == "Windows" ]]; then
        echo -e "${GREEN}Executable: dist/soplang/soplang.exe${NC}"
        echo -e "${GREEN}Installer: windows/Output/soplang-setup.exe${NC}"
    elif [[ "$PLATFORM" == "Linux" ]]; then
        echo -e "${GREEN}Executable: dist/soplang/soplang${NC}"
        echo -e "${GREEN}Debian package: linux/soplang_*_amd64.deb (if built)${NC}"
        echo -e "${GREEN}RPM package: linux/soplang-*.rpm (if built)${NC}"
    elif [[ "$PLATFORM" == "macOS" ]]; then
        echo -e "${GREEN}App bundle: dist/Soplang.app${NC}"
        echo -e "${GREEN}DMG installer: macos/Soplang-*.dmg (if built)${NC}"
    fi
else
    echo -e "${RED}Build failed for ${PLATFORM}.${NC}"
    echo -e "${YELLOW}Please check the error messages above and try again.${NC}"
    echo -e "${YELLOW}If you continue to experience issues, try the legacy build scripts in scripts/build/.${NC}"
    exit 1
fi
