# Soplang for macOS

Build Soplang (Rust) on macOS and optionally create an app bundle and DMG.

**Full build guide (all platforms):** [../docs/BUILD_GUIDE.md](../docs/BUILD_GUIDE.md)

## Prerequisites

- **Rust** — [rustup.rs](https://rustup.rs)
- **create-dmg** (optional, for disk image) — `brew install create-dmg`

## Build

From the **project root**:

```bash
./build.sh
```

Or from this directory:

```bash
chmod +x build_macos.sh
./build_macos.sh
```

The script will:

1. Run `cargo build --release`
2. Create **dist/Soplang.app** (macOS app bundle)
3. If `create-dmg` is installed, create **macos/Soplang-&lt;version&gt;.dmg**

## Output

- **Binary:** `target/release/soplang`
- **App bundle:** `dist/Soplang.app`
- **DMG:** `macos/Soplang-<version>.dmg` (if create-dmg installed)

## Run

```bash
./target/release/soplang examples/01_hello.sop
./target/release/soplang -i
# Or: open dist/Soplang.app
```

## Optional: code signing and notarization

For distribution you may want to sign and notarize:

```bash
codesign --force --sign "Developer ID Application: Your Name" dist/Soplang.app
# Then notarize the DMG with Apple (see Apple docs).
```
