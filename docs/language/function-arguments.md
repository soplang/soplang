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
before any defaults. Calls may omit only a trailing sequence of parameters.
Named and variadic arguments are still planned.

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

## Embedding ABI

Compiled functions retain the vector entry `(values, supplied_count)`. Runtime
registration records total slots and the required prefix, including implicit
instance or closure-environment parameters. Before invoking an entry, the
runtime checks that the count is in range and pads omitted slots with null.
The count remains the original supplied count. The generated prologue tests
parameter presence, evaluates defaults, and checks types before running the
body. A null in a supplied slot is therefore distinct from an omitted slot.

Generated roots cover parameter slots and default-expression temporaries before
any collection safepoint. Defaults that allocate, call back into compiled code,
or trigger collection use the same ownership rules as the function body.
