# Soplang Testing Documentation

These pages describe how to test the Soplang compiler and how to check Soplang programs.

## Overview

Tests check user-visible behavior of the Rust compiler:

1. **Example programs** in `examples/*.sop` show language features. A file with a matching `.expected` is compared against stdout.
2. **Unit tests** cover the lexer, parser, semantic analysis, HIR, and runtime.
3. **Integration tests** run the `soplang` binary on short programs and on example files.
4. **Benchmarks** measure the Cranelift JIT. See [benchmarks/README.md](../../benchmarks/README.md).

## Pages

- [Test methodology](TEST_METHODOLOGY.md) — what the suites cover and how to run them
- [Example programs](EXAMPLES.md) — the programs under `examples/`
- [Debugging](DEBUGGING.md) — errors in Soplang programs, and `--ast` / `--hir` for the compiler

## Quick start

From the repository root:

```bash
# One program
cargo build --release
./target/release/soplang examples/01_hello.sop

# Tests. AOT tests compile a small Cargo project and are slower.
cargo test

# Skip the AOT tests
cargo test -- --skip test_aot

# One integration test
cargo test --test interpreter_tests test_hello_world

# Print the parse tree, or the flat IR, instead of running
./target/release/soplang --ast examples/08_functions.sop
./target/release/soplang --hir examples/08_functions.sop
```

`make test` is `cargo test`. `make bench` is `cargo bench`.

## Contributing

1. Add or extend an example when a language feature changes.
2. Add a Rust test for the behavior, not only a comment.
3. Run `cargo test` before sending a change.
4. If an example has a `.expected` file, update that file when the printed output changes on purpose.
