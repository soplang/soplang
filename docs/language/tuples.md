# Tuples

`xirmo` is a fixed sequence with immutable positions. Tuple values and positional
type annotations are implemented in both execution paths as part of the 3.2
work. Sets and remaining library methods are still required for the milestone.

## Construction and conversion

```soplang
xirmo empty = xirmo()
xirmo[abn, qoraal] person = xirmo(25, "Ayaan")
xirmo[abn] single = xirmo(7)
qor(person)                  // xirmo(25, Ayaan)
qor(person.teed())           // [25, Ayaan]
qor([1, 2].xirmo())          // xirmo(1, 2)
```

`xirmo(...)` stores its positional arguments in order. Zero arguments create an
empty tuple; one creates a one-element tuple. Passing a list or another tuple
stores that value as one element. Use `list.xirmo()` to copy list elements into
a fresh tuple, or `tuple.teed()` to copy tuple elements into a fresh list.
`teed(tuple)` retains the existing list constructor's one-argument meaning.

Tuple display uses `xirmo(...)`. Like other collection displays, it is readable
text rather than an escaped serialization format. `nooc` returns `xirmo`.
An empty tuple is false; any nonempty tuple is true, including `xirmo(been)`.
Equality compares length and element values using existing collection equality:
for example, `xirmo(1)` differs from both `[1]` and `xirmo(1.0)`.

The constructor remains shadowable as an ordinary identifier. In declaration
and annotation positions, `xirmo` denotes the built-in type. A class cannot use
this type name. A declaration-shaped contextual prefix takes priority over
separate expression statements; use a semicolon or parentheses to disambiguate
an indexed expression followed by an assignment when necessary.

## Immutable positions and shared members

```soplang
door values = [1]
xirmo[teed[abn]] holder = xirmo(values)
values.geli(2)               // holder[0] is now [1, 2]
```

The tuple's positions, length, and ordering cannot be changed. Index/property
writes and mutating collection methods fail with catchable `E040` errors.
The contained values retain their own mutability: a list, object, byte array,
or other mutable member remains shared. Positional annotations enforce their
nested contracts through all aliases while the guarded tuple is live.

Copying, slicing, and concatenation create fresh outer storage. Nested mutable
members remain shared. A fresh outer copy acquires its own positional contract
only when annotated; contracts on any still-live original parent continue to
constrain shared mutable children. See [shared contracts](typed-collections.md).

## Indexes and methods

Indexes are integers. Negative indexes count backward from the end. An index
outside the tuple returns `maran`; a noninteger index fails.

| Operation | Result |
| --- | --- |
| `dherer(tuple)` / `tuple.dherer()` | Element count |
| `tuple.hel(index, default?)` | Element, supplied default, or `maran` |
| `tuple.hore()` / `tuple.dambe()` | First/last element, or `maran` |
| `tuple.jar(start?, end?)` | Fresh slice; exclusive end, negative offsets, clamped bounds |
| `tuple.nuqul()` | Fresh shallow tuple copy |
| `tuple.midee(other)` | Fresh concatenation with another tuple |
| `tuple.teed()` | Fresh shallow list copy |
| `tuple.leeyahay(value)` | Membership |
| `tuple.tiri(value)` | Occurrence count |
| `tuple.raadi(value)` | First index, or `maran` |
| `tuple.wareeg()` | Iterator over the immutable sequence |

Argument counts and slice arguments retain their existing checks. Tuple
operations do not add implicit element conversions or concatenation operators.

## Positional annotations

```soplang
xirmo[] zero = xirmo()
xirmo[abn | qoraal, bool?] row = xirmo("Ayaan", maran)
xirmo[abn]? optional = maran
hawl identity(xirmo[abn, qoraal] row): xirmo[abn, qoraal] { celi row }
```

`xirmo` accepts any tuple shape. `xirmo[T1, T2, ...]` requires exactly that many
positions and checks each element against its annotation. `xirmo[]` requires
an empty tuple. Element types may include classes, nested typed collections,
optional types, and unions. Numeric element kinds are exact, as with typed
lists. Known positional contracts are compared conservatively and invariantly;
unknown contents receive runtime checks.

Variables, constants, parameters, defaults, named calls, closures, methods,
constructors, returns, and loop binding targets use the same validation. Rest
collectors remain lists and objects: use `teed[xirmo[abn]] *rows` when collecting
tuple arguments. Tuple annotations obey the common type-expression depth and
node budgets.

## Iteration and lifetime

Collection loops, `wareeg`, `xiga`, `tirso`, and `lamaane` accept tuples.
Multiple loop bindings accept either a list or tuple with exactly the required
number of elements. Existing enumerate/zip helpers continue producing list rows.
Tuple cursors retain the immutable sequence and release it when exhausted or
collected; they do not need a shallow membership snapshot.

Runtime contexts own tuple handles. Repeated conversions preserve identity;
foreign or collected tagged handles fail with `E035`. The collector follows
tuple members, iterator references, and closure captures. It can clear only
unreachable tuple edges, so cycles through mutable children and deep immutable
chains can be reclaimed without changing a live tuple.

Structural equality walks complete collection graphs iteratively. Collection
display truncates after 64 active nested containers with `...`; this bounds
formatting stack use independently of graph depth. Native integrations can
construct `TupleData` and obtain read-only member borrows. Its positions have
no public mutation API.

See the [public example](../../examples/38_tuples.sop), the
[backend regression program](../../tests/fixtures/tuples.sop), and the
[Somali diagnostic inventory](runtime-error-wording.md#tuples).
