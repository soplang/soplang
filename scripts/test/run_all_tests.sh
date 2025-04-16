#!/bin/bash

# Soplang Test Runner Script
# This script runs all Soplang test files in sequence

echo "=== Soplang Test Runner ==="
echo "Running all test examples..."
echo

# Test files to run
TEST_FILES=(
    "examples/01_basic_output.so"
    "examples/02_user_input.so"
    "examples/03_conditionals.so"
    "examples/04_loops.so"
    "examples/05_dynamic_typing.so"
    "examples/06_static_typing.so"
    "examples/07_functions.so"
    "examples/08_lists.so"
    "examples/09_objects.so"
    "examples/10_error_handling.so"
    "examples/automated_test.so"
)

# Run each test file
for test_file in "${TEST_FILES[@]}"; do
    if [ -f "$test_file" ]; then
        echo "=== Running $test_file ==="
        python main.py "$test_file"
        echo
        echo "=== Test Complete: $test_file ==="
        echo "Press Enter to continue to the next test..."
        read
        echo
    else
        echo "ERROR: Test file not found: $test_file"
    fi
done

echo "=== All tests completed ===" 