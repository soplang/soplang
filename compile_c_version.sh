#!/bin/bash

# Soplang C Compilation Script
# This script compiles all the C files for the Soplang interpreter

echo "=== Compiling Soplang C Implementation ==="

# Python version detection
PYTHON_VERSION=$(python3 -c 'import sys; print(f"{sys.version_info.major}.{sys.version_info.minor}")')
echo "Detected Python version: $PYTHON_VERSION"

# Python include path detection
PYTHON_INCLUDE=$(python3-config --includes | cut -d' ' -f1 | sed 's/-I//')
if [ -z "$PYTHON_INCLUDE" ]; then
    # Fallback to common paths
    PYTHON_INCLUDE="/usr/include/python$PYTHON_VERSION"
fi
echo "Python include path: $PYTHON_INCLUDE"

# Python library detection
PYTHON_LIB=$(python3-config --libs | grep -o -- "-lpython[^ ]*" | sed 's/-l//')
if [ -z "$PYTHON_LIB" ]; then
    # Fallback to common naming
    PYTHON_LIB="python$PYTHON_VERSION"
fi
echo "Python library: $PYTHON_LIB"

# Compilation flags
CFLAGS="-Wall -O3 -march=native"
LDFLAGS="-L/usr/lib -l$PYTHON_LIB"
echo "Compiler flags: $CFLAGS"
echo "Linker flags: $LDFLAGS"

# Create a directory for the object files if it doesn't exist
mkdir -p obj
mkdir -p csrc/bin

# Source files
FILES=(
    "main"
    "interpreter"
    "lexer"
    "parser"
    "tokens"
    "builtins"
    "ast"
    "errors"
)

# Step 1: Compile each C file into an object file
echo "=== Compiling object files ==="
for file in "${FILES[@]}"; do
    echo "Compiling $file.c into object file..."
    gcc -c -o obj/$file.o csrc/$file.c -I$PYTHON_INCLUDE $CFLAGS
    
    if [ $? -ne 0 ]; then
        echo "Error compiling $file.c"
        exit 1
    fi
done

# Step 2: Link all object files into a single executable
echo "=== Linking object files ==="
echo "Creating main executable..."
gcc -o csrc/bin/main.out obj/*.o $LDFLAGS

if [ $? -ne 0 ]; then
    echo "Error linking object files"
    exit 1
fi

echo "=== All files compiled successfully ==="
echo "Executable is available at csrc/bin/main.out"

# Create a symlink to the main output file in the top directory
echo "Creating symlink to executable..."
ln -sf csrc/bin/main.out csrc/main.out

# Create a main script to run the compiled code
echo "Creating runner script..."
cat > soplang_c <<EOL
#!/bin/bash

# Get the directory where the script is located
SCRIPT_DIR="\$( cd "\$( dirname "\${BASH_SOURCE[0]}" )" && pwd )"

# Set PYTHONPATH to include the script directory
export PYTHONPATH="\$SCRIPT_DIR"

# Run the compiled interpreter
"\$SCRIPT_DIR/csrc/bin/main.out" "\$@"
EOL

chmod +x soplang_c
echo "Created soplang_c runner script"

echo "=== Compilation complete ==="
echo "Run your Soplang programs with: ./soplang_c your_program.so" 