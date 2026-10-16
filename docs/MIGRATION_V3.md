# Migrating to Soplang 3

Soplang 3 establishes the Rust compiler, the Cranelift JIT, and the LLVM AOT backend as the supported implementation. This release also makes runtime behavior stricter and establishes the public vocabulary and compatible naming improvements.

## Vocabulary

Update programs that still use names from earlier Soplang 2 releases or documentation:

| Earlier spelling | Soplang 3 |
| --- | --- |
| `howl` | `hawl` |
| `soo_celi` | `celi` |
| `bandhig` | `qor` |
| `akhri` | `gelin` |
| `ku_celi` | `kuceli` |
| `inta_ay` | `intay` |
| `sii_wad` | `soco` |
| `tiro` | `abn` |
| `liis` | `teed` |
| `shey` | `walax` |
| `labadaran` | `bool` |
| `waxba` | `maran` |
| `kudhow` | `xul` |
| `raadso` | `raadi` |
| `null` | `maran` |

`null` remains accepted as a compatibility spelling. New code should use `maran`.

## Calls and constructors

Functions, closures, methods, built-ins, and constructors now enforce their argument counts. Calls that previously ignored extra arguments or filled missing arguments with `maran` report a catchable error.

```soplang
hawl laban(x) { celi x * 2 }

tijaabi {
    laban()
} qabo (qalad) {
    qor(qalad)
}
```

Constructor arguments are also checked against the inherited or local `dhis()` or legacy `dhaw()` definition.

## Numeric conversion and ranges

`abn()` and `jajab()` accept trimmed numeric strings:

```soplang
door count = abn(" 42 ")
door price = jajab("3.5")
```

`abn()` rejects decimal strings such as `"3.5"`. `baaxad()` accepts integer arguments only and reports an error when its step is zero.

## Indexing

String indexes count Unicode characters. Lists and strings support negative indexes from the end. An out-of-range read returns `maran`; an out-of-range list write reports a catchable error.

Objects now accept computed string keys:

```soplang
door qof = {}
qof["magac"] = "Ayaan"
qor(qof["magac"])
```

## Logical operators

`&&` and `||` now short circuit. The right expression runs only when its result is needed.

## Semicolons

Semicolons are optional and supported consistently. Existing newline-oriented source remains valid.

```soplang
door x = 1;
qor(x);
```

## New standard library APIs

- Numbers: `duub`, `sugan`, `yar`, `weyn`, `jibbaar`, `xidid`
- Lists: `geli`, `midee`, `saar`, `hore`, `dambe`, `ururi`, `dhammaan`, `qaar`
- Objects: `hel`, `dhig`, `saar`, `midee`, `furayaal`, `qiimayaal`, `lammaanayaal`, plus bracket access
- Strings: `dherer`, `masax`, `tiri`, `kuceli`, `raadi`, `xiriiri`, `weynee`, `yaree`, `beddel`

See the [language reference](language/keywords.md) for signatures and examples.

## Shared collection methods and canonical names

New code may use the following names while earlier spellings remain supported:

| Earlier API | Canonical API |
| --- | --- |
| Constructor `dhaw()` | `dhis()` |
| Try block `fasax` | `tijaabi` |
| Exponentiation `awood(a, b)` | `jibbaar(a, b)` |
| List `muuji(item)` | `raadi(item)` |
| String `dhamaad(sub)` | `dhammaad(sub)` |
| String `masax_dhamaad()` | `masax_dhammaad()` |
| List `kasaar()` | `saar()` |
| List `ka_saar(index)` | `saar(index)` |
| Object `ka_qaad(key)` | `saar(key)` |

