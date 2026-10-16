# Soplang Testing Methodology

Tests check what a Soplang program prints and which errors the compiler reports. They live in `tests/` and are Rust integration tests run with Cargo.

## What is covered

| Suite | File | What it checks |
|-------|------|----------------|
| Lexer | `tests/lexer_tests.rs` | Tokens and lexical errors |
| Parser | `tests/parser_tests.rs` | Statements and expressions become the expected AST |
| Semantic | `tests/semantic_tests.rs` | Names, classes, `waalid`, and class-typed variables |
| HIR | `tests/hir_tests.rs` | Example programs lower without error |
| Runtime | `tests/runtime_tests.rs` | Tagged values, arithmetic, lists, objects, builtins |
| Programs | `tests/interpreter_tests.rs` | The `soplang` binary runs short programs (`-c` or a file) and stdout matches |
| Diagnostics | `tests/diagnostic_tests.rs` | Somali errors, original files, exact source positions and caret spacing in JIT/AOT |
| Examples | `tests/examples_tests.rs` | All numbered examples match exit status, stderr, and expected stdout in JIT and AOT |
| AOT | `tests/aot_tests.rs` | `soplang --build` produces a binary that prints the same output |

`tests/interpreter_tests.rs` is an integration suite. It spawns the compiler binary. It does not walk an AST.

Benchmarks live under `benchmarks/` and run with `cargo bench`. They time the Cranelift JIT, not a second implementation.

## Running

```bash
cargo test --locked
cargo test --locked -- --skip test_aot
cargo test --test semantic_tests
cargo test --test interpreter_tests test_nested_function_keeps_outer_local
cargo bench
```

AOT tests invoke `cargo build` for a generated runner, so the full `cargo test` is slower than the other suites.

`make test` and `make bench` call the same Cargo commands.

## When a feature changes

1. Write a short program that shows the behavior, as an example or as a `run_program` test.
2. Cover the failure case in `tests/semantic_tests.rs` when the compiler should reject the program.
3. Run `cargo test --locked -- --skip test_aot` while iterating. Run `cargo test` before a release if `--build` is involved.
4. Update a `.expected` file only when the new stdout is the intended result.

There is no separate Python or C implementation to keep in sync.
