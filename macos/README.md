# Soplang 3 for macOS

The [v3.0.1 release](https://github.com/soplang/soplang/releases/tag/v3.0.1)
provides an Apple Silicon archive whose app bundle declares macOS 14.8.9 as its
minimum. Native debug/release suites and extracted-package JIT/LLVM checks
are covered by the [release gates](../docs/testing/LOCALIZATION_3_0_1.md). The archive
is not notarized. See [Installing Soplang](../docs/installation.md).

## Build packages locally

Install Rust via rustup, Python 3.9+, and Xcode Command Line Tools. Optionally
install create-dmg for a disk image. Run on macOS:

```sh
bash macos/build_macos.sh
```

The locked builder stages `dist/macos/Soplang.app` with its runtime libraries,
creates `macos/soplang-<version>-macos-<arch>.tar.gz`, and optionally creates an
architecture-specific DMG. Checksums are in `macos/SHA256SUMS`. The app declares
the build host's macOS version as its minimum; older releases need separate
validation. This is a command-line application.

```sh
python3 scripts/verify_package.py dist/macos/Soplang.app/Contents/MacOS/soplang
(cd macos && shasum -a 256 -c SHA256SUMS)
```

Repeat verification after extracting the actual archive and mounting the DMG.
For future releases, repeat native JIT/LLVM verification. A locally generated
DMG needs its own checks; signing and notarization require a separate workflow. See [package requirements](../docs/INSTALLING_PACKAGES.md)
and the [verification record](../docs/testing/LOCALIZATION_3_0_1.md).
