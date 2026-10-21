# Soplang Keywords Reference

This document provides a reference for all keywords in the Soplang programming language, along with their meanings and examples of usage.

The [terminology review](terminology.md) records current names, source evidence,
and naming decisions. This reference uses implemented canonical spellings;
compatibility names are listed below. The collection design record explains shared operation contracts.

## Variable Declaration Keywords

| Keyword  | Meaning                       | English Equivalent | Example                                |
| -------- | ----------------------------- | ------------------ | -------------------------------------- |
| `door`   | Dynamic variable declaration  | `var`/`let`        | `door magac = "Sharafdin"`             |
| `madoor` | Constant variable declaration | `const`            | `madoor PI = 3.14159`                  |
| `abn`    | Integer type                  | `int`               | `abn da = 25`                          |
| `jajab`  | Binary floating-point type            | `float`/`double`   | `jajab qiimo = 3.14`                   |
| `qoraal` | String type                   | `string`           | `qoraal magac = "Sharafdin"`           |
| `bool`   | Boolean type                  | `bool`             | `bool waaRun = run`                    |
| `walax`  | Object type                   | `object`           | `walax person = { name: "Sharafdin" }` |
| `teed`   | List/array type               | `array`            | `teed numbers = [1, 2, 3]`             |
| `baytyo` | Byte array type and constructor (contextual) | `bytes` | `baytyo data = baytyo([0, 255])` |
| `maran`  | Null value                    | `null`             | `door a = maran`                       |

## Control Flow Keywords

| Keyword       | Meaning            | English Equivalent | Example                                      |
| ------------- | ------------------ | ------------------- | --------------------------------------------- |
| `haddii`      | If statement       | `if`                | `haddii (x > 10) { qor("Weyn") }`             |
| `kale` | Else if statement  | `else if`           | `kale (x == 10) { qor("Dhexe") }`      |
| `kale`  | Else/default statement     | `else`              | `kale { qor("Yar") }`                   |
| `dooro`       | Switch statement   | `switch`            | `dooro (x) { xaalad 1 { qor("Hal") } }`       |
| `xaalad`      | Case clause        | `case`              | `xaalad "A" { qor("Case A") }`                |
| `kuceli`      | For loop           | `for`               | `kuceli (i 1 ilaa 5) { qor(i) }`              |
| `ilaa`        | Loop range end     | `to`                | `kuceli (i 1 ilaa 5) { qor(i) }`              |
| `::`          | Loop step/increment amount (optional, follows the range) | `step` | `kuceli (i 1 ilaa 10 :: 2) { qor(i) }` — loops from 1 to 10, incrementing `i` by 2 each time |
| `intay`       | While loop         | `while`             | `intay (x < 5) { qor(x) }`                    |
| `jooji`       | Break statement    | `break`             | `haddii (x == 3) { jooji }`                   |
| `soco`        | Continue statement | `continue`          | `haddii (x == 3) { soco }`                    |

`kale` is contextual. After an `haddii` branch, `kale (condition) { ... }`
adds another conditional branch, while `kale { ... }` is the fallback. Inside
`dooro`, `kale { ... }` supplies the default case. Existing names such as
`hawl kale(x)` and calls such as `kale(1)` remain valid; a conditional branch
header must be followed by `{`.

```soplang
haddii (da < 18) {
    qor("Ilmo")
} kale (da < 60) {
    qor("Qof weyn")
} kale {
    qor("Waayeel")
}
```

## Function Keywords

| Keyword | Meaning              | English Equivalent | Example                            |
| ------- | -------------------- | ------------------- | ----------------------------------- |
| `hawl`  | Function declaration | `function`          | `hawl isuGee(a, b) { celi a + b }` |
| `celi`  | Return statement     | `return`            | `celi x * 2`                       |

Functions have fixed argument counts, checked before invocation. Compiled
functions, methods, constructors, and captured closures support signatures
with more than eight arguments. Implicit instance and capture values do not
reduce the number of user arguments available.

Any expression that produces a function or closure can be called:

```soplang
hawl samee(bilow) { celi hawl(x) { celi bilow + x } }
door hawlo = [samee(2)]
qor(samee(1)(2))
qor(hawlo[0](3))
qor((hawl(x) { celi x * 2 })(4))
```

The callee is evaluated before its arguments, which are evaluated from left
to right. A failed callee or argument stops the call. A chained call suffix
starts on the same line as the expression it invokes; its argument list may
span lines. This keeps a parenthesized statement on the next line independent.

Objects and instances can hold callable fields. `object.callback(args)` calls
such a field when no built-in object method or declared class method has that
name. Indexed calls such as `object["callback"](args)` invoke the stored value
directly. Stored functions receive the explicitly supplied arguments; only
declared class methods receive an implicit instance.

## Special Values

| Somali Value | English Equivalent | Description         | Example                  |
| ------------ | ------------------- | -------------------- | ------------------------- |
| `run`        | `true`              | Boolean true value   | `haddii (run) { ... }`   |
| `been`       | `false`             | Boolean false value  | `haddii (!been) { ... }` |
| `maran`      | `null`              | Empty/null value     | `door val = maran`       |

