# Soplang examples

Every numbered program has a matching `.expected` file. Tests check its exit
status, stdout, and empty stderr in both Cranelift JIT and LLVM AOT. Intentional
failures live separately in [errors/](errors/README.md).

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
| `05_conditionals.sop` | `haddii` and contextual `kale` |
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
| `17_try_catch.sop` | Errors with `tijaabi` and `qabo` |
| `18_closures.sop` | Captures, map, filter, fold, every, and some |
| `19_collection_contracts.sop` | Append, combine, remove, safe lookup, slicing, and counting |
| `20_unicode_strings.sop` | Character indexes, endpoints, case conversion, replacement, repetition, and joining |
| `21_global_updates.sop` | Shared globals and numeric representation |
| `22_numbers.sop` | Conversion, rounding, extrema, powers, square root, and ranges |
| `23_strict_types.sop` | Typed function parameters and returns; also checked with `--strict` |
| `24_caught_errors.sop` | Recovery and continuing after runtime failures |
| `25_compatibility.sop` | Earlier names alongside canonical names |
| `26_loop_control.sop` | Descending loops, break, continue, and short circuiting |
| `27_inheritance.sop` | Typed base instances, constructors, and `waalid` |
| `28_null_defaults.sop` | Stored nulls, defaults, empty endpoints, and search misses |
| `29_output.sop` | Empty and multiple-argument output, mixed values, return text, and evaluation order |
| `30_iteration.sop` | Collection loops, Unicode characters, lazy ranges, enumerate, and zip |
| `31_iterator_protocol.sop` | Class iterators, inherited methods, explicit cursor steps, and exhaustion |

`_math_lib.sop` is imported by `16_import.sop`; it is not a standalone demo.

All numbered examples are checked. The random example prints type and range
invariants rather than a random value, so its expected output is deterministic.
The input example receives `TestUser` through stdin. Helpers beginning with `_`
and programs inside `errors/` are excluded from `--example N`.

## Full example verification

```bash
cargo test --locked --test examples_tests
cargo test --locked --test diagnostic_tests
cargo test --locked --test llvm_ir_tests
cargo test --locked
```

These commands require system `clang` and `rustc` for native builds. The full
suite includes the AOT example and error runs; `--skip test_aot` skips those
slower checks. Add only valid programs to the numbered list.

## Adding a regression example

Add a `.sop` file and record its expected standard output in a file with the
same stem:

```bash
./target/release/soplang examples/18_feature.sop > examples/18_feature.expected
cargo test --locked --test examples_tests
```
