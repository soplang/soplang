# Soplang 3.0 verification

## Local compiler and examples

Verified on Linux x86_64 with Rust 1.94.0 and clang 22.1.8.

| Check | Result |
| --- | --- |
| `cargo test --locked` | 99 passed; no ignored or skipped tests |
| `cargo test --release --locked` | 99 passed; no ignored or skipped tests |
| `cargo test --locked --test llvm_ir_tests` | 12 passed |
| `cargo build --release --locked` | Passed |
| Targeted Rust formatting and `git diff --check` | Passed |

Both full profiles run all 28 numbered examples and all 63 intentional failures
through Cranelift JIT and LLVM AOT. Tests check exit status, stdout, stderr,
original source files, one-based lines/columns, and exact diagnostic snapshots.
Strict mode and stdin have dedicated example coverage. Random output checks
range/membership invariants so expected stdout remains deterministic.

Additional diagnostic checks cover command-expression echo, REPL compile
errors, combining characters, tabs, wide characters, large line numbers, and
argument errors that use the spelling from the call site. A standalone native
binary retains the correct imported-file diagnostic after its source files are
deleted. Caught errors remain message strings.

Detailed local test logs are generated under `target/release-verification/`;
that build directory is not part of the source distribution. The checked
[example catalog](../../examples/README.md), [failure manifest](../../examples/errors/cases.tsv),
and `.expected` / `.stderr` files provide reproducible source-level evidence.

## Remaining distribution gates

These are separate from the successful local compiler tests.

| Gate | Status and required work |
| --- | --- |
| Installed Linux package | Pending. Package scripts currently copy only the executable; `--build` still searches the original Cargo target directory for its runtime/dependency rlibs. Bundle and locate those artifacts, then test an extracted install away from the checkout. |
| Windows installer | Pending native build and execution on Windows, including the JIT and LLVM calling/linking conventions and toolchain requirements. |
| macOS package | Pending native build and execution on macOS, including imports, REPL, and standalone builds. |
| Container | Pending. `.dockerignore` currently excludes Cargo manifests used by `Dockerfile`, and the runtime image omits the `--build` toolchain/artifacts. This session also cannot access the Docker daemon socket. |
| Release assets/checksums | Pending final package builds; compute checksums from the actual assets after those gates pass. |
| Release notes | Draft prepared in [RELEASE_NOTES_V3.md](../RELEASE_NOTES_V3.md). |

The local results do not establish installed-package or cross-platform release
readiness. Keep publication gated on those distribution checks. Releases remain
on the Soplang 3 line.
