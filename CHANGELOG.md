# CHANGELOG

All notable changes to this project will be documented in this file.

---

## [Unreleased]

### Added

* [3.2.0 implementation milestone notes](docs/releases/3.2.0.md) record completed language foundations. A version tag and distribution builds remain pending.

* Text file builtins `akhrifayl`, `qor_fayl`, and `kudar_fayl` read, overwrite, and append UTF-8 text using paths relative to the working directory. I/O failures remain catchable with Somali messages.

* Filesystem builtins add `fayl_jiraa`, `fayl_xog`, `isku_xidh`, `baabi_jid`, `nuqul_fayl`, `wareeji_fayl`, `tirtir_fayl`, `abuur_gal`, `tir_gal`, and `tirtir_gal` for metadata, paths, files, and directories.

* Binary file builtins `akhrif_baytyo`, `qor_baytyo`, and `kudar_baytyo` read and write whole files as `baytyo` values without UTF-8 conversion.


* Numeric builtins add logarithms, radian trigonometry with optional inverse mode, `atan2`-style angles, named constants, clamping, and context-local seeded random streams. Integer-only extrema keep exact 64-bit results; invalid random calls do not advance the generator.

* Object methods add insert-if-absent, atomic in-place merging, sorted-snapshot value and entry mapping, entry filtering, and explicit nested paths. Mapping can rename keys through checked pair results; path lookup preserves mutable aliases and supports existing indexed value kinds.

* List methods add explicit removal by value, atomic in-place iterable extension, stable generic uniqueness, numeric sums, enumerate/zip adapters, and stable callback-key sorting. Sorting preserves large-integer precision and rejects changed receiver slots without discarding callback writes.

* Unicode text methods add classification, capitalization, title casing, scalar-width padding, tuple partitioning, Unicode line splitting, and default whitespace splitting. Unicode 17 titlecase mappings and word boundaries work through both compiler backends.

* Typed sets (`urur[T]`) enforce immutable element contracts through aliases, nested collections, parameters, defaults, returns, and typed rest collectors. Every mutation validates all affected parent contracts before changing membership or insertion order.

* Insertion-ordered `urur` hash sets support unique immutable keys, set algebra, conversion, snapshot iteration, and bare type annotations through both compiler backends. Tuple keys participate in context ownership and collection.

* Immutable `xirmo` tuples support indexing, slicing, conversion, shared collection methods, iteration, row bindings, and positional annotations. Nested mutable members retain shared contracts. Tuple handles participate in ownership checks and tracing collection.
* Typed lists (`teed[T]`) and string-keyed objects (`walax[qoraal, T]`) enforce shared element contracts through aliases, nested containers, parameters, returns, and typed rest collectors. Mutation validates all affected parents before publishing data. Weak guards participate in context collection and use a dependency index.
* Optional (`T?`), union (`A | B`), and null-only (`maran`) annotations constrain variables, constants, parameters, and returns. Both backends preserve exact numeric alternatives, apply existing numeric conversions when needed, and report recoverable Somali contract errors.
* Compact `baytyo` byte arrays support checked mutation, byte indexing and slicing, snapshot iteration, strict UTF-8 encoding/decoding, shared collection methods, and concrete type annotations. Both backends retain byte storage through tracing roots; stale handles and conflicting host borrows produce recoverable errors.
* Named arguments (`name: value`) support reordered values and omitted defaults in user functions, closures, methods, constructors, and parent calls. `**options` collects extra named arguments into a fresh object. Known signatures receive static checks; dynamic binding errors remain catchable.
* Collection callbacks are checked before iteration, including empty lists. User functions, closures, and builtins expose fixed, optional, variadic, or pairwise argument contracts without running the callback to inspect it.
* Variadic positional parameters (`*items`) collect surplus arguments into a fresh list. Required and defaulted parameters can precede them; methods, closures, constructors, and callbacks share the same behavior.
* User functions, closures, methods, constructors, and callbacks accept default parameters evaluated at call time. Defaults can refer to earlier parameters and lexical captures; mutable literals are fresh on each call, and both backends preserve type checks, errors, and collection roots.
* Collection loops use `kuceli (item ku source)` and exact-length row bindings. Lists use shallow snapshots, strings yield Unicode scalars, and objects yield sorted keys. `wareeg`/`xiga`, lazy `taxane`, `tirso`, and `lamaane` support shared cursors and user-defined class/object protocols in both backends.
* Iterator state participates in context ownership and tracing collection, including cyclic snapshots and native roots held across callbacks. Invalid protocols, reentrant advances, excessive native nesting, and conflicting snapshot borrows produce recoverable Somali errors.

