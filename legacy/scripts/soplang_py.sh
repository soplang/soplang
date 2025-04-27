#!/bin/bash

# Soplang Python Interpreter Runner
# This script automatically sets PYTHONPATH and runs the Python-based Soplang interpreter

# Get the directory where the script is located
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

# Check for debug flag
DEBUG=0
if [ "$1" == "--debug" ]; then
    DEBUG=1
    shift
fi

if [ $# -lt 1 ]; then
    echo "Usage: $0 [--debug] <soplang_file.sop>"
    echo ""
    echo "Options:"
    echo "  --debug    Enable debug output"
    echo ""
    echo "Available examples:"
    ls -1 "$SCRIPT_DIR/examples/"*.sop
    # Also show .so files as secondary
    ls -1 "$SCRIPT_DIR/examples/"*.so 2>/dev/null || true
    exit 1
fi

# Set PYTHONPATH to include the script directory
export PYTHONPATH="$SCRIPT_DIR"

# Run the Python-based interpreter with strace if debug is enabled
if [ $DEBUG -eq 1 ]; then
    echo "Running in debug mode..."

    # Get Python version
    PY_VER=$(python -V 2>&1 | cut -d' ' -f2 | cut -d'.' -f1-2)
    echo "Python version: $PY_VER"

    # Run with strace to see what's happening
    strace -f python "$SCRIPT_DIR/main.py" "$1" 2> debug_output_py.log

    echo "Debug output written to debug_output_py.log"
else
    # Run normally
    python "$SCRIPT_DIR/main.py" "$@"
fi
