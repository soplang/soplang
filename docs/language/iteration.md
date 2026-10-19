# Collection iteration

Implemented in the iteration stage of 3.2. The remaining 3.2 function and data
type work is separate; this stage does not publish a new release.

## Loop syntax

```soplang
kuceli (magac ku ["Ayaan", "Cali"]) { qor(magac) }
kuceli (tiro, magac ku tirso(["Ayaan", "Cali"], 1)) { qor(tiro, magac) }
kuceli (tiro ku taxane(5, 0, -2)) { qor(tiro) }
kuceli (xaraf ku "é😀") { qor(xaraf) }
```

The source is evaluated once, then advanced until exhausted. `ku` is contextual
in this header and remains available as a variable, function, or property name.
Existing numeric `kuceli (i start ilaa end :: step)` loops remain inclusive.
When a header beginning with a variable or call named `ku` also forms a complete
numeric header, the numeric form takes precedence. Parenthesize the foreach
source to disambiguate: `kuceli (item ku (ilaa(3)))`.

One binding receives each value unchanged. Multiple bindings require a list
with exactly that many elements. `tirso` and `lamaane` produce such rows:

```soplang
kuceli (magac, da ku lamaane(["Ayaan", "Cali"], [20,25])) { qor(magac, da) }
kuceli (fure, qiime ku {magac: "Ayaan", da: 7}.lamaane()) { qor(fure, qiime) }
```

Binding names must differ. Existing typed variables retain their type checks,
including conversion from `abn` to `jajab`; constants cannot be loop targets.
Row shape and all target types are checked before any target is assigned.
A rejected row has already been consumed, so a retained cursor can resume at
the following row. An empty source preserves previously assigned values.
New loop bindings follow the existing function/global scope rules and are only
assigned when a value is yielded. Declare a default beforehand if it must be
available after an empty loop. Implicit loop bindings are also accepted in
strict mode, as in numeric loops.

`jooji`, `soco`, `celi`, `tuur`, and `dhammee` work inside collection loops.
Breaking does not exhaust a retained iterator. Local closure captures keep the
value captured when the closure was created; top-level bindings remain globals
and retain their existing lookup behavior.

## Sources and snapshots

| Source | Values yielded |
| --- | --- |
| List | Elements from a shallow snapshot made when iteration begins |
| String | Unicode scalar characters, each returned as a string |
| Byte array | Integer byte values from a compact snapshot |
| Plain object | Keys sorted lexicographically at iterator creation |
| Native iterator | Its remaining values, using the same cursor |
| Class or object with the protocol below | Values produced by its `xiga()` method |

Combining marks are separate Unicode scalars; iteration does not group grapheme
clusters. List/object membership changes after snapshot creation do not change
the iteration order. Nested mutable values remain shared.

Use `object.qiime()` for values and `object.lamaane()` for key/value rows. These
existing methods return lists that can be iterated normally. A plain object's
callable `wareeg` or `xiga` member opts into the custom protocol. Non-callable
fields with those names remain ordinary data.

## Iterator functions

| Function | Contract |
| --- | --- |
| `wareeg(value)` | Create an iterator or return an existing native iterator unchanged |
| `value.wareeg()` | Shared method form; custom methods still run their own implementation |
| `xiga(iterator)` / `iterator.xiga()` | Advance a native iterator and return a step object |
| `taxane(stop)` | Lazy integers from zero up to, excluding, `stop` |
| `taxane(start, stop, step?)` | Lazy exclusive range; default step is one |
| `tirso(source, start?)` | Lazy `[index, value]` rows; default index is zero |
| `lamaane(source, ...)` | Lazy rows from zero or more sources, stopping at the shortest |

Range arguments and enumeration starts must be `abn`; zero range steps fail.
Negative steps and negative enumeration starts are accepted. Range arithmetic
stops before integer overflow. Enumeration reports an error if an index would
exceed `abn` while another item remains. Existing `baaxad` still creates a list;
`taxane` does not allocate all range elements.

Adapters initialize their sources when constructed and produce rows on demand.
Snapshot creation may copy a mutable collection. `lamaane()` with no sources
is empty; one source produces one-element rows. Sources advance from left to
right. When a later source ends, an earlier source may already have consumed
one extra item. Passing the same iterator twice shares its cursor; passing the
same list twice creates independent snapshots.

```soplang
door socod = wareeg([maran])
door hore = xiga(socod)
qor(hore.dhammaad, hore.qiime)   // been maran
door dambe = xiga(socod)
qor(dambe.dhammaad, dambe.qiime) // run maran
```

A native step always contains `dhammaad` and `qiime`. Exhaustion remains latched,
and its value is `maran`. A yielded `maran` is therefore distinct from exhaustion.
Native iterator values have type name `wareeg`, display as `<wareeg>`, compare
by identity, and are truthy even after exhaustion. They are not lists;
`teed(iterator)` retains the existing constructor meaning of a one-item list.
Use a loop to collect values if needed.

## User-defined protocol

An iterable defines `wareeg()` returning a native iterator or an object with a
callable `xiga()` method. An object with `xiga()` alone is also accepted. A class
without either method is not implicitly treated as a dictionary.

```soplang
qaab Tiriye {
    hawl dhis(xad) { nafta.hadda = 0 nafta.xad = xad }
    hawl wareeg() { celi nafta }
    hawl xiga() {
        haddii (nafta.hadda >= nafta.xad) { celi {dhammaad: run} }
        door qiime = nafta.hadda
        nafta.hadda = qiime + 1
        celi {dhammaad: been, qiime: qiime}
    }
}
kuceli (qiime ku cusub Tiriye(3)) { qor(qiime) }
```

`xiga()` takes no arguments and returns an object with a boolean `dhammaad`.
When false, `qiime` is required; when true it may be omitted. The wrapper
normalizes terminal steps and remembers exhaustion. Inheritance applies to
both methods and constructors, including subclasses that declare no methods.
Factories control whether repeated calls restart or share their own state.
Use `wareeg(custom)` before passing a custom object to the global `xiga` helper;
calling the custom object's method directly retains ordinary method behavior.

A factory or callback failure preserves its original structured error and
source trace. A failed advance closes that native wrapper; later steps report
exhaustion. An iterator cannot advance itself while it is already advancing.
Native factory/advance nesting is limited to 32 active calls, with a recoverable
error at the limit. Sequential steps and ordinary nested loop bodies do not
retain this depth between operations.

Invalid sources, steps, protocol results, and row shapes report Somali `E037`
errors. Constant bindings use the existing semantic error; duplicate names are
parser errors. See [intentional examples](../../examples/errors/README.md).

## Ownership

Iterators belong to their runtime context. Foreign or expired handles produce
`E035`. Their snapshots, adapter sources, and protocol callbacks participate in
tracing collection, including cycles. Rust-held iterator values and pinned
handles keep live state reachable. `MemoryStats.iterators` counts arena entries.
Context teardown releases cursor state even if Rust retains an unusable handle.

The runtime releases cursor borrows before invoking user code. Reentrant calls
are diagnosed, and nested callback collection preserves native temporary roots.
Rust integrations should release collection borrows before executing code that
accesses those collections; a conflicting snapshot borrow is reported as an
iteration error. General resource close/flush APIs remain part of later I/O work.