### Fixed

* Object method dispatch reports conflicting native borrows as recoverable errors before probing class metadata.

* List sorting uses consistent type ordering, preserves large integer precision, and hides opaque callable handles during comparisons. Collection validation after callbacks reports the outer call site while callback failures retain their inner locations and traces.

* Structural collection equality uses an iterative graph walk, and display bounds active nesting to 64 containers. Unreachable tuple edges are cleared during collection so deep immutable chains and mixed cycles can be reclaimed without recursive destruction.
* Functions that fall through validate their implicit null return against their annotation, closing a gap for primitive and class return types.
* Callable parameters and variables shadow global function signatures during static call validation and return-type inference. Windows LLVM wrappers retain the argument-signature registration bridges.
* Native Rust callbacks receive reusable context-owned handles when converted to Soplang values.
* LLVM registers empty classes and subclasses, preserving inherited constructors and iterator methods even when the subclass declares no methods.

## [3.1.0]

Implementation milestone; distribution and publication are pending.

### Added

* `qor()` accepts zero or more values, separates them with spaces, and prints a newline. It returns the printed text without the newline. File execution, the REPL, and LLVM builds support the same calls, including more than eight built-in arguments.

* Owned runtime contexts isolate JIT programs and REPL sessions, including their globals, classes, function registrations, source locations, and pending errors. Rust integrations can inspect globals through their owning session or backend.
* Closure environments are released with their runtime context. Dropping a JIT backend also frees its generated code and data allocations.
* Runtime strings, collections, functions, and closures use opaque allocation handles. Foreign, expired, and malformed handles are rejected with catchable Somali errors instead of indexing another allocation or dereferencing a closure address.
* Recoverable execution boundaries return uncaught runtime errors to Rust callers and the REPL. Failed expressions stop before later arguments or side effects run, while standalone programs retain Somali diagnostics and unsuccessful exit status.
* Contextual `tuur` raises error messages. `dhammee` guarantees cleanup after protected blocks, handlers, returns, and loop exits in both compiler backends.
* Immutable `khalad` values retain error codes, causes, source locations, and compiled function traces. Optional `qabo (fariin, xog)` exposes these details while existing catches keep receiving text. Cleanup failures preserve the original error as their cause.
* Compiled functions, closures, constructors, and methods accept more than eight arguments through a shared argument-vector ABI. Returned and indexed functions can be invoked directly, including callable object fields. LLVM reuses argument buffers inside loops.

* Tracing collection reclaims unreachable strings, errors, collections, instances, and closure environments, including cycles. Both backends publish explicit stack roots. Rust integrations can pin tagged values and inspect collection reports; repeated collection conversion preserves identity.
* Explicit `e` string prefixes decode control, quote, and Unicode escapes; `r` selects raw text. Triple quotes support multiline content. Existing unprefixed strings preserve literal backslashes. Editor ranges and diagnostics retain original source bytes.
* `qaabee(value, format?)` formats values with alignment, padding, integer bases, and floating-point precision. String `qaabee(...)` fills automatic, indexed, or named template slots and reports invalid formats as catchable Somali errors.
* Default float display preserves large values and negative zero instead of narrowing whole-valued floats to 64-bit integers.

* `f` string literals interpolate expressions with optional format descriptors. Nested literals, escaped braces, multiline content, closures, and original source locations work in both compiler backends. Fields evaluate once in order, and failures stop later fields. Editor indexing resolves references inside interpolation.

