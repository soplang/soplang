#!/usr/bin/env bash
# Build a relocatable app bundle and archive; optionally create a DMG.
set -euo pipefail
PROJECT_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$PROJECT_ROOT"
[[ "$(uname -s)" == Darwin ]] || { echo 'Run this package build on macOS.' >&2; exit 1; }
VERSION="$(sed -n 's/^version = "\([^"]*\)"/\1/p' Cargo.toml | head -n 1)"
[[ -n "$VERSION" ]] || { echo 'Missing Cargo version' >&2; exit 1; }
# Keep the DMG staging directory separate from other platform outputs.
APP_DIR=dist/macos/Soplang.app
python3 scripts/package.py --output "$APP_DIR/Contents/MacOS"
# Derive the declared minimum from this build; do not advertise an untested 10.13.
MIN_MACOS="$(sw_vers -productVersion)"
NUMERIC_VERSION="${VERSION%%-*}"
cat > "$APP_DIR/Contents/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
<key>CFBundleExecutable</key><string>soplang</string>
<key>CFBundleIdentifier</key><string>org.soplang</string>
<key>CFBundleName</key><string>Soplang</string>
<key>CFBundlePackageType</key><string>APPL</string>
<key>CFBundleShortVersionString</key><string>$NUMERIC_VERSION</string>
<key>CFBundleVersion</key><string>1</string>
<key>LSMinimumSystemVersion</key><string>$MIN_MACOS</string>
</dict></plist>
PLIST
ARCHIVE="macos/soplang-${VERSION}-macos-$(uname -m).tar.gz"
tar -czf "$ARCHIVE" -C dist/macos Soplang.app
ASSETS=("$ARCHIVE")
if command -v create-dmg >/dev/null 2>&1; then
    DMG="macos/Soplang-${VERSION}-$(uname -m).dmg"
    rm -f "$DMG"
    create-dmg --volname Soplang --window-size 500 300 --app-drop-link 380 150 \
        "$DMG" dist/macos/
    ASSETS+=("$DMG")
fi
python3 scripts/checksums.py "${ASSETS[@]}" --output macos/SHA256SUMS
printf 'Created %s\n' "${ASSETS[@]}"
