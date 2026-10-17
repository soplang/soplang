#!/usr/bin/env python3
"""Compare equivalent Python, Soplang v2, v3 JIT, and v3 LLVM programs."""

import argparse
from datetime import datetime, timezone
import hashlib
import io
import json
import math
import os
from pathlib import Path
import platform
import random
import re
import statistics
import subprocess
import sys
import tarfile
import tempfile
import time
import csv

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
ENGINES = {
    "python": "Python",
    "soplang2": "Soplang 2 interpreter",
    "soplang3_jit": "Soplang 3 JIT",
    "soplang3_llvm": "Soplang 3 LLVM",
}


def checked_run(command, expected=None, *, env=None, timeout=120):
    start = time.perf_counter_ns()
    result = subprocess.run(
        list(map(str, command)), cwd=ROOT, env=env, stdin=subprocess.DEVNULL,
        stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True,
        encoding="utf-8", timeout=timeout,
    )
    elapsed = time.perf_counter_ns() - start
    if result.returncode or (expected is not None and
                             (result.stdout != expected or result.stderr)):
        raise RuntimeError(
            f"Command failed validation: {result.args!r}\n"
            f"exit={result.returncode}\nexpected={expected!r}\n"
            f"stdout={result.stdout!r}\nstderr={result.stderr!r}"
        )
    return result.stdout, elapsed


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def summarize(samples):
    quartiles = statistics.quantiles(samples, n=4, method="inclusive")
    return {
        "samples": len(samples), "median_ns": statistics.median(samples),
        "mean_ns": statistics.mean(samples), "stdev_ns": statistics.stdev(samples),
        "min_ns": min(samples), "max_ns": max(samples),
        "p25_ns": quartiles[0], "p75_ns": quartiles[2],
    }


def export_v2(revision, destination):
    commit = checked_run(["git", "rev-parse", "--verify", f"{revision}^{{commit}}"])[0].strip()
    archive = subprocess.run(
        ["git", "archive", commit, "src"], cwd=ROOT, capture_output=True, check=True,
    ).stdout
    with tarfile.open(fileobj=io.BytesIO(archive)) as source:
        source.extractall(destination, filter="data")
    version_file = destination / "src/core/version.py"
    version = re.search(r'^VERSION = "([^"]+)"', version_file.read_text(), re.MULTILINE)
    if not version:
        raise RuntimeError("Selected revision does not contain the v2 interpreter")
    return {"ref": revision, "commit": commit, "reported_version": version[1],
            "archive_sha256": hashlib.sha256(archive).hexdigest(),
            "launcher_sha256": digest(HERE / "run_v2.py")}


def build_v3(env):
    command = ["cargo", "build", "--release", "--locked", "--lib", "--bin", "soplang",
               "--message-format=json"]
    output, elapsed = checked_run(command, env=env, timeout=900)
    for line in output.splitlines():
        message = json.loads(line)
        if (message.get("reason") == "compiler-artifact" and message.get("executable")
                and message["target"]["name"] == "soplang"):
            executable = Path(message["executable"]).resolve()
    if "executable" not in locals():
        raise RuntimeError("Cargo did not report the Soplang executable")
    return executable, {"command": command, "elapsed_ns": elapsed}


def machine_info():
    cpu = platform.processor()
    cpuinfo = Path("/proc/cpuinfo")
    if cpuinfo.exists():
        match = re.search(r"^model name\s*:\s*(.+)$", cpuinfo.read_text(), re.MULTILINE)
        if match:
            cpu = match[1]
    result = {"platform": platform.platform(), "cpu": cpu, "logical_cpus": os.cpu_count()}
    if hasattr(os, "sched_getaffinity"):
        result["cpu_affinity"] = sorted(os.sched_getaffinity(0))
    if hasattr(os, "getloadavg"):
        result["load_average_at_start"] = os.getloadavg()
    return result


def source_info():
    paths = [ROOT / name for name in ("Cargo.toml", "Cargo.lock", "rust-toolchain.toml")]
    paths += sorted((ROOT / "src").rglob("*.rs"))
    value = hashlib.sha256()
    for path in paths:
        value.update(str(path.relative_to(ROOT)).encode() + b"\0" + path.read_bytes() + b"\0")
    return {
        "commit": checked_run(["git", "rev-parse", "HEAD"])[0].strip(),
        "compiler_sources_sha256": value.hexdigest(),
        "compiler_changes": checked_run([
            "git", "status", "--porcelain", "--untracked-files=all", "--",
            "src", "Cargo.toml", "Cargo.lock", "rust-toolchain.toml",
        ])[0].splitlines(),
    }


