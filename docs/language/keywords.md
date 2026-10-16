# Soplang Keywords Reference

This document provides a reference for all keywords in the Soplang programming language, along with their meanings and examples of usage.

The [terminology review](terminology.md) records current names, source evidence,
and proposed alternatives. Pending alternatives are not part of this reference.

## Variable Declaration Keywords

| Keyword  | Meaning                       | English Equivalent | Example                                |
| -------- | ----------------------------- | ------------------ | -------------------------------------- |
| `door`   | Dynamic variable declaration  | `var`/`let`        | `door magac = "Sharafdin"`             |
| `madoor` | Constant variable declaration | `const`            | `madoor PI = 3.14159`                  |
| `abn`    | Integer type                  | `int`               | `abn da = 25`                          |
| `jajab`  | Decimal/float type            | `float`/`double`   | `jajab qiimo = 3.14`                   |
| `qoraal` | String type                   | `string`           | `qoraal magac = "Sharafdin"`           |
| `bool`   | Boolean type                  | `bool`             | `bool waaRun = run`                    |
| `walax`  | Object type                   | `object`           | `walax person = { name: "Sharafdin" }` |
| `teed`   | List/array type               | `array`            | `teed numbers = [1, 2, 3]`             |
| `maran`  | Null value                    | `null`             | `door a = maran`                       |

## Control Flow Keywords

| Keyword       | Meaning            | English Equivalent | Example                                      |
| ------------- | ------------------ | ------------------- | --------------------------------------------- |
| `haddii`      | If statement       | `if`                | `haddii (x > 10) { qor("Weyn") }`             |
| `haddii_kale` | Else if statement  | `else if`           | `haddii_kale (x == 10) { qor("Dhexe") }`      |
| `ugudambeyn`  | Else statement     | `else`              | `ugudambeyn { qor("Yar") }`                   |
| `dooro`       | Switch statement   | `switch`            | `dooro (x) { xaalad 1 { qor("Hal") } }`       |
| `xaalad`      | Case clause        | `case`              | `xaalad "A" { qor("Case A") }`                |
| `kuceli`      | For loop           | `for`               | `kuceli (i 1 ilaa 5) { qor(i) }`              |
| `ilaa`        | Loop range end     | `to`                | `kuceli (i 1 ilaa 5) { qor(i) }`              |
| `::`          | Loop step/increment amount (optional, follows the range) | `step` | `kuceli (i 1 ilaa 10 :: 2) { qor(i) }` — loops from 1 to 10, incrementing `i` by 2 each time |
| `intay`       | While loop         | `while`             | `intay (x < 5) { qor(x) }`                    |
| `jooji`       | Break statement    | `break`             | `haddii (x == 3) { jooji }`                   |
| `soco`        | Continue statement | `continue`          | `haddii (x == 3) { soco }`                    |

## Function Keywords

| Keyword | Meaning              | English Equivalent | Example                            |
| ------- | -------------------- | ------------------- | ----------------------------------- |
| `hawl`  | Function declaration | `function`          | `hawl isuGee(a, b) { celi a + b }` |
| `celi`  | Return statement     | `return`            | `celi x * 2`                       |

## Special Values

| Somali Value | English Equivalent | Description         | Example                  |
| ------------ | ------------------- | -------------------- | ------------------------- |
| `run`        | `true`              | Boolean true value   | `haddii (run) { ... }`   |
| `been`       | `false`             | Boolean false value  | `haddii (!been) { ... }` |
| `maran`      | `null`              | Empty/null value     | `door val = maran`       |

`maran` is the canonical null literal. `null` remains accepted as a compatibility spelling.

## Data Types

| Somali Type | English Equivalent | Description       | Example                          |
| ----------- | -------------------- | ------------------ | ---------------------------------- |
| `abn`       | `int`/`number`        | Integer numbers    | `abn age = 25`                    |
| `jajab`     | `float`/`decimal`     | Decimal numbers    | `jajab pi = 3.14`                  |
| `qoraal`    | `string`              | Text values         | `qoraal name = "Ahmed"`            |
| `bool`      | `boolean`             | Truth values        | `bool isValid = run`               |
| `teed`      | `list`/`array`        | List of items       | `teed numbers = [1, 2, 3]`         |
| `walax`     | `object`              | Key-value pairs     | `walax person = { name: "Ali" }`   |

