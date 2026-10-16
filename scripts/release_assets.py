#!/usr/bin/env python3
"""Verify platform assets and assemble one checksum file for a GitHub release."""

import argparse
import hashlib
from pathlib import Path
import re


def digest(path):
    value = hashlib.sha256()
    with path.open("rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            value.update(block)
    return value.hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("directory", type=Path)
    args = parser.parse_args()
    root = args.directory.resolve(strict=True)
    output = root / "SHA256SUMS"
    assets = {}
    for path in root.rglob("*"):
        if path.is_file() and path.name.endswith((".tar.gz", ".deb", ".zip", ".exe", ".dmg")):
            if path.name in assets:
                raise RuntimeError(f"duplicate release asset name: {path.name}")
            assets[path.name] = path
    if not assets:
        raise RuntimeError("no release assets found")
    verified = {}
    for checksum in sorted(root.rglob("SHA256SUMS")):
        if checksum == output:
            continue
        for row in checksum.read_text(encoding="utf-8").splitlines():
            match = re.fullmatch(r"([0-9a-f]{64})  ([^/\\]+)", row)
            if not match:
                raise RuntimeError(f"invalid checksum record in {checksum}: {row}")
            expected, name = match.groups()
            if name not in assets or assets[name] not in checksum.parent.rglob(name):
                raise RuntimeError(f"missing release asset: {name}")
            if name in verified:
                raise RuntimeError(f"duplicate checksum record: {name}")
            actual = digest(assets[name])
            if actual != expected:
                raise RuntimeError(f"checksum mismatch: {name}")
            verified[name] = actual
    missing = sorted(assets.keys() - verified.keys())
    if missing:
        raise RuntimeError(f"assets without platform checksums: {', '.join(missing)}")
    output.write_bytes("".join(f"{verified[name]}  {name}\n" for name in sorted(verified)).encode("utf-8"))
    print(f"Verified {len(verified)} release assets; wrote {output}")


if __name__ == "__main__":
    main()
