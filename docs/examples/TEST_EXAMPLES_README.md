# Soplang Test Examples

This directory contains various test examples for the Soplang programming language. Each example tests specific features of the language.

## Available Test Examples

1. **01_basic_output.so** - Tests the basic output functionality using the `qor()` function.
   * Printing strings, numbers, and expressions
   * String concatenation
   * Printing boolean values

2. **02_user_input.so** - Tests the user input functionality using the `gelin()` function.
   * Reading user input and storing in variables
   * Converting input strings to numbers
   * Using input in expressions

3. **03_conditionals.so** - Tests conditional statements.
   * Basic if statements (`haddii`)
   * If-else statements (`haddii`, `haddii_kale`)
   * If-elseif-else statements (`haddii`, `haddii_kale`, `ugudambeyn`)
   * Nested conditionals

4. **04_loops.so** - Tests loop constructs.
   * Basic for loops (`kuceli`)
   * Loops with break statements (`jooji`)
   * Loops with continue statements (`soco`)
   * Nested loops
   * While loops simulation (`intay`)

5. **05_dynamic_typing.so** - Tests dynamic typing using the `door` keyword.
   * Variable initialization with different types
   * Type changes during runtime
   * Type checking with `nooc()`
   * Using dynamic types in expressions

6. **06_static_typing.so** - Tests static typing using type-specific keywords.
   * Integer variables (`abn`)
   * String variables (`qoraal`)
   * Boolean variables (`bool`)
   * Type enforcement
   * Type conversion between static types

7. **07_functions.so** - Tests function definitions and calls.
   * Functions with no parameters
   * Functions with parameters
   * Functions with return values (`celi`)
   * Conditional returns
   * Nested function calls

8. **08_lists.so** - Provides placeholders for list operations.
   * List creation (dynamic and static)
   * Access, add, and remove items
   * List length, iteration, and searching
   * List operations (sorting, etc.)

9. **09_objects.so** - Provides placeholders for object operations.
   * Object creation (dynamic and static)
   * Access and modify properties
   * Check property existence
   * Delete properties
   * Nested objects and objects with lists

10. **10_error_handling.so** - Tests error handling mechanisms.
    * Syntax errors
    * Type errors
    * Reference errors
    * Division by zero
    * Array/list index errors
    * Custom error handling
    * Somali error messages

11. **automated_test.so** - An automated test example covering multiple features.
    * Variable declarations
    * Conditionals
    * Loops
    * Functions
    * Basic input/output

## Running the Tests

You can run individual tests using:

```bash
python main.py examples/01_basic_output.so
```

Or run all tests in sequence using the provided script:

```bash
./scripts/test/run_all_tests.sh
```

## Notes on Test Examples

- Some features like lists and objects currently use placeholders until those features are fully implemented.
- Error handling examples are commented out to prevent actual errors during testing. Uncomment specific sections to test error handling.
- All error messages should be displayed in Somali.
- The test examples can be expanded as more features are added to the language.