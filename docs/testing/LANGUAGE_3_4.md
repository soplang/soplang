# Local verification for the 3.4 module stage

These checks cover the implemented slice. Explicit export lists, selective
imports, and packages remain on [ROADMAP.md](../../ROADMAP.md).

## Namespaced imports

```sh
cargo test --locked --test module_tests
```

The suite loads a sibling file with `keen "math.sop" sida xisaab`, calls an
exported function through the namespace, checks that the function name is not
global, rejects a private `_` name and a name omitted from `soosaar`, rejects
an unknown export, and builds a program with LLVM.
Flat `keen` imports stay covered by the existing import example.
