# Soplang for Linux

Build Soplang (Rust) on Linux and optionally create a tarball or .deb package.

**Full build guide (all platforms):** [../docs/BUILD_GUIDE.md](../docs/BUILD_GUIDE.md)

## Prerequisites

- **Rust** — [rustup.rs](https://rustup.rs)
- **dpkg-dev**, **fakeroot** (optional, for .deb) — `sudo apt install dpkg-dev fakeroot`

## Build

From the **project root**:

```bash
./build.sh
```

Or from this directory:

```bash
chmod +x build_linux.sh
./build_linux.sh
```

The script will:

1. Run `cargo build --release`
2. Copy binary to **dist/soplang/soplang**
3. Create **linux/soplang-&lt;version&gt;-linux-$(uname -m).tar.gz**
4. If `dpkg-deb` is available, create **linux/soplang_&lt;version&gt;_amd64.deb** (or your arch)

## Output

- **Binary:** `target/release/soplang`
- **Dist:** `dist/soplang/`
- **Tarball:** `linux/soplang-<version>-linux-<arch>.tar.gz`
- **Debian package:** `linux/soplang_<version>_amd64.deb` (if dpkg-deb installed)

## Install from .deb

```bash
sudo dpkg -i linux/soplang_<version>_amd64.deb
soplang examples/01_hello.sop
```

## Run without installing

```bash
./target/release/soplang examples/01_hello.sop
./target/release/soplang -i
```
