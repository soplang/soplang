#!/usr/bin/env python3
"""Build a relocatable Soplang distribution using Cargo's exact artifact list."""

import argparse
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[1]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, default=ROOT / "dist/soplang")
    args = parser.parse_args()
    output = args.output.resolve()
    if output == ROOT or output in ROOT.parents:
        parser.error("output must be a separate package directory")

    rustc = os.environ.get("RUSTC", "rustc")
    version = subprocess.check_output([rustc, "-vV"], text=True)
    # Request both targets explicitly. Cargo reports fresh and newly built
    # dependencies, so stale rlibs from other builds never enter the package.
    result = subprocess.run(
        ["cargo", "build", "--release", "--locked", "--lib", "--bin", "soplang",
         "--message-format=json-render-diagnostics"],
        cwd=ROOT, stdout=subprocess.PIPE, text=True, check=True,
    )
    artifacts = []
    binary = None
    runtime = None
    for line in result.stdout.splitlines():
        message = json.loads(line)
        if message.get("reason") != "compiler-artifact":
            continue
        target = message["target"]
        if target["name"] == "soplang" and "bin" in target["kind"]:
            binary = Path(message["executable"])
        for name in message["filenames"]:
            path = Path(name)
            if path.suffix == ".rlib":
                if target["name"] == "soplang":
                    runtime = path
                else:
                    artifacts.append(path)
    if binary is None or runtime is None:
        raise RuntimeError("Cargo did not report the Soplang executable and runtime")

    output.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix=".soplang-package-", dir=output.parent) as temp:
        stage = Path(temp) / "package"
        deps = stage / "lib/soplang/deps"
        deps.mkdir(parents=True)
        shutil.copy2(binary, stage / binary.name)
        shutil.copy2(runtime, deps.parent / "libsoplang.rlib")
        for artifact in artifacts:
            shutil.copy2(artifact, deps / artifact.name)
        (deps.parent / "rustc-version.txt").write_text(version, encoding="utf-8")
        for name in ("LICENSE", "README.md"):
            shutil.copy2(ROOT / name, stage / name)
        shutil.copy2(ROOT / "docs/INSTALLING_PACKAGES.md", stage / "INSTALLING.md")
        if output.exists():
            shutil.rmtree(output)
        shutil.move(str(stage), output)
    print(f"Packaged {binary.name}, runtime, and {len(artifacts)} dependency libraries in {output}")


if __name__ == "__main__":
    main()
