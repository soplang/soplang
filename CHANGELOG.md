# CHANGELOG

All notable changes to this project will be documented in this file.

---

## [Unreleased]

### Added

* **Cranelift pipeline.** `soplang file.sop` lexes, parses, resolves `keen`, analyzes, lowers to HIR, and JIT-compiles with Cranelift. The tree-walking interpreter is not in this repo.
* **REPL session.** The shell keeps one Cranelift context, so functions, classes, and globals survive across lines.
* **Standalone `--build`.** `soplang --build file.sop` inlines `keen` imports and embeds that program in a Cargo runner. The binary JIT-compiles at process start. Direct LLVM IR is not wired up yet.
* **Native `abn` and `jajab`.** Static integer and float slots emit native arithmetic. `door`, strings, and other dynamic values stay on the tagged runtime. Division and modulo by zero still use that runtime so `fasax` can catch them.
* **Tests and benchmarks.** Unit tests, integration tests (examples vs `.expected`), and Criterion benchmarks. Makefile targets: `build`, `run`, `shell`, `test`, `bench`.

### Changed

* **Runtime is the Cranelift JIT.** Run via `cargo build --release` and `./target/release/soplang`.
* README and [COMPILER_PLAN.md](COMPILER_PLAN.md) describe the current pipeline. Phases 1–4 and 6 are done. Phase 5 (real LLVM) is still ahead.
* **Soplang 2.0 vocabulary.** The language reference and examples now use the final keyword spellings, including `qor`, `gelin`, `abn`, `teed`, `xul`, `muuji`, `tir`, and `maran`.

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
