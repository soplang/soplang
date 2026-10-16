#!/bin/bash
# Soplang Linux Build — build Rust binary and optional tarball / .deb.
# Prerequisites: Rust (rustup.rs). For .deb: dpkg-dev, fakeroot.

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$PROJECT_ROOT"

GREEN='\033[0;32m'
CYAN='\033[0;36m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

CARGO_VERSION="$(sed -n 's/^version = "\([^"]*\)"/\1/p' Cargo.toml | head -n 1)"
if [[ -z "$CARGO_VERSION" ]]; then
    echo -e "${RED}Error: could not read the package version from Cargo.toml.${NC}"
    exit 1
fi
VERSION="$CARGO_VERSION"
ARCH="$(uname -m)"
[[ "$ARCH" == "x86_64" ]] && DEB_ARCH="amd64" || DEB_ARCH="$ARCH"

echo -e "${CYAN}Soplang Linux Build${NC}"
echo -e "${CYAN}===================${NC}"

# Check Rust
if ! command -v cargo &>/dev/null; then
    echo -e "${RED}Error: cargo not found. Install Rust from https://rustup.rs${NC}"
    exit 1
fi

# Build release binary
echo -e "${CYAN}Building release binary...${NC}"
cargo build --release
BINARY="$PROJECT_ROOT/target/release/soplang"
if [[ ! -f "$BINARY" ]]; then
    echo -e "${RED}Build failed: binary not found.${NC}"
    exit 1
fi
echo -e "${GREEN}Binary: $BINARY${NC}"

# Prepare dist layout
DIST_DIR="dist/soplang"
rm -rf "$DIST_DIR"
mkdir -p "$DIST_DIR"
cp "$BINARY" "$DIST_DIR/soplang"
chmod +x "$DIST_DIR/soplang"
echo -e "${GREEN}Dist: $DIST_DIR/${NC}"

# Optional: tarball
TARBALL="linux/soplang-${VERSION}-linux-${ARCH}.tar.gz"
mkdir -p linux
tar -czf "$TARBALL" -C dist soplang
echo -e "${GREEN}Tarball: $TARBALL${NC}"

# Optional: .deb if dpkg-deb available
if command -v dpkg-deb &>/dev/null; then
    DEB_DIR="linux/soplang_${VERSION}_${DEB_ARCH}"
    rm -rf "$DEB_DIR"
    mkdir -p "$DEB_DIR/DEBIAN"
    mkdir -p "$DEB_DIR/usr/local/bin"
    cp "$BINARY" "$DEB_DIR/usr/local/bin/soplang"
    chmod 755 "$DEB_DIR/usr/local/bin/soplang"
    cat > "$DEB_DIR/DEBIAN/control" << EOF
Package: soplang
Version: $VERSION
Section: devel
Priority: optional
Architecture: $DEB_ARCH
Maintainer: Soplang Software Foundation <info@soplang.org>
Description: Soplang - The Somali Programming Language
 Soplang is a programming language with Somali-inspired syntax.
Homepage: https://www.soplang.org/
EOF
    fakeroot dpkg-deb --build "$DEB_DIR" "linux/soplang_${VERSION}_${DEB_ARCH}.deb" 2>/dev/null || dpkg-deb --build "$DEB_DIR" "linux/soplang_${VERSION}_${DEB_ARCH}.deb"
    rm -rf "$DEB_DIR"
    echo -e "${GREEN}Debian package: linux/soplang_${VERSION}_${DEB_ARCH}.deb${NC}"
else
    echo -e "${YELLOW}Tip: install dpkg-dev for .deb: sudo apt install dpkg-dev fakeroot${NC}"
fi

echo -e "${GREEN}Done.${NC}"
