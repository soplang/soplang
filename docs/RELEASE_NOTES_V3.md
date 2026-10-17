# Soplang 3.0.0

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
[language reference](https://github.com/soplang/soplang/blob/v3.0.0/docs/language/keywords.md) gives signatures, return values,
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
See the [error examples](https://github.com/soplang/soplang/blob/v3.0.0/examples/errors/README.md).

All 28 numbered examples have expected output. The diagnostic corpus checks 63
intentional failures with the same snapshots in Cranelift and LLVM. Native
builds and stdin/strict-mode behavior are included in the full test suite.

## Packages and installation

Release assets cover Linux x86_64, Windows x64, and macOS Apple Silicon. Keep the
whole extracted package together: native builds use the bundled runtime and
dependency libraries. A combined `SHA256SUMS` verifies every attached package.
See [installation requirements](https://github.com/soplang/soplang/blob/v3.0.0/docs/INSTALLING_PACKAGES.md).

Linux Debian packages and the Windows installer are installed, exercised through
JIT and LLVM, and removed in native release CI. Windows checks cover PATH and
file associations. Native jobs run the complete debug and release suites, with
102 tests per profile; the container runs the full installed-package checker.
The draft is created only after all release gates pass.

The Windows installer is unsigned, and the macOS archive is not notarized.

## Migration and current limits

See [Migrating to Soplang 3](https://github.com/soplang/soplang/blob/v3.0.0/docs/MIGRATION_V3.md) for compatibility names, stricter
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
