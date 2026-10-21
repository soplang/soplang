# Function arguments

## Default values

A parameter may supply an expression after `=`. Parameters without defaults
must come first. Parameter names must be unique.

```sop
hawl salaam(magac, hordhac = "Salaan") {
    celi f"{hordhac}, {magac}!"
}
qor(salaam("Ayaan"))
qor(salaam("Cali", "Soo dhowow"))
```

Omitting an argument evaluates its default **at call time**, in parameter order.
An explicitly supplied argument, including `maran`, `been`, `0`, or an empty
collection, skips that default. All supplied argument expressions are evaluated
before any defaults. Calls may omit only a trailing sequence of positional parameters.
Named arguments can also skip earlier defaulted parameters; see below.

Defaults can use earlier parameters, globals, and captured variables. A default
cannot use its own parameter or a later parameter, even if that later argument
will be supplied by a particular call. A nested function can shadow those names
with its own parameters.

```sop
hawl samee(a, b = a + 1, c = b + 1) {
    celi [a, b, c]
}
qor(samee(4))
qor(samee(4, 10))
```

Mutable defaults are created again for each omitted argument. Existing values
returned from a global or captured variable keep their usual shared identity;
a default does not clone arbitrary values.

```sop
hawl geli(qiime, waxyaabo = []) {
    waxyaabo.geli(qiime)
    celi waxyaabo
}
qor(geli(1))
qor(geli(2))
```

## Variadic positional arguments

A final parameter prefixed with `*` collects remaining positional arguments in
a fresh list. Required and defaulted parameters may precede it. Only a final `**` collector may follow it, and it cannot have a default of its own.

```sop
hawl wadarta(bilow = 0, *tirooyin) {
    kuceli (tiro ku tirooyin) { bilow = bilow + tiro }
    celi bilow
}
qor(wadarta())
qor(wadarta(10, 1, 2, 3))
```

`*tirooyin` is always a `teed`, including in strict mode. The equivalent explicit
annotation is `teed *tirooyin`. A [typed element contract](typed-collections.md)
can be written `teed[abn] *tirooyin`; scalar annotations such as `abn *tirooyin` are
rejected. List elements retain their original types and identities. An empty
rest list is still freshly allocated. Passing an existing list supplies one
argument; it does not expand that list. Call-site spread syntax is not yet
implemented.

The `*` parameter does not consume the implicit instance or closure environment.
Functions, closures, class methods, parent methods, inherited constructors, and
collection callbacks all follow the same rule. Calls with fewer than the
required fixed parameters fail before evaluating defaults or entering the body.
There is no fixed language arity limit; available memory and the ABI's signed
32-bit argument count bound individual call vectors.

## Named arguments and keyword collection

Use `name: value` to supply a declared parameter by name. Positional arguments
come first. Named arguments can appear in any order and can skip parameters
with defaults.

```sop
hawl samee(a=1,b=a+1,c=b+1) { celi [a,b,c] }
qor(samee(c:9))
qor(samee(c:8,a:4))
```

All supplied expressions run in their written order, before defaults. Binding
then places values in parameter order. Supplying a parameter both positionally
and by name is an error, as are duplicate names, unknown names without a
collector, and missing required arguments. Runtime binding failures do not run defaults
or the body; effects of supplied expressions have already happened.

A final `**options` parameter collects otherwise unmatched named arguments in
a fresh object. Its type is `walax`, also in strict mode; `walax **options` is
an equivalent explicit annotation. `walax[qoraal, abn] **options` also checks
the collected value types. It cannot have a default. A `*` collector
may precede it. Ordinary parameters cannot follow either collector.

```sop
hawl ururi(bilow=0,*tirooyin,**xulashooyin) {
    celi [bilow,tirooyin,xulashooyin]
}
qor(ururi())
qor(ururi(1,2,3,midab:"buluug"))
```

