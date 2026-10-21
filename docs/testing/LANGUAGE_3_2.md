# Local verification for the 3.2 language foundations

These checks cover implemented stages. They do not declare the complete 3.2
milestone or its distribution ready. The remaining scope is in [ROADMAP.md](../../ROADMAP.md).

## Iteration stage

Commit `78c3e44` adds collection loops and user-defined iterator protocols.
The debug suite, all diagnostic and public examples in both execution paths,
and 80 optimized tests passed. The optimized selection covers interpreter,
LLVM, iterator ownership, and collection tests.

## Default arguments stage

Commands run locally on Linux:

```sh
cargo test --locked -- --skip test_aot
cargo test --locked --test diagnostic_tests --test examples_tests
cargo test --locked --release --test default_argument_tests --test interpreter_tests --test llvm_ir_tests --test gc_tests
```

- The debug suite passes 182 tests, excluding tests whose names contain
  `test_aot`.
- The dedicated diagnostic and example suites pass 14 tests, including
  LLVM execution of all 87 error cases and all 32 public examples.
- The optimized selection passes 77 tests, including all 27 LLVM tests.
- The default-argument LLVM fixture runs at optimization levels 0 and 2.

The shared fixture covers omitted and supplied values, explicit null and false,
left-to-right effects, fresh lists and objects, earlier parameter conversion,
lexical captures, defaults containing anonymous functions, dynamic numeric
kinds, recursion, inherited constructors and methods, parent calls, callbacks,
structured failures, traces, recovery, and collection during later defaults.
Dedicated tests cover syntax and dependency rejection, strict types, persistent
REPL calls, and editor declarations and references inside parameter defaults.

Cross-platform distribution checks remain required before publication.

## Variadic positional arguments stage

The same debug command passes 187 tests after adding rest parameters. The
full diagnostic and example suites pass 14 tests covering all 92 error cases
and 33 public examples. Optimized checks pass 70 tests:

```sh
cargo test --locked --release --test variadic_argument_tests --test interpreter_tests --test llvm_ir_tests
```

The variadic LLVM fixture runs at optimization levels 0 and 2. It checks empty,
single, and wide rest lists; required and defaulted prefixes; dynamic calls;
closures; inherited constructors and methods; parent calls; callbacks;
argument evaluation order; shared element identity; fresh list ownership;
minimum-count failures; and collection while the rest list is live. Dedicated
tests check declaration constraints, strict typing, and persistent REPL calls.

## Callback signature validation stage

The debug suite passes 190 tests. The diagnostic and example suites pass 14
tests, including all 95 error cases and 33 public examples through both JIT
and LLVM. The shared callback fixture covers every callback-taking list method,
empty and nonempty collections, spelling of compatibility aliases, invalid
user and native signatures, fixed/defaulted/variadic contracts, captured
environments, native pairwise arguments, and validation without execution.

The host test checks that unknown Rust callbacks are not probed, that repeated
conversions reuse a context registration, and that foreign handles are rejected.
Actual callback calls remain responsible for validating parameter types.

Optimized checks pass 70 tests, including all 29 LLVM tests:

```sh
cargo test --locked --release --test callback_signature_tests --test interpreter_tests --test llvm_ir_tests
```

## Named arguments and keyword collectors stage

The debug suite passes 197 tests, including all 30 LLVM tests. The dedicated
diagnostic and public-example suites pass 14 tests covering all 108 error
cases and all 34 public examples in both execution paths. Optimized checks
pass 76 tests:

```sh
cargo test --locked --release --test named_argument_tests --test interpreter_tests --test llvm_ir_tests
```

The named-call fixture runs LLVM at levels 0 and 2. It covers reordered names,
omitted middle defaults, explicit null and false, written evaluation order,
dynamic and returned callables, keyword collection, combined rest collectors,
strict types and conversions, inherited constructors and methods, parent calls,
object-held callables, callback collectors, captured anonymous arguments,
collection during defaults, binding failures, recovery, and skipped defaults
and bodies after failed runtime binding.

Static tests cover names, required parameters, collisions, types, collector
syntax, strict mode, shadowed function signatures, persistent REPL metadata,
and malformed native descriptors.

Cross-target checks pass locally:

```sh
cargo check --locked --offline --target x86_64-pc-windows-msvc
cargo check --locked --offline --target aarch64-apple-darwin
```

The extracted LLVM linker wrapper also passes Rust metadata compilation against
both target libraries. This checks references to the runtime bridges and their
signatures; platform executable execution remains a distribution gate.

## Byte arrays stage

The debug suite passes 205 tests, including all 31 LLVM tests. The dedicated
error and example suites pass 14 tests covering all 116 diagnostic programs
and 35 public examples through JIT and LLVM. Six host tests cover byte handle
identity, foreign and stale handles, nested ownership, conflicting borrows,
concrete annotations, REPL recovery, and collection cycles.

