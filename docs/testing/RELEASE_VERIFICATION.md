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

## Package verification

The local Linux archive and Debian package include the executable, runtime rlib,
71 dependency rlibs, the full Rust compiler version, and installation guidance.
The package builder selects Cargo's exact artifact list, excluding stale builds.
The Debian package installs under `/usr/bin` and `/usr/lib/soplang`; its control
metadata records this host build's actual minimum, `libc6 (>= 2.39)`.
Do not advertise compatibility with older distributions from these host assets.

The installed-package checker relocates the executable and libraries into a
fresh temporary directory with spaces and uses a nonexistent Cargo target path.
It checks:

- Version, command execution, imports, stdin, strict mode, and a persistent REPL.
- All 28 public examples and all 63 diagnostic snapshots through JIT and LLVM.
- Concurrent builds with identical output basenames.
- Portable and Unix prefix layouts, missing runtimes, an invalid explicit runtime
  override, and mismatched Rust compiler metadata.
- Native execution after removing the source and installed runtime; JIT execution
  without Rust/clang on PATH.

Both the extracted tar archive and extracted Debian package passed. The host did
not have dpkg-deb installed; the Debian tool was extracted into `/tmp` from the
official downloaded Rust/Debian image, and the normal Linux packaging script
built the package with it. Both actual assets have generated and verified
SHA-256 records in `linux/SHA256SUMS` (generated output, not source control).
This checks extraction and execution, not a privileged system installation.

## Container

The final local image (`043ec9caa31a`) built and passed the complete installed-
package checker: all 28 examples and 63 diagnostics in both backends, REPL,
imports, concurrent builds, runtime discovery, toolchain mismatch handling, and
standalone execution. It runs as `soplang` with clang 16.0.6 and Rust 1.94.0.
Only the checker scripts and examples were mounted read-only; the image supplied
its own executable, runtime libraries, compiler, and linker.

The first attempt exposed clang 14 rejecting opaque LLVM pointer syntax. Docker
now installs clang 16, includes manifests in its context, and keeps its Rust
compiler matched to the bundled rlibs. Build and verification logs are under
`target/release-verification/container-build.log` and `container-install.log`.
The local image check does not establish a successful registry publication.

## Release automation

The native/container verification workflow covers Linux, Windows, and macOS.
Release branch drafts depend on all of its jobs succeeding, then attach actual
archives/packages and platform checksum records from the tested source revision.
Automation no longer rewrites the changelog or creates a commit. Prereleases
skip stable AUR publication and cannot replace stable Docker aliases. Workflow
syntax and embedded shell scripts passed actionlint locally; no remote workflow
or publication was triggered in this session.

## Remaining publication gates

| Gate | Status and required work |
| --- | --- |
| Linux system installation / compatibility floor | Archive and Debian extraction/execution passed on this x86_64 host. A clean system installation and the chosen oldest supported distribution still need validation. |
| Windows native compiler and installer | Pending native CI and a clean installation/removal check, including calling conventions, PATH, and file associations. |
| macOS native compiler and installer | Pending native CI, extracted app / mounted DMG execution, and any required signing/notarization. |
| Container registry | Local image verification passed. Validate the published registry image after publication. |
| Final candidate assets | Local Linux assets and checksums exist. Regenerate and verify every platform's assets from the selected candidate revision; optional installers must be tested if distributed. |
| Release notes/version | Draft prepared in [RELEASE_NOTES_V3.md](../RELEASE_NOTES_V3.md). Cargo remains 3.0.0; a candidate version has not been selected or published. |

Native workflow configuration is not a native result. Keep publication gated on
those checks. Releases remain on the Soplang 3 line.
