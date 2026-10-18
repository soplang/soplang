#!/usr/bin/env python3
"""Exercise an installed Soplang package outside its source checkout."""

import argparse
from concurrent.futures import ThreadPoolExecutor
import os
from pathlib import Path
import re
import shutil
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[1]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("executable", type=Path)
    args = parser.parse_args()
    original = args.executable.resolve(strict=True)
    runtime = original.parent / "lib/soplang"
    if not runtime.is_dir():
        runtime = original.parent.parent / "lib/soplang"
    if not runtime.is_dir():
        parser.error("the supplied executable has no packaged runtime")
    env = dict(os.environ, NO_COLOR="1", CARGO_TARGET_DIR="/nonexistent-soplang-target")
    env.pop("SOPLANG_RUNTIME_DIR", None)
    env.pop("SOPLANG_ERROR_DETAILS", None)
    suffix = ".exe" if os.name == "nt" else ""

    with tempfile.TemporaryDirectory(prefix="soplang installed check ") as temp:
        work = Path(temp)
        install = work / "installed package"
        install.mkdir()
        exe = install / original.name
        shutil.copy2(original, exe)
        shutil.copytree(runtime, install / "lib/soplang")
        shutil.copytree(ROOT / "examples", work / "examples")
        (work / "outputs").mkdir()

        def run(arguments, *, input="", executable=None, extra_env=None):
            return subprocess.run(
                [str(executable or exe), *map(str, arguments)], cwd=work,
                env=env | (extra_env or {}), input=input, text=True,
                encoding="utf-8", stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                timeout=120,
            )

        def success(result, expected=None):
            assert result.returncode == 0, result.stderr
            assert not result.stderr, result.stderr
            if expected is not None:
                assert result.stdout == expected, (result.stdout, expected)

        def build(path, flags=()):
            binary = work / "outputs" / (path.stem + suffix)
            result = run(["--no-color", "--build", "--quiet", *flags, path, "-o", binary])
            return binary, result

        version = run(["--version"])
        success(version)
        assert re.match(r"soplang 3\.", version.stdout), version.stdout
        help_output = run(["--help"])
        success(help_output)
        assert "Adeegsiga:" in help_output.stdout and "Xulashooyinka:" in help_output.stdout
        assert "Usage:" not in help_output.stdout and "[default:" not in help_output.stdout
        invalid = run(["--not-a-soplang-option"])
        assert invalid.returncode == 2 and "lama aqoonsan" in invalid.stderr
        assert not invalid.stdout and "unexpected argument" not in invalid.stderr
        failure = run(["missing-script.sop"])
        assert failure.returncode == 1 and "lama helin" in failure.stderr
        assert "No such file" not in failure.stderr and "safka 0" not in failure.stderr
        success(run(["-c", "1 + 2"]), "3\n")
        repl = run([], input='door n = 4\nn + 1\n/bixi\n')
        assert repl.returncode == 0 and re.search(r"(?m)^5\s*$", repl.stdout.replace("soplang> ", "")), repl
        examples = sorted(path.relative_to(work) for path in (work / "examples").glob("[0-9]*.sop"))
        assert examples, "no public examples found"
        for path in examples:
            flags = ["--strict"] if path.stem == "23_strict_types" else []
            stdin = "TestUser\n" if path.stem == "13_input" else ""
            expected = (work / path.with_suffix(".expected")).read_text(encoding="utf-8")
            success(run(["--no-color", *flags, path], input=stdin), expected)
            binary, result = build(path, flags)
            success(result)
            success(run([], input=stdin, executable=binary), expected)

        # Both compilers receive identical output basenames concurrently. Their
        # intermediate .ll/.o/wrapper files must be isolated as well.
        def concurrent_build(index):
            output = work / "concurrent" / str(index) / ("same-name" + suffix)
            result = run(["--build", "--quiet", "examples/27_inheritance.sop", "-o", output])
            success(result)
            success(run([], executable=output), "Bilan: Mia\nBilan\n")

        with ThreadPoolExecutor(max_workers=2) as pool:
            list(pool.map(concurrent_build, range(2)))

        count = 0
        manifest = (work / "examples/errors/cases.tsv").read_text(encoding="utf-8")
        for row in manifest.splitlines():
            if not row or row.startswith("#"):
                continue
            phase, name, line, col, message = row.split("\t")
            path = Path("examples/errors") / name
            flags = ["--strict"] if phase == "strict" else []
            expected = (work / path.with_suffix(".stderr")).read_text(encoding="utf-8").rstrip()

            def diagnostic(result):
                assert result.returncode == 1 and not result.stdout, result
                normalized = "".join(
                    line.replace("\\", "/") if line.startswith("     --> ") else line
                    for line in result.stderr.splitlines(keepends=True)
                )
                assert normalized.rstrip() == expected, (path, result.stderr, expected)
                assert f"safka {line}, tiirka {col}" in result.stderr
                assert message in result.stderr

            diagnostic(run(["--no-color", *flags, path]))
            binary, result = build(path, flags)
            if phase == "runtime":
                success(result)
                diagnostic(run([], executable=binary))
            else:
                diagnostic(result)
                assert not binary.exists(), binary
            count += 1

        # Test the Unix prefix/bin + prefix/lib layout too (without duplicating
        # the relatively large runtime directory).
        bin_dir = install / "bin"
        bin_dir.mkdir()
        moved = bin_dir / original.name
        exe.rename(moved)
        exe = moved
        sample = Path("examples/27_inheritance.sop")
        _, result = build(sample)
        success(result)
        missing = run(
            ["--build", sample], extra_env={"SOPLANG_RUNTIME_DIR": str(work / "missing runtime")}
        )
        assert missing.returncode != 0 and "Xogta nooca Rust" in missing.stderr
        version_path = install / "lib/soplang/rustc-version.txt"
        recorded = version_path.read_text(encoding="utf-8")
        version_path.write_text("rustc incompatible-test-version\n", encoding="utf-8")
        _, mismatch = build(sample)
        assert mismatch.returncode != 0 and "Nooca Rust ee rakiban kuma habboona" in mismatch.stderr
        version_path.write_text(recorded, encoding="utf-8")

        # A native output must continue to work without source or runtime files.
        retained = work / "outputs" / ("18_closures" + suffix)
        expected = (work / "examples/18_closures.expected").read_text(encoding="utf-8")
        shutil.rmtree(work / "examples")
        shutil.rmtree(install / "lib")
        success(run([], executable=retained), expected)
        failure = run(["--build", "does-not-exist.sop"])
        assert failure.returncode != 0
        # Parse a valid source so this checks discovery, rather than file I/O.
        (work / "simple.sop").write_text('qor(1)\n', encoding="utf-8")
        missing = run(["--build", "simple.sop"])
        assert missing.returncode != 0 and "Maktabadaha fulinta Soplang lama helin" in missing.stderr
        success(run(["-c", "1 + 2"], extra_env={"PATH": str(work / "no tools")}), "3\n")

    print(f"Installed package verified: version, REPL, {len(examples)} examples and {count} diagnostics in JIT/LLVM; runtime discovery, toolchain mismatch, and standalone execution.")


if __name__ == "__main__":
    main()
