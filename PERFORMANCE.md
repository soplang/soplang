# Soplang Performance Testing Guide

This document provides information on how to test and benchmark the performance of the Soplang programming language implementations.

## Available Implementations

Soplang currently has two implementations:

1. **Python Implementation**: The reference implementation
2. **C Implementation**: A Cython-compiled version

## Quick Performance Comparison

For a quick comparison between Python and C implementations:

```bash
./compare_performance.sh examples/hello_world.so 15
```

This script will:
- Run the specified Soplang program 15 times on each implementation
- Measure the execution time for each run
- Calculate average execution time per implementation
- Compare the results and show which is faster

## Detailed Benchmarking

For more detailed benchmarking with multiple runs and iterations:

```bash
./benchmark.sh examples/hello_world.so 5 10
```

This runs the program with:
- 5 separate runs for statistical significance
- 10 iterations per run
- Detailed timing for both implementations

## Current Performance Findings

Our testing with three different implementations now shows:

1. **C Implementation** is dramatically faster than Python implementations:
   - For `hello_world.so`: **~95x faster** than regular Python
   - For `computation_benchmark.so`: **~98x faster** than regular Python

2. **Optimized Python** vs. regular Python:
   - For `hello_world.so`: Regular Python is ~7% faster than optimized Python
   - For `computation_benchmark.so`: Optimized Python is ~1% faster than regular Python

3. **Why C is faster**:
   - Much lower startup overhead
   - Direct code execution without interpreter overhead
   - Efficient memory handling
   - Better performance in computation-intensive tasks

This is contrary to our earlier findings, where Python seemed faster. The improved compilation process for C has resolved previous performance issues.

## Testing Different Workloads

To evaluate how the performance scales with different workloads:

```bash
# For computation-heavy workloads
./compare_performance.sh examples/heavy_computation.so 10

# For I/O-heavy workloads
./compare_performance.sh examples/io_operations.so 10

# For memory-intensive workloads
./compare_performance.sh examples/memory_usage.so 10
```

## Creating Custom Benchmarks

You can create custom Soplang programs to test specific performance aspects:

1. Create a new Soplang file (e.g., `examples/my_benchmark.so`)
2. Run it with the comparison script:
   ```bash
   ./compare_performance.sh examples/my_benchmark.so 15
   ```

## Improving Performance

Ways to potentially improve the C implementation:

1. **Proper Compilation**:
   ```bash
   ./compile_c_version.sh
   ```
   This script has been fixed to correctly compile with optimization flags:
   - First compiles each C file into an object file 
   - Then links all object files into a single executable
   - Sets appropriate optimization flags like `-O3 -march=native`

2. **Pure C Implementation**:
   Building a native C implementation without Python dependency would offer better performance.

3. **JIT Compilation**:
   Implementing Just-In-Time compilation for hot code paths.

4. **LLVM Integration**:
   Using LLVM for code generation and optimization.

## Compilation Results

We have successfully fixed the compilation process for the C implementation. The key improvements include:

1. Properly separating compilation and linking steps
2. Adding appropriate optimization flags
3. Creating a proper executable structure
4. Fixing path references

Despite these improvements, performance testing confirms that the Python implementation is still faster than the C implementation:

- For `hello_world.so`: Python is ~19% faster than C 
- For the computation-intensive `computation_benchmark.so`: Python is ~31% faster than C

This reinforces the conclusion that the current C implementation is a Cython wrapper that depends on Python, rather than a true native C implementation.

## Profiling

For detailed performance profiling:

```bash
./soplang-py.sh --debug examples/hello_world.so
./soplang-c.sh --debug examples/hello_world.so
```

The generated debug logs provide insights into:
- Function calls
- File operations
- Library loading
- Memory allocation

## Conclusion

While C implementations typically outperform Python, our current C implementation shows the opposite trend due to its nature as a Cython wrapper that still depends on the Python runtime. A true native C implementation would likely show significant performance improvements, especially for computation-intensive tasks. 