* Integer arithmetic wraps consistently in native and dynamic execution, including minimum-value negation and remainder. Floating remainder preserves extreme magnitudes and signed zero. Numeric ranges retain exact integer bounds, stop before overflow, and report non-advancing floating-point steps.

* JIT backends own immutable copies of HIR strings and source spans referenced by generated code. Embedding callers can discard compiler input safely; REPL snippets transfer to the same owner without duplicate retention.

## [3.0.1] - 2026-10-18

### Fixed

* Translate CLI help, generated argument errors, REPL descriptions, and HIR section headings into Somali.
* Explain file, input, JIT, LLVM, and toolchain failures in Somali. Retain original external errors behind `SOPLANG_ERROR_DETAILS=1` without changing caught-error messages.
* Handle REPL startup and program output failures through diagnostics, preserve source carets, and omit nonexistent line and column numbers for infrastructure failures.
* Verify localized help and package failures, and select release notes for the version being packaged.

### Added

* **Python/v2/v3 benchmark comparison.** Ten equivalent workloads exercise the v2 interpreter, v3 Cranelift JIT, v3 LLVM executables, and Python. Validated runs record medians, sample spread, raw observations, source/toolchain metadata, and separate LLVM build costs.

### Changed

* **Current performance and release documentation.** README summaries use measured comparison results. Internal Criterion reports have a separate destination. Installation, native-build, architecture, and verification guides reflect the published v3.0.0 packages and completed platform checks.

## [3.0.0] - 2026-10-17

### Added

* **Relocatable compiler packages.** Linux archives/Debian packages, Windows archives/installers, macOS bundles, and Docker include matching runtime libraries and Rust toolchain metadata. Package checks run every public example and diagnostic outside the source checkout.
* **Release verification gates.** Native Linux, Windows, macOS, and container checks gate release drafts. Builders generate SHA-256 checksums for real assets; prereleases preserve stable Docker tags and skip stable AUR publication.

* **Somali diagnostics with source locations.** Lexer, parser, semantic, and runtime failures retain original files, token columns, source lines, and caret spans through Cranelift and LLVM, including imports. Carets account for tabs, accents, combining marks, emoji, and wide characters.
* **Complete example checks.** All 28 numbered programs have expected output; 63 intentional failures have diagnostic snapshots and expected locations. Both compiler paths check exit statuses and output, with dedicated strict-mode, command-expression, and REPL coverage.

* **Collection symmetry.** Strings support `hore()` and `dambe()` for first/last Unicode characters; lists support `tiri(value)` for occurrence counting. Both lists and strings accept zero, one, or two integer slice bounds.

* **Explicit collection combination.** `geli(value)` appends one value, including nested lists; `midee` combines lists or objects into a new shallow collection; separator strings use `xiriiri` to join values. Legacy `kudar` behavior remains supported.
* **Simpler standard library vocabulary.** `ururi`, `weynee`, `yaree`, string `kuceli`, numeric `duub`, and object `furayaal`, `qiimayaal`, and `lammaanayaal` are available alongside earlier spellings. Trim and search accept `"hore"`/`"dambe"`; replacement accepts a limit or `maran` for all matches.

* **Short paired names.** Lists support `hore()` and `dambe()`, numeric selection supports `yar()` and `weyn()`, and contextual `kale` handles conditional branches and switch defaults. Previous spellings remain supported.

* **Shared collection operations.** Lists and strings support `hel(index, default?)`, lists support `dhig(index, value)`, objects support `dherer()`, and lists and objects share `saar` removal methods. Existing APIs remain supported.
* **Canonical vocabulary.** `dhis`, contextual `tijaabi`, `jibbaar`, list `raadi`, string `dhammaad`, and `masax_dhammaad` are available alongside their earlier spellings. Constructor aliases work across inheritance and parent calls.