`maran` is the canonical null literal. `null` remains accepted as a compatibility spelling. An empty string, list, or object is a value of its own type, distinct from `maran`.

## Data Types

| Somali Type | English Equivalent | Description       | Example                          |
| ----------- | -------------------- | ------------------ | ---------------------------------- |
| `abn`       | `int`        | Integer numbers    | `abn age = 25`                    |
| `jajab`     | `float`/`double`     | Binary floating-point numbers    | `jajab pi = 3.14`                  |
| `qoraal`    | `string`              | Text values         | `qoraal name = "Ahmed"`            |
| `bool`      | `boolean`             | Truth values        | `bool isValid = run`               |
| `teed`      | `list`/`array`        | List of items       | `teed numbers = [1, 2, 3]`         |
| `walax`     | `object`              | Key-value pairs     | `walax person = { name: "Ali" }`   |

`abn` uses signed 64-bit integers. Addition, subtraction, multiplication, and
negation wrap in two's-complement arithmetic in both static and dynamic code.
Integer `%` keeps the dividend's sign; the minimum integer modulo `-1` is zero.
Division produces a `jajab`. Division and modulo by zero remain catchable errors.
`jajab` uses 64-bit binary floating-point values; it does not provide exact
decimal arithmetic. `qoraal` is a string,
`teed` is a mutable list, and `walax` stores string keys and values.

### Collection loops

`kuceli (item ku source)` iterates collections and lazy ranges. Multiple names
bind row elements, as in `kuceli (index, item ku tirso(source))`. `ku` is
contextual inside the header. See [iteration.md](iteration.md) for snapshots,
Unicode characters, object entries, `wareeg`/`xiga`, `taxane`, `tirso`, `lamaane`,
and user-defined class iterators.

### String literals

Unprefixed single and double quotes keep their existing literal-backslash
behavior. `r` explicitly selects the same raw behavior. Use `e` to decode escapes:

```soplang
door jid = r"C:\temp\new"
door fariin = e"Salaan\nSadar cusub\t!"
door xarfo = e"\u{e9}\u{1F600}"
door tuduc = """First line with "quotes"
Second line"""
```

Escaped strings accept `\n`, `\r`, `\t`, `\0`, `\\`, `\'`, `\"`, and
`\u{HEX}` with one to six hexadecimal digits naming a valid Unicode scalar.
Surrogates, values above U+10FFFF, incomplete escapes, and unknown escapes
report Somali lexer errors at the backslash. A backslash followed by an actual
LF or CRLF continues an escaped literal without adding a line break.

Triple single or double quotes allow multiline content and ordinary quote
characters inside the string. They work with `e` and `r`, preserve indentation
and line breaks, and close at the matching three-character delimiter. String
prefixes attach directly to the opening quote. `e` and `r` remain ordinary
identifiers elsewhere. Existing path strings require no migration.

### Interpolated strings

Attach `f` directly to single, double, or triple quotes to insert expressions.
These literals decode the same escapes as `e` strings. Use `{{` and `}}` for
literal braces; plain and raw strings do not interpolate.

```soplang
door magac = "Ayaan"
qor(f"Salaan, {magac}!")
qor(f"Wadarta: {2 + 3}; qiimaha: {3.14159:.2f}")
qor(f"{{{magac}}}")
qor(f"{ {magac: magac}.magac }: {[10,20][1]}")
```

Every field contains an ordinary expression and an optional static format
descriptor after `:`. The descriptors are the same ones accepted by `qaabee`.
Put expressions with a top-level colon, such as typed anonymous functions,
inside parentheses. Nested literals may use the same quote character as the
outer string. Nesting is limited to 64 interpolated literals.

Fields evaluate once, from left to right. A failed expression or format stops
the remaining fields. The result is always `qoraal`, including a literal with
only one field. Rebinding `qaabee` does not change interpolation. Runtime errors
retain the original expression location; invalid descriptors point at their
own source span. `f` remains an ordinary identifier when it is not followed
immediately by a quote.

### Value and template formatting

`qaabee(value, format?)` returns a string. Omitting the descriptor uses the
same cycle-safe conversion as `qoraal(value)`.

```soplang
qor(qaabee(3.14159, ".2f"))
qor(qaabee(42, "#08x"))
qor("{magac}: {da:03d}".qaabee({ magac: "Ayaan", da: 7 }))
qor("{} + {}".qaabee(2, 3))
```

Descriptors follow `[fill][alignment][sign][#][0][width][.precision][type]`.
Alignment is `<`, `>`, or `^`; an optional preceding character supplies the
fill. Width and string precision count Unicode characters. Numbers default to
right alignment and other values to left alignment. Signs `+`, `-`, and space
apply to numbers. Zero padding without explicit alignment follows the sign and
any base prefix.

| Type | Meaning |
| --- | --- |
| `s` | String, optionally truncated to precision |
| `d` | Signed decimal integer |
| `x`, `X`, `b`, `o` | Hexadecimal, binary, or octal integer; `#` includes a base prefix |
| `f` | Fixed decimal; default six fractional digits |
| `e`, `E` | Scientific notation; default six fractional digits |
| `%` | Multiply a number by 100 and append `%` |

