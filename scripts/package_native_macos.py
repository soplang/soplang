#!/usr/bin/env python3
"""Build a relocatable Apple Silicon Soplang package with bundled LLVM and LLD."""

import argparse
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile

ROOT = Path(__file__).resolve().parents[1]


def copy_stub(source, destination, extra_symbols=()):
    """Add public runtime symbols missing from the generated link interface."""
    text = source.read_text(encoding="utf-8")
    if extra_symbols:
        end = text.rfind("\n...\n")
        if end < 0 or "exports:" not in text[:end]:
            raise RuntimeError(f"invalid macOS link stub: {source}")
        additions = (
            "\n  - targets: [ arm64-macos ]\n"
            f"    symbols: [ {', '.join(extra_symbols)} ]"
        )
        text = text[:end] + additions + text[end:]
    destination.write_text(text, encoding="utf-8")


def output(*argv, env=None):
    return subprocess.check_output(argv, text=True, cwd=ROOT, env=env).strip()


def dependencies(path):
    return [line.strip().split(" (", 1)[0]
            for line in output("otool", "-L", str(path)).splitlines()[1:]]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, default=ROOT / "dist/soplang-native-macos")
    args = parser.parse_args()
    if sys.platform != "darwin" or os.uname().machine != "arm64":
        parser.error("this package builder requires macOS ARM64")
    llvm = Path(os.environ.get("SOPLANG_LLVM_PREFIX") or output("brew", "--prefix", "llvm@22"))
    lld = Path(os.environ.get("SOPLANG_LLD_PREFIX") or output("brew", "--prefix", "lld@22"))
    env = dict(os.environ, SOPLANG_LLVM_CONFIG=str(llvm / "bin/llvm-config"))
    with tempfile.TemporaryDirectory(prefix="soplang-macos-build-") as temp_dir:
        temp = Path(temp_dir)
        subprocess.run(["cargo", "build", "--release", "--locked", "--lib"], cwd=ROOT, env=env, check=True)
        archive = temp / "libsoplang.a"
        shutil.copy2(ROOT / "target/release/libsoplang.a", archive)
        subprocess.run(["cargo", "build", "--release", "--locked", "--features", "embedded-llvm",
                        "--bin", "soplang"], cwd=ROOT, env=env, check=True)

        stage = temp / "package"
        runtime = stage / "lib/soplang"
        libraries = runtime / "llvm"
        tools = runtime / "tools"
        sdk = runtime / "sdk"
        for directory in (libraries, tools, sdk / "usr/lib"):
            directory.mkdir(parents=True)
        binary = stage / "soplang"
        linker = tools / "ld64.lld"
        shutil.copy2(ROOT / "target/release/soplang", binary)
        shutil.copy2(lld / "bin/ld64.lld", linker)
        shutil.copy2(archive, runtime / archive.name)
        system_stub = ROOT / "third_party/zig/libSystem.tbd"
        shutil.copy2(system_stub, sdk / "usr/lib/libSystem.tbd")
        # Framework stubs re-export this install name, not libSystem.dylib.
        shutil.copy2(system_stub, sdk / "usr/lib/libSystem.B.tbd")
        generated = ROOT / "third_party/apple-sdk-gen"
        for stub in (generated / "usr/lib").glob("*.tbd"):
            shutil.copy2(stub, sdk / "usr/lib" / stub.name)
        for stub in (generated / "Frameworks").glob("*.tbd"):
            destination = sdk / "System/Library/Frameworks" / (stub.stem + ".framework")
            destination.mkdir(parents=True, exist_ok=True)
            extra = {
                "CoreFoundation": ("_CFRelease", "_CFRetain"),
                "Security": ("_AuthorizationExecuteWithPrivileges",),
            }.get(stub.stem, ())
            copy_stub(stub, destination / stub.name, extra)

        search = [llvm / "lib", lld / "lib", Path("/opt/homebrew/lib")]
        pending = [binary, linker]
        visited = set()
        while pending:
            item = pending.pop()
            if item in visited:
                continue
            visited.add(item)
            for dependency in dependencies(item):
                if dependency.startswith(("/usr/lib/", "/System/Library/")):
                    continue
                name = Path(dependency).name
                if item.parent == libraries and name == item.name:
                    continue
                source = Path(dependency)
                if dependency.startswith("@loader_path/"):
                    source = item.parent / dependency.removeprefix("@loader_path/")
                if not source.is_file():
                    source = next((directory / name for directory in search
                                   if (directory / name).is_file()), None)
                if source is None or not source.is_file():
                    raise RuntimeError(f"cannot package macOS library {dependency} needed by {item}")
                destination = libraries / name
                if not destination.exists():
                    shutil.copy2(source, destination)
                    pending.append(destination)
                subprocess.run(["install_name_tool", "-change", dependency, f"@rpath/{name}", str(item)],
                               check=True)
            if item == linker:
                subprocess.run(["install_name_tool", "-add_rpath", "@loader_path/../llvm", str(item)], check=True)
            elif item.parent == libraries:
                subprocess.run(["install_name_tool", "-id", f"@rpath/{item.name}", str(item)], check=True)
                subprocess.run(["install_name_tool", "-add_rpath", "@loader_path", str(item)], check=True)
            subprocess.run(["codesign", "--force", "--sign", "-", str(item)], check=True)

        for name in ("LICENSE", "README.md"):
            shutil.copy2(ROOT / name, stage / name)
        shutil.copy2(ROOT / "docs/INSTALLING_PACKAGES.md", stage / "INSTALLING.md")
        shutil.copytree(ROOT / "licenses", stage / "licenses")
        shutil.copy2(ROOT / "third_party/zig/LICENSE", stage / "licenses/ZIG.txt")
        shutil.copy2(generated / "LICENSE", stage / "licenses/APPLE-SDK-GEN.txt")
        output_path = args.output.resolve()
        output_path.parent.mkdir(parents=True, exist_ok=True)
        if output_path.exists():
            shutil.rmtree(output_path)
        shutil.move(str(stage), output_path)
    print(f"Packaged native macOS Soplang compiler in {output_path}")


if __name__ == "__main__":
    main()