## Operators

| Somali Operator | English Equivalent | Description              | Example                   |
| --------------- | -------------------- | -------------------------- | --------------------------- |
| `+`             | `+`                  | Addition                   | `x = a + b`                |
| `-`             | `-`                  | Subtraction                | `x = a - b`                |
| `*`             | `*`                  | Multiplication             | `x = a * b`                |
| `/`             | `/`                  | Division                   | `x = a / b`                |
| `%`             | `%`                  | Modulo                     | `x = a % b`                |
| `==`            | `==`                 | Equal to                   | `haddii (a == b) {...}`    |
| `!=`            | `!=`                 | Not equal to               | `haddii (a != b) {...}`    |
| `>`             | `>`                  | Greater than               | `haddii (a > b) {...}`     |
| `<`             | `<`                  | Less than                  | `haddii (a < b) {...}`     |
| `>=`            | `>=`                 | Greater than or equal to   | `haddii (a >= b) {...}`    |
| `<=`            | `<=`                 | Less than or equal to      | `haddii (a <= b) {...}`    |
| `&&`            | `&&`                 | Logical AND                | `haddii (a && b) {...}`    |
| `\|\|`          | `\|\|`               | Logical OR                 | `haddii (a \|\| b) {...}`  |
| `!`             | `!`                  | Logical NOT                | `haddii (!a) {...}`        |

> **Note:** Soplang supports the use of comparison operators directly in expressions without requiring additional parentheses. For example, `door x = a > b` is valid syntax to store the result of a comparison in a variable.

## Built-in Functions

| Function | Meaning                                    | English Equivalent | Example                                  |
| -------- | -------------------------------------------- | -------------------- | ------------------------------------------ |
| `qor`    | Print to console                             | `print`              | `qor("Salaan, Adduunka!")`                |
| `gelin`  | Read input from user                         | `input`              | `door magac = gelin("Magacaaga geli: ")`  |
| `nooc`   | Get type of variable                         | `typeof`             | `qor(nooc(magac))`                        |
| `abn`    | Convert value to a number (also the `int` type keyword above) | `int`/`float` | `door n = abn("5")`                 |
| `jajab`  | Convert a number or numeric string to a decimal | `float` | `door n = jajab("3.5")`             |
| `qoraal` | Convert to string                            | `str`                | `door s = qoraal(25)`                     |
| `bool`   | Convert to boolean                           | `bool`               | `door b = bool(1)`                        |
| `teed`   | Create a list                                | `list/array`         | `door list = teed(1, 2, 3)`               |
| `walax`  | Create an object                             | `object/dict`        | `door obj = walax("name", "Ali", "age", 25)` |
| `daji`   | Round down to integer                        | `Math.floor()`       | `door n = daji(4.7)`                      |
| `kor`    | Round up to integer                          | `Math.ceil()`        | `door n = kor(4.2)`                       |
| `dherer` | Count list items, object keys, or string characters | `len()`/`.length` | `door n = dherer(qoraal)`             |
| `xul`    | Get random value                             | `random()`           | `door n = xul(1, 6)`                      |
| `baaxad` | Create an integer range                      | `range()`            | `door n = baaxad(1, 5)`                   |
| `dhammeystir` | Round to the nearest integer            | `round()`            | `door n = dhammeystir(4.6)`               |
| `sugan`  | Get the absolute value                       | `abs()`              | `door n = sugan(-5)`                      |
| `ugu_yar` | Get the smallest numeric argument           | `min()`              | `door n = ugu_yar(4, 2, 8)`               |
| `ugu_weyn` | Get the largest numeric argument           | `max()`              | `door n = ugu_weyn(4, 2, 8)`              |
| `awood`  | Raise a number to a power                    | `pow()`              | `door n = awood(2, 3)`                    |
| `xidid`  | Calculate a square root                      | `sqrt()`             | `door n = xidid(9)`                       |

