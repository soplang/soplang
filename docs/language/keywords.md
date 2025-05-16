# Soplang Keywords Reference

This document provides a reference for all keywords in the Soplang programming language, along with their meanings and examples of usage.

## Variable Declaration Keywords

| Keyword  | Meaning                       | English Equivalent | Example                                |
| -------- | ----------------------------- | ------------------ | -------------------------------------- |
| `door`   | Dynamic variable declaration  | `var`/`let`        | `door magac = "Sharafdin"`             |
| `madoor` | Constant variable declaration | `const`            | `madoor PI = 3.14159`                  |
| `tiro`   | Integer type                  | `int`              | `tiro da = 25`                         |
| `jajab`  | Decimal/float type            | `float`/`double`   | `jajab qiimo = 3.14`                   |
| `qoraal` | String type                   | `string`           | `qoraal magac = "Sharafdin"`           |
| `bool`   | Boolean type                  | `bool`             | `bool waaRun = run`                    |
| `walax`  | Object type                   | `object`           | `walax person = { name: "Sharafdin" }` |
| `liis`   | List/array type               | `array`            | `liis numbers = [1, 2, 3]`             |
| `maran`  | Null value                    | `null`             | `door a = maran`                       |

## Control Flow Keywords

| Keyword       | Meaning            | English Equivalent | Example                                      |
| ------------- | ------------------ | ------------------ | -------------------------------------------- |
| `haddii`      | If statement       | `if`               | `haddii (x > 10) { bandhig("Weyn") }`        |
| `haddii_kale` | Else if statement  | `else if`          | `haddii_kale (x == 10) { bandhig("Dhexe") }` |
| `ugudambeyn`  | Else statement     | `else`             | `ugudambeyn { bandhig("Yar") }`              |
| `dooro`       | Switch statement   | `switch`           | `dooro (x) { xaalad 1 { bandhig("Hal") } }`  |
| `xaalad`      | Case clause        | `case`             | `xaalad "A" { bandhig("Case A") }`           |
| `kuceli`      | For loop           | `for`              | `kuceli (i 1 ilaa 5) { bandhig(i) }`         |
| `ilaa`        | Loop range end     | `to`               | `kuceli (i 1 ilaa 5) { bandhig(i) }`         |
| `::`          | Loop increment     | `step`             | `kuceli (i 1 ilaa 10 :: 2) { bandhig(i) }`   |
| `intay`       | While loop         | `while`            | `intay (x < 5) { bandhig(x) }`               |
| `jooji`       | Break statement    | `break`            | `haddii (x == 3) { jooji }`                  |
| `soco`        | Continue statement | `continue`         | `haddii (x == 3) { soco }`                   |

## Function Keywords

| Keyword | Meaning              | English Equivalent | Example                            |
| ------- | -------------------- | ------------------ | ---------------------------------- |
| `hawl`  | Function declaration | `function`         | `hawl isuGee(a, b) { celi a + b }` |
| `celi`  | Return statement     | `return`           | `celi x * 2`                       |

## Special Values

| Somali Value | English Equivalent | Description         | Example                  |
| ------------ | ------------------ | ------------------- | ------------------------ |
| `run`        | `true`             | Boolean true value  | `haddii (run) { ... }`   |
| `been`       | `false`            | Boolean false value | `haddii (!been) { ... }` |
| `maran`      | `null`             | Empty/null value    | `door val = maran`       |

## Data Types

| Somali Type | English Equivalent | Description     | Example                          |
| ----------- | ------------------ | --------------- | -------------------------------- |
| `tiro`      | `int`/`number`     | Integer numbers | `tiro age = 25`                  |
| `jajab`     | `float`/`decimal`  | Decimal numbers | `jajab pi = 3.14`                |
| `qoraal`    | `string`           | Text values     | `qoraal name = "Ahmed"`          |
| `bool`      | `boolean`          | Truth values    | `bool isValid = run`             |
| `liis`      | `list`/`array`     | List of items   | `liis numbers = [1, 2, 3]`       |
| `walax`     | `object`           | Key-value pairs | `walax person = { name: "Ali" }` |

## Operators

| Somali Operator | English Equivalent | Description              | Example                   |
| --------------- | ------------------ | ------------------------ | ------------------------- |
| `+`             | `+`                | Addition                 | `x = a + b`               |
| `-`             | `-`                | Subtraction              | `x = a - b`               |
| `*`             | `*`                | Multiplication           | `x = a * b`               |
| `/`             | `/`                | Division                 | `x = a / b`               |
| `%`             | `%`                | Modulo                   | `x = a % b`               |
| `==`            | `==`               | Equal to                 | `haddii (a == b) {...}`   |
| `!=`            | `!=`               | Not equal to             | `haddii (a != b) {...}`   |
| `>`             | `>`                | Greater than             | `haddii (a > b) {...}`    |
| `<`             | `<`                | Less than                | `haddii (a < b) {...}`    |
| `>=`            | `>=`               | Greater than or equal to | `haddii (a >= b) {...}`   |
| `<=`            | `<=`               | Less than or equal to    | `haddii (a <= b) {...}`   |
| `&&`            | `&&`               | Logical AND              | `haddii (a && b) {...}`   |
| `\|\|`          | `\|\|`             | Logical OR               | `haddii (a \|\| b) {...}` |
| `!`             | `!`                | Logical NOT              | `haddii (!a) {...}`       |

