#!/bin/bash

# Run all Soplang examples

echo "=== Running all Soplang examples ==="
echo ""

# Gather all .sop files (primary) and .so files (secondary) in the examples directory
examples=$(find examples -name "*.sop")
# Also add .so files that don't have a corresponding .sop version
for so_file in $(find examples -name "*.so"); do
    sop_equivalent=${so_file%.so}.sop
    if [ ! -f "$sop_equivalent" ]; then
        examples+=" $so_file"
    fi
done

# Run each example
for example in $examples; do
    echo "====================================================="
    echo "Running example: $example"
    echo "====================================================="
    # Add --no-interactive if available to prevent input prompts from blocking
    python main.py $example || echo "Failed to run $example"
    echo ""
    echo "Press Enter to continue to the next example..."
    read
done

echo "=== All examples completed ==="
