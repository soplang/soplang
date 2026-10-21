# Local verification for the 3.3 file and data stage

These checks cover implemented slices. Later 3.3 scope (directories, binary
files, JSON, HTTP) remains on [ROADMAP.md](../../ROADMAP.md).

## Initial UTF-8 text file access

Commands run locally on Linux:

```sh
cargo test --locked --test text_file_tests --test interpreter_tests --test llvm_ir_tests
cargo test --locked -- --skip test_aot
```

The dedicated text-file suite covers read/write/append round trips, catchable
missing-file errors, and invalid UTF-8 reads. LLVM runs the same operations at
optimization levels 0 and 2 with paths relative to the binary working directory.

Cross-platform executables and packaging remain distribution gates.

## Paths and filesystem management

```sh
cargo test --locked --test filesystem_tests --test text_file_tests --test llvm_ir_tests
```

The filesystem suite covers metadata, join/absolute paths, directory lifecycle,
copy/move/delete guards, and LLVM execution at optimization levels 0 and 2.

## Whole-file binary access

```sh
cargo test --locked --test binary_file_tests --test llvm_ir_tests
```

The binary suite covers byte round trips, append, strict `baytyo` arguments, and
LLVM execution at optimization levels 0 and 2.

## JSON

```sh
cargo test --locked --test json_tests --test llvm_ir_tests
```

The JSON suite covers value mapping, sorted object keys, a preserved `__class__`
field that stays a plain object, catchable parse and serialization failures, and
LLVM execution at optimization levels 0 and 2.
