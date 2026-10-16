# Soplang for macOS

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
Signing, notarization, and native JIT/LLVM results remain release gates until
verified on macOS. See [package requirements](../docs/INSTALLING_PACKAGES.md)
and the [verification record](../docs/testing/RELEASE_VERIFICATION.md).
