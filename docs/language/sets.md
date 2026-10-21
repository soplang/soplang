# Sets

`urur(...)` constructs a mutable set of unique, immutable values. Iteration
and display preserve insertion order; equality compares membership regardless
of order. Membership uses a hash table, including for tuple keys.

```sop
urur tirooyin = urur(3, 1, 3, 2)
qor(tirooyin.teed())                // [3, 1, 2]
qor(tirooyin == urur(2, 3, 1))      // run
qor(tirooyin.leeyahay(1))           // run
tirooyin.saar(1)
tirooyin.geli(1)
qor(tirooyin.teed())                // [3, 2, 1]
```

## Keys and equality

Accepted keys are `maran`, booleans, integers, floating-point numbers other
than NaN, strings, and tuples whose members are all accepted keys. Mutable
lists, objects, byte arrays, and other sets cannot be keys. Functions, closures,
errors, and iterators cannot be keys either.

Key equality keeps numeric kinds distinct, as existing collection equality
does: integer `1`, floating-point `1.0`, and boolean `run` are separate keys.
Positive and negative floating-point zero are the same key. This also applies
inside tuple keys. NaN is rejected because it does not equal itself; infinities
are valid. Tuple keys may contain at most 64 nested tuple levels. Deeper tuples
remain valid language values but are rejected as keys with a catchable error.

Insertion of an existing key preserves its position and stored value. Removing
and reinserting a key places it last. An empty set is false in conditions and
`bool(...)`; a nonempty set is true. `nooc(...)` returns `"urur"`.

## Construction and conversion

Each constructor argument is one element. `urur(xirmo(1, 2))` has one tuple
key; the constructor does not expand a single iterable argument.

Use `.urur()` on a list, tuple, byte array, string, or native iterator to convert
its elements. Strings yield Unicode scalar characters. The conversion rejects
the first unsupported key. Iterator conversion consumes input through that
key; a failed conversion does not rewind the iterator.

```sop
qor([1, 1, 2].urur())       // urur(1, 2)
qor("aba".urur())           // urur(a, b)
qor(xirmo(1, 2, 1).urur())  // urur(1, 2)
```

`set.teed()` and `set.xirmo()` produce independent outer containers in insertion
order. `.nuqul()` and `.urur()` on a set return independent sets. Immutable
tuple keys may be shared safely. `teed(set)` and `xirmo(set)` keep the set as
one element, following the existing constructor rules.

## Methods

| Method | Behavior |
| --- | --- |
| `dherer()` | Return the number of keys; global `dherer(set)` also works. |
| `leeyahay(value)` | Test membership. Unsupported key kinds raise an error. |
| `geli(value)` | Add one key and return the receiver. |
| `saar(value)` | Remove a key and return its stored value, or `maran` when absent. |
| `nadiifi()` | Remove all keys and return the receiver. |
| `nuqul()` / `urur()` | Return an independent set. |
| `teed()` / `xirmo()` | Copy the elements to a list or tuple. |
| `wareeg()` | Return a cursor over a snapshot of the current keys. |
| `midee(other)` | Return the union: left keys followed by unseen right keys. |
| `wadaag(other)` | Return the intersection, preserving left order. |
| `reeb(other)` | Return left keys absent from the right, preserving left order. |
| `kaladuwan(other)` | Return keys present in exactly one operand, left keys first. |
| `hoosaad(other)` | Test whether every left key belongs to the right. |

Set operations require another set. They return fresh results and preserve
both operands. `saar(maran)` returns `maran` both when it removes that key and
when it was absent; use `leeyahay(maran)` beforehand when that matters.

Sets have no positional index. Indexed reads, indexed writes, and property
writes raise `E041`. Sequence-only methods such as `hore()`, `jar()`, and
`habee()` are not defined. Invalid keys, unsupported methods, and invalid
set-operation operands also raise `E041`. Arity failures retain the existing
shared argument diagnostics.

## Iteration, annotations, and lifetime

`kuceli (item ku set)` uses a shallow snapshot. Adding or removing keys during
the loop does not change that loop's input. Tuple keys work with multiple row
bindings. Cursors remain valid after the source set is cleared or collected.

Bare `urur`, `urur?`, and unions constrain variables, parameters, returns, and
nested collection members. The constructor remains shadowable as an ordinary
identifier; type positions recognize the builtin name. A class cannot be named
`urur`. [Typed element annotations](typed-sets.md), `urur[T]`, enforce shared
contracts through aliases and nested collections.

Aliases share mutations. Copies have independent membership. Handles belong to
their runtime context; stale and foreign handles raise `E035`. The collector
traces tuple keys retained by sets and snapshots, and reclaims their storage
when unreachable. Native `SetData` exposes construction and read-only snapshots;
its internal key table is private so host code cannot invalidate key hashes.
