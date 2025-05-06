# Soplang Keywords Reference

This document provides a reference for all keywords in the Soplang programming language, along with their meanings and examples of usage.

## Variable Declaration Keywords

| Keyword     | Meaning                      | English Equivalent | Example                                 |
| ----------- | ---------------------------- | ------------------ | --------------------------------------- |
| `door`      | Dynamic variable declaration | `var`/`let`        | `door magac = "Sharafdin"`              |
| `tiro`      | Integer type                 | `int`              | `tiro da = 25`                          |
| `qoraal`    | String type                  | `string`           | `qoraal magac = "Sharafdin"`            |
| `labadaran` | Boolean type                 | `bool`             | `labadaran waaRun = true`               |
| `shey`      | Object type                  | `object`           | `shey person = { "name": "Sharafdin" }` |
| `liis`      | List/array type              | `array`            | `liis numbers = [1, 2, 3]`              |
| `maran`     | maran value                  | `maran`            | `door a = maran`                        |

## Control Flow Keywords

| Keyword         | Meaning           | English Equivalent | Example                                  |
| --------------- | ----------------- | ------------------ | ---------------------------------------- |
| `haddii`        | If statement      | `if`               | `haddii (x > 10) { qor("Weyn") }`        |
| `haddii_kale`   | Else if statement | `else if`          | `haddii_kale (x == 10) { qor("Dhexe") }` |
| `haddii_kalena` | Else statement    | `else`             | `haddii_kalena { qor("Yar") }`           |

## Loop Keywords

| Keyword   | Meaning             | English Equivalent | Example                                 |
| --------- | ------------------- | ------------------ | --------------------------------------- |
| `ku_celi` | For loop            | `for`              | `ku_celi i min 1 ilaa 5 { qor(i) }`     |
| `inta_ay` | While loop          | `while`            | `inta_ay (x < 5) { qor(x); x = x + 1 }` |
| `jooji`   | Break statement     | `break`            | `haddii (x == 3) { jooji }`             |
| `sii_wad` | Continue statement  | `continue`         | `haddii (x == 3) { sii_wad }`           |
| `min`     | From (in for loops) | `from`             | `ku_celi i min 1 ilaa 5 { qor(i) }`     |
| `ilaa`    | To (in for loops)   | `to`               | `ku_celi i min 1 ilaa 5 { qor(i) }`     |

## Function Keywords

| Keyword    | Meaning              | English Equivalent | Example                                |
| ---------- | -------------------- | ------------------ | -------------------------------------- |
| `howl`     | Function declaration | `function`         | `howl isuGee(a, b) { soo_celi a + b }` |
| `soo_celi` | Return statement     | `return`           | `soo_celi x * 2`                       |

## Error Handling Keywords

| Keyword    | Meaning            | English Equivalent | Example                         |
| ---------- | ------------------ | ------------------ | ------------------------------- |
| `isku_day` | Try block          | `try`              | `isku_day { /* code */ }`       |
| `qabo`     | Catch block        | `catch`            | `qabo (khalad) { qor(khalad) }` |
| `throw`    | Throw an exception | `throw`            | `throw "Error message"`         |

## Object-Oriented Programming Keywords

| Keyword     | Meaning                       | English Equivalent | Example                                              |
| ----------- | ----------------------------- | ------------------ | ---------------------------------------------------- |
| `fasalka`   | Class declaration             | `class`            | `fasalka Qof { /* properties and methods */ }`       |
| `nafta`     | Reference to current instance | `this`             | `nafta.magac = magac`                                |
| `ka_dhaxal` | Inheritance                   | `extends`          | `fasalka Ardayga ka_dhaxal Qof { /* class body */ }` |
| `cusub`     | New object instantiation      | `new`              | `door ardayga = cusub Ardayga()`                     |

## Module Keywords

| Keyword   | Meaning          | English Equivalent | Example                 |
| --------- | ---------------- | ------------------ | ----------------------- |
| `ka_keen` | Import statement | `import`           | `ka_keen "math_lib.so"` |

## Special Values