* **Cranelift pipeline.** `soplang file.sop` lexes, parses, resolves `keen`, analyzes, lowers to HIR, and JIT-compiles with Cranelift. The tree-walking interpreter is not in this repo.
* **REPL session.** The shell keeps one Cranelift context, so functions, classes, and globals survive across lines.
* **Standalone LLVM builds.** `soplang --build file.sop` resolves `keen`, lowers the program to textual LLVM IR, and links a native executable with `clang` and `rustc`.
* **Native `abn` and `jajab`.** Static integer and float slots emit native arithmetic. `door`, strings, and other dynamic values stay on the tagged runtime. Division and modulo by zero still use that runtime so `fasax` can catch them.
* **Classes and closures.** Inheritance, `waalid`, class-typed variables, captured environments, and anonymous callback functions work in both compiler backends.
* **V3 collection APIs.** Lists support indexed insertion and removal, first/last access, reduce, every, and some. Objects support computed keys, safe lookup, assignment, and removal. Strings include length, directional trim, occurrence count, repetition, and last-index lookup.
* **V3 numeric APIs.** Numeric strings can be converted with `abn` and `jajab`; rounding, absolute value, minimum, maximum, power, and square root are available as built-ins.
* **Tests and benchmarks.** Unit tests, integration tests (examples vs `.expected`), and Criterion benchmarks. Makefile targets: `build`, `run`, `shell`, `test`, `bench`.

### Changed

* **Consistent slicing.** List and string `jar` bounds must be integers, support negative offsets, and clamp to the receiver. Equal or reversed bounds return an empty result. Previously truncated fractional bounds now report catchable errors.

* **Clearer terminology.** Runtime and strict-mode diagnostics use consistent Somali explanations. The language reference distinguishes integer conversion, binary floating point, range materialization, and collection copy and lookup behavior.

* **Two compiler backends.** Normal execution uses the Cranelift JIT, while `--build` uses the LLVM AOT backend. Compiler phases 1–6 are complete.
* **Soplang 3 vocabulary.** The language reference and examples use the supported keyword spellings, including `qor`, `gelin`, `abn`, `teed`, `xul`, `muuji`, `tir`, and `maran`.
* **Predictable evaluation.** Logical operators short circuit, strings index by Unicode characters, negative indexes have consistent bounds, and callable argument counts are enforced at runtime.
* **Optional semicolons.** Statements may use semicolons consistently while newline-oriented source remains valid.

### Fixed

* `gelin` removes LF and CRLF line endings while preserving surrounding whitespace and EOF input. Both backends check Windows-style input explicitly.

* Native executables publish through a same-volume rename, avoiding writable-handle races in parallel execution. Windows installation preserves PATH expansion, and the command launcher forwards arguments and exit status.

* Windows linker sidecars stay in the build scratch directory; only the finished executable is copied to the requested output path. Native release checks also install, execute, and remove Linux and Windows packages.

* JIT function references use far calls so macOS ARM64 allocations can exceed the direct branch range. Windows builds retain native import-library paths and packages bundle those libraries.
* Windows x64 generated calls use System V bridges to the platform C runtime, including callbacks and explicit two-word return values.
* Release uploads combine verified platform checksums into one manifest and skip optional AUR publication when credentials are absent.

* Installed `--build` locates its bundled runtime instead of searching the build machine's Cargo directory and reports mismatched Rust toolchains before linking.
* Concurrent LLVM builds with identical output filenames use isolated temporary directories and clean up on failures.
* Docker includes Cargo manifests, clang 16, the linker, and the matching Rust toolchain. Registry metadata separates image names from the optional Docker Hub output.

* Canonical API argument errors report the spelling used at the call site; large line numbers preserve caret alignment.
* Unclosed strings and comments point to the opening delimiter. Invalid class contents point to the offending statement. EOF diagnostics retain empty final source lines.
* LLVM builds resolve imports as source-located ASTs, preserving imported-file errors instead of reporting positions in a concatenated source buffer.
* CLI and REPL expression echo wraps the AST, preserving the original source columns.
* Unknown variable reads produce a Somali error instead of silently returning `maran`.

* Global updates from functions and callbacks reach top-level reads and calls, including variables that shadow built-in names. Numeric loop indexes are shared with functions, and REPL snippets observe the current values.
* Function writes to typed globals enforce type and class checks. Failed assignments preserve previous global values, and accepted numeric values use their annotated runtime representation.
* Reversed string slices no longer panic.