Without a type, string precision truncates characters and float precision
selects fractional digits. Integer precision is rejected. Width and precision
are each limited to 1,000,000 to bound a single formatting operation.

String templates use `{}` for automatic positional slots, `{0}` for explicit
indexes, and `{name}` for keys in the first object argument. A colon introduces
a descriptor, as in `{0:.2f}`. Automatic and numbered slots cannot be mixed.
`{{` and `}}` produce literal braces. Missing values and invalid descriptors
report catchable errors with code `E036`. Template descriptors are static;
nested replacement fields are not supported.

`door`, `madoor`, and `abn` are Soplang short identifiers. The terminology
review relates them to *doorsoome*, *madoorsoome*, and *abyoone*, respectively;
these explanations do not establish a historical abbreviation or SLS approval.

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
| `qor`    | Print zero or more values separated by spaces | `print`              | `qor("Salaan", "Adduunka!", 3)`          |
| `gelin`  | Read input from user                         | `input`              | `door magac = gelin("Magacaaga geli: ")`  |
| `akhrifayl` | Read an entire UTF-8 text file          | (read file)          | `door text = akhrifayl("magac.txt")`      |
| `qor_fayl` | Create or overwrite a text file           | (write file)         | `qor_fayl("magac.txt", "Salaan")`          |
| `kudar_fayl` | Append text to a file                   | (append file)        | `kudar_fayl("magac.txt", "\n")`            |
| `akhrif_baytyo` | Read an entire file as `baytyo`        | (read bytes)         | `door data = akhrif_baytyo("raw.bin")`     |
| `qor_baytyo` | Overwrite a file with `baytyo`           | (write bytes)        | `qor_baytyo("raw.bin", data)`              |
| `kudar_baytyo` | Append `baytyo` to a file              | (append bytes)       | `kudar_baytyo("raw.bin", chunk)`           |
| `fayl_jiraa` | Test whether a path exists              | (path exists)        | `haddii (fayl_jiraa("magac.txt")) {...}`   |
| `fayl_xog` | Query path metadata                     | (path stat)          | `door x = fayl_xog("magac.txt")`           |
| `isku_xidh` | Join path segments                     | (path join)          | `isku_xidh("data", "magac.txt")`           |
| `baabi_jid` | Resolve an absolute path                | (absolute path)      | `baabi_jid("magac.txt")`                   |
| `nuqul_fayl` | Copy a file                            | (copy file)          | `nuqul_fayl("a.txt", "b.txt")`             |
| `wareeji_fayl` | Move or rename a path                | (move file)          | `wareeji_fayl("a.txt", "b.txt")`           |
| `tirtir_fayl` | Delete a file                         | (delete file)        | `tirtir_fayl("magac.txt")`                 |
| `abuur_gal` | Create a directory tree                 | (mkdir)              | `abuur_gal("data/pkg")`                    |
| `tir_gal` | List directory entry names                | (list dir)           | `door names = tir_gal("data")`             |
| `tirtir_gal` | Remove a directory                     | (rmdir)              | `tirtir_gal("data", run)`                  |
| `nooc`   | Get type of variable                         | `typeof`             | `qor(nooc(magac))`                        |
| `abn`    | Convert value to an integer (also the `int` type keyword above) | `int` | `door n = abn("5")`                 |
| `jajab`  | Convert a number or numeric string to floating point | `float` | `door n = jajab("3.5")`             |
| `qoraal` | Convert to string                            | `str`                | `door s = qoraal(25)`                     |
| `bool`   | Convert to boolean                           | `bool`               | `door b = bool(1)`                        |
| `teed`   | Create a list                                | `list/array`         | `door list = teed(1, 2, 3)`               |
| `walax`  | Create an object                             | `object/dict`        | `door obj = walax("name", "Ali", "age", 25)` |
| `daji`   | Round down to integer                        | `Math.floor()`       | `door n = daji(4.7)`                      |
| `kor`    | Round up to integer                          | `Math.ceil()`        | `door n = kor(4.2)`                       |
| `dherer` | Count list items, object keys, or string characters | `len()`/`.length` | `door n = dherer(qoraal)`             |
| `xul`    | Get random value                             | `random()`           | `door n = xul(1, 6)`                      |
| `abuur`  | Seed the context random stream               | (seed PRNG)          | `abuur(42)`                               |
| `loog`   | Natural or base logarithm                    | `log()`              | `door n = loog(100, 10)`                  |
| `sin`    | Sine or inverse sine                         | `sin()` / `asin()`   | `door n = sin(1, "rogan")`                |
| `cos`    | Cosine or inverse cosine                     | `cos()` / `acos()`   | `door n = cos(0)`                         |
| `tan`    | Tangent or inverse tangent                   | `tan()` / `atan()`   | `door n = tan(1)`                         |
| `xagal`  | Angle from `y` and `x`                       | `atan2()`            | `door n = xagal(1, -1)`                   |
| `joogto` | Named math constant                          | (constants)          | `door pi = joogto("pi")`                  |
| `xadid`  | Clamp a value between bounds                 | `clamp()`            | `door n = xadid(5, 0, 10)`                |
| `baaxad` | Create an integer range                      | `range()`            | `door n = baaxad(1, 5)`                   |
| `duub` | Round to the nearest integer            | `round()`            | `door n = duub(4.6)`               |
| `sugan`  | Get the absolute value                       | `abs()`              | `door n = sugan(-5)`                      |
| `yar` | Get the smallest numeric argument           | `min()`              | `door n = yar(4, 2, 8)`               |
| `weyn` | Get the largest numeric argument           | `max()`              | `door n = weyn(4, 2, 8)`              |
| `jibbaar`  | Raise a number to a power                    | `pow()`              | `door n = jibbaar(2, 3)`                    |
| `xidid`  | Calculate a square root                      | `sqrt()`             | `door n = xidid(9)`                       |

