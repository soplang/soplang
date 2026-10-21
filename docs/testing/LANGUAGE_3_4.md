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
Public class tests construct a namespace class, extend it, check `m.Qof` in
strict mode, bind `ka Qof sida Student`, and reject a private or unknown class.
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

## TOML manifest and lock schemas

The locked debug suite passes 370 tests, including 52 LLVM tests. The document
checks cover quoted/escaped keys, literal and multiline strings, comments,
structured local paths, duplicate tables, unknown fields, wrong value types,
escaped path round trips, duplicate lock entries, versions and SHA-256 syntax.
Package integration checks accept harmless TOML formatting and reject malformed
or dangling locks before a missing dependency can mask the metadata failure.

## Git repositories and reproducible package graphs

The locked debug suite passes 383 tests, including 52 dedicated LLVM tests.
The optimized library/Git/package/module/LLVM selection passes 127 tests:

```sh
cargo test --locked --offline -- --skip test_aot
cargo test --locked --offline --release --lib --test git_package_tests --test package_tests --test module_tests --test llvm_ir_tests
```

All Git tests use temporary local repositories and isolated package caches.
They cover dirty worktrees versus committed bytes, URL/path spaces and UTF-8,
annotated tags and full commits, SHA-256 object repositories, lock generation,
branch movement, explicit updates, root/transitive selector agreement,
conflicting commits, and dependency-owned locks that cannot override the root.

Offline JIT and LLVM builds run without the origin. LLVM levels 0 and 2 execute
after both source and cache are moved away. Corrupt caches, injected Git metadata,
missing offline content, changed locked sources, name mismatches, cycles,
snapshot-relative import escapes and symlinks fail without executing source.
Six concurrent installers share one complete content tree and lock. Transport
stall tests cover both a live child and an exited parent with inherited pipes.
A separate REPL process retains its selected graph after an external lock update;
a fresh process observes the new version.

The Git source format stores regular file bytes, with the documented size and
path limits. Native Windows/macOS execution remains a platform verification gate.
This stage does not complete the web framework, ORM, tooling or concurrency
milestones, and does not create a release tag.

The complete diagnostic/example suites also pass all 14 tests through JIT and
AOT, covering 175 error programs and 43 public examples.
