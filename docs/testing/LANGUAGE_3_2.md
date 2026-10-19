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
