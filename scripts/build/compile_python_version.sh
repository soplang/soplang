#!/bin/bash

# Soplang Python Compilation Script
# This script compiles the Python implementation of Soplang using PyInstaller

echo "=== Compiling Soplang Python Implementation ==="

# Check if PyInstaller is installed
if ! command -v pyinstaller &> /dev/null; then
    echo "PyInstaller is not installed. Attempting to install now..."
    
    # Create a temporary virtual environment to avoid import conflicts
    python3 -m venv /tmp/soplang_venv
    source /tmp/soplang_venv/bin/activate
    
    # Install PyInstaller in the virtual environment
    pip install pyinstaller
    
    if [ $? -ne 0 ]; then
        echo "Failed to install PyInstaller. Please install it manually:"
        echo "  pip install pyinstaller"
        exit 1
    fi
fi

# Create output directory
mkdir -p py_compiled

# Get the full path to the workspace
WORKSPACE_PATH=$(pwd)

# Compilation command
echo "=== Compiling main.py ==="
echo "This may take a few minutes..."

# We need to temporarily rename our src/ast.py to avoid conflicts
if [ -f "./src/ast.py" ]; then
    echo "Temporarily renaming src/ast.py to src/ast_soplang.py to avoid conflicts..."
    cp ./src/ast.py ./src/ast_soplang.py
    
    # Update imports in other files
    for file in ./src/*.py; do
        if [ "$file" != "./src/ast_soplang.py" ]; then
            sed -i 's/from src.ast/from src.ast_soplang/g' "$file"
            sed -i 's/import src.ast/import src.ast_soplang/g' "$file"
        fi
    done
fi

# Use PyInstaller to compile the main.py script
# --onefile: Create a single executable
# --distpath: Output directory
# --name: Name of the output file
# --clean: Clean PyInstaller cache before building
pyinstaller --onefile --distpath py_compiled --name soplang_py_compiled --clean main.py

# Restore original file names and imports
if [ -f "./src/ast_soplang.py" ]; then
    echo "Restoring original file names and imports..."
    for file in ./src/*.py; do
        if [ "$file" != "./src/ast_soplang.py" ]; then
            sed -i 's/from src.ast_soplang/from src.ast/g' "$file"
            sed -i 's/import src.ast_soplang/import src.ast/g' "$file"
        fi
    done
    
    # Keep both files around in case we need them
    echo "Original ast.py preserved"
fi

# Check if compilation succeeded
if [ ! -f "./py_compiled/soplang_py_compiled" ]; then
    echo "Compilation failed!"
    exit 1
fi

# Make the compiled file executable
chmod +x ./py_compiled/soplang_py_compiled

# Create a runner script
echo "Creating runner script..."
cat > soplang_py_compiled <<EOL
#!/bin/bash

# Get the directory where the script is located
SCRIPT_DIR="\$( cd "\$( dirname "\${BASH_SOURCE[0]}" )" && pwd )"

# Set PYTHONPATH to include the script directory
export PYTHONPATH="\$SCRIPT_DIR"

# Run the compiled interpreter
"\$SCRIPT_DIR/py_compiled/soplang_py_compiled" "\$@"
EOL

chmod +x soplang_py_compiled
echo "Created soplang_py_compiled runner script"

echo "=== Compilation complete ==="
echo "Run your Soplang programs with: ./soplang_py_compiled your_program.so" 