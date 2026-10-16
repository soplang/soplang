#!/usr/bin/env python3
"""Write SHA-256 checksums for actual release assets (run beside the assets)."""

import argparse
import hashlib
from pathlib import Path

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("assets", nargs="+", type=Path)
parser.add_argument("--output", type=Path, default=Path("SHA256SUMS"))
args = parser.parse_args()
if len({path.name for path in args.assets}) != len(args.assets):
    parser.error("asset filenames must be unique")
rows = []
for path in sorted(args.assets):
    with path.open("rb") as stream:
        digest = hashlib.sha256()
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(block)
    rows.append(f"{digest.hexdigest()}  {path.name}\n")
args.output.write_bytes("".join(rows).encode("utf-8"))
print(args.output)