```sh
cargo test --locked -- --skip test_aot
cargo test --locked --test byte_tests --test diagnostic_tests --test examples_tests
cargo test --locked --release --test byte_tests --test interpreter_tests --test llvm_ir_tests
cargo check --locked --offline --target x86_64-pc-windows-msvc
cargo check --locked --offline --target aarch64-apple-darwin
```

The optimized selection passes 79 tests. The byte fixture runs LLVM at levels
0 and 2. It checks encoding and strict decoding, embedded zero bytes, malformed
UTF-8, range validation, copy and alias behavior, negative and extreme indexes,
atomic failed mutations, slices, method contracts, snapshot iteration,
enumerate/zip, annotations, fresh defaults, dynamic parameter and return checks,
callback signatures, and live values across collection safepoints.

Both cross-target checks pass. Executable tests on Windows and macOS remain
required for distribution. The complete 3.2 milestone remains untagged.

## Optional and union annotations stage

The debug suite passes 213 tests, including all 32 LLVM tests. The dedicated
error and example suites pass 14 tests covering all 125 diagnostic programs
and all 36 public examples in both execution paths. Optimized checks pass
81 tests:

```sh
cargo test --locked -- --skip test_aot
cargo test --locked --test diagnostic_tests --test examples_tests
cargo test --locked --release --test union_type_tests --test interpreter_tests --test llvm_ir_tests
cargo check --locked --offline --target x86_64-pc-windows-msvc
cargo check --locked --offline --target aarch64-apple-darwin
```

The shared optional/union fixture runs LLVM at levels 0 and 2. It covers null,
all current concrete type families, exact numeric matches and conversions,
class inheritance, required/default/named calls, captured closures, argument
and return checks, implicit returns, branch fallthrough, failed-write atomicity,
loop targets, dynamic arithmetic, and managed roots across collection.

Six dedicated tests cover syntax and alternative limits, conservative static
compatibility, strict annotations, editor/AST displays, persistent REPL metadata,
malformed native descriptors, foreign handles, and borrowed class metadata.
Cross-target checks and metadata compilation of the extracted LLVM linker
wrapper pass for Windows x64 and macOS ARM64, including the new runtime bridge.
Actual platform executable checks remain distribution gates.

## Shared typed lists and objects stage

The debug suite passes 221 tests, including all 33 LLVM tests. The dedicated
error and example suites pass 14 tests covering all 133 diagnostic programs
and 37 public examples through JIT and LLVM. Optimized checks pass 83 tests:

```sh
cargo test --locked -- --skip test_aot
cargo test --locked --test diagnostic_tests --test examples_tests
cargo test --locked --release --test typed_collection_tests --test interpreter_tests --test llvm_ir_tests
cargo check --locked --offline --target x86_64-pc-windows-msvc
cargo check --locked --offline --target aarch64-apple-darwin
```

The shared fixture runs LLVM at levels 0 and 2. It checks all current list and
object mutation paths, method aliases, nested aliases, replacement and removal
of dependencies, correlated whole-container unions, intersections of shared
contracts, all-parent mutation atomicity, nominal class members, typed defaults,
rest collectors, named calls, returned closures, return contracts, optional
containers, equivalent unions, cycles, and contracts across GC safepoints.

Six host tests cover syntax and depth limits, key types, invariance, strict
annotations, weak guard lifetime, repeated identity checks, direct ABI and
standard-library writes, conflicting borrows, unrelated fast paths, failed
registration, malformed native descriptors, REPL metadata, and cyclic cleanup.

Windows x64 and macOS ARM64 cross-target checks pass. The extracted LLVM linker
wrapper passes Rust metadata compilation against both target libraries. Native
platform execution and packaging remain distribution gates. Sets, tuples, their
typed forms, and the remaining library operations are still required for 3.2.

## Immutable tuples and positional contracts stage

The debug suite passes 228 tests, including all 34 LLVM tests. The dedicated
diagnostic and example suites pass 14 tests covering 141 error programs and
38 public examples through JIT and LLVM. Optimized checks pass 84 tests:

```sh
cargo test --locked -- --skip test_aot
cargo test --locked --test diagnostic_tests --test examples_tests
cargo test --locked --release --test tuple_tests --test interpreter_tests --test llvm_ir_tests
cargo check --locked --offline --target x86_64-pc-windows-msvc
cargo check --locked --offline --target aarch64-apple-darwin
```

The shared tuple fixture runs LLVM at optimization levels 0 and 2. It covers
construction, immutable writes, indexing, slicing, conversions, snapshot
iteration, row bindings, positional annotations, optional and union contracts,
defaults, named calls, closures, class members, nested mutable aliases, cycles,
and managed roots across collection.

Five host tests cover contextual syntax, invariant shapes, handle ownership,
stale handles, weak contract lifetime, REPL persistence, and cyclic cleanup.
A small-stack test compares, displays, and collects 12,000-level tuple chains.

