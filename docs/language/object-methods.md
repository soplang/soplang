# Expanded object methods

The 3.2 object-library stage adds insert-if-absent, in-place merging, callback
mapping and filtering, and explicit nested paths. These operations work through
both Cranelift JIT and LLVM AOT and share existing collection contracts.

## Insert when absent

`object.geli(key, value)` inserts one string key only if it is missing and
returns the receiver. An existing value, including `maran`, is preserved.
The value expression is evaluated normally even when the key already exists.
An ignored value is not checked against the stored element contract.

```soplang
door scores = {Ayaan: 90}
scores.geli("Cali", 80)
scores.geli("Ayaan", 100)
qor(scores) // {Ayaan: 90, Cali: 80}
```

Use `dhig(key, value)` to replace an existing value unconditionally. `geli`
retains the add-one-item vocabulary and receiver return contract used by lists
and sets. Missing-key insertion validates every affected shared contract before
publishing a change.

## Merge in place

`object.ballaadhi(other)` merges another object into the receiver and returns
the receiver. Right-hand values win. The input is copied before mutation, so
self-merging is supported. The merge is shallow: nested values remain shared.
A rejected contract leaves all receiver fields unchanged, including fields
that would individually have been valid.

```soplang
door scores = {Ayaan: 90}
scores.ballaadhi({Ayaan: 95, Cali: 85})
qor(scores) // {Ayaan: 95, Cali: 85}
```

`midee(other)` and its earlier `kudar(other)` spelling continue returning a
merged copy. `ballaadhi` explicitly changes the receiver, matching list extension.
Object operations include all stored fields; no metadata keys are silently
removed from input maps.

## Map values or entries

`object.aaddin(callback, mode?)` returns a fresh object. The callback receives
`(key, value)`. Keys are visited in sorted string order, matching `furayaal()`.
The complete outer entry list is snapshotted before the first callback.

The default mode, `"qiime"`, preserves each key and stores the callback's
returned value. Lists and tuples returned in this mode remain ordinary values.

```soplang
door scores = {Ayaan: 90, Cali: 85}
door doubled = scores.aaddin(hawl(key, value) { celi value * 2 })
qor(doubled) // {Ayaan: 180, Cali: 170}
```

Explicit `"lamaane"` mode supports renaming keys. Each callback must return
an exact two-element list or tuple, whose first member is the new string key
and whose second member is its value. Repeated output keys raise `E043` rather
than silently discarding a mapped entry. Pair contents are read once when
returned; nested references remain shared.

```soplang
door renamed = scores.aaddin(hawl(key, value) {
    celi xirmo(key.yaree(), value)
}, "lamaane")
```

## Filter entries

`object.shaandhee(callback)` also receives `(key, value)` in sorted snapshot
order. It returns a fresh object containing entries whose callback result is
truthy. Stored values are preserved, including `maran` when selected.

Mapping and filtering validate callback signatures even for an empty object.
Empty inputs never invoke the callback. Callback errors stop further calls and
propagate their original source location and trace. No partial result is
returned. Callback side effects remain: clearing or adding source fields does
not change the snapshot's visitation order, and nested mutable values are shared.

New result objects have independent outer storage and no copied outer contract.
An annotation on the result checks and constrains it normally. Nested mutable
members can remain constrained by other live guarded parents.

## Nested paths

`object.hel(path, default?)` accepts an explicit list or tuple of address
components. Strings address object fields and error properties. Integers address
list, tuple, byte, and string positions; negative positions count from the end.
String positions count Unicode scalars, not bytes or graphemes.

```soplang
door data = {users: [{name: "Ayaan"}]}
qor(data.hel(["users", 0, "name"])) // Ayaan
qor(data.hel(xirmo("users", -1, "name"))) // Ayaan
```

A string argument keeps the earlier literal-key behavior. `hel("a.b")` looks
for the exact key `a.b`; it does not split dots. Explicit paths can therefore
address keys containing dots, spaces, or other punctuation without escaping.

An empty path returns the receiver itself. Missing keys, out-of-range positions,
and attempts to descend through a non-indexable value return the default,
which is `maran` when omitted. A stored `maran` at the final address is returned
unchanged, even when another default was supplied. Further descent through that
null value is missing and returns the default.

Every component must be text or an integer; invalid component kinds are checked
before traversal, even when an earlier key is missing. Object/error steps need
text, and sequence steps need integers. A wrong addressing kind raises `E043`.
Class fields are read as stored data without invoking declared methods. Known
structured error fields retain their existing null values; unknown field names
are missing. Borrowed byte storage keeps the existing byte diagnostic `E038`.

Paths are traversed iteratively, so deep paths through cyclic data do not consume
one call frame per component. Returned mutable values retain their original
identity and aliases. Lookup performs no writes and invokes no user code.

## Dispatch, errors, and lifetime

These methods apply to plain objects. Class instances keep their declared
method dispatch. New builtin object names take precedence over plain-object
callable fields, as existing builtin names do. Use indexed calls such as
`object["aaddin"](...)` to invoke a stored callable field explicitly.

Invalid keys, modes, pair shapes, duplicate output keys, path components, and
conflicting read borrows raise Somali `E043` errors. Invalid callback signatures
retain existing callback diagnostics. Rejected shared contracts retain `E039`;
wrong arities retain shared argument diagnostics. Post-callback validation
points at the outer operation, while callback failures preserve their inner
caret and call trace.

No native borrow survives a callback. Context ownership, tracing roots, weak
contract lifetimes, and nested dependency updates follow the existing runtime
and [typed-collection contracts](typed-collections.md). See the
[public example](../../examples/43_object_operations.sop),
[shared fixture](../../tests/fixtures/object_operations.sop), and
[verification notes](../testing/LANGUAGE_3_2.md).
