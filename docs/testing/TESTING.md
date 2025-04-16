# Soplang Testing Guide

This document provides information on testing different aspects of the Soplang programming language implementation.

## Table of Contents
- [Basic Testing](#basic-testing)
- [Example Programs](#example-programs)
- [Performance Testing](#performance-testing)
- [Implementation Comparison](#implementation-comparison)
- [Debug Mode](#debug-mode)

## Basic Testing

### Running a Soplang Program (Python Implementation)

To run a Soplang program using the Python implementation:

```bash
./soplang-py.sh examples/hello_world.so
```

### Running a Soplang Program (C Implementation)

To run a Soplang program using the C implementation:

```bash
./soplang-c.sh examples/hello_world.so
```

### Testing All Examples

To run all available example programs:

```bash
./run_examples.sh
```

### Running All Tests

To run all automated tests:

```bash
./run_all_tests.sh
```

## Example Programs

Soplang comes with several example programs demonstrating different language features:

1. **Hello World** (`examples/hello_world.so`)
   - Basic syntax and structure
   - Simple output and variables

2. **Basics** (`examples/01_basics.so`)
   - Variable declarations
   - Type checking
   - Arithmetic operations
   - Type enforcement

3. **Control Flow** (`examples/02_control_flow.so`)
   - Conditional statements
   - Loops
   - Break and continue

4. **Functions** (`examples/03_functions.so`)
   - Function declarations
   - Parameter passing
   - Return values

5. **Lists** (`examples/04_lists.so`)
   - List creation and manipulation
   - List traversal
   - List methods

6. **Objects** (`examples/05_objects.so`)
   - Object creation
   - Property access
   - Object methods

To run a specific example:

```bash
./soplang-py.sh examples/01_basics.so
```

## Performance Testing

### Basic Performance Comparison

To compare the performance of Python and C implementations:

```bash
./compare_performance.sh examples/hello_world.so 15
```

This will run the specified program 15 times with each implementation and display timing results.

### Detailed Benchmarking

For more detailed benchmarking with multiple runs and iterations:

```bash
./benchmark.sh examples/hello_world.so 5 10
```

This runs the program 5 times, with 10 iterations per run for both Python and C implementations.

## Implementation Comparison

Our testing has shown:

1. **Python Implementation**:
   - Faster for smaller programs
   - Lower startup overhead
   - Easier to modify and debug

2. **C Implementation (current)**:
   - Currently slower than Python for most examples
   - Requires PYTHONPATH to be set correctly
   - Linked against Python libraries

3. **Potential Native C Implementation**:
   - Would need to be compiled with proper optimization
   - Could be significantly faster for computation-heavy tasks
   - Would eliminate Python dependency

## Debug Mode

Both Python and C implementations support a debug mode:

```bash
./soplang-py.sh --debug examples/hello_world.so
./soplang-c.sh --debug examples/hello_world.so
```

This will generate detailed logs (debug_output.log and debug_output_py.log) showing:
- Function calls
- File operations
- System calls
- Import resolution

Useful for diagnosing issues or understanding the internal working of each implementation.

## Compilation

If you have a development environment with a working GCC compiler, you can compile the C implementation:

```bash
./compile_c_version.sh
```

This script will:
1. Detect your Python version and libraries
2. Compile all C files with optimization flags
3. Create a `bin/` directory with the compiled executables
4. Generate a `soplang_c` script to run the compiled version

Note: Compilation requires Python development headers and a working C compiler. 