`qor()` accepts zero or more positional arguments, converts each value using
the normal Soplang string representation, separates them with one space, and
adds a newline. With no arguments it prints an empty line. It returns the
printed text without the final newline, including an empty string for `qor()`.
Arguments are evaluated from left to right before output; an argument error
prevents that call from printing. Functions called while evaluating arguments
may produce their own output. Existing one-argument calls keep their behavior.

```soplang
qor("Ayaan", "Cali")
qor("Da:", 25, "Firfircoon:", run)
qor()
```

`daji()` rounds toward negative infinity and `kor()` toward positive infinity.
`duub()` rounds to the nearest integer, with halfway values rounded away
from zero. `xidid()` computes only a square root, not an arbitrary root.

`abuur(seed)` resets the context-local random stream used by `xul`. Each runtime
context keeps its own stream; reusing the same seed reproduces the same draws.

`xul()` returns a floating-point number in `[0, 1)`. `xul(a, b)` returns a
floating-point number in `[a, b)` and excludes the upper endpoint. `xul(teed)`
returns one item and reports an error for an empty list. Failed calls do not
advance the stream.

`loog(x)` and `loog(x, base)` require positive finite inputs with a base other
than `1`. Trigonometric builtins use radians; pass `"rogan"` as the second
argument for inverse sine, cosine, or tangent. `joogto("pi")`, `joogto("e")`,
`joogto("tau")`, `joogto("inf")`, and `joogto("nan")` return named constants.
`xadid(value, low, high)` clamps using the same numeric ordering as comparisons.
Domain and type failures for these builtins use error code `E044`.

> **Note:** `abn`, `qoraal`, `bool`, `teed`, and `walax` are used both as static **type keywords** (see Variable Declaration Keywords / Data Types above) and as **built-in conversion/constructor functions** here. The meaning depends on context: e.g. `abn age = 25` declares a typed variable, while `abn("5")` converts a string to a number.

## baaxad (Range)

`baaxad()` waa hawl ku dhex jirta Soplang oo soo saarta teed abn ah.

Unlike a lazy range iterator, it creates a list containing all the requested
integers. The stop bound is excluded.

**Isticmaalka:**

```soplang
// baaxad(stop): 0 ilaa stop-1
qor(baaxad(5))  // [0, 1, 2, 3, 4]

// baaxad(start, stop): start ilaa stop-1
qor(baaxad(2, 6))  // [2, 3, 4, 5]

// baaxad(start, stop, step): dhammaadka kuma jiro; tallaabadu waa step
qor(baaxad(1, 10, 2))  // [1, 3, 5, 7, 9]
```

`baaxad` accepts integers only. A zero step reports an error that can be caught with `tijaabi` and `qabo`.

## Shared collection operations

Lists and objects share `dherer`, `leeyahay`, `hel`, `dhig`, `saar`, `nadiifi`,
and `nuqul` where applicable. Strings share `dherer`, `leeyahay`, and `hel`;
strings remain immutable.

- `hel(position, default?)` reads a list item or Unicode character;
  `hel(key, default?)` reads an object value. Missing entries return the supplied
  default or `maran`. A present `maran` value is returned unchanged. Defaults
  are evaluated normally even when an entry exists.
- List and string positions must be integers and may be negative. Object keys
  must be strings. Invalid argument types report catchable errors.
- `dhig(index, value)` replaces an existing list item; `dhig(key, value)` inserts
  or replaces an object entry. Both mutate and return the receiver. An invalid
  list position reports an error before mutation; use `geli` for insertion.
- List `saar()` removes the last item; `saar(index)` removes an indexed item.
  Both return the removed value and reject empty or out-of-range removals.
  Object `saar(key)` returns the removed value or `maran` for a missing key.
- `nadiifi()` mutates and returns the receiver. `nuqul()` returns a shallow copy.

```soplang
door tirooyin = [10, 20, 30]
door qof = { magac: "Ayaan", da: 25 }
qor(tirooyin.hel(-1))
qor(qof.dherer())
tirooyin.dhig(0, 15)
qof.dhig("da", 26)
qor(tirooyin.saar())
qor(qof.saar("da"))
```

### Slicing, endpoints, counting, and search

Lists and strings accept `jar()`, `jar(start)`, or `jar(start, end)`. Bounds
must be integers. The default start is zero and the default end is the length;
negative bounds count from the end. Bounds are clamped to the collection. The
start is included and the end excluded. Equal or reversed normalized bounds
produce an empty result. Slicing leaves the receiver unchanged; list slices are
shallow copies. String positions count Unicode characters.

