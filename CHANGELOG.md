# CHANGELOG

All notable changes to this project will be documented in this file.

---

## [Unreleased]

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
