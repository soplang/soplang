#!/bin/bash
# Soplang Docker Helper Script
#
# This is just a convenience script to make running Soplang in Docker easier.
# You can absolutely run Docker manually with these commands:
#
# Build the image:
#   docker build -t soplang:latest .
#
# Run the interactive shell:
#   docker run --rm -it -v "$(pwd):/scripts" soplang:latest
#
# Run a specific file:
#   docker run --rm -it -v "$(pwd):/scripts" soplang:latest "path/to/your/file.so"

# Check if Docker image exists, if not build it
if ! docker image inspect soplang:latest >/dev/null 2>&1; then
    echo "Soplang image not found. Building it now..."
    docker build -t soplang:latest .
fi

# Function to run the interactive shell
run_shell() {
    echo "Starting Soplang interactive shell..."
    docker run --rm -it -v "$(pwd):/scripts" soplang:latest
}

# Function to run a specific file
run_file() {
    local file="$1"
    if [ ! -f "$file" ]; then
        echo "Error: File '$file' not found."
        exit 1
    fi

    echo "Running $file with Soplang..."
    docker run --rm -it -v "$(pwd):/scripts" soplang:latest "$file"
}

# Parse command line arguments
if [ $# -eq 0 ]; then
    run_shell
else
    run_file "$1"
fi
