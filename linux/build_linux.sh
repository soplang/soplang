#!/usr/bin/env bash
# Build the complete portable package and an optional Debian package.
set -euo pipefail
PROJECT_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$PROJECT_ROOT"
VERSION="$(sed -n 's/^version = "\([^"]*\)"/\1/p' Cargo.toml | head -n 1)"
[[ -n "$VERSION" ]] || { echo 'Missing Cargo version' >&2; exit 1; }
# Debian sorts prereleases below the final release using a tilde.
DEB_VERSION="${VERSION/-/~}"
ARCH="$(uname -m)"
case "$ARCH" in
    x86_64) DEB_ARCH=amd64 ;;
    aarch64) DEB_ARCH=arm64 ;;
    *) DEB_ARCH="$ARCH" ;;
esac
python3 scripts/package.py --output dist/soplang
TARBALL="linux/soplang-${VERSION}-linux-${ARCH}.tar.gz"
tar -czf "$TARBALL" -C dist soplang
ASSETS=("$TARBALL")
if command -v dpkg-deb >/dev/null 2>&1; then
    DEB_DIR="$(mktemp -d)"
    trap 'rm -rf "$DEB_DIR"' EXIT
    mkdir -p "$DEB_DIR/DEBIAN" "$DEB_DIR/usr/bin" "$DEB_DIR/usr/lib" \
        "$DEB_DIR/usr/share/doc/soplang"
    cp dist/soplang/soplang "$DEB_DIR/usr/bin/"
    cp -R dist/soplang/lib/soplang "$DEB_DIR/usr/lib/"
    cp dist/soplang/LICENSE dist/soplang/INSTALLING.md "$DEB_DIR/usr/share/doc/soplang/"
    # Record the binary's actual glibc floor rather than accepting installation
    # on distributions too old to start it.
    GLIBC_VERSION="$(LC_ALL=C readelf --version-info dist/soplang/soplang | \
        grep -oE 'GLIBC_[0-9]+(\.[0-9]+)+' | sed 's/GLIBC_//' | sort -Vu | tail -n 1)"
    [[ -n "$GLIBC_VERSION" ]] || { echo 'Cannot determine glibc requirement' >&2; exit 1; }
    cat > "$DEB_DIR/DEBIAN/control" <<CONTROL
Package: soplang
Version: $DEB_VERSION
Section: devel
Priority: optional
Architecture: $DEB_ARCH
Maintainer: Soplang Software Foundation <info@soplang.org>
Depends: libc6 (>= $GLIBC_VERSION), libgcc-s1
Suggests: clang, rustc
Description: The Somali programming language
 Somali syntax, Cranelift JIT, and LLVM native builds.
 Native builds require clang and the matching Rust toolchain.
Homepage: https://www.soplang.org/
CONTROL
    DEB="linux/soplang_${DEB_VERSION}_${DEB_ARCH}.deb"
    dpkg-deb --root-owner-group --build "$DEB_DIR" "$DEB"
    ASSETS+=("$DEB")
else
    echo 'dpkg-deb unavailable; Debian packaging still needs verification.'
fi
python3 scripts/checksums.py "${ASSETS[@]}" --output linux/SHA256SUMS
printf 'Created %s\n' "${ASSETS[@]}"