`hore()`/`dambe()` return the first/last item or character, or `maran` when
empty. List `tiri(value)` counts matching values using the same equality rules
as list membership and search, including structural collection equality and
distinct integer/float values. String `tiri(sub)` counts non-overlapping matches
and rejects an empty substring.

Search has one compatibility exception: missing list `raadi`/`muuji` results
are `maran`, while string `raadi`/`raadi_dambe` results are `-1`. A successful
search returns an integer position, including zero. Test misses explicitly
rather than relying on the truthiness of a search result.

## List Methods

| Method            | English Equivalent        | Description                     | Example                                 |
| ------------------ | --------------------------- | ---------------------------------- | ------------------------------------------ |
| `dherer()`         | `length()`                  | Get list length                    | `numbers.dherer()`                        |
| `hel(index, default?)` | `get()` | Read by index or return a default | `numbers.hel(-1, maran)` |
| `dhig(index, value)` | indexed assignment | Replace an existing item and return the list | `numbers.dhig(0, 5)` |
| `geli(item)`      | `push()` or `append()`      | Add item to the original list      | `numbers.geli(5)`                        |
| `saar()`           | `pop()`                      | Remove and return last item        | `door last = numbers.saar()`            |
| `midee(teed)`      | `concat()`                   | Concatenate lists                  | `door all = list1.midee(list2)`           |
| `leeyahay(x)`      | `contains()`/`includes()`   | Check if item exists               | `haddii (list.leeyahay(x)) {...}`         |
| `nuqul()`          | `copy()`                     | Create a shallow copy              | `door copy = list.nuqul()`                |
| `nadiifi()`        | `clear()`                    | Remove all items from the original list | `list.nadiifi()`                     |
| `rog()`            | `reverse()`                  | Reverse the original list          | `list.rog()`                              |
| `habee()`          | `sort()`                     | Sort the original list in ascending order | `list.habee()`                    |
| `jar(start?, end?)`        | `slice(a, b)`                | Return a new list; start included, end excluded         | `door subset = numbers.jar(1, 3)`         |
| `aaddin(func)`     | `map(func)`                   | Transform items with function      | `door doubled = nums.aaddin(hawl(x) { celi x * 2 })` |
| `shaandhee(func)`  | `filter(func)`                | Filter items with function         | `door evens = nums.shaandhee(hawl(x) { celi x % 2 == 0 })` |
| `raadi(item)`      | `indexOf(item)`               | Find index of item                 | `door idx = nums.raadi(5)`                |
| `tiri(value)` | `count()` | Count equal list items | `numbers.tiri(5)` |
| `geli(index, item)` | `insert(index, item)`         | Insert into the original list      | `numbers.geli(1, 5)`                      |
| `saar(index)`   | `remove(index)`                | Remove and return an indexed item  | `door item = numbers.saar(1)`          |
| `hore()`        | `first()`                      | Return first item or `maran`       | `door item = numbers.hore()`           |
| `dambe()`   | `last()`                       | Return last item or `maran`        | `door item = numbers.dambe()`      |
| `ururi(func, bilow)` | `reduce(func, initial)`  | Fold with an arbitrary callback, not only addition          | `nums.ururi(hawl(a, b) { celi a + b }, 0)` |
| `dhammaan(func)`   | `every(func)`                  | Test whether every item matches    | `nums.dhammaan(hawl(x) { celi x > 0 })`   |
| `qaar(func)`       | `some(func)`                   | Test whether any item matches      | `nums.qaar(hawl(x) { celi x == 5 })`      |

`midee(teed)` returns a concatenated copy without mutating either input. `geli(item)` always appends one item, including a nested list. `nuqul()` and `jar()` also return new lists. `saar()` and `saar(index)` return the removed value. Methods that otherwise mutate a list return the original list after changing it. Nested lists and objects remain shared when copied.

`hore()` and `dambe()` return the first and last list items without modifying
the list. Strings use the same names for Unicode characters. Both return
`maran` for an empty receiver. Objects have no built-in
positional first/last methods.

`raadi(item)` returns the first matching index or `maran` when no item matches.
`ururi(func, bilow)` is a general left fold: its callback receives the
accumulated value and the next item. It can multiply values, construct text, or
perform other accumulation; its behavior is not restricted to summation.

Map, filter, fold, every, and some validate callbacks even on empty lists.
Each iterates a shallow snapshot, allowing callbacks to modify the original
list without changing the remaining iteration sequence. Callback errors stop
the operation and propagate to `qabo`. `dhammaan` and `qaar` stop at the first
decisive result; on an empty list they return `run` and `been`, respectively.

## Object Methods

