# Typed lists and objects

Typed lists and string-keyed objects are implemented in the 3.2 work.
[Positional tuples](tuples.md) are implemented separately; sets and their
annotations remain planned. The complete milestone
is not yet ready for a tag.

## Syntax and validation

```soplang
teed[abn] numbers = [1, 2]
walax[qoraal, abn] scores = {Ayaan: 90}
walax[abn] other_scores = {Cali: 85}
teed[teed[abn]] rows = [[1, 2], [3]]
teed[abn | qoraal?] values = [1, "two", maran]
teed[abn]? optional = maran
```

`teed[T]` constrains every element to `T`. `walax[qoraal, T]` constrains every
stored value to `T`; object keys remain strings. `walax[T]` is shorthand for
the same string-keyed object. Other key types are rejected. Class instances
normally use their nominal class annotation; map contracts examine all stored
fields, including any class metadata.

Nested contracts can use all existing concrete annotations, known class names,
optional types, and unions. These annotations do not introduce parameterized
user classes or generic byte arrays. `nooc` and value display retain the existing
`teed` and `walax` behavior.

Semantic analysis checks outer kinds, known class names, and mutable generic
compatibility. Contents are validated at runtime, including literals and empty
collections. A failed initial check raises `E022` and installs no guard.

Element numeric types are exact: `teed[jajab]` needs float elements, and
`teed[abn]` needs integers. Use explicit numeric conversion when inserting a
different numeric kind. This prevents a check from changing data visible to
another alias. Scalar optional/union annotations keep their existing conversion
rules.

Mutable generic annotations are invariant. A known `teed[Child]` cannot be
assigned directly to `teed[Base]`, even when Child extends Base; writes through
the latter could violate the former. The same applies to different numeric
contracts. Equivalent element unions compare independently of alternative order.
An untyped alias can request another contract, subject to runtime validation;
all accepted contracts then apply to the shared collection.

## Aliases and nested values

```soplang
teed[abn] numbers = [1]
door alias = numbers
tijaabi { alias.geli("bad") } qabo (message) { qor(message) }
qor(numbers) // [1]
```

A successful annotation check attaches a shared contract to the collection's
identity. Index writes, property writes, append/insert/replace/remove/clear,
reverse, sorting, and compatibility method names all preserve it. Changing or
dropping a binding does not remove an accepted contract from a live collection.
Multiple annotations on one identity all apply. For example, an empty list
accepted by both integer-only and string-only contracts must remain empty.

Nested aliases are checked against their guarded parents. A child removed or
replaced in a parent stops inheriting that parent's constraint. Any contract
attached directly to the child remains in force. A shallow copy has independent
outer storage; its nested mutable values remain shared with the original.
Fresh concatenations and other newly allocated results also have independent
outer storage until annotated.

A union of whole containers is different from a container of union elements:

```soplang
teed[abn] | teed[qoraal] homogeneous = []
teed[abn | qoraal] mixed = [1, "two"]
```

Every mutation of `homogeneous` must leave the entire container valid under at
least one alternative. Nested siblings must satisfy the same chosen whole-root
alternative; each child cannot independently choose a different branch.

## Calls, defaults, and returns

```soplang
hawl identity(teed[abn] values): teed[abn] { celi values }
hawl fresh(teed[abn] values = []): teed[abn] { celi values }
hawl gather(teed[abn] *values, walax[qoraal, abn] **options): teed[abn] {
    celi values
}
```

Functions, closures, methods, constructors, named calls, defaults, returns, and
loop binding targets use the same checks. Rest collectors may use typed list
or object annotations. Nullable collectors are rejected because a collector
always receives a collection. Defaults retain their call-time evaluation and
fresh-literal behavior.

A successful parameter check attaches its contract before later defaults and
the function body run. That accepted contract remains even if a later parameter
or later execution fails. A failed individual check installs no partial guards
on nested children. A declared generic return type checks and constrains the
returned storage before the caller receives it.

## Mutation failures and diagnostics

Writes first validate candidate contents against all affected guarded parents.
Only a valid candidate is published. Rejected writes leave both data and
tracked dependencies unchanged and raise catchable `E039` errors at the mutation
site. Argument expressions already evaluated before the mutation keep their
ordinary side effects.

Diagnostics use stable schema ordering when several guarded parents reject a
write. Parser errors identify invalid type arguments, unsupported keys, and
annotation limits. Missing class names retain `E021`. See the
[wording inventory](runtime-error-wording.md#typed-lists-and-objects).

Type annotations allow up to 32 levels and 32 source alternatives per union,
with a total budget of 1024 source type nodes. These limits bound compiler and
runtime type traversal. Finite nested annotations support cyclic values; they
do not introduce recursive type aliases.

## Native integration and lifetime

Runtime contexts retain weak guard roots and index the mutable identities each
contract reads. Unrelated collections keep the direct mutation path. Nested
replacement updates the index after successful validation. Writes to guarded data copy the changed container and may validate whole
affected roots; large guarded appends can require whole-container work. Schema
parsing uses a bounded cache. Garbage collection removes guards for reclaimed containers,
including cyclic graphs; guard metadata does not keep language values alive.
`MemoryStats.type_contracts` counts guarded collection identities.

Rust integrations must activate the owning `RuntimeContext` and use ABI calls
or the guarded standard-library mutation functions. Direct edits to the public
`Rc<RefCell<...>>` backing values are trusted host operations and bypass language
checks. Contract validation and candidate publication report conflicting host borrows
as recoverable errors.

See the [public example](../../examples/37_typed_collections.sop), the
[shared regression program](../../tests/fixtures/typed_collections.sop), and
[local verification](../testing/LANGUAGE_3_2.md).