`xul()` returns a decimal in `[0, 1)`. `xul(a, b)` returns a decimal in `[a, b)`. `xul(teed)` returns one item and reports an error for an empty list.

> **Note:** `abn`, `qoraal`, `bool`, `teed`, and `walax` are used both as static **type keywords** (see Variable Declaration Keywords / Data Types above) and as **built-in conversion/constructor functions** here. The meaning depends on context: e.g. `abn age = 25` declares a typed variable, while `abn("5")` converts a string to a number.

## baaxad (Range)

`baaxad` waa hawl dhisay oo soo saarta liis tirooyin ah, sida Python `range`.

**Isticmaalka:**

```soplang
// baaxad(stop): 0 ilaa stop-1
qor(baaxad(5))  // [0, 1, 2, 3, 4]

// baaxad(start, stop): start ilaa stop-1
qor(baaxad(2, 6))  // [2, 3, 4, 5]

// baaxad(start, stop, step): start ilaa stop-1, talaabo kasta
qor(baaxad(1, 10, 2))  // [1, 3, 5, 7, 9]
```

`baaxad` accepts integers only. A zero step reports an error that can be caught with `fasax` and `qabo`.

## List Methods

| Method            | English Equivalent        | Description                     | Example                                 |
| ------------------ | --------------------------- | ---------------------------------- | ------------------------------------------ |
| `dherer()`         | `length()`                  | Get list length                    | `numbers.dherer()`                        |
| `kudar(item)`      | `push()` or `append()`      | Add item to the original list      | `numbers.kudar(5)`                        |
| `kasaar()`         | `pop()`                      | Remove and return last item        | `door last = numbers.kasaar()`            |
| `kudar(teed)`      | `concat()`                   | Concatenate lists                  | `door all = list1.kudar(list2)`           |
| `leeyahay(x)`      | `contains()`/`includes()`   | Check if item exists               | `haddii (list.leeyahay(x)) {...}`         |
| `nuqul()`          | `copy()`                     | Create a shallow copy              | `door copy = list.nuqul()`                |
| `nadiifi()`        | `clear()`                    | Remove all items from the original list | `list.nadiifi()`                     |
| `rog()`            | `reverse()`                  | Reverse the original list          | `list.rog()`                              |
| `habee()`          | `sort()`                     | Sort the original list in ascending order | `list.habee()`                    |
| `jar(a, b)`        | `slice(a, b)`                | Return sublist from a to b         | `door subset = numbers.jar(1, 3)`         |
| `aaddin(func)`     | `map(func)`                   | Transform items with function      | `door doubled = nums.aaddin(hawl(x) { celi x * 2 })` |
| `shaandhee(func)`  | `filter(func)`                | Filter items with function         | `door evens = nums.shaandhee(hawl(x) { celi x % 2 == 0 })` |
| `muuji(item)`      | `indexOf(item)`               | Find index of item                 | `door idx = nums.muuji(5)`                |
| `geli(index, item)` | `insert(index, item)`         | Insert into the original list      | `numbers.geli(1, 5)`                      |
| `ka_saar(index)`   | `remove(index)`                | Remove and return an indexed item  | `door item = numbers.ka_saar(1)`          |
| `koowaad()`        | `first()`                      | Return first item or `maran`       | `door item = numbers.koowaad()`           |
| `ugu_dambeeya()`   | `last()`                       | Return last item or `maran`        | `door item = numbers.ugu_dambeeya()`      |
| `isku_gee(func, bilow)` | `reduce(func, initial)`  | Fold items into one value          | `nums.isku_gee(hawl(a, b) { celi a + b }, 0)` |
| `dhammaan(func)`   | `every(func)`                  | Test whether every item matches    | `nums.dhammaan(hawl(x) { celi x > 0 })`   |
| `qaar(func)`       | `some(func)`                   | Test whether any item matches      | `nums.qaar(hawl(x) { celi x == 5 })`      |