> **Note:** Soplang supports the use of comparison operators directly in expressions without requiring additional parentheses. For example, `door x = a > b` is valid syntax to store the result of a comparison in a variable.

## Built-in Functions

| Function  | Meaning               | English Equivalent | Example                                  |
| --------- | --------------------- | ------------------ | ---------------------------------------- |
| `bandhig` | Print to console      | `print`            | `bandhig("Salaan, Adduunka!")`           |
| `gelin`   | Read input from user  | `input`            | `door magac = gelin("Magacaaga geli: ")` |
| `nooc`    | Get type of variable  | `typeof`           | `bandhig(nooc(magac))`                   |
| `tiro`    | Convert to number     | `int`/`float`      | `door n = tiro("5")`                     |
| `qoraal`  | Convert to string     | `str`              | `door s = qoraal(25)`                    |
| `bool`    | Convert to boolean    | `bool`             | `door b = bool(1)`                       |
| `liis`    | Create a list         | `list/array`       | `door list = liis(1, 2, 3)`              |
| `walax`   | Create an object      | `object/dict`      | `door obj = walax(name: "Ali", age: 25)` |
| `daji`    | Round down to integer | `Math.floor()`     | `door n = daji(4.7)`                     |
| `kor`     | Round up to integer   | `Math.ceil()`      | `door n = kor(4.2)`                      |
| `dherer`  | Get length of value   | `len()`/`.length`  | `door n = dherer(qoraal)`                |
| `kudhow`  | Get random value      | `random()`         | `door n = kudhow(1, 6)`                  |

## List Methods

| Method            | English Equivalent        | Description                   | Example                                 |
| ----------------- | ------------------------- | ----------------------------- | --------------------------------------- |
| `dherer()`        | `length()`                | Get list length               | `numbers.dherer()`                      |
| `kudar()`         | `push()` or `append()`    | Add item to end               | `numbers.kudar(5)`                      |
| `kasaar()`        | `pop()`                   | Remove and return last item   | `door last = numbers.kasaar()`          |
| `kudar(liis)`     | `concat()`                | Concatenate lists             | `door all = list1.kudar(list2)`         |
| `leeyahay(x)`     | `contains()`/`includes()` | Check if item exists          | `haddii (list.leeyahay(x)) {...}`       |
| `nuqul()`         | `copy()`                  | Create a shallow copy         | `door copy = list.nuqul()`              |
| `nadiifi()`       | `clear()`                 | Remove all items from list    | `list.nadiifi()`                        |
| `rog()`           | `reverse()`               | Reverse the list in-place     | `list.rog()`                            |
| `habee()`         | `sort()`                  | Sort the list in-place        | `list.habee()`                          |
| `jar(a, b)`       | `slice(a, b)`             | Return sublist from a to b    | `door subset = numbers.jar(1, 3)`       |
| `aaddin(func)`    | `map(func)`               | Transform items with function | `door doubled = nums.aaddin("laban")`   |
| `shaandhee(func)` | `filter(func)`            | Filter items with function    | `door evens = nums.shaandhee("isEven")` |
| `raadso(item)`    | `indexOf(item)`           | Find index of item            | `door idx = nums.raadso(5)`             |

## Object Methods

| Method        | English Equivalent   | Description           | Example                               |
| ------------- | -------------------- | --------------------- | ------------------------------------- |
| `fure()`      | `keys()`             | Get all keys          | `door keys = obj.fure()`              |
| `qiime()`     | `values()`           | Get all values        | `door values = obj.qiime()`           |
| `lamaane()`   | `entries()`          | Get key-value pairs   | `door pairs = obj.lamaane()`          |
| `leeyahay(x)` | `hasOwnProperty()`   | Check if key exists   | `haddii (obj.leeyahay("name")) {...}` |
| `tir(x)`   | `delete property`    | Delete a property     | `obj.tir("oldProp")`               |
| `kudar(obj)`  | `merge()`/`assign()` | Merge/copy properties | `door merged = obj1.kudar(obj2)`      |
| `nuqul()`     | `copy()`             | Create a shallow copy | `door copy = obj.nuqul()`             |
| `nadiifi()`   | `clear()`            | Remove all properties | `obj.nadiifi()`                       |

## String Methods

| Method            | English Equivalent | Description                           | Example                                  |
| ----------------- | ------------------ | ------------------------------------- | ---------------------------------------- |
| `qeybi(xad)`    | `split()`          | Split string by delimiter             | `door parts = text.qeybi(",")`           |
| `leeyahay(sub)`   | `includes()`       | Check if string contains substring    | `haddii (text.leeyahay("search")) {...}` |
| `dhamaad(sub)`    | `endsWith()`       | Check if string ends with substring   | `haddii (text.dhamaad("ing")) {...}`     |
| `bilow(sub)`      | `startsWith()`     | Check if string starts with substring | `haddii (text.bilow("http")) {...}`      |
| `beddel(x, y)`    | `replace()`        | Replace substring x with y            | `door new = text.beddel("old", "new")`   |
| `kudar(liis)`     | `join()`           | Join list of strings with separator   | `door text = ", ".kudar(names)`          |
| `jar(start, end)` | `slice()`          | Extract substring from start to end   | `door sub = text.jar(0, 3)`              |