def markdown(data):
    config = data["configuration"]
    lines = [
        "# Python vs Soplang 2 vs Soplang 3", "",
        f"Measured {data['started_at']} on {data['machine']['cpu']}.", "",
        f"- OS: `{data['machine']['platform']}`.",
        f"- Python baseline and v2 host: `{data['python']['version']}` "
        f"({data['python']['implementation']}).",
        f"- Python build: debug={data['python']['debug']}, "
        f"GIL enabled={data['python']['gil_enabled']}, JIT enabled={data['python']['jit_enabled']}.",
        f"- v2 source: `{data['v2']['ref']}` at `{data['v2']['commit']}`; "
        f"its internal version is `{data['v2']['reported_version']}`.",
        f"- v3: `{data['v3']['version']}` at `{data['v3']['commit']}`; "
        f"Cargo release build; LLVM optimization level {config['opt_level']}.",
        f"- {config['samples']} measured runs and {config['warmups']} warmups per cell; "
        f"interleaved engine order, random seed {config['seed']}.",
        f"- CPU affinity: `{data['machine'].get('cpu_affinity', 'OS default')}`; "
        f"initial load average: `{data['machine'].get('load_average_at_start', 'unavailable')}`.",
        "", "## Median process wall time", "",
        "Milliseconds; lower is better. Each cell is a fresh process, including startup, "
        "source reading, parsing, execution, and output capture. v3 JIT includes compilation. "
        "LLVM executable timing excludes its separate build step.", "",
        "| Workload | Python | Soplang 2 | Soplang 3 JIT | Soplang 3 LLVM |",
        "| --- | ---: | ---: | ---: | ---: |",
    ]
    for row in data["workloads"]:
        times = [f"{row['engines'][engine]['median_ns'] / 1e6:.3f}" for engine in ENGINES]
        lines.append(f"| {row['name']} | " + " | ".join(times) + " |")
    lines += [
        "", "## Ratios of median times", "",
        "The numerator is the reference language's median. Values above 1 mean v3 "
        "finished sooner; values below 1 mean v3 finished later. These ratios include "
        "startup and must not be read as execution-only speedups.", "",
        "| Workload | v2 / v3 JIT | v2 / v3 LLVM | Python / v3 JIT | Python / v3 LLVM |",
        "| --- | ---: | ---: | ---: | ---: |",
    ]
    for row in data["workloads"]:
        med = {engine: row["engines"][engine]["median_ns"] for engine in ENGINES}
        ratios = [med[ref] / med[engine] for ref, engine in (
            ("soplang2", "soplang3_jit"), ("soplang2", "soplang3_llvm"),
            ("python", "soplang3_jit"), ("python", "soplang3_llvm"),
        )]
        lines.append(f"| {row['name']} | " + " | ".join(f"{ratio:.2f}×" for ratio in ratios) + " |")
    lines += ["", "## Sample spread", "",
              "Interquartile range (25th–75th percentile) in milliseconds. No outliers removed.", "",
              "| Workload | Python | Soplang 2 | Soplang 3 JIT | Soplang 3 LLVM |",
              "| --- | ---: | ---: | ---: | ---: |"]
    for row in data["workloads"]:
        spreads = []
        for engine in ENGINES:
            stat = row["engines"][engine]
            spreads.append(f"{stat['p25_ns']/1e6:.3f}–{stat['p75_ns']/1e6:.3f}")
        lines.append(f"| {row['name']} | " + " | ".join(spreads) + " |")
    lines += ["", "## LLVM build costs", "",
              "One observed build per workload, in seconds. Not a statistical benchmark; "
              "toolchain startup, linking, and existing build caches affect these values.", "",
              "| Workload | Build seconds |", "| --- | ---: |"]
    for row in data["workloads"]:
        lines.append(f"| {row['name']} | {row['llvm_build_ns']/1e9:.3f} |")
    lines += ["", "## Workloads and expected output", "",
              "| Workload | Work performed | Output |", "| --- | --- | ---: |"]
    for row in data["workloads"]:
        lines.append(f"| {row['name']} | {row['description']} | `{row['expected'].strip()}` |")
    lines += ["", "## Interpretation and limits", "",
              "- The same `.sop` files run in v2 and v3. Python uses explicit equivalent loops, "
              "recursion, append/index operations, and dictionaries; no closed-form replacements.",
              "- The v2 launcher calls the tag's original `run_soplang_file` pipeline, including "
              "normal package imports. It avoids initializing the interactive shell and writing "
              "history. It does not modify or port the interpreter.",
              "- `integer_sum` uses dynamic `door`; `integer_sum_typed` uses `abn`. "
              "The Python programs are identical for these two rows. `float_sum_typed` uses "
              "`jajab` and exact halves, converting the final result to an integer for validation.",
              "- Short programs are sensitive to process and interpreter startup. Warmups warm "
              "filesystem/bytecode caches; each measured process starts a fresh VM/JIT. "
              "No startup subtraction, timing inside a persistent VM, or manual disabling of GC.",
              "- Results describe these workloads on one shared workstation, not all programs. "
              "CPU frequency, background work, allocation strategies, garbage collection, and "
              "the chosen Python build affect results. The suite does not measure peak memory "
              "or sustained server workloads; unreclaimed runtime allocations can favor short runs.",
              "- Every warmup and measured process must exit successfully with exactly the "
              "expected stdout and no stderr. Failures abort the comparison.",
              "", "Raw samples, hashes, commands, tool versions, and summary statistics are in "
              "[results.json](results.json) and [samples.csv](samples.csv). "
              "See [README.md](README.md) to reproduce the run.", ""]
    return "\n".join(lines)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--python", default=sys.executable, help="Python used for both baseline and v2")
    parser.add_argument("--v2-ref", default="v2.0.0")
    parser.add_argument("--v3-bin", type=Path, help="Use an existing v3 executable instead of building")
    parser.add_argument("--samples", type=int, default=25)
    parser.add_argument("--warmups", type=int, default=5)
    parser.add_argument("--timeout", type=float, default=90, help="Timeout for each program in seconds")
    parser.add_argument("--seed", type=int, default=1729)
    parser.add_argument("--cpu", type=int, help="Pin this runner and its children to one Linux logical CPU")
    parser.add_argument("--opt-level", type=int, choices=range(4), default=2)
    parser.add_argument("--only", nargs="+", help="Workload names for a focused run")
    parser.add_argument("--output", type=Path, default=HERE)
    args = parser.parse_args()
    if args.samples < 3 or args.warmups < 0 or not math.isfinite(args.timeout) or args.timeout <= 0:
        parser.error("Use at least 3 samples, nonnegative warmups, and a positive finite timeout")
    if args.cpu is not None:
        if not hasattr(os, "sched_setaffinity") or args.cpu not in os.sched_getaffinity(0):
            parser.error("--cpu must select an available Linux logical CPU")
        os.sched_setaffinity(0, {args.cpu})
    env = dict(os.environ, PYTHONHASHSEED="0", NO_COLOR="1")
    env.pop("SOPLANG_RUNTIME_DIR", None)
    started = datetime.now(timezone.utc).isoformat()
    machine = machine_info()
    python_probe = (
        "import json,platform,sys,sysconfig; print(json.dumps(dict("
        "version=sys.version, executable=sys.executable, implementation=platform.python_implementation(),"
        "compiler=platform.python_compiler(), configure_args=sysconfig.get_config_var('CONFIG_ARGS'),"
        "debug=bool(sysconfig.get_config_var('Py_DEBUG')),"
        "gil_enabled=getattr(sys,'_is_gil_enabled',lambda:True)(),"
        "jit_enabled=hasattr(sys,'_jit') and sys._jit.is_enabled())))"
    )
    python = json.loads(checked_run([args.python, "-c", python_probe], env=env)[0])
    workloads = json.loads((HERE / "workloads.json").read_text())
    if args.only:
        unknown = set(args.only) - {row["name"] for row in workloads}
        if unknown:
            parser.error(f"Unknown workloads: {sorted(unknown)}")
        workloads = [row for row in workloads if row["name"] in args.only]
    print("Building/checking v3...", flush=True)
    if args.v3_bin:
        binary, build = args.v3_bin.resolve(strict=True), None
    else:
        binary, build = build_v3(env)
    version = checked_run([binary, "--version"], env=env)[0].strip()
    if not re.fullmatch(r"soplang 3\.[\w.+-]+", version):
        raise RuntimeError(f"Expected a v3 executable, got {version!r}")
    v3 = {**source_info(), "version": version, "binary": str(binary),
          "binary_sha256": digest(binary), "build": build,
          "source_attribution": "working tree" if build else "external binary; source revision unverified"}
    if args.v3_bin:
        v3["commit"] = "unverified external binary"
    tool_versions = {}
    for tool in ("rustc", "cargo", "clang"):
        tool_versions[tool] = checked_run([tool, "--version"], env=env)[0].strip()
    data = {
        "schema_version": 1, "started_at": started, "machine": machine,
        "python": python, "v3": v3, "tool_versions": tool_versions,
        "configuration": {**vars(args), "python": python["executable"],
                          "output": str(args.output.resolve()),
                          "v3_bin": str(binary), "environment": {"PYTHONHASHSEED": "0", "NO_COLOR": "1"}},
        "runner_sha256": digest(Path(__file__)),
        "manifest_sha256": digest(HERE / "workloads.json"), "workloads": [],
    }
    cache = ROOT / "target/benchmark-comparison"
    cache.mkdir(parents=True, exist_ok=True)
    rng = random.Random(args.seed)
    all_samples = []
    with tempfile.TemporaryDirectory(prefix="run-", dir=cache) as directory:
        work = Path(directory)
        snapshot = work / "v2"
        snapshot.mkdir()
        data["v2"] = export_v2(args.v2_ref, snapshot)
        for case in workloads:
            name = case["name"]
            print(f"Benchmarking {name}...", flush=True)
            sop = HERE / "programs" / f"{name}.sop"
            py = sop.with_suffix(".py")
            native = work / (name + (".exe" if os.name == "nt" else ""))
            build_command = [str(binary), "--build", "--quiet", "--opt-level", str(args.opt_level),
                             str(sop), "-o", str(native)]
            _, build_ns = checked_run(build_command, env=env, timeout=args.timeout)
            commands = {
                "python": [python["executable"], str(py)],
                "soplang2": [python["executable"], str(HERE / "run_v2.py"), str(snapshot), str(sop)],
                "soplang3_jit": [str(binary), str(sop)], "soplang3_llvm": [str(native)],
            }
            values = {engine: [] for engine in ENGINES}
            for round_index in range(-args.warmups, args.samples):
                order = list(ENGINES)
                rng.shuffle(order)
                for engine in order:
                    _, elapsed = checked_run(commands[engine], case["expected"], env=env, timeout=args.timeout)
                    if round_index >= 0:
                        values[engine].append(elapsed)
                        all_samples.append({"workload": name, "engine": engine,
                                            "round": round_index, "elapsed_ns": elapsed,
                                            "execution_order": len(all_samples)})
            row = {**case, "commands": commands, "llvm_build_command": build_command,
                   "llvm_build_ns": build_ns, "source_sha256": {"soplang": digest(sop), "python": digest(py)},
                   "engines": {engine: summarize(samples) for engine, samples in values.items()}}
            data["workloads"].append(row)
            print("  " + ", ".join(f"{engine}: {row['engines'][engine]['median_ns']/1e6:.3f} ms"
                                    for engine in ENGINES), flush=True)
    data["finished_at"] = datetime.now(timezone.utc).isoformat()
    data["samples"] = all_samples
    args.output.mkdir(parents=True, exist_ok=True)
    (args.output / "results.json").write_text(json.dumps(data, indent=2) + "\n", encoding="utf-8")
    (args.output / "RESULTS.md").write_text(markdown(data), encoding="utf-8")
    with (args.output / "samples.csv").open("w", newline="", encoding="utf-8") as stream:
        writer = csv.DictWriter(stream, fieldnames=list(all_samples[0]))
        writer.writeheader()
        writer.writerows(all_samples)
    print(f"Wrote results to {args.output.resolve()}", flush=True)


if __name__ == "__main__":
    main()
