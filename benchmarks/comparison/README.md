# Python vs Soplang 2 vs Soplang 3

This suite compares equivalent programs in four modes:

| Mode | Execution path |
| --- | --- |
| Python | CPython running a `.py` file |
| Soplang 2 | The original interpreter exported from the local `v2.0.0` tag |
| Soplang 3 JIT | Cargo release compiler running the same `.sop` file through Cranelift |
| Soplang 3 LLVM | An executable built from that `.sop` file with `--build --opt-level 2` |

See [measured results](RESULTS.md), the [chart](results.svg),
[raw samples](samples.csv), and [full metadata](results.json).

## Reproduce

Requirements: Python 3.12 or newer, the repository's pinned Rust toolchain,
clang, a native linker, and a local `v2.0.0` tag. No checkout switch is needed;
the interpreter is exported into an isolated directory below ignored `target/`.
It is never copied into the v3 compiler or modified.

From the repository root on Linux/macOS:

```sh
python3 -m venv target/benchmark-comparison/venv
target/benchmark-comparison/venv/bin/python -m pip install -r benchmarks/comparison/requirements.txt
target/benchmark-comparison/venv/bin/python benchmarks/comparison/run.py
```

On Windows use `target/benchmark-comparison/venv/Scripts/python.exe` instead.
Native LLVM builds require a Visual Studio developer shell, clang, and the
matching Rust toolchain, as described in [package requirements](../../docs/INSTALLING_PACKAGES.md).

The defaults are **25 measured runs and 5 warmups per mode per workload**.
Each process must produce exactly the expected stdout, no stderr, and exit 0.
The benchmark fails immediately on an incorrect result or timeout.

The recorded workstation run also pinned execution to Linux logical CPU 2:

```sh
target/benchmark-comparison/venv/bin/python benchmarks/comparison/run.py --cpu 2
```

Choose a CPU available on your machine; omit `--cpu` on other platforms.
Avoid running other CPU-intensive work during measurements. The script does
not change the machine's power settings or stop other applications.

Useful options:

```sh
# A shorter correctness/pilot run, keeping the published results intact.
target/benchmark-comparison/venv/bin/python benchmarks/comparison/run.py \
  --samples 3 --warmups 1 --output target/benchmark-comparison/pilot

# Focus on particular workloads.
target/benchmark-comparison/venv/bin/python benchmarks/comparison/run.py \
  --only fibonacci integer_sum --output target/benchmark-comparison/focused

# Compare another v2 revision or an existing v3 executable.
target/benchmark-comparison/venv/bin/python benchmarks/comparison/run.py \
  --v2-ref v2.0.0 --v3-bin /absolute/path/to/soplang \
  --output target/benchmark-comparison/custom
```

When `--v3-bin` is supplied, its source revision is explicitly unverified.
The default builds the working tree with `cargo build --release --locked --lib --bin soplang`,
records the compiler source hash and any compiler changes, and times only the
resulting executable. Cargo is never part of an execution sample.

## Chart

Plotting is separate from timing and is optional. Install its dependency into
the same virtual environment, then generate the SVG and an optional PNG:

```sh
target/benchmark-comparison/venv/bin/python -m pip install -r benchmarks/comparison/requirements-plot.txt
MPLCONFIGDIR="$PWD/target/benchmark-comparison/matplotlib" \
  target/benchmark-comparison/venv/bin/python benchmarks/comparison/plot.py \
  --png target/benchmark-comparison/results.png
```

The chart uses a logarithmic time axis, displays medians and interquartile
ranges, and includes every measured workload. Its Python label comes from the
actual measured Python version.

## Method

- The manifest fixes the work and expected result for ten programs. Soplang
  ranges are inclusive, so Python's exclusive `range` endpoints are adjusted
  explicitly. There are no library shortcuts replacing loops or recursion.
- Numeric loops run in functions in all languages, avoiding a Python global
  variable lookup penalty. Integer results stay within signed 64-bit range.
  Float additions use exactly representable halves. Object fields map to
  Python dictionary keys; list `kudar` maps to `append`.
- The same Python executable hosts the baseline and v2. Both use
  `PYTHONHASHSEED=0`. Python's debug/GIL/JIT configuration is recorded, rather
  than assuming all CPython builds behave the same way.
- Engine order is shuffled within each measurement round using a recorded
  seed. Processes run sequentially. Warmups populate filesystem and Python
  bytecode caches, but each sample has a fresh language runtime.
- Times include process creation, imports, file reading, parsing, execution,
  and output capture. v3 JIT includes JIT compilation. LLVM's separate build
  time is reported per workload, outside execution samples. No startup time
  is subtracted, and no sample is discarded as an outlier.
- The v2 launcher calls the original `src.runtime.main.run_soplang_file`.
  Package imports, including its shell import and pinned `colorama`, still
  occur. It does not construct the interactive shell or read/write history.
  The tag's internal version says `2.0.0-beta`; results identify the release
  tag, exact commit, and reported version without relabeling the source.
- JSON records every elapsed nanosecond, execution order, commands, source and
  binary hashes, source revisions, tool versions, CPU affinity, and initial
  system load. CSV contains all measured observations. Existing reports are
  overwritten only after the whole run succeeds.

These are **whole-program latency measurements**, not pure execution timings
or a claim that one language wins every task. Startup is substantial for short
Python programs; a v3 latency win there does not establish a faster inner loop.
The suite also does not measure peak memory, sustained allocation, server
throughput, or cold disk caches. Runtime memory reclamation remains a separate
concern when comparing longer-running workloads.

The older [Criterion results](../RESULTS.md) measure another revision in-process
and cannot be compared directly to these fresh-process measurements.

## Validate the tooling

```sh
target/benchmark-comparison/venv/bin/python -m unittest discover \
  -s benchmarks/comparison -p 'test_*.py' -v
```

The runner tests ensure a fast wrong answer, a failed process, and stderr never
become benchmark results, and that outliers remain in the statistics. A pilot
run checks the actual interpreter, JIT, and LLVM outputs for every workload.