Windows x64 and macOS ARM64 cross-target checks pass. The extracted LLVM linker
wrapper also passes Rust metadata compilation against both target libraries.
Native platform execution and packaging remain distribution gates. Sets,
typed sets, and the remaining library operations are still required for 3.2.

## Ordered hash sets stage

The debug suite passes 235 tests, including all 35 LLVM tests. The diagnostic
and example suites pass 14 tests covering 146 error programs and 39 public
examples through JIT and LLVM. Optimized checks pass 85 tests:

```sh
cargo test --locked -- --skip test_aot
cargo test --locked --test diagnostic_tests --test examples_tests
cargo test --locked --release --test set_tests --test interpreter_tests --test llvm_ir_tests
cargo check --locked --offline --target x86_64-pc-windows-msvc
cargo check --locked --offline --target aarch64-apple-darwin
```

The shared set fixture runs LLVM at levels 0 and 2. It checks insertion order,
duplicates, shared mutation, independent copies, set algebra, strict numeric
keys, tuple keys, signed zero, invalid mutable keys, failed writes, conversion,
cursor consumption, snapshot iteration, row bindings, defaults, named calls,
bare and optional annotations, nested contracts, and live roots across GC.

Four host tests cover handle deduplication and ownership, stale handles,
snapshot retention of tuple keys, immutable key validation, NaN and infinity,
contextual names, and rejection of overly deep tuple keys on a small stack.
A unit test compares 20,000 mixed insert/remove operations with an ordered
reference model and verifies bounded tombstones and indexed membership.

Windows x64 and macOS ARM64 cross-target checks pass. The extracted LLVM linker
wrapper passes Rust metadata compilation against both target libraries.
Native execution and packaging remain distribution gates. Typed sets and the
remaining library operations are still required before the 3.2 milestone tag.

## Shared typed sets stage

The debug suite passes 243 tests, including all 36 LLVM tests. The diagnostic
and example suites pass 14 tests covering 153 error programs and 40 public
examples through JIT and LLVM. Optimized checks pass 88 tests:

```sh
cargo test --locked -- --skip test_aot
cargo test --locked --test diagnostic_tests --test examples_tests
cargo test --locked --release --test typed_set_tests --test interpreter_tests --test llvm_ir_tests
cargo check --locked --offline --target x86_64-pc-windows-msvc
cargo check --locked --offline --target aarch64-apple-darwin
```

The shared fixture runs LLVM at levels 0 and 2. It covers exact element kinds,
tuple shapes, null keys, equivalent unions, optional sets, shared aliases,
constraints from multiple parents, dependency replacement, correlated whole-root
unions, atomic failures, independent copies and algebra results, named calls,
defaults, returns, closures, constructors, typed rest collectors, constant
bindings, snapshot cursors, and guards across collection safepoints.

Five host tests cover annotation limits, impossible key types, invariance,
contextual identifiers, invalid native descriptors, failed registration,
handle ownership, weak guard lifetime, REPL persistence, and cyclic cleanup.
A unit test checks conflicting set-state borrows and the unrelated fast path.

Windows x64 and macOS ARM64 cross-target checks pass. The extracted LLVM linker
wrapper passes Rust metadata compilation against both target libraries.
Native platform execution and packaging remain distribution gates. Expanded
string, list, object, and numeric library operations remain before the 3.2 tag.

## Unicode text-library stage

The debug suite passes 248 tests, including all 37 LLVM tests. The diagnostic
and example suites pass 14 tests covering 159 error programs and 41 public
examples through JIT and LLVM. Optimized checks pass 99 tests:

```sh
cargo test --locked -- --skip test_aot
cargo test --locked --test diagnostic_tests --test examples_tests
cargo test --locked --release --lib --test interpreter_tests --test llvm_ir_tests
cargo check --locked --offline --target x86_64-pc-windows-msvc
cargo check --locked --offline --target aarch64-apple-darwin
```

The shared fixture runs LLVM at levels 0 and 2. It covers Unicode classification,
capitalization, titlecase expansion and digraphs, contextual final sigma,
apostrophe word boundaries, scalar-width padding, partitioning, whitespace
tokenization, Unicode line endings, invalid arguments, typed results, and roots
across collection. Three unit tests cover casing byte boundaries for every
cased Unicode scalar, selected titlecase conformance cases, padding limits and
multibyte fills, and exact line-ending reconstruction.

The generated Unicode table reproduces byte-for-byte from the official Unicode
17 input files. Python scripts and the Linux packaging shell script pass syntax
checks. A portable release package contains the Unicode notice and executes the
new public example through JIT and a standalone LLVM-built binary.

Windows x64 and macOS ARM64 cross-target checks pass. The extracted LLVM linker
wrapper passes Rust metadata compilation against both target libraries.
Native platform execution, installer builds, and the Docker build remain
distribution gates. List, object, and numeric library work remains before 3.2.
