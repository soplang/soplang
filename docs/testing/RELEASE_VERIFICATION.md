# Soplang 3.0 verification

## Local compiler checks

Verified on Linux x86_64 with Rust 1.94.0 and clang 22.1.8:

| Check | Result |
| --- | --- |
| `cargo test --locked` | 102 passed; none ignored or skipped |
| `cargo test --release --locked` | 102 passed; none ignored or skipped |
| `cargo test --locked --test llvm_ir_tests` | 13 passed |
| Windows x64 and macOS ARM64 target compile checks | Passed |
| Targeted formatting and whitespace checks | Passed |

The suites run all 28 numbered examples and all 63 intentional failures through
Cranelift and LLVM, including imports, closures, classes, strict mode, Unicode
source columns/carets, stdin, and standalone execution after source removal.
Dedicated input tests cover LF, CRLF, EOF, empty input, and retained whitespace.

The relocated Linux archive and Debian package and the local Docker image passed
the installed-package checker. It moves the compiler/runtime outside the
checkout, tests both installation layouts and concurrent builds, rejects missing
or mismatched runtimes, and runs native outputs after source/runtime removal.
Local logs live in the ignored `target/release-verification/` directory.

## Required native release gates

The `release/v3.0.0` branch runs the
[release workflow](https://github.com/soplang/soplang/actions/workflows/release.yml)
and creates a draft only when all verification jobs succeed. The published
[3.0.0 release](https://github.com/soplang/soplang/releases/tag/v3.0.0) records its
final workflow run and provides a combined `SHA256SUMS` for the actual assets.

| Gate | Required coverage |
| --- | --- |
| Linux x86_64 | Full debug/release suites; extracted tar/Debian packages; system installation, JIT/LLVM imports/closures/inheritance, and removal |
| Windows x64 | Full debug/release suites; extracted ZIP; installer execution and removal; installed JIT/LLVM; PATH and file associations |
| macOS ARM64 | Full debug/release suites; extracted app archive; full installed-package checker |
| Container | Pinned Rust/clang image build; full installed-package checker as the unprivileged `soplang` user |
| Asset integrity | Each platform checksum matches its artifacts; filenames are unique; one combined manifest covers every published asset |

Workflow configuration alone is not a passing result. Publication must wait for
all native gates. Optional installers are distributed only when built and checked.
Published container images are checked after the registry workflow completes.

## Platform fixes covered by these gates

The initial native runs exposed macOS ARM64 direct-call range failures and
missing Windows import-library search paths. JIT calls now use far references.
Windows x64 bridges match the generated System V value convention to the native
C runtime, including callback return values. Source builds reuse Cargo's native
library metadata; packages carry the required Windows import libraries.

Linker sidecars remain in scratch directories. Finished executables publish with
a same-volume rename, with a copy fallback for different volumes. Parallel LLVM
execution passed 65 tests across five additional local batches after a Linux
copy/execute race was found. Windows CRLF input has explicit regressions in both
backends, preserving whitespace and EOF behavior.

## Distribution limits

- Native `--build` requires the exact Rust compiler recorded in the package,
  clang with opaque pointer support, and the platform linker/SDK. JIT and REPL
  execution do not require Rust or clang.
- Linux compatibility follows the built executable's actual glibc requirement;
  the Debian control file records its minimum. A newer build host does not
  establish compatibility with older distributions.
- The macOS bundle records the build host's OS version as its minimum. The
  Windows installer is unsigned; macOS distribution is not notarized.
- Stable AUR publication runs only when its credentials are configured.
- Releases stay on the Soplang 3 line.
