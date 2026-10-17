# Soplang 3.0.0 verification record

## Published release

[Soplang 3.0.0](https://github.com/soplang/soplang/releases/tag/v3.0.0) was published
on **2026-10-17** from commit
`0a3d36e44dd65a0add6f6dac285d774e439d37ff`.
The [native and container verification run](https://github.com/soplang/soplang/actions/runs/38995837371)
completed successfully before the draft was created and published.

| Gate | Result | Verified coverage |
| --- | --- | --- |
| Linux x86_64 | Passed | Full debug/release suites; extracted tar/Debian packages; system install, installed JIT/LLVM imports/closures/inheritance, and removal |
| Windows x64 | Passed | Full debug/release suites; extracted ZIP; installed JIT/LLVM; installer removal, PATH restoration, and file associations |
| macOS ARM64 | Passed | Full debug/release suites; extracted app archive; full installed-package checker |
| Container | Passed | Pinned Rust/clang image; installed-package checker as the unprivileged `soplang` user |
| Asset integrity | Passed | Five downloaded packages matched the combined `SHA256SUMS`; unique asset filenames |
| Downloaded Linux archive | Passed | Version, JIT execution, and LLVM native executable smoke tests |

Each native job ran **102 tests in debug and 102 in release**, including the
13 LLVM integration tests. The suites execute all 28 numbered examples and all
63 intentional failures through Cranelift and LLVM. Coverage includes imports,
closures, inheritance, strict mode, Unicode carets, LF/CRLF/EOF input, and native
execution after source removal.

## Published assets

| Platform | Assets | Recorded requirements |
| --- | --- | --- |
| Linux x86_64 | `soplang-3.0.0-linux-x86_64.tar.gz`, `soplang_3.0.0_amd64.deb` | glibc 2.39 or later; libgcc |
| Windows x64 | `soplang-3.0.0-windows-AMD64.zip`, `soplang-setup.exe` | Native Windows x64 environment |
| macOS Apple Silicon | `soplang-3.0.0-macos-arm64.tar.gz` | Bundle minimum macOS 14.8.9 |

The release attaches one combined `SHA256SUMS`. The Windows installer is
unsigned, and the macOS archive is not notarized. RPM, Intel Mac, and DMG
packages are not attached to this release.

## Registry publication

The [Docker publication workflow](https://github.com/soplang/soplang/actions/runs/38996737793)
succeeded for Docker Hub and GHCR. The public Docker Hub image
`soplang/soplang:3.0.0` was pulled and passed version, JIT, and LLVM native-build
checks. Its labels identify version 3.0.0 and the exact tested source commit.
Its `latest` and `3.0.0` manifests matched.

Anonymous GHCR pulls were still rejected in the 2026-10-17 documentation check.
Its package visibility must be Public for unauthenticated use. Docker Hub is
the verified public registry. AUR credentials were absent at publication, so
the optional AUR job skipped the upload; no AUR release is claimed.

## Platform fixes covered by the release checks

- ARM64 JIT functions use far references when allocations exceed direct-call range.
- Windows x64 generated calls use System V bridges to the platform runtime,
  including callbacks with explicit two-word return values.
- Source builds use Cargo's native-library metadata; Windows packages bundle
  the required import libraries.
- Linker sidecars stay in scratch directories. Executables publish through a
  same-volume rename, with a copy fallback across volumes. An additional local
  stress check passed 65 LLVM tests after fixing a Linux copy/execute race.
- Windows input removes CRLF terminators while preserving spaces and EOF input.
- Windows uninstall removes PATH entries before deleting the directory; the
  installer checker waits for the copied uninstaller's log handle to close.

## Benchmark comparison

The post-release [Python/v2/v3 comparison](../../benchmarks/RESULTS.md) records
1,000 measured processes across ten equivalent workloads and four modes.
Every output matched. It reports medians, sample spread, separate LLVM build
costs, raw observations, and source/toolchain provenance. The compiler source
matches the v3.0.0 release; benchmark tooling was added afterward.

## Repeating verification

Follow the [release process](../RELEASE_PROCESS.md) for future 3.x versions.
Native builds require the exact Rust toolchain recorded in the package, clang,
and the platform linker/SDK. JIT and REPL do not need Rust or clang.

Local compiler and package logs are kept under ignored
`target/release-verification/`; benchmark data is under `benchmarks/comparison/`.
A platform's build script alone does not establish ABI, installer, or signing
support. Each new release must pass its own gates.
