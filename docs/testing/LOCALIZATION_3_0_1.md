# Soplang 3.0.1 localization verification

Date: 2026-10-18.

Status: local compiler and portable-package checks passed. Release publication
and the native CI matrix remain pending.

## Source

The localization commit is `1d24f875070399abdfc1db6c33d71ba20039256e`,
with Cargo version 3.0.1. It was tested in an isolated source snapshot based
on `e72c6668276fa801691ddb1c753ebbe2f73a417c`.
Unfinished compiler analysis work in the main checkout was excluded. Record
and verify the final committed release revision in CI before publication.

## Completed checks

| Check | Result |
| --- | --- |
| `cargo test --locked --offline` | 112 passed, including AOT tests |
| `cargo test --release --locked --offline` | 112 passed, including AOT tests |
| `cargo test --locked --offline --test llvm_ir_tests` | 13 passed |
| Localization integration tests | 10 passed, including LLVM input/caught-error parity |
| `bash linux/build_linux.sh` | Portable Linux x86_64 archive and SHA256SUMS created |
| `python3 scripts/verify_package.py dist/soplang/soplang` | Passed outside the source checkout |

The installed-package verifier exercised version/help/argument errors, REPL,
28 public examples, 63 intentional diagnostics in JIT and LLVM, runtime
relocation/discovery, toolchain mismatch, and standalone execution without
source or runtime files.

The localization tests cover generated help and defaults, invalid CLI values,
exit statuses and output streams, UTF-8 failures, paths with spaces/Unicode,
contextual I/O explanations, HIR wrapper text, preserved source carets,
internal tags, JIT startup state, failed tool fixtures, original stderr only
when requested, and broken output without a print panic. Invalid command
argument encoding is tested on Unix.

Installed-package verification initially caught a runtime dependency on
Clap's procedural-macro library. Moving the argument formatter into the CLI
binary fixed the packaged LLVM link without expanding runtime dependencies.
The final portable package passed verification after that change.

## Remaining release gates

- Native Linux CI, including Debian package build and clean installation.
  `dpkg-deb` was unavailable in the local test environment.
- Native Windows build, ZIP/installer verification, and checksums.
- Native macOS build, package verification, and checksums.
- Container build and installed-package verification.
- Combined release asset checksums and review of the exact tested revision.
- Stable release publication, registry publication/pull checks, and optional
  AUR publication when configured.
- Website version/fallback update from actual published asset metadata,
  documentation checks, production build, deployment, and live download checks.

Local Linux results do not establish success on other operating systems.
The [v3.0.0 verification record](RELEASE_VERIFICATION.md) remains historical.
