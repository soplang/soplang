# Local verification for the 3.4 module stage

These checks cover namespaced imports and the package registry.

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
program before it prints. A `[ku-tiirsan]` path must point at a package of the
same name. A `soplang.lock` file, when present, must match those resolved
dependencies. Registry installs pin `sumad`. `--cusbooneysii` moves a caret
constraint to a newer published version, and `keen "xisaab"` loads the cached
package. An HTTP `.sopkg` download is checked against a local server.

## Module regression audit

The full locked debug suite passes 353 tests, including 52 dedicated LLVM tests.
The ten module tests cover JIT and LLVM levels 0 and 2, lexical shadowing,
defaults, interpolation, private class annotations, repeated and diamond imports,
source-order initialization, cycle locations, strict imports, REPL collection,
transactional import discovery after analysis failure, and initializer retry.
Existing import diagnostic snapshots retain their relative file names and carets.

Windows and macOS checks currently stop in the `ring` C dependency because this
Linux host has no Windows SDK or macOS SDK headers. Native platform CI remains
required; these checks do not establish cross-platform release readiness.

## Project initialization

The full locked debug suite passes 362 tests, including 52 LLVM tests.
`project_init_tests` checks application execution and library imports through
JIT and LLVM at optimization levels 0 and 2, default/current directories,
package naming, Somali help and errors, preservation of existing files, and
refusal to overwrite dangling starter symlinks. Library entry resolution retains
the older package-named source layout as a fallback.
