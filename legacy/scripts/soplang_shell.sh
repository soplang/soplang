#!/bin/bash

# Soplang Interactive Shell Runner
# This script launches the interactive Soplang shell

# Get the directory where the script is located
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

# Set PYTHONPATH to include the script directory
export PYTHONPATH="$SCRIPT_DIR"

# Make sure we're in the right directory
cd "$SCRIPT_DIR"

# Display help if requested
if [ "$1" == "--help" ] || [ "$1" == "-h" ]; then
    echo "Soplang Interactive Shell"
    echo ""
    echo "Usage: ./soplang_shell.sh [OPTIONS] [FILE]"
    echo ""
    echo "Options:"
    echo "  -v, --version       Display Soplang version information"
    echo "  -f, --file FILE     Execute a Soplang file"
    echo "  -e, --example N     Run example program number N"
    echo "  -i, --interactive   Start interactive shell after executing file"
    echo "  -c, --command CODE  Execute Soplang code snippet"
    echo "  --debug             Run shell in debug mode"
    echo "  -h, --help          Display this help message"
    echo ""
    echo "Examples:"
    echo "  ./soplang_shell.sh                           Start interactive shell"
    echo "  ./soplang_shell.sh examples/hello_world.so   Run a Soplang file"
    echo "  ./soplang_shell.sh -e 1                      Run the first example"
    echo "  ./soplang_shell.sh -c 'qor(\"Hello\")'"      Execute code snippet"
    exit 0
fi

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
