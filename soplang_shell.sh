#!/bin/bash

# Soplang Interactive Shell Runner
# This script launches the interactive Soplang shell

# Get the directory where the script is located
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

# Set PYTHONPATH to include the script directory
export PYTHONPATH="$SCRIPT_DIR"

# Make sure we're in the right directory
cd "$SCRIPT_DIR"

# Check for debug mode
if [ "$1" == "--debug" ]; then
    # Run in debug mode
    echo "Running Soplang shell in debug mode..."
    python3 -m pdb "$SCRIPT_DIR/src/shell.py" "${@:2}"
    exit $?
fi

# Run the shell with any provided arguments
python3 "$SCRIPT_DIR/src/shell.py" "$@"
exit $? 