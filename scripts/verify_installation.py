#!/usr/bin/env python3
"""Test installation and removal on a clean, disposable native CI runner."""

import argparse
import os
from pathlib import Path
import re
import subprocess
import tempfile
import time

ROOT = Path(__file__).resolve().parents[1]
ENV = dict(os.environ, NO_COLOR="1")


def command(args, **kwargs):
    return subprocess.run(
        list(map(str, args)), env=ENV, text=True, encoding="utf-8",
        stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=180, **kwargs,
    )


def success(result):
    if result.returncode:
        raise RuntimeError(f"{result.args}: {result.stdout}\n{result.stderr}")
    return result.stdout


def installed_cli(executable, work):
    version = success(command([executable, "--version"], cwd=work))
    assert re.match(r"soplang 3\.", version), version
    for name in ("16_import", "18_closures", "27_inheritance"):
        source = ROOT / f"examples/{name}.sop"
        expected = source.with_suffix(".expected").read_text(encoding="utf-8")
        run = command([executable, source], cwd=work)
        assert success(run) == expected and not run.stderr, run
        binary = work / (name + (".exe" if os.name == "nt" else ""))
        success(command([executable, "--build", "--quiet", source, "-o", binary], cwd=work))
        run = command([binary], cwd=work)
        assert success(run) == expected and not run.stderr, run


def debian(package):
    present = command(["dpkg-query", "-W", "-f=${Status}", "soplang"])
    if present.returncode == 0 and "install ok installed" in present.stdout:
        raise RuntimeError("Soplang is already installed; use a clean runner")
    success(command(["sudo", "dpkg", "-i", package]))
    try:
        with tempfile.TemporaryDirectory(prefix="soplang system check ") as temp:
            installed_cli(Path("/usr/bin/soplang"), Path(temp))
    finally:
        success(command(["sudo", "dpkg", "-r", "soplang"]))
    assert not Path("/usr/bin/soplang").exists(), "executable remains after removal"
    assert not Path("/usr/lib/soplang").exists(), "runtime remains after removal"


def windows(installer):
    import winreg

    def normalized(path):
        return os.path.normcase(os.path.realpath(os.path.expandvars(str(path))))

    def path_value():
        with winreg.OpenKey(winreg.HKEY_LOCAL_MACHINE,
                            r"SYSTEM\CurrentControlSet\Control\Session Manager\Environment") as key:
            return winreg.QueryValueEx(key, "Path")

    def path_entries():
        return [normalized(entry) for entry in path_value()[0].split(";") if entry]

    original_path, original_kind = path_value()

    with tempfile.TemporaryDirectory(prefix="soplang installer check ") as temp:
        work = Path(temp)
        directory = work / "Soplang"
        success(command([installer, "/VERYSILENT", "/SUPPRESSMSGBOXES", "/NORESTART", "/SP-",
                         f"/DIR={directory}", "/TASKS=addtopath,fileassociation"]))
        try:
            installed_cli(directory / "soplang.exe", work)
            assert normalized(directory) in path_entries(), "PATH was not updated"
            assert path_value()[1] == winreg.REG_EXPAND_SZ, "PATH lost environment-variable expansion"
            for extension in (".sop", ".so"):
                with winreg.OpenKey(winreg.HKEY_CLASSES_ROOT, extension) as key:
                    assert winreg.QueryValueEx(key, "")[0] == "SoplangFile"
            with winreg.OpenKey(winreg.HKEY_CLASSES_ROOT, r"SoplangFile\shell\open\command") as key:
                association = winreg.QueryValueEx(key, "")[0]
                associated_exe = association.split('"')[1]
                assert Path(associated_exe).samefile(directory / "soplang.exe"), association
        finally:
            success(command([directory / "unins000.exe", "/VERYSILENT", "/SUPPRESSMSGBOXES", "/NORESTART"]))
        for _ in range(75):
            if not (directory / "soplang.exe").exists():
                break
            time.sleep(0.2)
        assert not (directory / "soplang.exe").exists(), "executable remains after removal"
        assert not (directory / "lib/soplang").exists(), "runtime remains after removal"
        assert normalized(directory) not in path_entries(), "PATH remains after removal"
        assert path_value()[0] == original_path, "existing PATH entries changed"
        if original_kind == winreg.REG_EXPAND_SZ:
            assert path_value()[1] == original_kind, "PATH registry type changed"
        for extension in (".sop", ".so"):
            try:
                with winreg.OpenKey(winreg.HKEY_CLASSES_ROOT, extension) as key:
                    value = winreg.QueryValueEx(key, "")[0]
                assert value != "SoplangFile", "association remains after removal"
            except FileNotFoundError:
                pass


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("platform", choices=("debian", "windows"))
    parser.add_argument("package", type=Path)
    args = parser.parse_args()
    package = args.package.resolve(strict=True)
    if args.platform == "debian":
        debian(package)
    else:
        windows(package)
    print(f"Verified {args.platform} installation, JIT/LLVM imports, closures, inheritance, and removal")


if __name__ == "__main__":
    main()