Collector names are not ordinary named parameters. For example, with the
signature above, `ururi(tirooyin: [1])` leaves `tirooyin` empty and stores the
extra key in `xulashooyin`. Arguments retain their usual shared value identity;
the collector containers themselves are fresh for each call. Call-site `*` and
`**` expansion and keyword-only ordinary parameters are not yet implemented.

Named arguments work with user functions, anonymous functions, captured and
returned closures, class methods, inherited methods and constructors, parent
calls, and object-held callables. Builtins and built-in collection/string
methods currently accept positional arguments only. Their public parameter
names are not inferred from implementation details.

Known function and constructor signatures receive static checks for names,
required arguments, collisions, and inferable types. Dynamic calls bind at
runtime and raise catchable `E022` errors for binding failures. A dynamically
provided value still undergoes the function's ordinary type checks. Callable
variables and parameters shadow same-named global function signatures.

## Types and errors

A parameter's type annotation applies to both supplied and default values.
Each parameter is prepared and type checked before the next default is
evaluated. Integer-to-float conversion therefore happens before a later
default reads that parameter. Strict mode still requires explicit parameter
and return types; a default is not a replacement for a type annotation.

```sop
hawl qiyaas(jajab qiime = 2, qoraal nooca = nooc(qiime)): qoraal {
    celi nooca
}
qor(qiyaas())
```

Default evaluation uses ordinary expression rules, including user functions,
closures, collection operations, and structured errors. A failure prevents the
remaining defaults and body from running. Effects of supplied arguments and
already evaluated defaults remain visible. Errors retain their source location
and function trace and can be caught by the caller.

Defaults work in named functions, anonymous functions, nested closures, class
methods, inherited methods, constructors, `waalid` calls, and functions used as
collection callbacks. Both Cranelift JIT and LLVM executables use the same
argument binding rules. A callback may have optional trailing parameters when
the collection supplies its required arguments.

## Collection callback signatures

List methods validate a callback before iterating, including when the list is
empty. `aaddin`, `shaandhee`, `dhammaan`, and `qaar` supply one argument;
`ururi` and its `isku_gee` alias supply two. A callback must accept that count.
Defaults, rest parameters, and closure environments are included in this check.

```sop
qor([1, 2].aaddin(hawl(qiime, kordhin = 3) { celi qiime + kordhin }))
qor([1, 2].aaddin(hawl(*qiimayaal) { celi qiimayaal }))
tijaabi {
    [].aaddin(hawl(a, b) { celi a + b })
} qabo (fariin, xog) {
    qor(xog.summad)
}
```

Invalid signatures raise `E022` at the collection method call. Validation does
not invoke the callback, evaluate its defaults, read input, print output, or
mutate its captured state. Actual argument types are checked when a call is
made. Builtins carry their argument-count contracts too, including aliases and
pairwise arguments to `walax`.

Rust host callbacks registered through `Value::NativeFunction` have no declared
parameter contract. They validate their own arguments when invoked; the runtime
does not probe them. Converting the same native function repeatedly reuses its
context-owned registration. Opaque handles from a different runtime context
remain invalid.

## Embedding ABI

Compiled entries retain `(values, supplied_count)`. Registration records names,
required slots, total slots, implicit parameters, and positional/keyword
collectors. Fixed calls with every parameter supplied can enter directly;
other calls pass through runtime binding. Names are copied into the owning
runtime context when compiled code is registered.

Binding creates a normalized vector with one slot per formal parameter,
including instance or closure environment slots. Omitted defaults use internal
marker tag `255`; it is never a language value. Prologues check presence by
that marker, so an explicit null and an omitted value remain distinct even
when named arguments leave gaps. The supplied count remains available in the
entry ABI and can differ from normalized vector size.

Rest lists and keyword objects are prepared before entry. Generated roots
cover parameter slots and default-expression temporaries before any collection
safepoint. Defaults that allocate, call compiled code, or trigger collection
use the same ownership rules as the function body. Binding errors occur before
function entry; default/body failures retain their usual trace frames.
