#!/bin/bash
# Soplang build — cargo build, or on macOS/Linux/Windows run platform build script.
set -e
cd "$(dirname "$0")"

if [[ "$OSTYPE" == "darwin"* ]]; then
    exec ./macos/build_macos.sh
elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
    exec ./linux/build_linux.sh
elif [[ "$OSTYPE" == "msys" || "$OSTYPE" == "cygwin" || "$OSTYPE" == "win32" ]]; then
    echo "On Windows run: windows\build_windows.bat or powershell -File windows\build_windows.ps1"
    cargo build --release --locked
    echo "Binary: ./target/release/soplang.exe"
else
    cargo build --release --locked
    echo "Binary: ./target/release/soplang"
fi
