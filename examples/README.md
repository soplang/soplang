# Soplang examples

These programs demonstrate the main language features. Files with a matching
`.expected` file are also checked by `tests/examples_tests.rs`.

## Running examples

```bash
cargo build --release
./target/release/soplang examples/01_hello.sop
```

You can also run an example by its position in the sorted file list:

```bash
./target/release/soplang --example 1
```

## Programs

| File | Topic |
| --- | --- |
| `01_hello.sop` | Output with `qor` |
| `02_variables.sop` | Dynamic and typed variables |
| `03_constants.sop` | Constants with `madoor` |
| `04_operators.sop` | Arithmetic, comparison, and logical operators |
| `05_conditionals.sop` | `haddii`, `haddii_kale`, and `ugudambeyn` |
| `06_loops.sop` | `kuceli` and `intay` loops |
| `07_switch.sop` | `dooro` and `xaalad` |
| `08_functions.sop` | Functions and `celi` |
| `09_lists.sop` | Lists, indexing, and list methods |
| `10_objects.sop` | Objects, properties, and object methods |
| `11_types.sop` | `nooc` and type conversion |
| `12_strings.sop` | String methods |
| `13_input.sop` | Input with `gelin` |
| `14_random.sop` | Random selection with `xul` |
| `15_classes.sop` | Classes, inheritance, and `nafta` |
| `16_import.sop` | Importing `_math_lib.sop` with `keen` |
| `17_try_catch.sop` | Errors with `fasax` and `qabo` |

`_math_lib.sop` is imported by `16_import.sop`; it is not a standalone demo.

The class, import, and try/catch examples currently have `.expected` files.
`14_random.sop` intentionally has none because its output varies.

## Adding a regression example

Add a `.sop` file and record its expected standard output in a file with the
same stem:

```bash
./target/release/soplang examples/18_feature.sop > examples/18_feature.expected
cargo test --test examples_tests
```