* Collection callback errors stop iteration and propagate unchanged, including from map and filter. Iteration uses a shallow snapshot so callbacks can safely mutate the source list; invalid callbacks are rejected on empty lists.

* Constructor argument counts and types are validated, including inherited constructors.
* `baaxad()` rejects fractional bounds and a zero step instead of silently truncating or returning an empty list.
* Invalid list writes and index types now produce catchable errors.

---

## [v2.0.0] - 2025-05-26

### Added

* Added random selection function `kudhow()`.
* Added `dherer()` to get length of lists, strings, and objects.
* Added `jar()` for slicing strings and lists.
* Added `kudar()` for joining string lists and merging values.
* Added `beddel()` for string replacement.
* Added `bilow()` to check string prefix.
* Added `dhamaad()` to check string suffix.
* Added `leeyahay()` to check substring existence.
* Added `qeybi()` to split strings by delimiter.
* Added `kor()` and `daji()` for number rounding.
* Added `lamaane()` to return key-value pairs from walax.
* Added `walax.qiime()` to get object values.
* Added `walax.nadiifi()` to clear object keys.
* Added `walax.nuqul()` for shallow object copy.
* Added list `nadiifi()` and `nuqul()` methods.
* Added `raadso()` and negative list indexing support.
* Added `aaddin()` for list transformations.
* Added `shaandhee()` for filtering lists.
* Added `habee()` to sort lists in place.
* Added `rog()` to reverse lists in place.
* Added `dooro (x)` / `xaalad` switch-case syntax.
* Added `madoor` keyword for constants with type validation.
* Added improved Somali error messages with line and position info.
* Added support for return statements with and without parentheses.
* Added Docker automation for builds and releases.
* Added automated CI release workflows.
* Added Windows, macOS, and Linux build support.
* Added interactive shell improvements.
* Added full unit test runners for lexer, parser, and interpreter.
* Added comprehensive installation and testing documentation.

### Fixed

* Fixed comparison and logical operator handling.
* Fixed parser issues with method calls and unary operators.
* Fixed negative number parsing.
* Fixed token inconsistencies in lexer and parser.
* Fixed multiple Windows build and launch issues.
* Fixed Docker workflow and authentication issues.
* Fixed changelog formatting and duplicate sections.
* Fixed error message formatting and translation placeholders.
* Fixed REPL and shell behavior issues.
* Fixed list concatenation edge cases.
* Fixed boolean type handling.
* Fixed file extension handling and version naming.
* Fixed tests to match actual implementation behavior.

### Changed

* Renamed `qor` to `bandhig`.
* Renamed many legacy keywords for consistency and clarity.
* Standardized keywords to snake_case format.
* Replaced loop `by` syntax with `::`.
* Updated boolean literals to `been` and `run`.
* Changed primary file extension from `.so` to `.sop`.
* Centralized version management system.
* Simplified output formatting in interpreter.
* Reorganized codebase into modular structure.
* Migrated interpreter implementation to C.
* Improved grammar consistency and naming standards across docs and source.

### Documentation

* Updated grammar definitions and keyword references.
* Updated examples to match new syntax and renamed keywords.
* Improved operator expression and switch-case documentation.
* Updated error message documentation.
* Reorganized documentation into structured directories.
* Simplified and modernized README.

### Build

* Improved CI workflows for releases.
* Added automated binary builds.
* Added platform-specific packaging support.
* Improved Windows installer configuration.

### Other

* Removed outdated grammar and legacy files.
* Removed unused and deprecated keywords.
* Cleaned up root directory and legacy scripts.
* Reorganized project structure.
* Updated publisher information.
* Added Code of Conduct and contribution guidelines.
* Improved development configuration and tooling.

---

## [v1.0.0] - 2023-10-23

### Added

* Added full README documentation.
* Added Soplang language sample.
* Added Soplang grammar definition.
* Added MIT License.
* Added Soplang language core files.

### Improved

* Improved and added `return` statement support.

### Completed

* Completed the initial interpreter for Soplang.
