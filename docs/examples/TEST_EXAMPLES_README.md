# Testing the examples

`examples/*.sop` are ordinary Soplang programs. The compiler test suite runs them.

```bash
# Every suite
cargo test

# Example stdout against examples/*.expected
cargo test --test examples_tests

# Short programs that do not need a file
cargo test --test interpreter_tests
```

A program is checked against a golden stdout only when `examples/<name>.expected` exists. See [testing/EXAMPLES.md](../testing/EXAMPLES.md) for the list.

To refresh one golden file after an intentional output change:

```bash
cargo build --release
./target/release/soplang examples/16_import.sop > examples/16_import.expected
```

Do that only when the new output is the behavior you want to keep.
