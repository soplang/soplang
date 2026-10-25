#!/usr/bin/env python3
"""Build a Windows x64 Soplang package with LLVM-C and a bundled COFF linker."""

import argparse
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile

ROOT = Path(__file__).resolve().parents[1]
IMPORT_LIBRARIES = (
    "kernel32.lib", "advapi32.lib", "bcrypt.lib", "userenv.lib", "ws2_32.lib",
    "ntdll.lib", "ole32.lib", "shell32.lib", "secur32.lib", "crypt32.lib",
    "user32.lib", "iphlpapi.lib", "vcruntime.lib", "libvcruntime.lib",
    "ucrt.lib", "libucrt.lib", "msvcrt.lib", "libcmt.lib", "oldnames.lib",
    "legacy_stdio_definitions.lib", "synchronization.lib", "normaliz.lib",
    "user32.lib", "comdlg32.lib", "rpcrt4.lib", "version.lib", "dbghelp.lib",
)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, default=ROOT / "dist/soplang-native-windows")
    args = parser.parse_args()
    if sys.platform != "win32" or os.environ.get("PROCESSOR_ARCHITECTURE", "").upper() != "AMD64":
        parser.error("this package builder requires Windows x64")
    sdk = Path(os.environ["SOPLANG_LLVM_SDK"])
    if not (sdk / "lib/LLVM-C.lib").is_file() or not (sdk / "bin/LLVM-C.dll").is_file():
        parser.error("the LLVM SDK needs LLVM-C.lib and LLVM-C.dll")
    env = dict(os.environ)
    env["SOPLANG_LLVM_LIB_DIR"] = str(sdk / "lib")
    env["PATH"] = str(sdk / "bin") + os.pathsep + env["PATH"]
    vc = Path(os.environ["VCToolsInstallDir"]) / "lib/x64"
    kit = Path(os.environ["WindowsSdkDir"]) / "Lib" / os.environ["WindowsSDKVersion"].strip("\\/")
    search = [vc, kit / "ucrt/x64", kit / "um/x64"]
    with tempfile.TemporaryDirectory(prefix="soplang-windows-build-") as temp_dir:
        temp = Path(temp_dir)
        subprocess.run(["cargo", "build", "--release", "--locked", "--lib"], cwd=ROOT, env=env, check=True)
        archive = temp / "soplang.lib"
        shutil.copy2(ROOT / "target/release/soplang.lib", archive)
        subprocess.run(["cargo", "build", "--release", "--locked", "--features", "embedded-llvm",
                        "--bin", "soplang"], cwd=ROOT, env=env, check=True)

        stage = temp / "package"
        runtime = stage / "lib/soplang"
        imports = runtime / "native"
        tools = runtime / "tools"
        imports.mkdir(parents=True)
        tools.mkdir()
        shutil.copy2(ROOT / "target/release/soplang.exe", stage / "soplang.exe")
        shutil.copy2(archive, runtime / archive.name)
        shutil.copy2(sdk / "bin/LLVM-C.dll", stage / "LLVM-C.dll")
        shutil.copy2(sdk / "bin/lld-link.exe", tools / "lld-link.exe")
        for name in dict.fromkeys(IMPORT_LIBRARIES):
            source = next((directory / name for directory in search if (directory / name).is_file()), None)
            if source is not None:
                shutil.copy2(source, imports / name)
        if not (imports / "kernel32.lib").is_file() or not (imports / "libcmt.lib").is_file():
            raise RuntimeError("Windows SDK or MSVC runtime import libraries are incomplete")

        for name in ("LICENSE", "README.md"):
            shutil.copy2(ROOT / name, stage / name)
        shutil.copy2(ROOT / "docs/INSTALLING_PACKAGES.md", stage / "INSTALLING.md")
        shutil.copytree(ROOT / "licenses", stage / "licenses")
        output = args.output.resolve()
        output.parent.mkdir(parents=True, exist_ok=True)
        if output.exists():
            shutil.rmtree(output)
        shutil.move(str(stage), output)
    print(f"Packaged native Windows Soplang compiler in {output}")


if __name__ == "__main__":
    main()
