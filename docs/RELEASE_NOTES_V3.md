# Soplang 3.0 release notes — draft

Soplang 3.0 establishes the Rust compiler as the supported implementation of the
Somali programming language. Programs run through Cranelift JIT; `--build`
produces standalone executables from textual LLVM IR using `clang` and `rustc`.

## Language and library

- Dynamic variables, constants, typed primitives, and class-typed variables.
- Functions, anonymous callbacks, captured environments, and recursion.
- Classes, inheritance, constructors, and parent calls with `waalid`.
- Conditionals, switches, numeric ranges, descending loops, break, continue,
  optional semicolons, and short-circuit logic.
- Lists, objects, Unicode strings, computed keys, safe lookup defaults, slicing,
  insertion, removal, counting, transformation, filtering, and folding.
- Numeric conversion, rounding, extrema, absolute value, powers, and square root.
- Catchable runtime failures, file imports, and a persistent REPL.

## Canonical vocabulary

New code can use `dhis`, `tijaabi`, `kale`, `jibbaar`, `yar`/`weyn`, `duub`,
`hore`/`dambe`, `saar`, `midee`, `xiriiri`, `ururi`, `weynee`/`yaree`, string
`kuceli`, and plural object methods. Earlier spellings remain supported. The
[language reference](language/keywords.md) gives signatures, return values,
mutation rules, and compatibility names.

## Correctness and diagnostics

Global updates completed by functions and callbacks reach top-level reads,
including callable replacements. Typed writes validate and normalize numeric
representations before native arithmetic. Callback failures propagate and stop
iteration; callbacks can mutate their source collection safely.

List and string slicing share integer bounds, negative offsets, clamping, and
empty results for reversed bounds. Fractional slice bounds now report errors.
String indexes and endpoint methods count Unicode characters.

Somali diagnostics show the original file, source line, token column, and
Unicode-aware caret in both compiler paths, including imported code. Native
executables retain relevant source lines when the original files are missing.
See the [error examples](../examples/errors/README.md).

All 28 numbered examples have expected output. The diagnostic corpus checks 63
intentional failures with the same snapshots in Cranelift and LLVM. Native
builds and stdin/strict-mode behavior are included in the full test suite.

## Migration and current limits

See [Migrating to Soplang 3](MIGRATION_V3.md) for compatibility names, stricter
argument/index validation, search-miss conventions, and intentional behavior
corrections.

- `--build` requires clang with opaque pointer support and the exact Rust
  compiler recorded in the package. Packages bundle the matching runtime and
  dependency libraries; Docker also includes the build toolchain. JIT and REPL
  execution need neither Rust nor clang.
- Runtime memory reclamation, compiled stack traces, structured user-raised
  errors, and guaranteed cleanup are planned for 3.1.
- More argument forms, iterators, and data types are planned for 3.2.
- General file I/O and networking are planned for 3.3. `keen` currently imports
  source files; it is not a file-I/O API.
- The compiled call ABI currently allows eight values, including implicit
  instance/environment arguments where applicable.

All planned releases remain on the 3.x line. There is no approved version 4.
Windows, macOS, container, and installed-package results must be verified before
this draft becomes a published release announcement.