`kudar(teed)` returns a concatenated copy. `nuqul()` and `jar()` also return new lists. `kasaar()` and `ka_saar()` return the removed value. Methods that otherwise mutate a list return the original list after changing it.

## Object Methods

| Method        | English Equivalent   | Description             | Example                               |
| -------------- | ---------------------- | -------------------------- | ---------------------------------------- |
| `fure()`      | `keys()`               | Get all keys              | `door keys = obj.fure()`               |
| `qiime()`     | `values()`             | Get all values             | `door values = obj.qiime()`            |
| `lamaane()`   | `entries()`            | Get key-value pairs        | `door pairs = obj.lamaane()`           |
| `leeyahay(x)` | `hasOwnProperty()`     | Check if key exists        | `haddii (obj.leeyahay("name")) {...}`  |
| `tir(x)`      | `delete property`      | Delete from the original object | `obj.tir("oldProp")`              |
| `kudar(obj)`  | `merge()`/`assign()`   | Return a merged copy       | `door merged = obj1.kudar(obj2)`       |
| `nuqul()`     | `copy()`               | Create a shallow copy      | `door copy = obj.nuqul()`              |
| `nadiifi()`   | `clear()`              | Remove all original properties | `obj.nadiifi()`                    |
| `hel(key, default?)` | `get()`            | Get a key or optional default  | `obj.hel("name", "Unknown")`     |
| `dhig(key, value)` | `set()`              | Set a computed key             | `obj.dhig("name", "Ali")`        |
| `ka_qaad(key)` | `remove()`              | Remove and return a value      | `door old = obj.ka_qaad("name")`  |

Objects also support computed bracket access: `obj["name"]` and `obj["name"] = "Ali"`.

## String Methods

| Method             | English Equivalent | Description                             | Example                                  |
| -------------------- | --------------------- | ------------------------------------------ | ------------------------------------------- |
| `qeybi(xad)`        | `split()`             | Split string by delimiter                  | `door parts = text.qeybi(",")`             |
| `leeyahay(sub)`     | `includes()`          | Check if string contains substring         | `haddii (text.leeyahay("search")) {...}`   |
| `dhamaad(sub)`      | `endsWith()`          | Check if string ends with substring        | `haddii (text.dhamaad("ing")) {...}`       |
| `bilow(sub)`        | `startsWith()`        | Check if string starts with substring      | `haddii (text.bilow("http")) {...}`        |
| `beddel(x, y)`      | `replace()`           | Replace substring x with y                 | `door new = text.beddel("old", "new")`     |
| `kudar(teed)`       | `join()`               | Join list of strings with separator        | `door text = ", ".kudar(names)`            |
| `jar(start, end)`   | `slice()`              | Slice by character position                 | `door sub = text.jar(0, 3)`                |
| `xarfaha_waaweyn()` | `toUpperCase()`        | Convert string to uppercase                | `door n = q.xarfaha_waaweyn()`             |
| `xarfaha_yaryar()`  | `toLowerCase()`        | Convert string to lowercase                | `door n = q.xarfaha_yaryar()`              |
| `masax()`           | `trim()`               | Remove leading/trailing whitespace         | `door n = q.masax()`                       |
| `raadi(sub)`        | `indexOf()`            | Find the index of a substring              | `door n = q.raadi("pro")`                  |
| `beddel_dhammaan(x, y)` | `replaceAll()`     | Replace all occurrences of x with y        | `door n = q.beddel_dhammaan("a", "x")`     |
| `dherer()`          | `length`              | Count Unicode characters                    | `door n = q.dherer()`                       |
| `masax_bilow()`     | `trimStart()`         | Remove leading whitespace                   | `door n = q.masax_bilow()`                  |
| `masax_dhamaad()`   | `trimEnd()`           | Remove trailing whitespace                  | `door n = q.masax_dhamaad()`                |
| `tiri(sub)`         | `count()`             | Count non-overlapping occurrences           | `door n = q.tiri("ab")`                    |
| `ku_celi(n)`        | `repeat()`            | Repeat the string                           | `door n = q.ku_celi(3)`                     |
| `raadi_dambe(sub)`  | `lastIndexOf()`       | Find the last character position            | `door n = q.raadi_dambe("ab")`             |

