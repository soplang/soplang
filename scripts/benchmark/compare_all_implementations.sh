#!/bin/bash

# Soplang Full Implementation Comparison
# Compares the performance of all Soplang implementations:
# 1. Regular Python
# 2. Optimized Python (-O flag)
# 3. C implementation

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

echo "=== Soplang Complete Implementation Comparison ==="
echo "File: $SOPLANG_FILE"
echo "Iterations: $ITERATIONS"
echo ""

# Ensure PYTHONPATH is set
export PYTHONPATH=$(pwd)

# Check if all implementations are available
if [ ! -f "./soplang_py_optimized" ]; then
    echo "Optimized Python runner not found!"
    exit 1
fi

if [ ! -f "./csrc/bin/main.out" ]; then
    echo "Compiled C implementation not found!"
    echo "Run ./scripts/build/compile_c_version.sh first"
    exit 1
fi

# 1. Run regular Python implementation
echo "=== 1. Regular Python Implementation ==="
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

# 2. Run optimized Python implementation
echo "=== 2. Optimized Python Implementation ==="
OPTIMIZED_PY_TOTAL=0
for i in $(seq 1 $ITERATIONS); do
    echo -n "."
    start=$(date +%s.%N)
    ./soplang_py_optimized $SOPLANG_FILE > /dev/null 2>&1
    end=$(date +%s.%N)
    
    # Calculate duration
    duration=$(echo "$end - $start" | bc)
    OPTIMIZED_PY_TOTAL=$(echo "$OPTIMIZED_PY_TOTAL + $duration" | bc)
done
echo ""

OPTIMIZED_PY_AVG=$(echo "scale=6; $OPTIMIZED_PY_TOTAL / $ITERATIONS" | bc)
echo "Total time: $OPTIMIZED_PY_TOTAL seconds"
echo "Average per iteration: $OPTIMIZED_PY_AVG seconds"
echo ""

# 3. Run C implementation
echo "=== 3. C Implementation ==="
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
echo "=== Results Summary ==="
echo "1. Regular Python:    $PYTHON_AVG seconds per iteration"
echo "2. Optimized Python:  $OPTIMIZED_PY_AVG seconds per iteration"
echo "3. C Implementation:  $C_AVG seconds per iteration"
echo ""

# Find the fastest implementation
FASTEST="Python"
FASTEST_TIME=$PYTHON_AVG

if (( $(echo "$OPTIMIZED_PY_AVG < $FASTEST_TIME" | bc -l) )); then
    FASTEST="Optimized Python"
    FASTEST_TIME=$OPTIMIZED_PY_AVG
fi

if (( $(echo "$C_AVG < $FASTEST_TIME" | bc -l) )); then
    FASTEST="C"
    FASTEST_TIME=$C_AVG
fi

echo "Fastest implementation: $FASTEST"
echo ""

# Calculate relative performance
echo "=== Relative Performance ==="
PYTHON_TO_OPTIMIZED=$(echo "scale=2; $PYTHON_AVG / $OPTIMIZED_PY_AVG" | bc)
PYTHON_TO_C=$(echo "scale=2; $PYTHON_AVG / $C_AVG" | bc)
OPTIMIZED_TO_C=$(echo "scale=2; $OPTIMIZED_PY_AVG / $C_AVG" | bc)

if (( $(echo "$PYTHON_TO_OPTIMIZED > 1" | bc -l) )); then
    PERCENT=$(echo "scale=2; ($PYTHON_AVG - $OPTIMIZED_PY_AVG) / $PYTHON_AVG * 100" | bc)
    echo "Optimized Python is ${PYTHON_TO_OPTIMIZED}x faster than regular Python (${PERCENT}% improvement)"
else
    INV=$(echo "scale=2; 1 / $PYTHON_TO_OPTIMIZED" | bc)
    PERCENT=$(echo "scale=2; ($OPTIMIZED_PY_AVG - $PYTHON_AVG) / $PYTHON_AVG * 100" | bc)
    echo "Regular Python is ${INV}x faster than optimized Python (${PERCENT}% faster)"
fi

if (( $(echo "$PYTHON_TO_C > 1" | bc -l) )); then
    PERCENT=$(echo "scale=2; ($PYTHON_AVG - $C_AVG) / $PYTHON_AVG * 100" | bc)
    echo "C is ${PYTHON_TO_C}x faster than regular Python (${PERCENT}% improvement)"
else
    INV=$(echo "scale=2; 1 / $PYTHON_TO_C" | bc)
    PERCENT=$(echo "scale=2; ($C_AVG - $PYTHON_AVG) / $PYTHON_AVG * 100" | bc)
    echo "Regular Python is ${INV}x faster than C (${PERCENT}% faster)"
fi

if (( $(echo "$OPTIMIZED_TO_C > 1" | bc -l) )); then
    PERCENT=$(echo "scale=2; ($OPTIMIZED_PY_AVG - $C_AVG) / $OPTIMIZED_PY_AVG * 100" | bc)
    echo "C is ${OPTIMIZED_TO_C}x faster than optimized Python (${PERCENT}% improvement)"
else
    INV=$(echo "scale=2; 1 / $OPTIMIZED_TO_C" | bc)
    PERCENT=$(echo "scale=2; ($C_AVG - $OPTIMIZED_PY_AVG) / $OPTIMIZED_PY_AVG * 100" | bc)
    echo "Optimized Python is ${INV}x faster than C (${PERCENT}% faster)"
fi 