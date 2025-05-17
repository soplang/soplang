# Soplang Example Programs Documentation

This document provides detailed explanations of all example programs included with Soplang. These examples serve as both demonstrations of language features and as regression tests for the interpreter.

## Table of Contents

- [Soplang Example Programs Documentation](#soplang-example-programs-documentation)
  - [Table of Contents](#table-of-contents)
  - [Running Examples](#running-examples)
  - [Basic Language Features](#basic-language-features)
    - [01\_dynamic\_typing.so](#01_dynamic_typingso)
    - [02\_static\_typing.so](#02_static_typingso)
    - [03\_type\_checking.so](#03_type_checkingso)
  - [Operators](#operators)
    - [04\_arithmetic\_operators.so](#04_arithmetic_operatorsso)
    - [05\_comparison\_operators.so](#05_comparison_operatorsso)
    - [06\_logical\_operators.so](#06_logical_operatorsso)
  - [Control Flow](#control-flow)
    - [07\_conditional\_statements.so](#07_conditional_statementsso)
    - [08\_for\_loops.so](#08_for_loopsso)
    - [09\_while\_loops.so](#09_while_loopsso)
  - [Functions and Objects](#functions-and-objects)
    - [10\_functions.so](#10_functionsso)
    - [11\_list\_operations.so](#11_list_operationsso)
    - [12\_object\_operations.so](#12_object_operationsso)
  - [Advanced Features](#advanced-features)
    - [13\_type\_conversion.so](#13_type_conversionso)
    - [14\_comparison\_assignment.so](#14_comparison_assignmentso)
    - [15\_user\_input.so](#15_user_inputso)
  - [Conclusion](#conclusion)

## Running Examples

To run any example, use:

```bash
python main.py examples/XX_example_name.so
```

Or, if using Docker:

```bash
docker run --rm -v "$(pwd):/scripts" soplang:latest examples/XX_example_name.so
```

## Basic Language Features

### 01_dynamic_typing.so

**Purpose**: Demonstrates Soplang's dynamic typing using the `door` keyword.

**Features Tested**:
- Dynamic variable declaration (`door` keyword)
- Variable reassignment with different types
- Type inference
- Variable scope

**Key Concepts**:
- Dynamic variables can change type during execution
- The `nooc()` function shows the current type of a variable

### 02_static_typing.so

**Purpose**: Demonstrates Soplang's static typing system with type-specific keywords.

**Features Tested**:
- Static variable declaration with `abn`, `qoraal`, `bool`, `teed`, and `walax`
- Type enforcement at runtime
- Error handling for type violations

**Key Concepts**:
- Static typing provides guarantees about variable types
- Attempting to assign a value of the wrong type will cause a runtime error

### 03_type_checking.so

**Purpose**: Tests the type checking mechanism for static types.

**Features Tested**:
- Runtime type validation
- Error reporting for type mismatches
- Type compatibility rules

**Expected Results**:
- Valid type assignments succeed
- Invalid type assignments generate clear error messages

## Operators

### 04_arithmetic_operators.so

**Purpose**: Tests arithmetic operations in Soplang.

**Features Tested**:
- Addition (+)
- Subtraction (-)
- Multiplication (*)
- Division (/)
- Modulo (%)
- Operator precedence
- Mixed type arithmetic

**Key Concepts**:
- Arithmetic follows standard precedence rules
- Numeric operations work with both integers and floating-point values
- String concatenation uses the + operator

### 05_comparison_operators.so

**Purpose**: Tests comparison operations in Soplang.

**Features Tested**:
- Equality (==)
- Inequality (!=)
- Greater than (>)
- Less than (<)
- Greater than or equal to (>=)
- Less than or equal to (<=)
- Comparing different types

**Key Concepts**:
- Comparison operations return `run` (true) or `been` (false)
- Comparison works with numbers, strings, booleans, and other types

### 06_logical_operators.so

**Purpose**: Tests logical operations in Soplang.

**Features Tested**:
- Logical AND (&&)
- Logical OR (||)
- Logical NOT (!)
- Short-circuit evaluation
- Operator precedence

**Key Concepts**:
- Logical operators combine boolean expressions
- Short-circuiting prevents unnecessary evaluation

## Control Flow

### 07_conditional_statements.so

**Purpose**: Demonstrates conditional execution in Soplang.

**Features Tested**:
- Basic if statements (`haddii`)
- If-else statements (`haddii` / `ugudambeyn`)
- If-else if-else chains (`haddii` / `haddii_kale` / `ugudambeyn`)
- Nested conditionals
- Complex conditions with logical operators

**Key Concepts**:
- Conditional execution controls program flow based on boolean conditions
- Multiple conditions can be chained together

### 08_for_loops.so

**Purpose**: Tests for loops in Soplang using `kuceli`.

**Features Tested**:
- Basic for loop syntax
- Looping through ranges of numbers
- Custom step values
- Loop control with `jooji` (break) and `soco` (continue)
- Nested loops

**Key Concepts**:
- The `kuceli` statement iterates from a start to an end value
- The step can be specified with the optional `by` clause

### 09_while_loops.so

**Purpose**: Tests while loops in Soplang using `inta_ay`.

**Features Tested**:
- While loop syntax
- Conditional looping
- Loop control with `jooji` (break) and `soco` (continue)
- Infinite loops with manual breaking
- Nested while loops

**Key Concepts**:
- The `intay` statement executes as long as a condition is true
- Proper loop control prevents infinite loops

## Functions and Objects

### 10_functions.so

**Purpose**: Demonstrates function definition and calling in Soplang.

**Features Tested**:
- Function definition with `hawl`
- Parameter passing
- Return values with `celi`
- Recursive functions
- Function scoping
- Default parameters

**Key Concepts**:
- Functions encapsulate reusable code
- Return values are specified with `celi`
- Functions have their own variable scope

### 11_list_operations.so

**Purpose**: Tests list operations in Soplang.

**Features Tested**:
- List creation and initialization
- Accessing elements by index
- Modifying elements
- List methods: `kudar` (push), `kasaar` (pop), `dherer` (length), `kudar` (concat), `leeyahay` (contains)
- List iteration
- Nested lists

**Key Concepts**:
- Lists are ordered collections of values that can be accessed by index
- Lists can contain mixed types
- Built-in methods provide common list operations

### 12_object_operations.so

**Purpose**: Tests object operations in Soplang.

**Features Tested**:
- Object creation and initialization
- Property access with dot notation
- Property modification
- Object methods: `fure` (keys), `leeyahay` (has), `tir` (remove), `kudar` (merge)
- Nested objects
- Objects with arrays
- Arrays of objects

**Key Concepts**:
- Objects are collections of key-value pairs
- Properties can be accessed and modified with dot notation
- Built-in methods provide common object operations

## Advanced Features

### 13_type_conversion.so

**Purpose**: Demonstrates type conversion in Soplang.

**Features Tested**:
- Number to string conversion
- String to number conversion
- Boolean to number/string conversion
- List to string conversion
- Object to string conversion
- Type detection with `nooc()`

**Key Concepts**:
- The type conversion functions (`abn`, `qoraal`, `bool`) convert values between types
- The `nooc()` function returns the type of a value as a string

### 14_comparison_assignment.so

**Purpose**: Tests using comparison results in assignments.

**Features Tested**:
- Assigning comparison results to variables
- Using both `door` and `bool` types for storing boolean results
- Complex boolean expressions in assignments
- Reassigning boolean values

**Key Concepts**:
- Comparison operations produce boolean values
- These can be stored in variables and used later

### 15_user_input.so

**Purpose**: Demonstrates user input in Soplang using the `gelin` function.

**Features Tested**:
- Basic input with `gelin`
- Displaying prompts
- Processing user input
- Type checking input with `nooc()`

**Key Concepts**:
- The `gelin` function gets input from the user
- Input is always returned as a string
- Input can be converted to other types as needed

## Conclusion

These examples provide a comprehensive test suite for Soplang's features. When making changes to the interpreter, run these examples to ensure compatibility and correct behavior. They also serve as valuable learning resources for new users of the language.

For more information on how these examples relate to the internal implementation, see the [TEST_METHODOLOGY.md](TEST_METHODOLOGY.md) document.
