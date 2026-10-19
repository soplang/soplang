# Byte arrays

`baytyo` is a mutable, compact sequence of unsigned bytes. Each element is an
`abn` from 0 through 255. This is an implemented part of the 3.2 work; the
complete milestone and its distribution remain pending.

Byte arrays prepare the value model for binary files and network bodies.
Filesystem and networking APIs remain separate roadmap stages.

## Construction and conversion

```soplang
baytyo empty = baytyo()
baytyo data = baytyo([0, 128, 255])
baytyo text = baytyo("é")
qor(text.teed())       // [195, 169]
qor(qoraal(text))      // é
qor(text.qoraal())     // é
qor(text)             // baytyo([195, 169])
```

`baytyo()` creates an empty array. With one argument it accepts a string, a
list containing only valid byte integers, or another byte array. Strings are
encoded as UTF-8. Lists and byte arrays are copied into fresh storage; changing
the source does not change the result. Integers, floats, booleans, null, and
other sources are rejected. No integer-length allocation shorthand is provided.

`qoraal(data)` and `data.qoraal()` decode UTF-8 strictly. Malformed sequences
raise a catchable error; they are never replaced with replacement characters.
Embedded zero bytes are preserved. `data.teed()` copies the elements to a normal
list. The existing `teed(data)` constructor still creates a one-element list
containing the byte array.

`baytyo` is also a contextual type annotation for variables, constants,
parameters, and returns. It does not perform automatic conversion from lists
or strings. A class cannot use this built-in type name. Ordinary identifiers
may still shadow the constructor; annotation positions always mean the byte
array type.

```soplang
hawl copy(baytyo data): baytyo { celi data.nuqul() }
madoor baytyo header = baytyo([83, 79, 80])
```

A constant binding does not freeze mutable contents, following existing list
and object behavior. Type checks apply to later assignments and dynamic calls.

## Indexes and methods

`data[index]` returns an integer, or `maran` when outside the array. Negative
indexes count backward from the end. Indexes must be integers. Assignment
requires an existing position and a valid byte; invalid writes leave the array
unchanged. Ordinary assignment shares storage, as with lists.

| Operation | Result and effects |
| --- | --- |
| `dherer(data)` / `data.dherer()` | Number of bytes, rather than Unicode characters |
| `data.hel(index, default?)` | Element, supplied default, or `maran` |
| `data.dhig(index, byte)` | Replace an existing element; return the receiver |
| `data.geli(byte)` | Append; return the receiver |
| `data.geli(index, byte)` | Insert before an existing position or at the end; return the receiver |
| `data.saar(index?)` | Remove and return an element; default is the last element; empty removal fails |
| `data.hore()` / `data.dambe()` | First or last byte, or `maran` |
| `data.jar(start?, end?)` | Fresh array with an exclusive end; integer bounds, negative offsets, and clamping follow list slicing |
| `data.nuqul()` | Independent copy |
| `data.midee(other)` | Fresh concatenation of two byte arrays; both inputs are preserved |
| `data.nadiifi()` | Clear the receiver; return it |
| `data.rog()` | Reverse the receiver in place; return it |
| `data.leeyahay(byte)` | Whether the byte is present |
| `data.tiri(byte)` | Occurrence count |
| `data.raadi(byte)` | First index, or `maran` |
| `data.qoraal()` | Strict UTF-8 decoding |
| `data.teed()` | Fresh list of byte integers |
| `data.wareeg()` | Iterator over a byte snapshot |

Byte methods accept positional arguments. An empty array is false in
conditions and `bool`; a nonempty array is true. Equality compares byte
contents. A byte array is distinct from a list with the same integer values.
Display, interpolation, and ordinary formatting show `baytyo([0, 255])`.
Explicit decoding is required to interpret those contents as text.

## Iteration and ownership

```soplang
kuceli (byte ku baytyo([0, 128, 255])) { qor(byte) }
kuceli (index, byte ku tirso(baytyo("SOP"))) { qor(index, byte) }
```

Iteration copies bytes when the cursor is created. Changes to the original
array do not alter the remaining values. Byte cursors store one byte per
pending element and work with `wareeg`, `xiga`, `tirso`, and `lamaane`.

Byte storage belongs to runtime contexts, preserves repeated-conversion
identity, and participates in tracing collection. Generated stack roots,
closures, nested collections, iterators, and explicit Rust pins retain live
values. Stale and foreign tagged handles fail with `E035`. Host-held Rust byte
values retain their data; tagged handles remain specific to their owner.
Growth and retained buffer capacity contribute to collection accounting.

## Errors

Invalid byte values, byte indexes and writes, UTF-8 decoding, unsupported byte
operations, and conflicting host borrows use `E038`, with Somali wording and
source locations. Argument counts and type annotations retain their existing
`E022` checks. Slice argument errors reuse list-slicing diagnostics. A rejected
class name uses `E020`.

See [35_bytes.sop](../../examples/35_bytes.sop), the shared
[regression program](../../tests/fixtures/bytes.sop), and the
[wording inventory](runtime-error-wording.md#byte-arrays).
