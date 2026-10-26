# Expanded list methods

The 3.2 list-library stage adds removal by value, in-place extension, stable
uniqueness, numeric sums, iterator adapters, and callback-key sorting. Existing
indexing, copying, slicing, replacement, fold/map/filter operations, and method
aliases remain available in both execution backends.

## Removing a value

`saar(value, "qiime")` removes the first structurally equal element and returns
that stored element. It returns `maran` when no element matches. All value kinds
are supported, including nested collections and cycles. Numeric kinds remain
distinct inside collection equality: integer `1` and float `1.0` are separate
matches. The receiver is mutated through its shared contracts.

```soplang
door numbers = [1, 2, 1]
qor(numbers.saar(1, "qiime")) // 1
qor(numbers)                 // [2, 1]
```

`saar(index)` still removes by integer position, including negative positions.
`saar()` still removes the last element. The explicit `"qiime"` mode avoids
ambiguity between an integer value and an integer index. Removing a stored
`maran` and missing a value both return `maran`; use `leeyahay` first if that
distinction matters.

## In-place extension

`ballaadhi(source)` appends all values from an iterable and returns the receiver.
It accepts lists, tuples, sets, bytes, text, object keys, lazy ranges, native
cursors, and user-defined iteration protocols. Iteration follows the existing
[iteration contract](iteration.md), including sorted plain-object keys.

```soplang
door numbers = [1]
numbers.ballaadhi(xirmo(2, 3))
qor(numbers) // [1, 2, 3]
```

The complete input is collected before appending. Self-extension uses a snapshot.
Failed iteration or a rejected element contract appends none of the collected
values. Input iterators may already have advanced, and user protocol side effects
remain. If protocol code changes the receiver successfully, extension appends to
its current contents after consumption finishes. No mutable receiver borrow is
held while invoking user code.

`midee(other)` retains its concatenated-copy behavior. `geli(value)` still adds
one element, including a nested list. `ballaadhi` explicitly changes the receiver.

## Stable uniqueness

`keli()` returns a new list containing the first occurrence of each distinct
value, preserving input order and sharing nested references. It supports every
value kind, including mutable and cyclic values. Immutable hashable keys use
the set key representation; other values use structural equality. The latter
path can require pairwise comparisons. Unlike `.urur()`, this operation does
not reject mutable elements or require all elements to be hashable.

Numeric kinds remain distinct; floating-point signed zeros compare equal.
Scalar NaN values remain distinct because they do not equal themselves.
Tuple and collection identity fast paths follow the existing equality contract.
The result has independent outer storage and no copied outer element contract.

## Numeric sums

`wadar(initial?)` sums list elements from left to right. The initial value defaults
to integer zero. Every element and the initial value must be `abn` or `jajab`;
booleans and strings are rejected. Integer addition wraps at 64 bits, matching
ordinary arithmetic. The first floating-point operand promotes the running sum.
An empty list returns its initial value.

```soplang
qor([1, 2, 3].wadar())      // 6
qor([1, 2].wadar(0.0))     // 3
```

## Iterator adapters

`list.tirso(start?)` is the method form of `tirso(list, start?)`.
`list.lamaane(other...)` is the method form of `lamaane(list, other...)`.
Both return native cursors with existing snapshot and ownership rules. Zip stops
at the shortest input. With no additional sources, it produces one-element rows.

## Stable sorting

`habee(key?, reverse?)` sorts in place and returns the receiver. Omitting the key
or passing `maran` uses each element as its key. The reverse flag is a boolean,
defaulting to `been`. A supplied key function receives one element and is checked
for a compatible signature even on an empty list.

```soplang
door people = [{age: 30, name: "Cali"}, {age: 20, name: "Ayaan"}]
people.habee(hawl(person) { celi person.age })
qor(people[0].name) // Ayaan
```

Keys are evaluated exactly once per snapshot element, from left to right. Equal
keys preserve their original order, including reverse sorting. A failed callback
stops further calls and leaves the sorting result unpublished. Callback side
effects remain. After all callbacks finish, mutable key values are rendered to
stable comparison data; comparisons invoke no user code.

Callbacks may change nested values. If they leave the receiver's outer slots
changed, sorting raises `E043` and preserves those callback writes. Changes
include replacing a nested collection with an equal but distinct collection.
Changing and restoring the same outer slots is allowed. This prevents publishing
a sorted snapshot over data supplied by the callback.

### Comparison order

Keys have a consistent order across types:

1. `maran`.
2. Booleans (`been` before `run`).
3. Integers and floating-point numbers, compared numerically without first
   rounding integers to floats.
4. Strings, compared lexically.
5. Lists, tuples, sets, bytes, objects, callables, errors, and iterators, in
   that type order.

Composite keys compare by bounded, cycle-safe display text within their type.
Callables and iterators are opaque ties; callable handles are also hidden inside
composite comparison text. Equal-looking composites therefore remain stable.
Use a scalar key callback when sorting structured values by a particular field.

Numeric signed zeros tie. Infinities retain their numeric order. NaNs follow
all other numbers and compare by IEEE total ordering among themselves. Integer
precision is preserved around the floating-point exactness boundary and at both
64-bit endpoints.

This corrects the earlier pairwise numeric/string fallback, which could form
inconsistent comparisons for mixed types. Existing homogeneous scalar sorting
keeps its order, apart from equal signed zeros now preserving input order.

## Errors, contracts, and lifetime

Invalid removal modes, sum operands, reverse flags, changed sorting receivers,
and conflicting read borrows use catchable Somali `E043` diagnostics. Iterator
failures retain `E037` or the original user error. Rejected element writes retain
`E039`. Callback signature failures retain existing callback diagnostics, and
wrong method arities retain existing argument-count diagnostics. Post-callback
validation points at the outer operation; errors raised by callbacks retain
their inner source position and call trace.

Mutating operations validate all affected shared contracts before publication.
Copies retain nested references, and cursor snapshots retain their values after
source collection. Managed values held during callbacks survive collection
safepoints. See the [public example](../../examples/42_list_operations.sop),
[shared fixture](../../tests/fixtures/list_operations.sop), and
[verification notes](../testing/LANGUAGE_3_2.md).
