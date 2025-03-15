#!/bin/bash

# Run all Soplang examples

echo "=== Running all Soplang examples ==="
echo ""

# Gather all .so files in the examples directory
examples=$(find examples -name "*.so")

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