String positions and bracket indexes count characters rather than UTF-8 bytes. `jar()` and bracket indexing accept negative positions from the end. Missing indexes return `maran`; `raadi()` and `raadi_dambe()` return `-1` when they find no match. Calls with the wrong number of arguments report an error that can be caught with `fasax` and `qabo`.

### xarfaha_waaweyn (Uppercase)
**Isticmaal:**

```soplang
door q = "soplang"
door natiijo = q.xarfaha_waaweyn()
qor(natiijo)  # SOPLANG
```

### xarfaha_yaryar (Lowercase)
**Isticmaal:**

```soplang
door q = "SOPLANG"
door natiijo = q.xarfaha_yaryar()
qor(natiijo)  # soplang
```

### masax (Trim Whitespace)
**Isticmaal:**

```soplang
door q = "  soplang  "
door natiijo = q.masax()
qor(natiijo)  # soplang
```

### raadi (Find Substring)
**Isticmaal:**

```soplang
door q = "soplang programming"
door natiijo = q.raadi("pro")
qor(natiijo)  # 8
```

### beddel_dhammaan (Replace All)
**Isticmaal:**

```soplang
door q = "abaaba"
door natiijo = q.beddel_dhammaan("a", "x")
qor(natiijo)  # xbxxbx
```

## Object-Oriented Programming (OOP)

| Keyword / concept | Meaning | English | Example |
| -------------------- | ---------- | ---------- | ---------- |
| `qaab`   | Class definition                            | `class`          | `qaab Bisad { ... }`                        |
| `dhaxal` | Inherit from parent class                   | `extends`        | `qaab Bisad dhaxal Xayawaan { ... }`        |
| `cusub`  | Create a new instance                       | `new`             | `door b = cusub Bisad("Mia")`               |
| `nafta`  | Current instance (self)                     | `this` / `self`  | `nafta.magac = magac`                       |
| `waalid` | Call a method on the parent class           | `super`          | `celi waalid.cod()`                         |
| `dhaw`   | Constructor method (auto-called by `cusub`) | `constructor`    | `hawl dhaw(magac) { nafta.magac = magac }`  |

**Class example:**

```soplang
qaab Xayawaan {
    hawl cod() {
        celi "xayawaan"
    }
}

qaab Bisad dhaxal Xayawaan {
    hawl dhaw(magac) {
        nafta.magac = magac
    }
    hawl salaam() {
        qor("Salaan, " + nafta.magac)
    }
}

door bisad = cusub Bisad("Mia")
Bisad b = cusub Bisad("Mia")
bisad.salaam()
qor(bisad.magac)
```

Notes:
- Class bodies may contain only `hawl` (method) definitions.
- `nafta` is injected automatically as the first parameter of every class method.
- Instance fields are stored as object properties; read them with `obj.field`.
- Method lookup walks the inheritance chain (child overrides parent).

## Modules and error handling

| Keyword | Meaning | English | Example |
| --------- | ---------- | ---------- | ---------- |
| `keen`  | Import another `.sop` file       | `import` | `keen "lib.sop"`                          |
| `fasax` | Start a protected try block      | `try`    | `fasax { ... } qabo (e) { ... }`          |
| `qabo`  | Catch block with error variable  | `catch`  | `qabo (khalad) { qor(khalad) }`           |

**Import example:**

```soplang
keen "_math_lib.sop"
qor(laba() + afar())
```

Imported files are resolved relative to the current file's directory. Top-level definitions (functions, variables, classes) from the imported file become available in the importing program.

**Try/catch example:**

```soplang
fasax {
    qor(1 / 0)
} qabo (khalad) {
    qor("Qabtay: " + khalad)
}
```

Runtime errors inside the `fasax` block are caught; the error message is bound to the variable in `qabo (...)`.