| Value   | Meaning       | English Equivalent | Example                  |
| ------- | ------------- | ------------------ | ------------------------ |
| `run`   | Boolean true  | `true`             | `labadaran check = run`  |
| `been`  | Boolean false | `false`            | `labadaran check = been` |
| `maran` | Null value    | `null`             | `door empty = maran`     |

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

| Function    | Meaning              | English Equivalent | Example                                  |
| ----------- | -------------------- | ------------------ | ---------------------------------------- |
| `qor`       | Print to console     | `print`            | `qor("Salaan, Adduunka!")`               |
| `akhri`     | Read input from user | `input`            | `door magac = akhri("Magacaaga geli: ")` |
| `nuuc`      | Get type of variable | `typeof`           | `qor(nuuc(magac))`                       |
| `tiro`      | Convert to number    | `int`/`float`      | `door n = tiro("5")`                     |
| `qoraal`    | Convert to string    | `str`              | `door s = qoraal(25)`                    |
| `labadaran` | Convert to boolean   | `bool`             | `door b = labadaran(1)`                  |
| `liis`      | Create a list        | `list/array`       | `door list = liis(1, 2, 3)`              |
| `shey`      | Create an object     | `object/dict`      | `door obj = shey(name: "Ali", age: 25)`  |

## List Methods

| Method      | Meaning              | English Equivalent  | Example                                |
| ----------- | -------------------- | ------------------- | -------------------------------------- |
| `kudar`     | Add item to list     | `push/append`       | `myList.kudar(newItem)`                |
| `kasaar`    | Remove last item     | `pop`               | `door lastItem = myList.kasaar()`      |
| `dherer`    | Get list length      | `length/size`       | `door size = myList.dherer()`          |
| `iskuxir`   | Concatenate lists    | `concat`            | `door combined = list1.iskuxir(list2)` |
| `ka_kooban` | Check if item exists | `contains/includes` | `door exists = myList.ka_kooban(item)` |

## Object Methods

| Method    | Meaning             | English Equivalent   | Example                               |
| --------- | ------------------- | -------------------- | ------------------------------------- |
| `furaha`  | Get object keys     | `keys`               | `door keys = myObj.furaha()`          |
| `haystaa` | Check if key exists | `has/hasOwnProperty` | `door exists = myObj.haystaa("name")` |
| `tirtir`  | Remove property     | `delete`             | `myObj.tirtir("oldProp")`             |
| `iskudar` | Merge objects       | `merge/assign`       | `door merged = obj1.iskudar(obj2)`    |

## Error Message Terminology

| Somali Term | Meaning            | English Equivalent | Example Usage                                                     |
| ----------- | ------------------ | ------------------ | ----------------------------------------------------------------- |
| `Khalad`    | Error              | `Error`            | `Khalad lexer: Xaraf aan la filayn: @`                            |
| `sadar`     | Line (in code)     | `line`             | `ee sadar 12, goobta 18`                                          |
| `goobta`    | Position (in code) | `position`         | `ee sadar 12, goobta 18`                                          |
| `lexer`     | Lexical analyzer   | `lexer`            | `Khalad lexer: Xaraf aan la filayn: @`                            |
| `parser`    | Syntax analyzer    | `parser`           | `Khalad parser: Waxaa la filayay ')', laakiin waxaa la helay '+'` |
| `runtime`   | Execution time     | `runtime`          | `Khalad runtime: Ma suurtogali karto qeybinta eber`               |
| `nuuc`      | Type               | `type`             | `Khalad nuuc: Qiimaheeda '10' ma ahan qoraal`                     |

## Special Syntax

| Syntax  | Meaning             | English Equivalent | Example                           |
| ------- | ------------------- | ------------------ | --------------------------------- |
| `//`    | Single-line comment | `//`               | `// Tani waa comment`             |
| `/* */` | Multi-line comment  | `/* */`            | `/* Tani waa comment dheer */`    |
| `{ }`   | Code block          | `{ }`              | `haddii (x > 10) { qor("Weyn") }` |
| `( )`   | Expression grouping | `( )`              | `(a + b) * c`                     |
| `[ ]`   | List/array access   | `[ ]`              | `numbers[0]`                      |
| `.`     | Property access     | `.`                | `person.name`                     |
