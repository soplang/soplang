# Soplang Testing Methodology

This document outlines the testing methodology used for Soplang. It describes the different types of tests, testing strategies, and best practices for ensuring the language implementation is correct and robust.

## Testing Philosophy

Soplang follows these testing principles:

1. **Feature-first testing**: Each language feature is tested individually before integration
2. **Example-driven development**: New features are accompanied by example programs
3. **Regression prevention**: Tests are run before merging changes to prevent regressions
4. **Dual implementation testing**: Both Python and C implementations must pass all tests
5. **User-focused validation**: Tests focus on user-visible behavior, not implementation details

## Types of Tests

### 1. Example Programs

The primary testing method for Soplang is through example programs (`examples/*.so` files). These serve multiple purposes:

- **Documentation**: Show users how to use language features
- **Feature validation**: Verify that features work as expected
- **Regression testing**: Ensure changes don't break existing functionality
- **Integration testing**: Test how different features interact

For details on the example programs, see [EXAMPLES.md](EXAMPLES.md).

### 2. Unit Tests

Unit tests focus on individual components of the Soplang implementation:

- **Lexer tests**: Verify token generation from source code
- **Parser tests**: Ensure AST construction is correct
- **Interpreter tests**: Test execution of AST nodes
- **Type system tests**: Validate type checking and enforcement
- **Standard library tests**: Test built-in functions and methods

Unit tests are written in Python using the standard `unittest` framework and are located in the `tests/` directory.

### 3. Property-Based Tests

Property-based tests generate random inputs to find edge cases and ensure properties hold:

- **Syntax validity**: Ensure valid syntax is accepted and invalid syntax is rejected
- **Type safety**: Verify type system correctly enforces constraints
- **Execution determinism**: Ensure same input always produces same output
- **Error handling**: Test that errors are caught and reported correctly

### 4. Performance Tests

Performance tests measure execution time and resource usage:

- **Baseline performance**: Establish performance baselines for core operations
- **Implementation comparison**: Compare Python and C implementations
- **Scaling tests**: Measure how performance scales with input size
- **Memory usage**: Track memory consumption during execution

## Testing Strategy

### Test-Driven Development

For new features, follow this process:

1. **Specification**: Define the syntax and semantics of the feature
2. **Example Creation**: Write example programs demonstrating the feature
3. **Test First**: Write unit tests for the feature
4. **Implementation**: Implement the feature until tests pass
5. **Integration**: Ensure the feature works with existing functionality
6. **Documentation**: Update documentation with the new feature

### Continuous Integration

All tests are run:

1. **Before commit**: Using pre-commit hooks to catch basic issues
2. **After push**: Automated CI runs all tests on each push
3. **Before release**: Complete test suite is run before each release

### Manual Testing

Some aspects require manual testing:

1. **User experience**: Testing the REPL and interactive features
2. **Error messages**: Ensuring error messages are clear and helpful
3. **Documentation accuracy**: Verifying documentation matches implementation
4. **Cross-platform behavior**: Testing on different operating systems

## Testing Tools

### Running Tests

Use these commands to run tests:

```bash
# Run all tests
python -m unittest discover tests

# Run specific test file
python -m unittest tests/test_lexer.py

# Run specific test class
python -m unittest tests.test_lexer.TestLexer

# Run with coverage reporting
pytest --cov=src tests/
```

### Benchmarking

For performance testing:

```bash
# Run basic benchmark
./scripts/benchmark/benchmark.sh examples/benchmark.so

# Compare implementations
./scripts/benchmark/compare_implementations.sh examples/benchmark.so
```

## Test Organization

Tests are organized as follows:

1. **Example Programs**: `examples/*.so`
   - Organized by feature and numbered for logical progression
   - Comprehensive examples of all language features

2. **Unit Tests**: `tests/`
   - `test_lexer.py`: Tests for the lexer
   - `test_parser.py`: Tests for the parser
   - `test_interpreter.py`: Tests for the interpreter
   - `test_builtins.py`: Tests for built-in functions
   - `test_types.py`: Tests for the type system

3. **Integration Tests**: `tests/integration/`
   - End-to-end tests of the entire system
   - Tests that span multiple components

4. **Performance Tests**: `tests/performance/`
   - Benchmarks for various operations
   - Tests for memory usage and scaling

## Best Practices

1. **Test coverage**: Aim for >90% code coverage
2. **Test isolation**: Each test should be independent of others
3. **Descriptive test names**: Test names should describe what they're testing
4. **Error testing**: Test both success and failure cases
5. **Edge cases**: Include tests for edge cases and corner cases
6. **Documentation**: Document the purpose and approach of each test
7. **Maintenance**: Update tests when behavior changes

## Handling Language Evolution

As Soplang evolves:

1. **Backward compatibility tests**: Ensure old code continues to work
2. **Version-specific tests**: Label tests with the version they apply to
3. **Deprecation testing**: Test that deprecated features are correctly handled
4. **Migration testing**: Test migration paths between versions

## Test Reporting

Test results are reported in several ways:

1. **Console output**: Pass/fail status and errors
2. **Coverage reports**: Code coverage statistics
3. **Performance dashboards**: Performance metrics over time
4. **Regression reports**: Any regressions from previous versions

## Conclusion

Testing is central to Soplang's development process. By combining example programs, unit tests, property-based tests, and performance tests, we ensure that the language implementation is correct, robust, and efficient.

For more specific information on examples, see [EXAMPLES.md](EXAMPLES.md). For debugging information, refer to [DEBUGGING.md](DEBUGGING.md).
