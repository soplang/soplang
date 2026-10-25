#!/usr/bin/env python3
"""Build a Windows x64 Soplang package with embedded LLVM and a bundled Zig linker."""

import argparse
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile

ROOT = Path(__file__).resolve().parents[1]
TARGET = "x86_64-pc-windows-gnullvm"


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, default=ROOT / "dist/soplang-native-windows")
    args = parser.parse_args()
    if sys.platform != "win32" or os.environ.get("PROCESSOR_ARCHITECTURE", "").upper() != "AMD64":
        parser.error("this package builder requires Windows x64")
    llvm = Path(os.environ["SOPLANG_LLVM_SDK"])
    zig = Path(os.environ["SOPLANG_ZIG_SDK"])
    if not (llvm / "lib/LLVM-C.lib").is_file() or not (llvm / "bin/LLVM-C.dll").is_file():
        parser.error("the LLVM SDK needs LLVM-C.lib and LLVM-C.dll")
    if not (zig / "zig.exe").is_file() or not (zig / "lib/libc/mingw/COPYING").is_file():
        parser.error("the Zig distribution needs zig.exe and its MinGW runtime")

    with tempfile.TemporaryDirectory(prefix="soplang-windows-gnu-build-") as temp_dir:
        temp = Path(temp_dir)
        # GNU import-library lookup uses the lib*.a spelling. LLVM-C.lib is a
        # COFF import archive, so only the filename needs to change.
        imports = temp / "imports"
        imports.mkdir()
        shutil.copy2(llvm / "lib/LLVM-C.lib", imports / "libLLVM-C.a")
        env = dict(os.environ)
        env["SOPLANG_LLVM_LIB_DIR"] = str(imports)
        env["PATH"] = str(llvm / "bin") + os.pathsep + env["PATH"]
        linker = temp / "zig-link.cmd"
        linker.write_text(
            f'@echo off\r\n"{zig / "zig.exe"}" cc -target x86_64-windows-gnu %*\r\n',
            encoding="utf-8",
        )
        env["CARGO_TARGET_X86_64_PC_WINDOWS_GNULLVM_LINKER"] = str(linker)
        subprocess.run(["rustup", "target", "add", TARGET], cwd=ROOT, env=env, check=True)
        mode = ["cargo", "build", "--release", "--locked", "--target", TARGET]
        subprocess.run([*mode, "--lib"], cwd=ROOT, env=env, check=True)
        release = ROOT / "target" / TARGET / "release"
        archive = temp / "libsoplang.a"
        shutil.copy2(release / archive.name, archive)
        subprocess.run([*mode, "--features", "embedded-llvm", "--bin", "soplang"],
                       cwd=ROOT, env=env, check=True)

        stage = temp / "package"
        runtime = stage / "lib/soplang"
        tools = runtime / "tools"
        tools.mkdir(parents=True)
        shutil.copy2(release / "soplang.exe", stage / "soplang.exe")
        shutil.copy2(archive, runtime / archive.name)
        shutil.copy2(llvm / "bin/LLVM-C.dll", stage / "LLVM-C.dll")
        shutil.copy2(zig / "zig.exe", tools / "zig.exe")
        shutil.copytree(zig / "lib", tools / "lib")
        for name in ("libgcc_s_seh-1.dll", "libwinpthread-1.dll", "libstdc++-6.dll"):
            source = shutil.which(name)
            if source:
                shutil.copy2(source, stage / name)
        for name in ("LICENSE", "README.md"):
            shutil.copy2(ROOT / name, stage / name)
        shutil.copy2(ROOT / "docs/INSTALLING_PACKAGES.md", stage / "INSTALLING.md")
        shutil.copytree(ROOT / "licenses", stage / "licenses")
        shutil.copy2(ROOT / "third_party/llvm/LICENSE.TXT", stage / "licenses/LLVM.txt")
        shutil.copy2(zig / "LICENSE", stage / "licenses/ZIG.txt")
        shutil.copy2(zig / "lib/libc/mingw/COPYING", stage / "licenses/MINGW.txt")
        output = args.output.resolve()
        output.parent.mkdir(parents=True, exist_ok=True)
        if output.exists():
            shutil.rmtree(output)
        shutil.move(str(stage), output)
    print(f"Packaged native Windows GNU Soplang compiler in {output}")


if __name__ == "__main__":
    main()
