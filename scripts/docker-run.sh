#!/bin/bash
# Helper script for running Soplang in Docker with proper file mounting

# Check if Docker is installed
if ! command -v docker &> /dev/null; then
    echo "Error: Docker is not installed. Please install Docker first."
    exit 1
fi

# Display usage information if no arguments provided
if [ $# -eq 0 ]; then
    echo "Soplang Docker Runner"
    echo "Usage: ./docker-run.sh [path/to/script.sop]"
    echo ""
    echo "This script runs Soplang in Docker with proper file mounting."
    echo "If no script is provided, it starts the interactive shell."
    echo ""
    echo "Examples:"
    echo "  ./docker-run.sh                           # Start interactive shell"
    echo "  ./docker-run.sh examples/hello.sop        # Run a specific script"
    echo ""
    echo "Starting Soplang interactive shell..."
    docker run -it --rm soplang/soplang
    exit 0
fi

# If a script path is provided, run it
SCRIPT_PATH="$1"
SCRIPT_DIR=$(dirname "$SCRIPT_PATH")
SCRIPT_NAME=$(basename "$SCRIPT_PATH")

# Ensure the script exists
if [ ! -f "$SCRIPT_PATH" ]; then
    echo "Error: Script file not found: $SCRIPT_PATH"
    exit 1
fi

# Run the script using Docker with proper volume mounting
if [ "$SCRIPT_DIR" = "." ]; then
    # Script is in current directory
    echo "Running $SCRIPT_NAME in Docker..."
    docker run -it --rm -v "$(pwd):/scripts" soplang/soplang "$SCRIPT_NAME"
else
    # Script is in a subdirectory - mount from project root
    echo "Running $SCRIPT_PATH in Docker..."
    docker run -it --rm -v "$(pwd):/scripts" soplang/soplang "$SCRIPT_PATH"
fi