| Method        | English Equivalent   | Description             | Example                               |
| -------------- | ---------------------- | -------------------------- | ---------------------------------------- |
| `dherer()` | `length()` | Count keys | `obj.dherer()` |
| `furayaal()`      | `keys()`               | Get all keys              | `door keys = obj.furayaal()`               |
| `qiimayaal()`     | `values()`             | Get all values             | `door values = obj.qiimayaal()`            |
| `lammaanayaal()`   | `entries()`            | Get key-value pairs        | `door pairs = obj.lammaanayaal()`           |
| `leeyahay(x)` | `hasOwnProperty()`     | Check if key exists        | `haddii (obj.leeyahay("name")) {...}`  |
| `midee(obj)`  | `merge()`/`assign()`   | Return a merged copy       | `door merged = obj1.midee(obj2)`       |
| `nuqul()`     | `copy()`               | Create a shallow copy      | `door copy = obj.nuqul()`              |
| `nadiifi()`   | `clear()`              | Remove all original properties | `obj.nadiifi()`                    |
| `hel(key, default?)` | `get()`            | Get a key or optional default  | `obj.hel("name", "Unknown")`     |
| `dhig(key, value)` | `set()`              | Set a computed key             | `obj.dhig("name", "Ali")`        |
| `saar(key)` | `remove()`              | Remove and return a value      | `door old = obj.saar("name")`  |

Objects also support computed bracket access: `obj["name"]` and `obj["name"] = "Ali"`. Object `nuqul()` is shallow: nested lists and objects remain shared. `tir(key)` returns the object, whereas `saar(key)` returns the removed value or `maran`.

Object key, value, and pair lists follow sorted-key order. `midee` returns a
new shallow object; the right input wins overlapping keys. Neither input is
mutated.

## String Methods

| Method             | English Equivalent | Description                             | Example                                  |
| -------------------- | --------------------- | ------------------------------------------ | ------------------------------------------- |
| `hore()` | first character | First Unicode character or `maran` | `text.hore()` |
| `dambe()` | last character | Last Unicode character or `maran` | `text.dambe()` |
| `hel(index, default?)` | character lookup | Read a Unicode character or return a default | `text.hel(-1, "")` |
| `qeybi(xad)`        | `split()`             | Split string by delimiter                  | `door parts = text.qeybi(",")`             |
| `leeyahay(sub)`     | `includes()`          | Check if string contains substring         | `haddii (text.leeyahay("search")) {...}`   |
| `dhammaad(sub)`      | `endsWith()`          | Check if string ends with substring        | `haddii (text.dhammaad("ing")) {...}`       |
| `bilow(sub)`        | `startsWith()`        | Check if string starts with substring      | `haddii (text.bilow("http")) {...}`        |
| `beddel(x, y, count?)`      | `replace()`           | Replace first match, a limited count, or all with `maran` | `door new = text.beddel("old", "new")`     |
| `xiriiri(teed)`       | `join()`               | Join string representations with separator        | `door text = ", ".xiriiri(names)`            |
| `jar(start?, end?)`   | `slice()`              | Slice by character position                 | `door sub = text.jar(0, 3)`                |
| `weynee()` | `toUpperCase()`        | Convert string to uppercase                | `door n = q.weynee()`             |
| `yaree()`  | `toLowerCase()`        | Convert string to lowercase                | `door n = q.yaree()`              |
| `masax(direction?)`           | `trim()`               | Trim both ends, or `"hore"`/`"dambe"` | `door n = q.masax()`                       |
| `raadi(sub, direction?)`        | `indexOf()`            | Find first/last character index | `door n = q.raadi("pro")`                  |
| `dherer()`          | `length`              | Count Unicode characters                    | `door n = q.dherer()`                       |
| `tiri(sub)`         | `count()`             | Count non-overlapping occurrences           | `door n = q.tiri("ab")`                    |
| `kuceli(n)`        | `repeat()`            | Repeat the string                           | `door n = q.kuceli(3)`                     |

String positions and bracket indexes count characters rather than UTF-8 bytes. `jar()` and bracket indexing accept negative positions from the end. Missing indexes return `maran`; `raadi()` returns `-1` when it finds no match. Calls with the wrong number of arguments report an error that can be caught with `tijaabi` and `qabo`.

### weynee (Uppercase)
**Isticmaal:**

```soplang
door q = "soplang"
door natiijo = q.weynee()
qor(natiijo)  # SOPLANG
```

### yaree (Lowercase)
**Isticmaal:**

```soplang
door q = "SOPLANG"
door natiijo = q.yaree()
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

### beddel (Replace All)
**Isticmaal:**

```soplang
door q = "abaaba"
door natiijo = q.beddel("a", "x", maran)
qor(natiijo)  # xbxxbx
```

`masax()` trims both ends; `masax("hore")` trims only leading whitespace and
`masax("dambe")` only trailing whitespace. `raadi(sub)` and `raadi(sub, "hore")`
search from the beginning; `raadi(sub, "dambe")` returns the last match.
Unsupported direction values report catchable errors.

`beddel(old, new)` replaces the first occurrence. A third nonnegative integer
limits replacements; zero leaves the text unchanged. Passing `maran` as the
third argument replaces all occurrences. Empty search text matches character
boundaries. `xiriiri(list)` joins the string representations of values, including
non-string values. `weynee` and `yaree` use Unicode case conversion. `kuceli`
requires a nonnegative integer count and is accepted after a dot without changing
the numeric loop syntax.

## Object-Oriented Programming (OOP)

| Keyword / concept | Meaning | English | Example |
| -------------------- | ---------- | ---------- | ---------- |
| `qaab`   | Class definition                            | `class`          | `qaab Bisad { ... }`                        |
| `dhaxal` | Inherit from parent class                   | `extends`        | `qaab Bisad dhaxal Xayawaan { ... }`        |
| `cusub`  | Create a new instance                       | `new`             | `door b = cusub Bisad("Mia")`               |
| `nafta`  | Current instance (self)                     | `this` / `self`  | `nafta.magac = magac`                       |
| `waalid` | Call a method on the parent class           | `super`          | `celi waalid.cod()`                         |
| `dhis`   | Constructor method (auto-called by `cusub`) | `constructor`    | `hawl dhis(magac) { nafta.magac = magac }`  |

**Class example:**

```soplang
qaab Xayawaan {
    hawl cod() {
        celi "xayawaan"
    }
}

