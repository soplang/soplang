# Soplang Example Programs

This document describes each example program available in the Soplang programming language. These examples demonstrate different features and capabilities of the language.

## Hello World (`examples/hello_world.so`)

A simple introduction to Soplang syntax and basic features.

**Features tested:**
- Basic output with `qor()`
- Variable declarations (dynamic and static)
- String concatenation
- Function definition and calls
- Conditional statements
- For and while loops

**Run with:**
```bash
./soplang-py.sh examples/hello_world.so
```

## Basics (`examples/01_basics.so`)

Demonstrates fundamental language features and operations.

**Features tested:**
- Dynamic typing with `door`
- Static typing with `qoraal`, `abn`, etc.
- Type checking with `nooc()`
- Type conversion
- Arithmetic operations
- Type enforcement

**Run with:**
```bash
./soplang-py.sh examples/01_basics.so
```

## Control Flow (`examples/02_control_flow.so`)

Demonstrates conditional statements and loops.

**Features tested:**
- If statements (`haddii`)
- If-else statements (`haddii_kale`)
- Else statements (`ugudambeyn`)
- Nested conditionals
- Complex conditions with logical operators
- For loops (`kuceli`)
- While loops (`intay`)
- Break statements (`jooji`)
- Continue statements (`soco`)
- Nested loops

**Run with:**
```bash
./soplang-py.sh examples/02_control_flow.so
```

## Functions (`examples/03_functions.so`)

Demonstrates function definitions and usage.

**Features tested:**
- Basic function declaration with `hawl`
- Functions with parameters
- Functions with multiple parameters
- Return values with `celi`
- Functions that compute values
- Conditional returns
- Functions working with complex data
- Nested functions
- Recursive functions

**Run with:**
```bash
./soplang-py.sh examples/03_functions.so
```

## Lists (`examples/04_lists.so`)

Demonstrates list operations and features.

**Features tested:**
- Dynamic list declarations
- Static list declarations with `teed` type
- Empty lists
- Lists with mixed types
- Lists with expressions
- Nested lists
- Accessing list elements
- Modifying list elements
- List methods (length, push, pop)
- List traversal
- Finding values in lists
- Summing values in lists
- Filtering lists
- Type enforcement for static lists

**Run with:**
```bash
./soplang-py.sh examples/04_lists.so
```

## Objects (`examples/05_objects.so`)

Demonstrates object operations and features.

**Features tested:**
- Dynamic object declaration
- Static object declaration with `walax` type
- Empty objects
- Objects with mixed value types
- Nested objects
- Accessing properties (dot and bracket notation)
- Accessing nested properties
- Modifying objects
- Adding new properties
- Removing properties
- Objects with lists
- Object traversal
- Objects with methods
- Advanced objects with self-referential methods
- Type enforcement for static objects

**Run with:**
```bash
./soplang-py.sh examples/05_objects.so
```

## Running All Examples

You can run all examples in sequence using:

```bash
./scripts/test/run_examples.sh
```

## Performance Testing

To compare the performance of different implementations on each example:

```bash
./scripts/benchmark/compare_performance.sh examples/01_basics.so 15
```

This will run the specified example 15 times with both Python and C implementations and display timing results. 