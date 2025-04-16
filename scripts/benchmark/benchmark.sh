#!/bin/bash

# Soplang Benchmark Script
# This script compares the performance of the C and Python interpreters

# Get the directory where the script is located
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

if [ $# -lt 1 ]; then
    echo "Usage: $0 <soplang_file.so> [num_runs=5] [iterations=1]"
    echo ""
    echo "Available examples:"
    ls -1 "$SCRIPT_DIR/examples/"*.so
    exit 1
fi

FILE=$1
RUNS=${2:-5}      # Default to 5 runs if not specified
ITERATIONS=${3:-1} # Default to 1 iteration per run

# Set PYTHONPATH to include the script directory
export PYTHONPATH="$SCRIPT_DIR"

echo "=== Soplang Benchmark ==="
echo "File: $FILE"
echo "Number of runs: $RUNS"
echo "Iterations per run: $ITERATIONS"
echo ""

# Benchmark Python version
echo "=== Python Version ==="
PYTHON_TOTAL=0
for ((i=1; i<=$RUNS; i++)); do
    echo "Run $i..."
    start_time=$(date +%s.%N)
    
    # Run multiple iterations to create more work
    for ((j=1; j<=$ITERATIONS; j++)); do
        python "$SCRIPT_DIR/main.py" "$FILE" > /dev/null
    done
    
    end_time=$(date +%s.%N)
    runtime=$(echo "$end_time - $start_time" | bc)
    PYTHON_TOTAL=$(echo "$PYTHON_TOTAL + $runtime" | bc)
    echo "  Time: $runtime seconds"
done
PYTHON_AVG=$(echo "scale=6; $PYTHON_TOTAL / $RUNS" | bc)
echo "Python average time: $PYTHON_AVG seconds"
echo ""

# Benchmark C version
echo "=== C Version ==="
C_TOTAL=0
for ((i=1; i<=$RUNS; i++)); do
    echo "Run $i..."
    start_time=$(date +%s.%N)
    
    # Run multiple iterations to create more work
    for ((j=1; j<=$ITERATIONS; j++)); do
        "$SCRIPT_DIR/csrc/main.out" "$FILE" > /dev/null
    done
    
    end_time=$(date +%s.%N)
    runtime=$(echo "$end_time - $start_time" | bc)
    C_TOTAL=$(echo "$C_TOTAL + $runtime" | bc)
    echo "  Time: $runtime seconds"
done
C_AVG=$(echo "scale=6; $C_TOTAL / $RUNS" | bc)
echo "C average time: $C_AVG seconds"
echo ""

# Calculate speedup
SPEEDUP=$(echo "scale=2; $PYTHON_AVG / $C_AVG" | bc)
echo "=== Results ==="
echo "Python version: $PYTHON_AVG seconds"
echo "C version: $C_AVG seconds"
echo "Speedup: ${SPEEDUP}x" 