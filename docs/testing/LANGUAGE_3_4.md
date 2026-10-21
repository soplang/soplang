# Local verification for the 3.4 module stage

These checks cover the implemented slice. Package imports remain on
[ROADMAP.md](../../ROADMAP.md).

## Namespaced imports

```sh
cargo test --locked --test module_tests
```

The suite loads a sibling file with `keen "math.sop" sida xisaab`, calls an
exported function through the namespace, checks that the function name is not
global, rejects a private `_` name and a name omitted from `soosaar`, rejects
an unknown export, binds selected names with `ka` and `sida`, and builds a
program with LLVM.
Flat `keen` imports stay covered by the existing import example.

## Package manifest

```sh
cargo test --locked --lib package:: --test package_tests
```

The parser checks the package name, version, language requirement, source
directory, and unknown keys. Running a file beside a compatible `soplang.toml`
still executes the program. An incompatible language requirement stops the
program before it prints.
