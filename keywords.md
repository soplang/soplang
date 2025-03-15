# Soplang Keywords Reference

This document provides a reference for all keywords in the Soplang programming language, along with their meanings and examples of usage.

## Variable Declaration Keywords

| Keyword  | Meaning                      | English Equivalent | Example                                 |
| -------- | ---------------------------- | ------------------ | --------------------------------------- |
| `door`   | Dynamic variable declaration | `var`/`let`        | `door magac = "Sharafdin"`              |
| `tiro`   | Integer type                 | `int`              | `tiro da = 25`                          |
| `qoraal` | String type                  | `string`           | `qoraal magac = "Sharafdin"`            |
| `boole`  | Boolean type                 | `bool`             | `boole waaRun = true`                   |
| `waxa`   | Object type                  | `object`           | `waxa person = { "name": "Sharafdin" }` |
| `liis`   | List/array type              | `array`            | `liis numbers = [1, 2, 3]`              |
| `fadhi`  | Float type                   | `float`            | `fadhi qiimo = 3.14`                    |
| `waxaba` | Null value                   | `null`             | `door a = waxaba`                       |

## Control Flow Keywords

| Keyword        | Meaning           | English Equivalent | Example                                 |
| -------------- | ----------------- | ------------------ | --------------------------------------- |
| `haddii`       | If statement      | `if`               | `haddii (x > 10) { qor("Weyn") }`       |
| `haddiiKale`   | Else if statement | `else if`          | `haddiiKale (x == 10) { qor("Dhexe") }` |
| `haddiiKalena` | Else statement    | `else`             | `haddiiKalena { qor("Yar") }`           |

## Loop Keywords

| Keyword  | Meaning             | English Equivalent | Example                                |
| -------- | ------------------- | ------------------ | -------------------------------------- |
| `kuCeli` | For loop            | `for`              | `kuCeli i min 1 ilaa 5 { qor(i) }`     |
| `intaAy` | While loop          | `while`            | `intaAy (x < 5) { qor(x); x = x + 1 }` |
| `jooji`  | Break statement     | `break`            | `haddii (x == 3) { jooji }`            |
| `siiWad` | Continue statement  | `continue`         | `haddii (x == 3) { siiWad }`           |
| `min`    | From (in for loops) | `from`             | `kuCeli i min 1 ilaa 5 { qor(i) }`     |
| `ilaa`   | To (in for loops)   | `to`               | `kuCeli i min 1 ilaa 5 { qor(i) }`     |

## Function Keywords

| Keyword    | Meaning              | English Equivalent | Example                                |
| ---------- | -------------------- | ------------------ | -------------------------------------- |
| `howl`     | Function declaration | `function`         | `howl isuGee(a, b) { soo_celi a + b }` |
| `soo_celi` | Return statement     | `return`           | `soo_celi x * 2`                       |

## Error Handling Keywords

| Keyword    | Meaning     | English Equivalent | Example                         |
| ---------- | ----------- | ------------------ | ------------------------------- |
| `isku_day` | Try block   | `try`              | `isku_day { /* code */ }`       |
| `qabo`     | Catch block | `catch`            | `qabo (khalad) { qor(khalad) }` |

## Object-Oriented Programming Keywords

| Keyword  | Meaning                       | English Equivalent | Example                                         |
| -------- | ----------------------------- | ------------------ | ----------------------------------------------- |
| `fasal`  | Class declaration             | `class`            | `fasal Qof { /* properties and methods */ }`    |
| `this`   | Reference to current instance | `this`             | `this.magac = magac`                            |
| `dhaxal` | Inheritance                   | `extends`          | `fasal Ardayga dhaxal Qof { /* class body */ }` |

## Module Keywords

| Keyword    | Meaning          | English Equivalent | Example                                         |
| ---------- | ---------------- | ------------------ | ----------------------------------------------- |
| `keeno`    | Import statement | `import`           | `keeno "math_lib.so"`                           |
| `siidaayo` | Export statement | `export`           | `siidaayo howl isuGee(a, b) { soo_celi a + b }` |

## Operators

| Operator | Meaning                  | English Equivalent | Example                                    |
| -------- | ------------------------ | ------------------ | ------------------------------------------ |
| `+`      | Addition                 | `+`                | `x = a + b`                                |
| `-`      | Subtraction              | `-`                | `x = a - b`                                |
| `*`      | Multiplication           | `*`                | `x = a * b`                                |
| `/`      | Division                 | `/`                | `x = a / b`                                |
| `%`      | Modulus                  | `%`                | `x = a % b`                                |
| `==`     | Equal to                 | `==`               | `haddii (a == b) { /* code */ }`           |
| `!=`     | Not equal to             | `!=`               | `haddii (a != b) { /* code */ }`           |
| `>`      | Greater than             | `>`                | `haddii (a > b) { /* code */ }`            |
| `<`      | Less than                | `<`                | `haddii (a < b) { /* code */ }`            |
| `>=`     | Greater than or equal to | `>=`               | `haddii (a >= b) { /* code */ }`           |
| `<=`     | Less than or equal to    | `<=`               | `haddii (a <= b) { /* code */ }`           |
| `&&`     | Logical AND              | `&&`               | `haddii (a > 0 && b > 0) { /* code */ }`   |
| `\|\|`   | Logical OR               | `\|\|`             | `haddii (a > 0 \|\| b > 0) { /* code */ }` |
| `!`      | Logical NOT              | `!`                | `haddii (!waaRun) { /* code */ }`          |

## Built-in Functions

| Function | Meaning                   | English Equivalent | Example                                  |
| -------- | ------------------------- | ------------------ | ---------------------------------------- |
| `qor`    | Print to console          | `print`            | `qor("Salaan, Adduunka!")`               |
| `akhri`  | Read input from user      | `input`            | `door magac = akhri("Magacaaga geli: ")` |
| `nuuc`   | Get type of variable      | `typeof`           | `qor(nuuc(magac))`                       |
| `dherer` | Get length of string/list | `len`/`length`     | `qor(dherer(magacayga))`                 |
| `tiro`   | Convert to number         | `int`/`float`      | `door n = tiro("5")`                     |
| `qoraal` | Convert to string         | `str`              | `door s = qoraal(25)`                    |
| `boole`  | Convert to boolean        | `bool`             | `door b = boole(1)`                      |

## Special Syntax

| Syntax  | Meaning             | English Equivalent | Example                           |
| ------- | ------------------- | ------------------ | --------------------------------- |
| `//`    | Single-line comment | `//`               | `// Tani waa comment`             |
| `/* */` | Multi-line comment  | `/* */`            | `/* Tani waa comment dheer */`    |
| `{ }`   | Code block          | `{ }`              | `haddii (x > 10) { qor("Weyn") }` |
| `( )`   | Expression grouping | `( )`              | `(a + b) * c`                     |
| `[ ]`   | List/array access   | `[ ]`              | `numbers[0]`                      |
| `.`     | Property access     | `.`                | `person.name`                     |