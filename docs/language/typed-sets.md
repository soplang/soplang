# Typed sets

`urur[T]` constrains every set key to `T`. Checks apply to variables, constants,
parameters, returns, and nested collections in both execution backends.
The [set reference](sets.md) defines construction, key equality, insertion order,
and set operations. This syntax does not change which values can be set keys.

```soplang
urur[abn] numbers = urur(1, 2)
urur[abn | qoraal] labels = urur(1, "one")
urur[xirmo[abn, qoraal?]] rows = urur(xirmo(1, "Ayaan"), xirmo(2, maran))
urur[maran] nulls = urur(maran)
urur[abn]? optional = maran
```

## Key annotations

Elements may use immutable primitive types, `maran`, bare `xirmo`, positional
tuples, optional types, and unions. Tuple positions may themselves use those
forms. `urur[xirmo[]]` accepts only empty tuple keys. Bare `urur` constrains
only the outer kind.

An element annotation must admit at least one valid key. For example,
`urur[teed]`, `urur[walax]`, `urur[baytyo]`, `urur[urur]`, and
`urur[xirmo[teed]]` are rejected during analysis with `E022`. Class instances
are objects and cannot be keys. Unknown class names are diagnosed first with
`E021`, including inside nested annotations.

A union may include alternatives that are not valid keys if another alternative
admits valid keys: `urur[abn | teed]` can contain integers, and `urur[teed?]`
can contain `maran`. Trying to insert a list still fails with the usual `E041`
key diagnostic. A union does not make a mutable value hashable.

Element numeric kinds are exact. `urur[jajab]` does not accept integer keys,
and `urur[abn]` does not convert floating-point keys. Use explicit conversion
before insertion. This preserves key identity and aliases. The same rule applies
to tuple positions. Existing numeric conversions for scalar annotations remain
unchanged.

Annotations take exactly one element type. Empty arguments, extra arguments,
and trailing commas are parser errors. The existing 32-level, 32-alternative,
and 1024-source-node annotation limits also apply to sets.

`urur` remains an ordinary identifier outside type positions. As with tuples,
`urur[Type] name = value` is parsed as a declaration. If a shadowed `urur`
is indexed with an identifier immediately before a separate assignment,
use a semicolon or parentheses to separate the expression from the assignment.

## Shared contracts and mutations

```soplang
urur[abn] numbers = urur(1)
door alias = numbers
tijaabi { alias.geli("bad") } qabo (message) { qor(message) }
qor(numbers) // urur(1)
```

A successful annotation check attaches a contract to the shared set identity.
`geli`, `saar`, and `nadiifi` validate candidate contents against every affected
contract before publishing the change. Failure raises catchable `E039` and
preserves membership, insertion order, and tracked dependencies. Argument
expressions evaluated before the method call keep their ordinary side effects.
Invalid key validation happens before mutation validation and uses `E041`.

Multiple accepted contracts all remain in force while their collection lives.
An empty set accepted through both integer-only and string-only annotations must
remain empty. Reassigning or dropping an annotated binding does not remove its
contract from a set retained by another alias.

Mutable generic types are invariant. A known `urur[abn]` cannot be assigned to
`urur[jajab]` or `urur[abn | qoraal]`. An untyped alias can request another
contract after its current contents pass validation. Equivalent unions compare
independently of their alternative order, including inside tuple annotations.

## Nested collections

```soplang
door child = urur(1)
teed[urur[abn]] rows = [child]
walax[qoraal, urur[abn]] named = {first: child}
xirmo[urur[abn]] tuple = xirmo(child)
```

Mutating `child` must preserve every guarded parent. A child replaced or removed
from a parent stops inheriting that parent's constraint after the successful
parent mutation. A discarded parent retains its constraints until collected.
Contracts attached directly to the child remain independent of parent lifetime.

Whole-container unions retain their correlation rules. For
`teed[urur[abn]] | teed[urur[qoraal]]`, a write must leave the complete root
valid under one alternative. Each child cannot choose an alternative independently.

Set copies, conversions, unions, intersections, and differences return fresh
outer storage without copying attached contracts. An annotation on a new result
checks and constrains it normally. Tuple keys are immutable and may be shared.

## Functions and lifetime

```soplang
hawl fresh(urur[abn] values = urur()): urur[abn] { celi values }
hawl gather(teed[urur[abn]] *values, walax[urur[abn]] **options) {
    celi [values, options]
}
```

Defaults, named arguments, closures, methods, constructors, and return checks
share the same runtime contract. A failed initializer, argument, assignment,
or return check raises `E022` without installing a partial guard. An accepted
parameter contract persists even if later execution fails. Rest collectors must
remain lists or objects; their elements may be typed sets.

Weak guard roots do not keep sets, tuple keys, or cyclic parent graphs alive.
Collection removes dead guards and dependencies. Unrelated unguarded sets use
the direct mutation path. Guarded writes copy the changed set and may traverse
whole affected roots. Rust integrations must activate the owning runtime context
and use the checked ABI; `SetData` exposes no public mutable backing storage.
