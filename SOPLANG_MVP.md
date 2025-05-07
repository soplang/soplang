# Soplang MVP (Minimal Viable Product)

## What's New

Soplang has been continuously improving with new features and optimizations:

### Latest Improvements

- **Enhanced Expression Handling**: Comparison operators can now be used directly in expressions without additional parentheses. For example, `door result = x > 10` or `dooro (score >= 90)` are valid.
- **Switch-Case Statement**: The new `dooro`/`xaalad` (switch/case) functionality allows for elegant conditional branching based on values or expressions.
- **Consistent API**: List and object methods now have a consistent API with methods like `kudar`, `kasaar`, `leeyahay`, and `fure`.
- **Improved Type Support**: Better support for decimal types with the `jajab` keyword and proper handling of boolean types with `bool`.
- **Parser Optimizations**: Expression parsing has been improved for better handling of complex logical expressions.

## Introduction

# Soplang MVP: The Somali Programming Language

> Soplang is a programming language with syntax inspired by Somali language, making programming more accessible to Somali speakers. It uniquely combines static and dynamic typing systems in one elegant language with a focus on clarity and ease of use.

This document outlines the core features included in the Minimum Viable Product (MVP) of Soplang, with examples to help you get started.

## Table of Contents

- [Soplang MVP (Minimal Viable Product)](#soplang-mvp-minimal-viable-product)
  - [What's New](#whats-new)
    - [Latest Improvements](#latest-improvements)
  - [Introduction](#introduction)
- [Soplang MVP: The Somali Programming Language](#soplang-mvp-the-somali-programming-language)
  - [Table of Contents](#table-of-contents)
  - [Variable Declaration](#variable-declaration)
    - [Example](#example)
  - [Control Flow Statements](#control-flow-statements)
    - [Example](#example-1)
  - [Loops](#loops)
    - [Example](#example-2)
  - [Functions](#functions)
    - [Example](#example-3)
  - [Data Types and Values](#data-types-and-values)
    - [Example](#example-4)
  - [Operators](#operators)
    - [Example](#example-5)
  - [Built-in Functions](#built-in-functions)
    - [Example](#example-6)
  - [Lists and Methods](#lists-and-methods)
    - [Example](#example-7)
  - [Objects and Methods](#objects-and-methods)
    - [Example](#example-8)
  - [Comments](#comments)
    - [Example](#example-9)
  - [Complete Example Program](#complete-example-program)

## Variable Declaration

Soplang provides both dynamic typing (variables can change types) and static typing (variable type is fixed).

| Somali Keyword | English Equivalent | Description                  | Example                                |
| -------------- | ------------------ | ---------------------------- | -------------------------------------- |
| `door`         | `var`/`let`        | Dynamic variable declaration | `door magac = "Ahmed"`                 |
| `tiro`         | `int`/`number`     | Static number type           | `tiro da = 25`                         |
| `jajab`        | `float`/`decimal`  | Static decimal type          | `jajab qiimo = 3.14`                   |
| `qoraal`       | `string`           | Static string type           | `qoraal magac = "Sharafdin"`           |
| `bool`         | `bool`/`boolean`   | Static boolean type          | `bool waaRun = run`                    |
| `liis`         | `array`/`list`     | Static list type             | `liis numbers = [1, 2, 3]`             |
| `walax`        | `object`           | Static object type           | `walax person = { name: "Sharafdin" }` |

### Example

```
// Dynamic typing (type can change)
door magac = "Sharafdin"
door da = 30
da = "soddon"      // Valid: type can change with 'door'

// Static typing (type is fixed)
tiro tirada = 42
qoraal cinwaan = "Xamar"
bool waa_run = run

// This would cause an error:
// tirada = "42" // Error: Cannot assign string to tiro
```

## Control Flow Statements

Conditional statements allow your program to make decisions.

| Somali Keyword | English Equivalent | Description       | Example                                      |
| -------------- | ------------------ | ----------------- | -------------------------------------------- |
| `haddii`       | `if`               | If condition      | `haddii (x > 10) { bandhig("Weyn") }`        |
| `haddii_kale`  | `else if`          | Else if condition | `haddii_kale (x == 10) { bandhig("Dhexe") }` |
| `ugudambeyn`   | `else`             | Else condition    | `ugudambeyn { bandhig("Yar") }`              |
| `dooro`        | `switch`           | Switch statement  | `dooro (x) { xaalad 1 { bandhig("Hal") } }`  |
| `xaalad`       | `case`             | Case clause       | `xaalad "A" { bandhig("Option A") }`         |

### Example

```
// If-else statements
door buundada = 85

haddii (buundada >= 90) {
    bandhig("Darajada A")
} haddii_kale (buundada >= 80) {
    bandhig("Darajada B")
} haddii_kale (buundada >= 70) {
    bandhig("Darajada C")
} ugudambeyn {
    bandhig("Waa lagu dhacay")
}

// Switch-case statement
door doorasho = "B"

dooro (doorasho) {
    xaalad "A" {
        bandhig("Waa doorasho A")
    }
    xaalad "B" {
        bandhig("Waa doorasho B")
    }
    xaalad "C" {
        bandhig("Waa doorasho C")
    }
    ugudambeyn {
        bandhig("Doorasho aan la aqoon")
    }
}
```

## Loops

Loops allow you to repeat code multiple times.

| Somali Keyword | English Equivalent | Description              | Example                                   |
| -------------- | ------------------ | ------------------------ | ----------------------------------------- |
| `kuceli`       | `for`              | For loop                 | `kuceli (i 1 ilaa 5) { bandhig(i) }`      |
| `intay`        | `while`            | While loop               | `intay (x < 5) { bandhig(x); x = x + 1 }` |
| `jooji`        | `break`            | Break from loop          | `haddii (x == 3) { jooji }`               |
| `soco`         | `continue`         | Skip to next iteration   | `haddii (x == 3) { soco }`                |
| `ilaa`         | `to`               | Ending value in for loop | `kuceli (i 1 ilaa 5) { bandhig(i) }`      |

### Example

```
// For loop
kuceli (i 1 ilaa 5) {
    bandhig("Tirsi: " + i)
}

// For loop with step
kuceli (j 1 ilaa 10 :: 2) {
    bandhig("Step: " + j)
}

// While loop
door j = 1
intay (j <= 5) {
    bandhig("J waa: " + j)
    j = j + 1
}

// Using break and continue
door k = 0
intay (run) {  // Infinite loop with break
    k = k + 1

    haddii (k == 3) {
        soco  // Skip printing 3
    }

    haddii (k > 5) {
        jooji  // Exit loop when k > 5
    }

    bandhig(k)
}
```

## Functions

Functions allow you to organize and reuse code.

| Somali Keyword | English Equivalent | Description                | Example                            |
| -------------- | ------------------ | -------------------------- | ---------------------------------- |
| `hawl`         | `function`         | Function declaration       | `hawl isuGee(a, b) { celi a + b }` |
| `celi`         | `return`           | Return value from function | `celi x * 2`                       |

### Example

```
// Function that returns the sum of two numbers
hawl isuDar(tiro a, tiro b) {
    celi a + b
}

// Function that greets a person
hawl salaan(qoraal magac) {
    celi "Salaam, " + magac + "!"
}

// Function with no parameters
hawl sayHello() {
    bandhig("Salaam, Adduunka!")
}

// Using the functions
bandhig(isuDar(5, 3))    // Outputs: 8
bandhig(salaan("Ahmed")) // Outputs: Salaam, Ahmed!
sayHello()           // Outputs: Salaam, Adduunka!
```

## Data Types and Values

Soplang supports various data types and special values.

| Somali Keyword/Value | English Equivalent | Description         | Example                    |
| -------------------- | ------------------ | ------------------- | -------------------------- |
| `tiro`               | `number`/`int`     | Integer type        | `tiro x = 42`              |
| `jajab`              | `decimal`/`float`  | Decimal type        | `jajab x = 3.14`           |
| `qoraal`             | `string`           | Text type           | `qoraal s = "soplang"`     |
| `bool`               | `boolean`          | Truth value type    | `bool b = run`             |
| `liis`               | `array`/`list`     | List type           | `liis l = [1, 2, 3]`       |
| `walax`              | `object`           | Object type         | `walax o = { a: 1, b: 2 }` |
| `run`                | `true`             | Boolean true value  | `door check = run`         |
| `been`               | `false`            | Boolean false value | `door check = been`        |
| `maran`              | `null`             | Null/empty value    | `door empty = maran`       |

### Example

```
// Number
tiro da = 25

// Decimal
jajab qiimo = 3.14

// String
qoraal magac = "Sharafdin"

// Boolean
bool waa_arday = run

// List/Array
liis tirooyin = [1, 2, 3, 4, 5]

// Object
walax qof = {
    "magac": "Ahmed Ali",
    "da": 30,
    "waa_arday": been
}

// Special values
door empty = maran
```

## Operators

Operators allow you to perform calculations and comparisons.

| Operator | English Equivalent | Description              | Example                                    |
| -------- | ------------------ | ------------------------ | ------------------------------------------ |
| `+`      | `+`                | Addition/concatenation   | `x = a + b` or `s = "Hello" + "World"`     |
| `-`      | `-`                | Subtraction              | `x = a - b`                                |
| `*`      | `*`                | Multiplication           | `x = a * b`                                |
| `/`      | `/`                | Division                 | `x = a / b`                                |
| `%`      | `%`                | Modulus/remainder        | `x = a % b`                                |
| `==`     | `==`               | Equal to                 | `haddii (a == b) { /* code */ }`           |
| `!=`     | `!=`               | Not equal to             | `haddii (a != b) { /* code */ }`           |
| `>`      | `>`                | Greater than             | `haddii (a > b) { /* code */ }`            |
| `<`      | `<`                | Less than                | `haddii (a < b) { /* code */ }`            |
| `>=`     | `>=`               | Greater than or equal to | `haddii (a >= b) { /* code */ }`           |
| `<=`     | `<=`               | Less than or equal to    | `haddii (a <= b) { /* code */ }`           |
| `&&`     | `&&`               | Logical AND              | `haddii (a > 0 && b > 0) { /* code */ }`   |
| `\|\|`   | `\|\|`             | Logical OR               | `haddii (a > 0 \|\| b > 0) { /* code */ }` |
| `!`      | `!`                | Logical NOT              | `haddii (!waaRun) { /* code */ }`          |

> **Note:** Soplang supports using comparison operators directly in expressions without requiring additional parentheses. This means you can use comparison expressions in variable assignments, function arguments, switch statements, and more.

### Example

```
// Arithmetic operators
door a = 10
door b = 3
door c = a + b    // 13
door d = a - b    // 7
door e = a * b    // 30
door f = a / b    // 3.33...
door g = a % b    // 1 (remainder)

// String concatenation
door magac = "Ahmed"
door salaan = "Salaam, " + magac + "!"  // "Salaam, Ahmed!"

// Comparison operators
haddii (a > b) {
    bandhig("A waa ka weyn yahay B")
}

haddii (a == 10) {
    bandhig("A waa 10")
}

// Direct use of comparison operators in expressions
door passing_score = 60
door student_score = 85
// This works without parentheses:
door passed = student_score > passing_score  // Assigns 'run' (true)

// Using comparison expressions in switch statements
dooro (student_score >= 90) {
    xaalad run {
        bandhig("Waxaad heshay darajo A")  // You got an A grade
    }
    xaalad been {
        bandhig("Waxaad heshay darajo kale")  // You got another grade
    }
}

// Complex expressions in assignments
door is_eligible = (age >= 18) && (score > 70 || experience >= 2)

// Logical operators
haddii (a > 5 && b < 5) {
    bandhig("Labada shuruud waa run")
}

haddii (a > 20 || b < 5) {
    bandhig("Mid ka mid ah shuruudaha waa run")
}

haddii (!been) {
    bandhig("Been ma ahan run")
}
```

## Built-in Functions

Soplang provides several built-in functions for common operations.

| Function  | English Equivalent | Description        | Example                                  |
| --------- | ------------------ | ------------------ | ---------------------------------------- |
| `bandhig` | `print`            | Output to console  | `bandhig("Salaam, Adduunka!")`           |
| `gelin`   | `input`            | Get user input     | `door magac = gelin("Magacaaga: ")`      |
| `nooc`    | `typeof`           | Get variable type  | `bandhig(nooc(x))`                       |
| `tiro`    | `int`              | Convert to integer | `door n = tiro("42")`                    |
| `jajab`   | `float`/`decimal`  | Convert to decimal | `door n = jajab("3.14")`                 |
| `qoraal`  | `str`              | Convert to string  | `door s = qoraal(42)`                    |
| `bool`    | `bool`             | Convert to boolean | `door b = bool(1)`                       |
| `liis`    | `list`/`array`     | Create a list      | `door list = liis(1, 2, 3)`              |
| `walax`   | `object`/`dict`    | Create an object   | `door obj = walax(name: "Ali", age: 25)` |

### Example

```
// Printing to console
bandhig("Salaam, Adduunka!")

// Getting user input
bandhig("Fadlan gali magacaaga:")
door magac = gelin()
bandhig("Salaam, " + magac + "!")

// Type conversion
door number_string = "42"
door actual_number = tiro(number_string)
door result = actual_number + 8    // 50

// Getting type
door x = 42
door y = "hello"
bandhig(nooc(x))    // "tiro"
bandhig(nooc(y))    // "qoraal"
```

## Lists and Methods

Lists (arrays) are a collection of items that can be accessed by index.

| Somali Function | English Equivalent    | Description          | Example                               |
| --------------- | --------------------- | -------------------- | ------------------------------------- |
| `kudar`         | `push`/`append`       | Add item to list     | `myList.kudar(newItem)`               |
| `kasaar`        | `pop`                 | Remove last item     | `door lastItem = myList.kasaar()`     |
| `dherer`        | `length`/`size`       | Get list length      | `door size = myList.dherer()`         |
| `kudar`         | `concat`              | Combine lists        | `door combined = list1.kudar(list2)`  |
| `leeyahay`      | `contains`/`includes` | Check if item exists | `door exists = myList.leeyahay(item)` |

### Example

```
// Creating a list
liis ardayda = ["Ali", "Farah", "Muna", "Hassan"]

// Accessing by index (zero-based)
bandhig(ardayda[0])    // "Ali"
bandhig(ardayda[2])    // "Muna"

// Getting the length
door tirada = ardayda.dherer()
bandhig("Tirada ardayda: " + tirada)    // "Tirada ardayda: 4"

// Adding an item
ardayda.kudar("Amina")
bandhig(ardayda)    // ["Ali", "Farah", "Muna", "Hassan", "Amina"]

// Removing the last item
door last = ardayda.kasaar()
bandhig(last)      // "Amina"
bandhig(ardayda)   // ["Ali", "Farah", "Muna", "Hassan"]

// Checking if an item exists
door exists = ardayda.leeyahay("Farah")
bandhig(exists)    // run (true)

// Combining lists
liis fasalka1 = ["Ali", "Farah"]
liis fasalka2 = ["Muna", "Hassan"]
liis dhamaan = fasalka1.kudar(fasalka2)
bandhig(dhamaan)   // ["Ali", "Farah", "Muna", "Hassan"]
```

## Objects and Methods

Objects are collections of key-value pairs that represent real-world entities.

| Somali Function | English Equivalent     | Description         | Example                                |
| --------------- | ---------------------- | ------------------- | -------------------------------------- |
| `fure`          | `keys`                 | Get all object keys | `door keys = myObj.fure()`             |
| `leeyahay`      | `has`/`hasOwnProperty` | Check if key exists | `door exists = myObj.leeyahay("name")` |
| `tirtir`        | `delete`               | Remove a property   | `myObj.tirtir("oldProp")`              |
| `kudar`         | `merge`/`assign`       | Combine objects     | `door merged = obj1.kudar(obj2)`       |

### Example

```
// Creating an object
walax qof = {
    "magac": "Ahmed Ali",
    "da": 30,
    "deggan": "Xamar"
}

// Accessing properties
bandhig(qof["magac"])    // "Ahmed Ali"
bandhig(qof.da)          // 30

// Adding/modifying properties
qof.shaqo = "Macallin"
qof.da = 31

// Getting all keys
door furayaasha = qof.fure()
bandhig(furayaasha)    // ["magac", "da", "deggan", "shaqo"]

// Checking if a property exists
door haystaa_shaqo = qof.leeyahay("shaqo")
bandhig(haystaa_shaqo)    // run (true)

// Removing a property
qof.tirtir("deggan")
bandhig(qof.leeyahay("deggan"))    // been (false)

// Merging objects
walax faahfaahin = {
    "jinsiyad": "Soomaali",
    "luuqadaha": ["Soomaali", "Carabi", "Ingiriisi"]
}

door qof_dhamaystiray = qof.kudar(faahfaahin)
bandhig(qof_dhamaystiray.luuqadaha[0])    // "Soomaali"
```

## Comments

Comments allow you to add notes to your code that are not executed.

| Syntax  | English Equivalent | Description         | Example                        |
| ------- | ------------------ | ------------------- | ------------------------------ |
| `//`    | `//`               | Single-line comment | `// Tani waa comment`          |
| `/* */` | `/* */`            | Multi-line comment  | `/* Tani waa comment dheer */` |

### Example

```
// Tani waa single-line comment

/* Tani waa multi-line comment
   Waxaad ku qori kartaa waxyaabo badan
   Dhowr sadar ah */

// Comments that explain code
door a = 5    // Assign 5 to variable a
door b = 10   // Assign 10 to variable b
door sum = a + b    // Calculate sum
bandhig(sum)      // Print the result
```

## Complete Example Program

Here's a complete example program that demonstrates the key features of Soplang:

```
// Soplang Example Program
// Student Grade Calculator

/* This program:
   1. Takes student names and their scores
   2. Calculates the grade based on the score
   3. Displays the results
*/

// Function to calculate the grade
hawl xisaabi_darajada(tiro dhibcaha) {
    haddii (dhibcaha >= 90) {
        celi "A"
    } haddii_kale (dhibcaha >= 80) {
        celi "B"
    } haddii_kale (dhibcaha >= 70) {
        celi "C"
    } haddii_kale (dhibcaha >= 60) {
        celi "D"
    } haddii_kalena {
        celi "F"
    }
}

// Main program
bandhig("=== Xisaabiyaha Darajooyinka Ardayda ===")

// Create a list to store student information
liis ardayda = []

// Get number of students
bandhig("Fadlan gali tirada ardayda:")
door tirada_ardayda = tiro(gelin())

// Input loop
kuceli i min 1 ilaa tirada_ardayda {
    bandhig("\nArday #" + i)
    bandhig("Fadlan gali magaca ardayga:")
    door magac = gelin()

    bandhig("Fadlan gali dhibcaha ardayga (0-100):")
    door dhibco = tiro(gelin())

    // Validate score
    inta_ay (dhibco < 0 || dhibco > 100) {
        bandhig("Dhibcaha waa inay u dhexeeyaan 0 iyo 100. Fadlan mar kale gali:")
        dhibco = tiro(gelin())
    }

    // Calculate grade
    door darajada = xisaabi_darajada(dhibco)

    // Create student object
    walax arday = {
        "magac": magac,
        "dhibco": dhibco,
        "darajo": darajada
    }

    // Add to list
    ardayda.kudar(arday)
}

// Display results
bandhig("\n=== Natiijooyinka Ardayda ===")

door wadarta_dhibcaha = 0
door tirada_guulaystay = 0

kuceli i min 0 ilaa ardayda.dherer() - 1 {
    door arday = ardayda[i]

    bandhig("Magaca: " + arday.magac)
    bandhig("Dhibcaha: " + arday.dhibco)
    bandhig("Darajada: " + arday.darajo)

    haddii (arday.dhibco >= 60) {
        bandhig("Xaaladda: Guulaystay!")
        tirada_guulaystay = tirada_guulaystay + 1
    } haddii_kalena {
        bandhig("Xaaladda: Ku dhacay")
    }

    bandhig("-----------------------")

    wadarta_dhibcaha = wadarta_dhibcaha + arday.dhibco
}

// Calculate statistics
door celceliska = wadarta_dhibcaha / ardayda.dherer()
door boqolkiiba_guul = (tirada_guulaystay / ardayda.dherer()) * 100

bandhig("\n=== Tirakoobka Fasalka ===")
bandhig("Tirada Ardayda: " + ardayda.dherer())
bandhig("Celceliska Dhibcaha: " + celceliska)
bandhig("Boqolkiiba Guulaystay: " + boqolkiiba_guul + "%")
```

This program demonstrates:
- Variable declarations (static and dynamic)
- Functions with return values
- Conditional statements
- Loops
- Lists and their methods
- Objects and property access
- Input/output operations
- Type conversion
- Arithmetic operations
- Logical conditions
- Comments

---

This MVP documentation covers the core features of Soplang, making programming accessible to Somali speakers. The language combines intuitive Somali keywords with familiar programming concepts, creating a bridge to computer science education.
