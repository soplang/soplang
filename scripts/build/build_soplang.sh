#!/bin/bash
# ==================================================
# Soplang Build Script
# Compiles Soplang components using Nuitka for both 
# Windows and Unix platforms
# ==================================================

# Set current timestamp for build ID
BUILD_ID=$(date +"%Y%m%d_%H%M%S")
echo "Starting Soplang build process (ID: $BUILD_ID)"
echo "====================================================="

# Check if Nuitka is installed
if ! python -c "import nuitka" &> /dev/null; then
    echo "❌ Error: Nuitka is not installed. Please install it with:"
    echo "   pip install nuitka"
    exit 1
fi

# Create output directories
echo "Creating output directories..."
mkdir -p dist/win dist/unix

# Display build environment info
echo "====================================================="
echo "Build Environment:"
echo "Python version: $(python --version)"
echo "Nuitka version: $(python -m nuitka --version 2>&1 | head -1)"
echo "System: $(uname -a)"
echo "====================================================="

# Record start time for entire build
BUILD_START_TIME=$(date +%s)

# Compile for Linux/macOS (Unix)
echo "Building Unix versions..."
UNIX_START_TIME=$(date +%s)

echo "Compiling main.py..."
time python -m nuitka --onefile --standalone --output-dir=dist/unix --output-filename=soplang main.py

echo "Compiling components..."
time python -m nuitka --onefile --standalone --output-dir=dist/unix --output-filename=shell src/shell.py
time python -m nuitka --onefile --standalone --output-dir=dist/unix --output-filename=interpreter src/interpreter.py
time python -m nuitka --onefile --standalone --output-dir=dist/unix --output-filename=lexer src/lexer.py
time python -m nuitka --onefile --standalone --output-dir=dist/unix --output-filename=parser src/parser.py
time python -m nuitka --onefile --standalone --output-dir=dist/unix --output-filename=tokens src/tokens.py

UNIX_END_TIME=$(date +%s)
UNIX_BUILD_TIME=$((UNIX_END_TIME - UNIX_START_TIME))
echo "Unix build completed in $UNIX_BUILD_TIME seconds."
echo "====================================================="

# Check if mingw is available for Windows build
if command -v x86_64-w64-mingw32-gcc &> /dev/null; then
    echo "Building Windows versions..."
    WIN_START_TIME=$(date +%s)
    
    echo "Compiling main.py..."
    time python -m nuitka --onefile --standalone --mingw64 --output-dir=dist/win --output-filename=soplang.exe main.py
    
    echo "Compiling components..."
    time python -m nuitka --onefile --standalone --mingw64 --output-dir=dist/win --output-filename=shell.exe src/shell.py
    time python -m nuitka --onefile --standalone --mingw64 --output-dir=dist/win --output-filename=interpreter.exe src/interpreter.py
    time python -m nuitka --onefile --standalone --mingw64 --output-dir=dist/win --output-filename=lexer.exe src/lexer.py
    time python -m nuitka --onefile --standalone --mingw64 --output-dir=dist/win --output-filename=parser.exe src/parser.py
    time python -m nuitka --onefile --standalone --mingw64 --output-dir=dist/win --output-filename=tokens.exe src/tokens.py
    
    WIN_END_TIME=$(date +%s)
    WIN_BUILD_TIME=$((WIN_END_TIME - WIN_START_TIME))
    echo "Windows build completed in $WIN_BUILD_TIME seconds."
    echo "====================================================="
else
    echo "⚠️ MinGW compiler not found. Skipping Windows build."
    echo "To build for Windows, install MinGW with:"
    echo "   sudo apt-get install mingw-w64 (for Ubuntu/Debian)"
    echo "   brew install mingw-w64 (for macOS with Homebrew)"
    WIN_BUILD_TIME=0
fi

# Calculate total build time
BUILD_END_TIME=$(date +%s)
TOTAL_BUILD_TIME=$((BUILD_END_TIME - BUILD_START_TIME))

# Verify output
echo "✅ Compilation Complete!"
echo "====================================================="
echo "Unix Binaries:"
ls -lh dist/unix/

if [ $WIN_BUILD_TIME -gt 0 ]; then
    echo "Windows Binaries:"
    ls -lh dist/win/
fi

# Create a simple report
echo "====================================================="
echo "Build Summary (ID: $BUILD_ID):"
echo "Total build time: $TOTAL_BUILD_TIME seconds"
echo "Unix build time: $UNIX_BUILD_TIME seconds"
if [ $WIN_BUILD_TIME -gt 0 ]; then
    echo "Windows build time: $WIN_BUILD_TIME seconds"
fi
echo "Build completed on: $(date)"
echo "====================================================="

# Create run scripts
echo "Creating launcher scripts..."

# Unix script
cat > dist/unix/soplang.sh <<EOF
#!/bin/bash
# Soplang Launcher Script
SCRIPT_DIR="\$(cd "\$(dirname "\${BASH_SOURCE[0]}")" && pwd)"
"\$SCRIPT_DIR/soplang" "\$@"
EOF
chmod +x dist/unix/soplang.sh

# Windows batch file
cat > dist/win/soplang.bat <<EOF
@echo off
REM Soplang Launcher Script
"%~dp0soplang.exe" %*
EOF

echo "✅ All done! Launcher scripts created."
echo "---------------------------------------------------"
echo "To run Soplang:"
echo "  Linux/macOS: ./dist/unix/soplang.sh"
echo "  Windows: dist\\win\\soplang.bat"
echo "=====================================================" 