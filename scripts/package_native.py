#!/usr/bin/env python3
"""Build a Linux x86_64 Soplang package with embedded LLVM and LLD."""

import argparse
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys
import tempfile

ROOT = Path(__file__).resolve().parents[1]
SYSTEM_LIBRARIES = {
    "libc.so.6", "libm.so.6", "libpthread.so.0", "libdl.so.2",
    "librt.so.1", "libutil.so.1", "ld-linux-x86-64.so.2",
}


def run(*args, env=None):
    subprocess.run(args, cwd=ROOT, env=env, check=True)


def installed_package(path):
    for command in (["pacman", "-Qqo", str(path)], ["dpkg-query", "-S", str(path)]):
        if shutil.which(command[0]) is None:
            continue
        result = subprocess.run(command, capture_output=True, text=True)
        if result.returncode == 0:
            first = result.stdout.splitlines()[0]
            return first.split(": ", 1)[0].split(":", 1)[0]
    return None


def copy_license(path, licenses):
    package = installed_package(path)
    if package is None:
        return None
    destination = licenses / "third-party" / package
    candidates = (Path("/usr/share/licenses") / package,
                  Path("/usr/share/doc") / package / "copyright")
    for source in candidates:
        if source.is_dir():
            shutil.copytree(source, destination, dirs_exist_ok=True)
        elif source.is_file():
            destination.mkdir(parents=True, exist_ok=True)
            shutil.copy2(source, destination / source.name)
    return package


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, default=ROOT / "dist/soplang-native")
    parser.add_argument("--debug", action="store_true", help="build a debug package for local tests")
    args = parser.parse_args()
    if sys.platform != "linux" or os.uname().machine != "x86_64":
        parser.error("this package builder currently supports Linux x86_64 only")
    if shutil.which("patchelf") is None:
        parser.error("patchelf is required to make packaged LLVM dependencies relocatable")
    output = args.output.resolve()
    if output == ROOT or output in ROOT.parents:
        parser.error("output must be a separate package directory")
    profile = "debug" if args.debug else "release"
    mode = [] if args.debug else ["--release"]
    sdk = Path(os.environ.get("SOPLANG_LLD_SDK", "/usr"))
    if not (sdk / "include/lld/Common/Driver.h").is_file():
        parser.error("LLD headers are missing; set SOPLANG_LLD_SDK to the LLD SDK prefix")
    env = dict(os.environ)
    env["SOPLANG_LLD_SDK"] = str(sdk)
    env["LD_LIBRARY_PATH"] = str(sdk / "lib") + ":" + env.get("LD_LIBRARY_PATH", "")
    with tempfile.TemporaryDirectory(prefix="soplang-native-build-") as temp:
        temp = Path(temp)
        run("cargo", "build", "--locked", *mode, "--lib", env=env)
        archive = temp / "libsoplang.a"
        shutil.copy2(ROOT / "target" / profile / "libsoplang.a", archive)
        run("cargo", "build", "--locked", *mode, "--features", "embedded-llvm", "--bin", "soplang", env=env)
        binary = ROOT / "target" / profile / "soplang"
        stage = temp / "package"
        runtime = stage / "lib/soplang"
        libraries = runtime / "llvm"
        libraries.mkdir(parents=True)
        (runtime / "crt").mkdir()
        shutil.copy2(binary, stage / "soplang")
        shutil.copy2(archive, runtime / archive.name)
        startup = next((p for p in (
            Path("/usr/lib/Scrt1.o"), Path("/usr/lib/x86_64-linux-gnu/Scrt1.o"),
        ) if p.is_file()), None)
        if startup is None:
            parser.error("glibc startup object Scrt1.o is missing on the packaging machine")
        shutil.copy2(startup, runtime / "crt/Scrt1.o")

        # ldd reports the transitive shared-library closure. Package LLVM and
        # its non-glibc dependencies under the executable's relative RPATH.
        inspection = subprocess.check_output(["ldd", str(binary)], env=env, text=True)
        copied = {}
        for line in inspection.splitlines():
            match = re.match(r"\s*(\S+)\s+=>\s+(\/\S+)\s+\(", line)
            if not match:
                if "not found" in line:
                    raise RuntimeError(f"unresolved compiler library: {line.strip()}")
                continue
            name, source = match.groups()
            if name in SYSTEM_LIBRARIES:
                continue
            destination = libraries / name
            if destination.exists():
                continue
            shutil.copy2(source, destination)
            copied[name] = Path(source)
            # The executable's RPATH does not reliably cover dependencies of
            # Homebrew or distro LLVM shared libraries. Give each copied ELF
            # library a relative RPATH for the rest of the packaged closure.
            run("patchelf", "--force-rpath", "--set-rpath", "$ORIGIN", str(destination))
        if not any(libraries.glob("libLLVM*.so*")):
            raise RuntimeError("LLVM was not found in the compiler dependencies")
        if (sdk / "lib/liblldELF.so").is_file() and not any(libraries.glob("liblldELF*.so*")):
            raise RuntimeError("shared LLD was not found in the compiler dependencies")
        for name in ("LICENSE", "README.md"):
            shutil.copy2(ROOT / name, stage / name)
        shutil.copy2(ROOT / "docs/INSTALLING_PACKAGES.md", stage / "INSTALLING.md")
        shutil.copytree(ROOT / "licenses", stage / "licenses")
        # These licenses cover the LLVM and LLD binaries included in the archive.
        for source, name in (
            (Path("/usr/share/licenses/llvm-libs/LICENSE"), "LLVM.txt"),
            (sdk / "share/licenses/lld/LICENSE", "LLD.txt"),
            (Path("/usr/share/doc/libllvm18/copyright"), "LLVM-Ubuntu.txt"),
            (Path("/usr/share/doc/liblld-18/copyright"), "LLD-Ubuntu.txt"),
        ):
            if source.is_file():
                shutil.copy2(source, stage / "licenses" / name)
        notices = ["Bundled native libraries and their build-machine packages:\n"]
        for name, source in sorted(copied.items()):
            package = copy_license(source, stage / "licenses")
            if package is None and name.startswith("liblld"):
                package = "LLD SDK (see LLD.txt)"
            notices.append(f"{name}: {package or 'package unknown'}\n")
        startup_package = copy_license(startup, stage / "licenses")
        notices.append(f"Scrt1.o: {startup_package or 'package unknown'}\n")
        (stage / "licenses/NATIVE-LIBRARIES.txt").write_text("".join(notices), encoding="utf-8")
        output.parent.mkdir(parents=True, exist_ok=True)
        if output.exists():
            shutil.rmtree(output)
        shutil.move(str(stage), output)
    print(f"Packaged native Soplang compiler in {output}")


if __name__ == "__main__":
    main()