Lists and strings now support `hel(index, default?)`. Lists also support
`dhig(index, value)`, and objects support `dherer()` in addition to the global
length function. Existing `leeyahay`, `nuqul`, and `nadiifi` methods keep their
behavior. See the [shared operation contracts](language/keywords.md#shared-collection-operations).

A missing read returns the supplied default or `maran`; an entry storing
`maran` is still present and does not select the default. List removal rejects
invalid positions and popping an empty list; object removal of a missing key
returns `maran`. Legacy `tir(key)` still returns the object.

The `tijaabi` spelling does not become globally reserved. Existing functions,
variables, or properties with that name continue to parse.

Constructor lookup walks classes from child to parent, trying `dhis` before
`dhaw` within each class. A child's legacy constructor therefore still takes
precedence over an ancestor's canonical constructor. Explicit method calls
prefer the requested spelling within each class and then try its counterpart,
including calls through `waalid`.

No old spelling is scheduled for removal. `kudar` retains its earlier mutation
and copying behavior. Prefer the explicit operations described below.

## Short paired names and contextual branches

| Earlier spelling | Preferred spelling |
| --- | --- |
| List `koowaad()` | `hore()` |
| List `ugu_dambeeya()` | `dambe()` |
| `ugu_yar(...)` | `yar(...)` |
| `ugu_weyn(...)` | `weyn(...)` |
| `haddii_kale (condition) { ... }` | `kale (condition) { ... }` |
| `ugudambeyn { ... }` | `kale { ... }` |

First/last list methods return `maran` for empty lists. Numeric selection keeps
its existing argument validation and result types. Every earlier spelling in
this table remains supported.

`kale (condition) { ... }` adds a conditional branch after `haddii`, while
`kale { ... }` supplies a fallback. Switches also accept `kale { ... }` as the
default case. Canonical and compatibility branches may be mixed. `kale` remains
an identifier elsewhere, including function declarations, arguments, variables,
and properties. An ordinary `kale(...)` call after an if statement is preserved
when it is not followed by a branch block.

## Insertion, combination, and text operations

- `list.geli(value)` appends one item, even when the value is a list.
  `list.geli(index, value)` keeps indexed insertion. Both mutate the receiver.
- `list.midee(other_list)` and `object.midee(other_object)` return new shallow
  collections, leaving both inputs unchanged. Object keys on the right win.
- `separator.xiriiri(list)` joins string representations. Legacy `kudar`
  remains supported on all three receivers with its earlier contracts.

| Earlier spelling | Preferred call |
| --- | --- |
| `isku_gee(func, initial)` | `ururi(func, initial)` |
| `xarfaha_waaweyn()`, `xarafaha_weyn()` | `weynee()` |
| `xarfaha_yaryar()` | `yaree()` |
| `masax_bilow()` | `masax("hore")` |
| `masax_dhamaad()`, `masax_dhammaad()` | `masax("dambe")` |
| `raadi_dambe(sub)` | `raadi(sub, "dambe")` |
| `beddel_dhammaan(old, new)` | `beddel(old, new, maran)` |
| String `ku_celi(count)` | `kuceli(count)` |
| `dhammeystir(value)` | `duub(value)` |
| `fure()`, `qiime()`, `lamaane()` | `furayaal()`, `qiimayaal()`, `lammaanayaal()` |

`masax()` still trims both ends; `raadi(sub)` still searches first. Direction
arguments accept only `"hore"` and `"dambe"`. `beddel(old, new)` still replaces
one match. Its optional third argument is a nonnegative integer limit or
`maran` for all matches. Replacement and repetition reject fractional and
negative counts. Numeric loops still use `kuceli`; the parser also accepts
`.kuceli(count)` after a dot.

`ururi` is a general left fold. Map, filter, fold, every, and some now iterate
a shallow snapshot, allowing callbacks to change the original list. Callback
errors stop iteration and propagate to `qabo`, including errors from map and
filter that previously could be lost. Invalid callbacks are rejected even on
empty lists. Every/some continue to short circuit.

Object key/value/pair lists follow sorted-key order. `duub` rounds to the
nearest integer with ties away from zero. `xidid` continues to mean square root.
All additions belong to Soplang 3; no version 4 transition is planned.

## Collection consistency and corrected global updates

Lists now accept `jar()` and `jar(start)` alongside `jar(start, end)`. Lists and
strings both require integer bounds; fractional values previously truncated by
slicing now report a catchable error. Negative bounds count from the end and
bounds are clamped. The end is excluded. Reversed or equal bounds return an
empty result; reversed string bounds no longer panic. List slices are shallow
copies and neither receiver is modified.

Strings add `hore()` and `dambe()` for first/last Unicode characters, returning
`maran` for empty strings, matching list endpoint behavior. Lists add
`tiri(value)` using existing list equality rules. String `tiri(sub)` still counts
non-overlapping substring matches and rejects an empty substring. Search misses
continue to return `maran` on lists and `-1` on strings for compatibility.

Global writes from functions, methods, and callbacks are now visible on the next
top-level read, including replacements of collection and callable values. Local
variables and parameters still shadow globals. Top-level numeric loop indexes
are shared with functions, and updates from the loop body feed the next increment.
REPL snippets observe the same current global values.

Writes to typed globals inside functions now enforce type and class checks.
Rejected assignments and failed right-hand expressions preserve the previous
global value. Accepted numeric values are converted to their annotated runtime
representation: integral floats become `abn` and integers become `jajab`.
Fractional floats remain invalid for integer annotations. This corrects stale
reads, invalid writes, and mismatched native numeric representations.
