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
