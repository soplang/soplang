#!/bin/bash

# Soplang Performance Comparison Script
# Compares the performance of Python and C implementations

# Check if an input file is provided
if [ $# -lt 1 ]; then
    echo "Usage: $0 <soplang_file> [iterations]"
    echo "Example: $0 examples/hello_world.so 10"
    exit 1
fi

# Default iterations
ITERATIONS=10
if [ $# -ge 2 ]; then
    ITERATIONS=$2
fi

SOPLANG_FILE=$1

echo "=== Soplang Performance Comparison ==="
echo "File: $SOPLANG_FILE"
echo "Iterations: $ITERATIONS"
echo ""

# Ensure PYTHONPATH is set
export PYTHONPATH=$(pwd)

# Run Python implementation
echo "=== Python Implementation ==="
PYTHON_TOTAL=0
for i in $(seq 1 $ITERATIONS); do
    echo -n "."
    start=$(date +%s.%N)
    python3 main.py $SOPLANG_FILE > /dev/null 2>&1
    end=$(date +%s.%N)
    
    # Calculate duration
    duration=$(echo "$end - $start" | bc)
    PYTHON_TOTAL=$(echo "$PYTHON_TOTAL + $duration" | bc)
done
echo ""

PYTHON_AVG=$(echo "scale=6; $PYTHON_TOTAL / $ITERATIONS" | bc)
echo "Total time: $PYTHON_TOTAL seconds"
echo "Average per iteration: $PYTHON_AVG seconds"
echo ""

# Run C implementation
echo "=== C Implementation ==="
C_TOTAL=0
for i in $(seq 1 $ITERATIONS); do
    echo -n "."
    start=$(date +%s.%N)
    ./soplang_c $SOPLANG_FILE > /dev/null 2>&1
    end=$(date +%s.%N)
    
    # Calculate duration
    duration=$(echo "$end - $start" | bc)
    C_TOTAL=$(echo "$C_TOTAL + $duration" | bc)
done
echo ""

C_AVG=$(echo "scale=6; $C_TOTAL / $ITERATIONS" | bc)
echo "Total time: $C_TOTAL seconds"
echo "Average per iteration: $C_AVG seconds"
echo ""

# Compare results
echo "=== Results ==="
echo "Python average: $PYTHON_AVG seconds per iteration"
echo "C average: $C_AVG seconds per iteration"

# Calculate improvement
if (( $(echo "$PYTHON_AVG > $C_AVG" | bc -l) )); then
    RATIO=$(echo "scale=2; $PYTHON_AVG / $C_AVG" | bc)
    PERCENT=$(echo "scale=2; ($PYTHON_AVG - $C_AVG) / $PYTHON_AVG * 100" | bc)
    echo "C implementation is ${RATIO}x faster (${PERCENT}% improvement)"
else
    RATIO=$(echo "scale=2; $C_AVG / $PYTHON_AVG" | bc)
    PERCENT=$(echo "scale=2; ($C_AVG - $PYTHON_AVG) / $PYTHON_AVG * 100" | bc)
    echo "Python implementation is ${RATIO}x faster (${PERCENT}% improvement)"
fi 