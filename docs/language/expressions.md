# Expressions in Soplang

This document explains how expressions work in Soplang, with a focus on the improved handling of comparison operators and logical expressions.

## Overview

Expressions in Soplang are combinations of values, variables, operators, and function calls that can be evaluated to produce a value. The language supports a wide range of expressions, from simple arithmetic to complex logical conditions.

## Expression Types

### Arithmetic Expressions

Arithmetic expressions involve numeric operations:

```
door a = 5 + 10        // Addition: 15
door b = 20 - 7        // Subtraction: 13
door c = 4 * 5         // Multiplication: 20
door d = 15 / 3        // Division: 5
door e = 10 % 3        // Modulus (remainder): 1
```

### String Operations

String concatenation using the `+` operator:

```
qoraal first_name = "Ahmed"
qoraal last_name = "Mohamed"
qoraal full_name = first_name + " " + last_name  // "Ahmed Mohamed"
```

### Comparison Expressions

Soplang supports direct use of comparison operators in expressions:

```
tiro x = 10
tiro y = 20

// These work without needing additional parentheses
door is_greater = x > y         // been (false)
door is_less = x < y           // run (true)
door is_equal = x == y         // been (false)
door is_not_equal = x != y     // run (true)
door is_greater_equal = x >= y // been (false)
door is_less_equal = x <= y    // run (true)
```

### Logical Expressions

Logical expressions combine boolean values or comparison results:

```
bool condition1 = run      // true
bool condition2 = been     // false

door result1 = condition1 && condition2  // AND: been (false)
door result2 = condition1 || condition2  // OR: run (true)
door result3 = !condition1               // NOT: been (false)
```

### Complex Expressions

You can create complex expressions by combining multiple operations:

```
tiro age = 25
tiro experience = 3
tiro score = 85

// Complex logical expression
door is_eligible = (age >= 21) && (experience >= 2 || score > 80)
```

## Using Expressions

### In Variable Assignments

Expressions can be directly assigned to variables:

```
door passed = student_score > passing_score
```

### In Function Arguments

Expressions can be used as function arguments:

```
bandhig(name + " is " + (age >= 18 ? "an adult" : "a minor"))
```

### In Switch Statements

Expressions can be used in switch conditions:

```
dooro (score >= 90) {
  xaalad run {
    bandhig("Excellent score!")
  }
  xaalad been {
    bandhig("Keep practicing")
  }
}
```

### In List and Object Literals

Expressions can be used in list elements and object property values:

```
liis results = [score1 > 80, score2 > 80, score3 > 80]

walax student = {
  name: full_name,
  passed: final_score >= passing_score,
  grade: final_score >= 90 ? "A" : final_score >= 80 ? "B" : "C"
}
```

## Operator Precedence

Soplang follows standard operator precedence rules:

1. Parentheses `()`
2. Unary operators (`!`, unary `-`)
3. Multiplication, division, modulus (`*`, `/`, `%`)
4. Addition, subtraction (`+`, `-`)
5. Comparison operators (`<`, `>`, `<=`, `>=`, `==`, `!=`)
6. Logical AND (`&&`)
7. Logical OR (`||`)

Example showing precedence:
```
door result = 5 + 3 * 2          // 11 (not 16, because * has higher precedence than +)
door result2 = (5 + 3) * 2        // 16 (parentheses override default precedence)
door result3 = 10 > 5 && 6 < 8   // run (true, comparisons evaluate before logical AND)
```

## Type Conversion in Expressions

Soplang may perform automatic type conversion in certain expressions:

```
// String concatenation with non-string values
door output = "The score is: " + 85  // "The score is: 85"

// Boolean conversion in logical expressions
door result = 10 && 20  // run (true, because both 10 and 20 are truthy)
```

## Benefits of Direct Operator Expressions

The ability to use comparison operators directly in expressions without additional parentheses makes Soplang code:

1. **More readable** - Fewer nested parentheses means cleaner code
2. **More intuitive** - Expressions can be written in a natural way
3. **More concise** - Expressions require less syntactic overhead

This feature brings Soplang closer to the way we think about conditions and expressions in natural language.