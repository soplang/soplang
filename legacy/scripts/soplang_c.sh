#!/bin/bash

# Soplang C Interpreter Runner
# This script automatically sets PYTHONPATH and runs the C-based Soplang interpreter

# Get the directory where the script is located
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

# Check for debug flag
DEBUG=0
if [ "$1" == "--debug" ]; then
    DEBUG=1
    shift
fi

if [ $# -lt 1 ]; then
    echo "Usage: $0 [--debug] <soplang_file.so>"
    echo ""
    echo "Options:"
    echo "  --debug    Enable debug output"
    echo ""
    echo "Available examples:"
    ls -1 "$SCRIPT_DIR/examples/"*.so
    exit 1
fi

# Set PYTHONPATH to include the script directory
export PYTHONPATH="$SCRIPT_DIR"

# Run the C-based interpreter with strace if debug is enabled
if [ $DEBUG -eq 1 ]; then
    echo "Running in debug mode..."

    # Get Python version from the main.py file
    PY_VER=$(python -V 2>&1 | cut -d' ' -f2 | cut -d'.' -f1-2)
    echo "Python version: $PY_VER"

    # Check if the C interpreter exists
    if [ ! -f "$SCRIPT_DIR/csrc/main.out" ]; then
        echo "Error: C interpreter not found at $SCRIPT_DIR/csrc/main.out"
        exit 1
    fi

    # Run with strace to see what's happening
    strace -f "$SCRIPT_DIR/csrc/main.out" "$1" 2> debug_output.log

    echo "Debug output written to debug_output.log"
else
    # Run normally
    "$SCRIPT_DIR/csrc/main.out" "$@"
fi
