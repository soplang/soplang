# Soplang

> The Somali Programming Language

Soplang is a programming language with syntax inspired by Somali, making programming more accessible to Somali speakers. It combines static and dynamic typing in one language with a focus on clarity and ease of use.

**The implementation is in Rust.** Soplang runs through a compiled pipeline: Cranelift JIT for `run` and an AOT build path for standalone binaries. See [COMPILER_PLAN.md](COMPILER_PLAN.md).

## Project structure

| Path | Description |
|------|-------------|
| **src/** | Rust implementation: lexer, parser, semantic, HIR, runtime, Cranelift JIT, and AOT build backend. |
| **examples/** | Soplang example programs (`.sop` files). |
| **benchmarks/** | [Python/v2/v3 comparison](benchmarks/comparison/RESULTS.md), paired workloads, raw timings, charts, and internal Criterion benchmarks. |
| **legacy-interpreter/** | Legacy interpreter code, now in the separate repo [`soplang/soplang-interpreter`](https://github.com/soplang/soplang-interpreter). |
| **docs/** | Documentation (installation, language reference, contributing). |
| **windows/**, **linux/**, **macos/** | Platform-specific build and packaging scripts. |

## Features

- **Dual type system** — Static typing (`abn`, `qoraal`, etc.) and dynamic typing (`door`)
- **Somali-based syntax** — Keywords and concepts in Somali
- **Modern paradigms** — Functional, procedural, and object-oriented support
- **Interactive shell** — one Cranelift session, so functions, classes, and globals survive across lines
- **Compiled execution** — Cranelift JIT for running files; `--build` produces a standalone binary

## Example

```sop
qor("Salaan, Adduunka!")   // Hello, World!

door magac = "Sharafdin"
abn age = 25

hawl salaam(qof) {
    celi "Salaan, " + qof + "!"
}
qor(salaam(magac))
```

## Running Soplang

Build and run:

```bash
cargo build --release

./target/release/soplang examples/01_hello.sop     # JIT run
./target/release/soplang -i    # REPL
./target/release/soplang -c 'qor("Salaan!")'
./target/release/soplang --build examples/01_hello.sop -o hello_aot
./hello_aot
```

Or: `make build`, `make run FILE=examples/01_hello.sop`, `make shell`, `make test`, `make bench`.

### CLI notes

- `soplang <file.sop>`: run via Cranelift JIT.
- `soplang --build <file.sop> -o <out>`: build a standalone native binary.
- `soplang --build ... --opt-level 0..3`: tune AOT optimization level.
- `soplang --strict`: enable stricter static typing checks.

### How a program runs

`soplang file.sop` and the REPL share one pipeline: lex, parse, resolve `keen`, analyze, lower to HIR, then Cranelift. Static `abn` and `jajab` arithmetic is native integer and float code. `door` and other dynamic values, including string concatenation, go through the tagged runtime.

`soplang --build file.sop` resolves `keen`, lowers the program to HIR, emits textual LLVM IR, and links a native binary with system `clang` (15+) and the matching `rustc`. Installed packages include the required runtime libraries; keep the package intact. See [package installation](docs/INSTALLING_PACKAGES.md). `--opt-level` selects optimization level 0–3. The binary runs without Soplang or the source files beside it.

## Benchmark results

The [Python vs Soplang 2 vs Soplang 3 comparison](benchmarks/comparison/RESULTS.md)
measures ten equivalent workloads with checked output: Python, the tagged v2
interpreter, v3 Cranelift JIT, and v3 LLVM executables. It includes
[raw samples](benchmarks/comparison/samples.csv), a [chart](benchmarks/comparison/results.svg),
and [reproduction instructions](benchmarks/comparison/README.md).

Times include process startup. JIT compilation is included; LLVM build time is
reported separately. Results apply to the measured workloads and machine.

`cargo bench` runs the separate in-process Criterion suite. See
[benchmarks/README.md](benchmarks/README.md) for both suites and the historical
compiler-only measurements.

## Legacy interpreter

The legacy tree‑walking interpreter has been moved to a dedicated repository:

- [`soplang/soplang-interpreter`](https://github.com/soplang/soplang-interpreter)

All new development (compiler, JIT, AOT, CLI) happens here in the Rust implementation.

## Documentation

- [Getting started](docs/index.md)
- [Build guide](docs/BUILD_GUIDE.md) — Build from source on Windows, macOS, and Linux
- [Language reference](docs/language/keywords.md)
- [Migrating to Soplang 3](docs/MIGRATION_V3.md)
- [Soplang 3 roadmap](ROADMAP.md) — Planned `3.x` language, runtime, library, and tooling work
- [Installation](docs/installation.md)
- [Contributing](docs/CONTRIBUTING.md)
- [Rust implementation plan (complete)](IMPLEMENTATION_PLAN.md) — Phases 1–7 done
- [Compiler plan](COMPILER_PLAN.md) — Cranelift run path and LLVM AOT build path

## License

This project is licensed under the [MIT License](LICENSE).
