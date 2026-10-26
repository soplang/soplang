# Optional and union types

Optional and union annotations are implemented as part of the 3.2 work. They
constrain variables, constants, parameters, and return values in both JIT and
LLVM execution. Distribution packaging for Soplang 3.2 remains pending.

## Syntax

```soplang
qoraal? name = maran
name = "Ayaan"
abn | qoraal value = 7
value = "toddoba"
maran nothing = maran

hawl aqoonso(qoraal? value = maran): qoraal? { celi value }
hawl identity(abn | qoraal value): abn | qoraal { celi value }
```

`T?` means `T | maran`. A union accepts any listed type. `maran` can also be
used alone as a concrete null-only annotation. Supported members are `abn`,
`jajab`, `qoraal`, `bool`, `teed`, `walax`, `baytyo`, `xirmo`, `urur`, `maran`, and known class
names. Classes follow the existing rule requiring their declaration before
use as an annotation.

The `?` suffix applies to its preceding type. Because unions are alternatives,
`abn | qoraal?` and `abn? | qoraal` both accept integers, strings, and null.
Duplicate alternatives are removed. A two-member union with null is displayed
as `T?` in the AST and editor type text. At most 32 source alternatives are
accepted per annotation.

`door` retains its meaning as an untyped declaration. It is not a union member.
This syntax does not introduce bitwise `|`, ternary expressions, grouping of
type expressions, flow narrowing, or type aliases. [Typed list and object
elements](typed-collections.md) use `teed[T]` and `walax[qoraal, T]`.
[Positional tuples](tuples.md) use `xirmo[T1, T2, ...]`;
[typed sets](typed-sets.md) use `urur[T]`.
Logical `||` retains its existing meaning.

## Required arguments and defaults

An optional type permits a null value. Argument omission still follows the
function's parameter defaults:

```soplang
hawl required(qoraal? value) { celi value }
hawl defaulted(qoraal? value = maran) { celi value }
required(maran)
defaulted()
```

`required()` fails its argument-count check. Explicit null and false count as
supplied values. Named calls, closures, methods, constructors, default
evaluation, and callback validation retain their existing argument contracts.
Rest collectors still require list or object annotations; nullable collector
annotations are rejected because the collector always receives a collection.

## Validation and numeric values

Known incompatible values fail semantic analysis. Unknown dynamic values are
checked when assigned, passed, or returned. A failed assignment preserves the
previous value, and a failed parameter check stops before the body runs.
Runtime contract failures are catchable `E022` errors with Somali messages,
locations, and function traces. Existing arity errors retain their behavior.

A union does not create a new runtime value kind. `nooc(value)` continues to
report the actual type. Exact matches take precedence over numeric conversion,
regardless of alternative order:

```soplang
hawl numeric(jajab | abn value): jajab | abn { celi value }
qor(nooc(numeric(3)))     // abn
qor(nooc(numeric(3.0)))   // jajab
```

When no exact member matches, the existing numeric annotation conversions
apply. An integer can become a float for `jajab?`; a finite integral float can
become an integer for `abn?`. Integer conversion retains the existing saturation
at the signed 64-bit bounds. Fractional or nonfinite floats do not satisfy an
integer-only alternative. Strings and booleans are never implicitly parsed or
converted by these annotations.

Class alternatives accept instances of the class and its subclasses. `walax`
retains its existing object contract. No automatic collection or byte encoding
conversion is performed. A nullable list constrains whether the value is a
list or null; parameterized list and object annotations enforce shared
[element contracts](typed-collections.md).

When a source variable already has a union annotation, every possible source
member must fit a statically checked destination. The analyzer currently does
not narrow that annotation inside `haddii (value != maran)` or similar guards.
Use an explicit conversion or a dynamically checked call when narrowing is
needed. Collection loop targets also retain optional/union checks.

## Return values

```soplang
hawl empty(): abn? {}         // implicit maran is allowed
hawl nothing(): maran { celi }
hawl missing(): abn | qoraal {} // calling this fails its return contract
```

Explicit returns and fallthrough both check the declared return contract.
Falling through returns `maran`. Nullable and null-only return types allow a
bare `celi`; a nonnullable declared return type rejects it during analysis.
The fallthrough check also fixes the earlier gap where a primitive or class
return annotation could return null silently when execution reached the end.

## References

- [Public example](../../examples/36_optional_union_types.sop)
- [Shared backend regression program](../../tests/fixtures/union_types.sop)
- [Argument contracts](function-arguments.md)
- [Diagnostic wording](runtime-error-wording.md#optional-and-union-types)
