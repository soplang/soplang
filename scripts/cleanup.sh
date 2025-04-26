#!/bin/bash
# Soplang Project Structure Cleanup Script
# This script organizes the project by moving redundant scripts to a legacy directory

# Colors for output
GREEN='\033[0;32m'
CYAN='\033[0;36m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${CYAN}Soplang Project Structure Cleanup${NC}"
echo -e "${CYAN}=================================${NC}"
echo ""

# Create a legacy directory to hold outdated scripts
echo -e "${YELLOW}Creating legacy directory...${NC}"
mkdir -p legacy/scripts

# List of scripts that are now redundant with the platform-specific build system
REDUNDANT_SCRIPTS=(
  "soplang_c.sh"
  "soplang_py.sh"
  "soplang_py_optimized"
  "soplang_shell.sh"
)

# Move redundant scripts to legacy directory
for script in "${REDUNDANT_SCRIPTS[@]}"; do
  if [ -f "$script" ]; then
    echo -e "${YELLOW}Moving $script to legacy/scripts/${NC}"
    mv "$script" legacy/scripts/
  fi
done

# Create a README in the legacy directory
echo -e "${YELLOW}Creating README in legacy directory...${NC}"
cat > legacy/README.md << 'EOF'
# Legacy Soplang Scripts

This directory contains scripts that were used in earlier versions of Soplang but have been replaced by the new platform-specific build system.

These scripts are kept for reference but are no longer maintained. Please use the platform-specific build scripts instead:

- **Windows**: `windows/build_windows.ps1` or `windows/build_windows.bat`
- **Linux**: `linux/build_linux.sh`
- **macOS**: `macos/build_macos.sh`

Or use the universal build script:

```
./build.sh
```

## Script Descriptions

- `soplang_c.sh` - Runner script for the C implementation
- `soplang_py.sh` - Runner script for the Python implementation
- `soplang_py_optimized` - Runner script for the optimized Python implementation
- `soplang_shell.sh` - Runner script for the interactive shell
EOF

echo -e "${GREEN}Cleanup complete!${NC}"
echo "The following scripts have been moved to legacy/scripts/:"
for script in "${REDUNDANT_SCRIPTS[@]}"; do
  if [ -f "legacy/scripts/$script" ]; then
    echo "- $script"
  fi
done

echo ""
echo -e "${CYAN}Next steps:${NC}"
echo "1. Review the contents of the legacy directory to ensure nothing important was moved"
echo "2. If everything looks good, commit the changes to your repository"
echo "3. Use the new platform-specific build scripts for all future development"
