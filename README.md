# Stopped this version of soplang

This version of Soplang has been discontinued as we have migrated to a new version with updated keywords and syntax. The new version features improved Somali language keywords and a more intuitive syntax structure. Please check our latest repository for the current implementation.


# Soplang Programming Language

Soplang is a simple programming language with Somali-based syntax. It's designed for educational purposes and supports variables, functions, loops, conditionals, and basic data structures.

## Features

- Somali-based keywords for better accessibility to Somali speakers
- Clean, Python-like syntax
- Support for variables, functions, and data structures
- Interactive shell with command history
- File execution support

## Installation

### Option 1: Install from source

```bash
# Clone the repository
git clone https://github.com/soplang/soplang.git
cd soplang

# Install the package
pip install -e .
```

### Option 2: Manual usage

```bash
# Clone the repository
git clone https://github.com/soplang/soplang.git
cd soplang

# Run the interactive shell
python shell.py

# Run a Soplang file
python soplang_run.py example.spl
```

## Usage

### Interactive Shell

Start the interactive shell:

```bash
soplang
```

Or if not installed:

```bash
python shell.py
```

### Running Files

Run a Soplang file:

```bash
soplang-run myprogram.spl
```

Or if not installed:

```bash
python soplang_run.py myprogram.spl
```

## Language Syntax

### Keywords

Soplang uses Somali-based keywords:

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

### Examples

#### Variables

```
keyd x = 5
keyd name = "John"
keyd list = [1, 2, 3]
```

#### Conditionals

```
haddii x < 10 markaas
    PRINT("x is less than 10")
haddii_kale x == 10 markaas
    PRINT("x equals 10")
kale
    PRINT("x is greater than 10")
dhamee
```

#### Loops

```
# For loop
ku_celi i = 0 ilaa 10 markaas
    PRINT(i)
dhamee

# While loop
keyd i = 0
inta_ay i < 5 markaas
    PRINT(i)
    keyd i = i + 1
dhamee
```

#### Functions

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

## Built-in Functions

- `PRINT(value)` - Print to console
- `PRINT_RET(value)` - Print and return value
- `INPUT()` - Get string input
- `INPUT_INT()` - Get integer input
- `APPEND(list, value)` - Add to list
- `POP(list, index)` - Remove from list and return
- `EXTEND(listA, listB)` - Add all items from listB to listA
- `LEN(list)` - Get list length
- `RUN("filename.spl")` - Run another Soplang file

## License

This project is open source and available under the [MIT License](LICENSE).