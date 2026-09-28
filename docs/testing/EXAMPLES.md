# Example programs

The programs in `examples/` demonstrate the language and, where a `.expected` file exists, act as regression checks. `tests/examples_tests.rs` runs each `examples/*.sop` that has a matching `.expected` and compares stdout.

## Run one

```bash
cargo build --release
./target/release/soplang examples/01_hello.sop
```

`--example N` runs the Nth `.sop` file in `examples/` after a name sort. `--example 1` is `01_hello.sop`. `_math_lib.sop` sorts last.

Docker, after a local image build:

```bash
docker run --rm -v "$(pwd):/scripts" soplang:local examples/01_hello.sop
```

## Programs

| File | What it shows |
|------|----------------|
| `01_hello.sop` | `qor` |
| `02_variables.sop` | `door` reassignment, `abn`, `qoraal`, `jajab`, `bool` |
| `03_constants.sop` | `madoor` |
| `04_operators.sop` | Arithmetic and comparison |
| `05_conditionals.sop` | `haddii` / `ugudambeyn` |
| `06_loops.sop` | `kuceli` and `intay` |
| `07_switch.sop` | `dooro` / `xaalad` |
| `08_functions.sop` | `hawl` and `celi` |
| `09_lists.sop` | List literals and list methods |
| `10_objects.sop` | `walax` and object methods |
| `11_types.sop` | `nooc` and conversions |
| `12_strings.sop` | String methods |
| `13_input.sop` | `gelin` |
| `14_random.sop` | `xul` (output varies) |
| `15_classes.sop` | `qaab`, `dhaxal`, `nafta` |
| `16_import.sop` | `keen` of `_math_lib.sop` |
| `17_try_catch.sop` | `fasax` / `qabo` |

`_math_lib.sop` is a library imported by `16_import.sop`, not a standalone demo.

Checked against `.expected` today: `15_classes.sop`, `16_import.sop`, `17_try_catch.sop`. Other examples are run by hand or covered by `tests/interpreter_tests.rs`.

`14_random.sop` calls `xul`, so it has no `.expected` file.
