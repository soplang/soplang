# Soplang Testing Documentation

Welcome to the Soplang testing documentation. This collection of documents provides comprehensive information about testing the Soplang language interpreter and writing testable Soplang programs.

## Table of Contents

- [Soplang Testing Documentation](#soplang-testing-documentation)
  - [Table of Contents](#table-of-contents)
  - [Overview](#overview)
  - [Test Methodology (TEST\_METHODOLOGY.md)](#test-methodology-test_methodologymd)
  - [Example Programs (EXAMPLES.md)](#example-programs-examplesmd)
  - [Debugging Guide (DEBUGGING.md)](#debugging-guide-debuggingmd)
  - [Performance Testing](#performance-testing)
  - [Quick Start](#quick-start)
    - [Running Examples](#running-examples)
    - [Running Tests](#running-tests)
    - [Debugging](#debugging)
  - [Contributing](#contributing)
  - [Conclusion](#conclusion)

## Overview

Soplang uses a multi-faceted testing approach:

1. **Example-based testing**: Each language feature has example programs that demonstrate usage and verify behavior
2. **Unit testing**: Core components (lexer, parser, interpreter) have dedicated unit tests
3. **Integration testing**: End-to-end tests verify complete program execution
4. **Performance testing**: Benchmarks measure execution efficiency

## Test Methodology ([TEST_METHODOLOGY.md](TEST_METHODOLOGY.md))

The [Test Methodology](TEST_METHODOLOGY.md) document explains:

- Testing philosophy
- Types of tests
- Testing strategies
- Test organization
- Best practices for testing
- Handling language evolution
- Test reporting

Read this document if you want to understand the testing approach for Soplang or if you're contributing new tests.

## Example Programs ([EXAMPLES.md](EXAMPLES.md))

The [Examples Documentation](EXAMPLES.md) provides:

- Detailed descriptions of each example program
- Features tested in each example
- Purpose of each test
- Expected results

Example programs are the primary way of testing Soplang features. Each example demonstrates a specific language feature or combination of features. They serve as both tests and documentation.

## Debugging Guide ([DEBUGGING.md](DEBUGGING.md))

The [Debugging Guide](DEBUGGING.md) covers:

- Understanding error messages
- Adding debug output
- Step-by-step debugging
- Common runtime errors
- Debugging the lexer, parser, and interpreter
- Using logging
- Advanced debugging techniques

Use this guide when you encounter problems with Soplang programs or when developing the interpreter itself.

## Performance Testing

Performance testing compares different implementations and measures execution efficiency:

- **Baseline Performance**: Establishing performance baselines for core operations
- **Implementation Comparison**: Comparing Python and C implementations
- **Scaling Tests**: Measuring how performance scales with input size
- **Memory Usage**: Tracking memory consumption during execution

## Quick Start

### Running Examples

```bash
# Run using Python implementation
python main.py examples/01_dynamic_typing.so

# Run using Docker
docker run --rm -v "$(pwd):/scripts" soplang:latest examples/01_dynamic_typing.so
```

### Running Tests

```bash
# Run all unit tests
python -m unittest discover tests

# Run a specific test file
python -m unittest tests/test_lexer.py
```

### Debugging

```bash
# Run with lexer debugging
python main.py --debug-lexer examples/my_program.so

# Run with parser debugging
python main.py --debug-parser examples/my_program.so

# Run with interpreter debugging
python main.py --debug-interpreter examples/my_program.so
```

## Contributing

When contributing to Soplang, follow these testing principles:

1. **Add examples** for new features
2. **Write unit tests** for internal components
3. **Verify backwards compatibility** with existing examples
4. **Document tested behavior** in the appropriate files
5. **Run all tests** before submitting changes

See the [Contributing Guide](../CONTRIBUTING.md) for more information.

## Conclusion

A comprehensive testing approach ensures Soplang remains reliable, performant, and easy to use. These documents guide you through testing both the Soplang interpreter and Soplang programs.