qaab Bisad dhaxal Xayawaan {
    hawl dhis(magac) {
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
| `tijaabi` | Start a protected try block      | `try`    | `tijaabi { ... } qabo (e) { ... }`          |
| `qabo`  | Catch block with error variable  | `catch`  | `qabo (khalad) { qor(khalad) }`           |
| `tuur` | Raise an error message | `raise` / `throw` | `tuur "Faylka lama helin"` |
| `dhammee` | Run guaranteed cleanup | `finally` | `tijaabi { ... } dhammee { ... }` |

**Import example:**

```soplang
keen "_math_lib.sop"
qor(laba() + afar())
```

Imported files are resolved relative to the current file's directory. Top-level definitions (functions, variables, classes) from the imported file become available in the importing program.

**Try/catch example:**

```soplang
tijaabi {
    qor(1 / 0)
} qabo (khalad) {
    qor("Qabtay: " + khalad)
}
```

Runtime errors inside the `tijaabi` block are caught; the error message is bound to the variable in `qabo (...)`.

**Raising errors and guaranteed cleanup:**

```soplang
tijaabi {
    tuur "Xogtu ma dhammaystirna"
} qabo (fariin) {
    qor(fariin)
} dhammee {
    qor("Hawshii waa dhammaatay")
}
```

`tuur` raises a string message or a structured `khalad` value. Other values
report a catchable type error. The value starts on the same line as `tuur`.
Calls such as `tuur(value)` remain ordinary function calls for compatibility;
to raise a computed message, use `tuur make_message()` or `tuur message`.

`dhammee` runs once when its protected block finishes, including after a
handler and on errors, `celi`, `jooji`, and `soco`. It can follow either a
`tijaabi` body alone or a `tijaabi`/`qabo` pair. A try block must have a handler,
cleanup, or both. Nested cleanup runs from the innermost block outward.

The return value is evaluated before cleanup. If cleanup completes normally,
the original return or error continues. A return or error from cleanup takes
precedence. Breaking out of a loop inside a protected body does not run that
body's cleanup until execution actually leaves the protected block.

`tuur` and `dhammee` are contextual forms. Existing variables, functions, and
properties with those names remain supported outside these statement contexts.

### Structured errors

`khalad(fariin, summad?, sabab?)` constructs an immutable error without raising
it. The message must be a string. The code is a string or `maran`; the cause
is another error or `maran`. The two optional arguments default to `maran`.

```soplang
tijaabi {
    tuur khalad("Xogtu ma dhammaystirna", "XOG001")
} qabo (fariin, xog) {
    qor(fariin, xog.summad)
    tuur khalad("Hawshu ma dhammaan", "HAWL001", xog)
}
```

The first catch binding remains message text. An optional second binding
receives the structured error. The two binding names must differ. Rethrowing
that error preserves its original location, trace, and cause. Raising a string
creates a new error at the `tuur` statement. If cleanup fails while another
error is pending, the new error retains that earlier error as its cause unless
an explicit cause is already present.

| Property | Value |
| --- | --- |
| `fariin` | Message string |
| `summad` | Error-code string, or `maran` |
| `sabab` | Original structured error, or `maran` |
| `fayl` | Source filename, or `maran` |
| `saf`, `tiir` | One-based source line and column, or `maran` |
| `raad` | Compiled function trace, innermost call first |

Trace entries are objects with `magac`, `fayl`, `saf`, and `tiir`. Their
positions identify call sites; the error's own position identifies the failing
operation. Trace entries are detached snapshots. Property assignment on an
error is rejected. String-key indexing supports the same properties.
`nooc(xog)` returns `"khalad"`, and `qoraal(xog)` returns its message.

Uncaught diagnostics show the original source caret, `Raadka wicitaannada`,
and any `Sababta` chain. Cause rendering is bounded to avoid unbounded output.

## Compatibility spellings

| Canonical API | Supported earlier spelling |
| --- | --- |
| Constructor `dhis()` | `dhaw()` |
| Try block `tijaabi { ... } qabo (e) { ... }` | `fasax { ... } qabo (e) { ... }` |
| `jibbaar(a, b)` | `awood(a, b)` |
| List `raadi(item)` | `muuji(item)` |
| String `dhammaad(suffix)` | `dhamaad(suffix)` |
| List `saar()` | `kasaar()` |
| List `saar(index)` | `ka_saar(index)` |
| Object `saar(key)` | `ka_qaad(key)` |
| Object removal returning the receiver | `tir(key)`; different return contract from `saar` |
| List `hore()` | `koowaad()` |
| List `dambe()` | `ugu_dambeeya()` |
| `yar(...)` | `ugu_yar(...)` |
| `weyn(...)` | `ugu_weyn(...)` |
| List `ururi(func, initial)` | `isku_gee(func, initial)` |
| List/object `midee(other)` | `kudar(other)`; list argument concatenates without mutation |
| String `xiriiri(list)` | `kudar(list)` |
| String `weynee()` | `xarfaha_waaweyn()`, `xarafaha_weyn()` |
| String `yaree()` | `xarfaha_yaryar()` |
| String `masax("hore")` | `masax_bilow()` |
| String `masax("dambe")` | `masax_dhammaad()`, `masax_dhamaad()` |
| String `raadi(sub, "dambe")` | `raadi_dambe(sub)` |
| String `beddel(old, new, maran)` | `beddel_dhammaan(old, new)` |
| String `kuceli(count)` | `ku_celi(count)` |
| `duub(value)` | `dhammeystir(value)` |
| Object `furayaal()`, `qiimayaal()`, `lammaanayaal()` | `fure()`, `qiime()`, `lamaane()` |
| `kale (condition) { ... }` | `haddii_kale (condition) { ... }` |
| `kale { ... }` | `ugudambeyn { ... }` in conditionals and switches |

`tijaabi` is contextual: it starts a try block only before `{`. Existing
variables, functions, and properties named `tijaabi` remain valid. `qabo` is
unchanged.

Automatic construction searches the nearest class first, preferring its `dhis`
over its `dhaw`, then walks to its parent. Explicit constructor calls, including
`waalid.dhis()` and `waalid.dhaw()`, prefer the requested spelling in each class
and fall back to the other spelling before moving to the next ancestor. If a
class defines both, both methods remain explicitly callable.

Legacy object `tir(key)` remains supported and returns the receiver; it is not
a direct alias of `saar(key)`, which returns the removed value. Existing `kudar`
overloads also preserve their mutation and copying behavior.

List `kudar(value)` retains its original type-dependent behavior: scalar values
append in place, while a list argument produces a concatenated copy. Prefer
`geli(value)` for adding exactly one item, including a nested list, and `midee`
for combining collections.

## Global updates

Functions, methods, and callbacks share the current value of a global variable
with top-level code. Global variables take precedence over built-in names,
matching local shadowing. Replacing a scalar, list, object, or callable is visible on
the next read. Function parameters and local declarations still shadow globals;
closure captures of enclosing local variables retain their existing snapshot
semantics.

Top-level numeric loop indexes are published to globals and can be read or
updated by functions during the loop. The increment uses the current index,
including any update from the body. The final index remains available after
the loop; `jooji` preserves the index at the break.

Typed global writes inside functions enforce the same type and class checks as
top-level writes. Rejected assignments preserve the previous global value.
An accepted integral float is converted to `abn` for an integer annotation, and
an accepted integer is converted to `jajab` for a floating-point annotation,
keeping the representation consistent with native arithmetic. Fractional floats
are rejected for `abn` annotations; use `abn(value)` for explicit truncation.

## Byte arrays

The [byte-array reference](bytes.md) defines construction, strict UTF-8 conversion,
indexing, checked writes, collection methods, type annotations, and iteration.

## Optional and union annotations

`T?` permits a value of type `T` or `maran`. `A | B` permits either type.
The [optional and union reference](optional-union-types.md) defines argument
requirements, conversions, class alternatives, and return validation.

## Typed collections

`teed[T]` constrains list elements; `walax[qoraal, T]` constrains string-keyed
object values. See the [shared collection contracts](typed-collections.md) for
aliases, nesting, numeric kinds, mutation, rest collectors, and lifetime.

## Tuples

`xirmo(...)` constructs an immutable sequence. `xirmo[T1, T2, ...]` annotates
its positions. The [tuple reference](tuples.md) defines indexing, slicing,
iteration, nested mutation contracts, conversions, and lifetime.

## Sets

`urur(...)` constructs a mutable set of unique immutable keys.
The [set reference](sets.md) defines numeric key equality, insertion order,
conversion, snapshot iteration, and set algebra.

`urur[T]` constrains set elements. See the [typed set reference](typed-sets.md)
for immutable key types, shared contracts, nested aliases, and diagnostics.

## Unicode text methods

`hubi`, optional `weynee` modes, `buuxi`, `kalaqaad`, `sadarro`, and
zero-argument `qeybi()` extend the string library. The [text reference](text-methods.md)
defines classification, word boundaries, scalar widths, Unicode line endings,
and compatibility with earlier spellings.

## Expanded list methods

`saar(value, "qiime")`, `ballaadhi`, `keli`, `wadar`, `tirso`, `lamaane`, and
optional `habee` key/reverse arguments extend the list library. The
[list reference](list-methods.md) defines snapshots, atomic publication,
structural equality, consistent sorting, and callback side effects.

## Expanded object methods

Object `geli`, `ballaadhi`, `aaddin`, `shaandhee`, and explicit paths in `hel`
reuse collection vocabulary. The [object reference](object-methods.md) defines
insert-if-absent, atomic merges, sorted snapshot callbacks, renamed entries,
and path semantics without changing literal string keys.
