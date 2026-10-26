# Example programs

[examples/README.md](../../examples/README.md) lists the numbered programs.
Each has expected stdout and runs through both Cranelift JIT and LLVM AOT.
Tests require a successful exit and empty stderr as well as matching output.

[examples/errors/README.md](../../examples/errors/README.md) describes the
intentional failures and their checked diagnostic snapshots. Coverage includes
Somali messages, original filenames, exact lines/columns, and Unicode/tab caret
alignment. Helpers are kept outside the numbered example list.

```bash
cargo test --locked --test examples_tests
cargo test --locked --test diagnostic_tests
cargo test --locked
```

The input example receives deterministic stdin. The random example checks range
and membership invariants instead of printing an unstable random number. The
strict example is run with `--strict`. Native tests require `clang` and `rustc`.

`--example N` selects the Nth numbered `.sop` after sorting. Library helpers
beginning with `_` and the error directory are excluded.
