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
Named arguments are still planned.

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
a fresh list. Required and defaulted parameters may precede it. No parameters
may follow it, and it cannot have a default of its own.

```sop
hawl wadarta(bilow = 0, *tirooyin) {
    kuceli (tiro ku tirooyin) { bilow = bilow + tiro }
    celi bilow
}
qor(wadarta())
qor(wadarta(10, 1, 2, 3))
```

`*tirooyin` is always a `teed`, including in strict mode. The equivalent explicit
annotation is `teed *tirooyin`; scalar annotations such as `abn *tirooyin` are
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

Compiled functions retain the vector entry `(values, supplied_count)`. Runtime
registration records total slots and the required prefix, including implicit
instance or closure-environment parameters. Before invoking an entry, the
runtime checks that the count is in range and pads omitted slots with null.
For a variadic function, the runtime instead builds the final slot as a list
of surplus arguments and initializes any omitted fixed slots. The count remains
the original supplied count, which can differ from the normalized vector size. The generated prologue tests
parameter presence, evaluates defaults, and checks types before running the
body. A null in a supplied slot is therefore distinct from an omitted slot.

Generated roots cover parameter slots and default-expression temporaries before
any collection safepoint. Defaults that allocate, call back into compiled code,
or trigger collection use the same ownership rules as the function body.
