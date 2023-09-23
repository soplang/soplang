# Soplang Grammar

This document describes the formal grammar of the Soplang programming language.

## Lexical Elements

### Keywords

Soplang uses the following Somali-based keywords:

| Keyword       | Meaning              | English Equivalent |
| ------------- | -------------------- | ------------------ |
| `keyd`        | Variable declaration | `VAR`              |
| `iyo`         | Logical AND          | `AND`              |
| `ama`         | Logical OR           | `OR`               |
| `ma`          | Logical NOT          | `NOT`              |
| `haddii`      | If statement         | `IF`               |
| `haddii_kale` | Else if statement    | `ELIF`             |
| `kale`        | Else statement       | `ELSE`             |
| `ku_celi`     | For loop             | `FOR`              |
| `ilaa`        | To (in for loops)    | `TO`               |
| `tallaabo`    | Step (in for loops)  | `STEP`             |
| `inta_ay`     | While loop           | `WHILE`            |
| `shaqo`       | Function definition  | `FUN`              |
| `markaas`     | Then                 | `THEN`             |
| `dhamee`      | End block            | `END`              |
| `soo_celi`    | Return from function | `RETURN`           |
| `sii_wad`     | Continue in loop     | `CONTINUE`         |
| `jooji`       | Break from loop      | `BREAK`            |

### Tokens

- **Identifiers**: Sequence of letters, digits, and underscores, starting with a letter
- **Numbers**: Integer or floating-point numbers
- **Strings**: Text enclosed in double quotes
- **Operators**: `+`, `-`, `*`, `/`, `^`, `==`, `!=`, `<`, `>`, `<=`, `>=`
- **Delimiters**: `(`, `)`, `[`, `]`, `,`, `=`, `->`, `;`, newline

## Grammar Rules

The grammar is defined in Extended Backus-Naur Form (EBNF) notation.

```ebnf
program       : statements

statements    : NEWLINE* statement (NEWLINE+ statement)* NEWLINE*

statement     : KEYWORD:soo_celi expr?
              | KEYWORD:sii_wad
              | KEYWORD:jooji
              | expr

expr          : KEYWORD:keyd IDENTIFIER EQ expr
              | comp-expr ((KEYWORD:iyo|KEYWORD:ama) comp-expr)*

comp-expr     : KEYWORD:ma comp-expr
              | arith-expr ((EE|NE|LT|GT|LTE|GTE) arith-expr)*

arith-expr    : term ((PLUS|MINUS) term)*

term          : factor ((MUL|DIV) factor)*

factor        : (PLUS|MINUS) factor
              | power

power         : call (POW factor)*

call          : atom (LPAREN (expr (COMMA expr)*)? RPAREN)?

atom          : INT|FLOAT|STRING|IDENTIFIER
              | LPAREN expr RPAREN
              | list-expr
              | if-expr
              | for-expr
              | while-expr
              | func-def

list-expr     : LSQUARE (expr (COMMA expr)*)? RSQUARE

if-expr       : KEYWORD:haddii expr KEYWORD:markaas
                (statement if-expr-b|if-expr-c?)
              | (NEWLINE statements KEYWORD:dhamee|if-expr-b|if-expr-c)

if-expr-b     : KEYWORD:haddii_kale expr KEYWORD:markaas
                (statement if-expr-b|if-expr-c?)
              | (NEWLINE statements KEYWORD:dhamee|if-expr-b|if-expr-c)

if-expr-c     : KEYWORD:kale
                statement
              | (NEWLINE statements KEYWORD:dhamee)

for-expr      : KEYWORD:ku_celi IDENTIFIER EQ expr KEYWORD:ilaa expr 
                (KEYWORD:tallaabo expr)? KEYWORD:markaas
                statement
              | (NEWLINE statements KEYWORD:dhamee)

while-expr    : KEYWORD:inta_ay expr KEYWORD:markaas
                statement
              | (NEWLINE statements KEYWORD:dhamee)

func-def      : KEYWORD:shaqo IDENTIFIER?
                LPAREN (IDENTIFIER (COMMA IDENTIFIER)*)? RPAREN
                (ARROW expr)
              | (NEWLINE statements KEYWORD:dhamee)
```

## Syntax Examples

### Variable Declaration

```
keyd x = 5
keyd name = "John"
keyd list = [1, 2, 3]
```

### Conditionals

```
haddii x < 10 markaas
    PRINT("x is less than 10")
haddii_kale x == 10 markaas
    PRINT("x equals 10")
kale
    PRINT("x is greater than 10")
dhamee
```

### Loops

#### For Loop

```
ku_celi i = 0 ilaa 10 markaas
    PRINT(i)
dhamee

# With step
ku_celi i = 0 ilaa 10 tallaabo 2 markaas
    PRINT(i)  # Prints 0, 2, 4, 6, 8
dhamee
```

#### While Loop

```
keyd i = 0
inta_ay i < 5 markaas
    PRINT(i)
    keyd i = i + 1
dhamee
```

### Functions

```
# Simple function
shaqo greet(name)
    PRINT("Hello, " + name + "!")
dhamee

# Function with return value
shaqo add(a, b)
    soo_celi a + b
dhamee

# One-line function using arrow syntax
shaqo square(n) -> n * n
```

### Lists

```
keyd fruits = ["apple", "banana", "orange"]
PRINT(fruits/0)  # Access by index (prints "apple")
APPEND(fruits, "mango")  # Add to list
PRINT(POP(fruits, 1))  # Remove and return (prints "banana")
PRINT(LEN(fruits))  # Length of list (prints 3)
```

## Operator Precedence

Operators are evaluated in the following order (from highest to lowest precedence):

1. Parentheses `()`
2. Unary operators (`+`, `-`, `ma`)
3. Exponentiation `^`
4. Multiplication and division (`*`, `/`)
5. Addition and subtraction (`+`, `-`)
6. Comparison operators (`==`, `!=`, `<`, `>`, `<=`, `>=`)
7. Logical NOT (`ma`)
8. Logical AND (`iyo`)
9. Logical OR (`ama`)

## Built-in Functions

Soplang provides the following built-in functions:

- `PRINT(value)` - Print to console
- `PRINT_RET(value)` - Print and return value
- `INPUT()` - Get string input
- `INPUT_INT()` - Get integer input
- `APPEND(list, value)` - Add to list
- `POP(list, index)` - Remove from list and return
- `EXTEND(listA, listB)` - Add all items from listB to listA
- `LEN(list)` - Get list length
- `RUN("filename.spl")` - Run another Soplang file
- `IS_NUM(value)` - Check if value is a number
- `IS_STR(value)` - Check if value is a string
- `IS_LIST(value)` - Check if value is a list
- `IS_FUN(value)` - Check if value is a function 