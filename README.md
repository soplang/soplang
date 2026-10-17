# Soplang

> The Somali Programming Language

Soplang is a programming language with syntax inspired by Somali, making programming more accessible to Somali speakers. It combines static and dynamic typing in one language with a focus on clarity and ease of use.

**Soplang 3.0.0 is available** for Linux x86_64, Windows x64, macOS Apple Silicon, and Docker. Get the [release packages](https://github.com/soplang/soplang/releases/tag/v3.0.0) or follow the [installation guide](docs/installation.md).

The implementation is in Rust. Files and the REPL use Cranelift JIT; `--build` produces standalone LLVM executables. See [COMPILER_PLAN.md](COMPILER_PLAN.md).

## Project structure

| Path | Description |
|------|-------------|
| **src/** | Rust implementation: lexer, parser, semantic, HIR, runtime, Cranelift JIT, and AOT build backend. |
| **examples/** | Soplang example programs (`.sop` files). |
| **benchmarks/** | [Python/v2/v3 comparison](benchmarks/comparison/RESULTS.md), paired workloads, raw timings, charts, and internal Criterion benchmarks. |
| **v2.0.0 tag** | Original interpreter exported by the comparison suite; reference repo [`soplang/soplang-interpreter`](https://github.com/soplang/soplang-interpreter). |
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
cargo build --release --locked

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

`soplang file.sop` and the REPL share one pipeline: lex, parse, resolve `keen`, analyze, lower to HIR, then Cranelift. Numeric operations with known static types can use native integer and float instructions. Dynamic values and remaining validation/conversion steps go through the tagged runtime; a type annotation alone does not guarantee a faster program.

`soplang --build file.sop` resolves `keen`, lowers the program to HIR, emits textual LLVM IR, and links a native binary with system `clang` (15+) and the matching `rustc`. Installed packages include the required runtime libraries; keep the package intact. See [package installation](docs/INSTALLING_PACKAGES.md). `--opt-level` selects optimization level 0–3. The binary runs without Soplang or the source files beside it.

## Benchmark results

Measured on 2026-10-17 with CPython 3.14.7 and an Intel Core i7-1185G7.
Ten equivalent workloads run in four modes, with **25 samples per cell** and every output checked.

**Median milliseconds; lower is better:**

| Workload | Python | Soplang 2 | V3 JIT | V3 LLVM |
| --- | ---: | ---: | ---: | ---: |
| Startup | 11.19 | 52.34 | 2.85 | 1.97 |
| Integer sum, 100k | 13.96 | 222.05 | 42.77 | 41.79 |
| Fibonacci, 25 | 20.88 | 1265.46 | 171.61 | 143.00 |
| List append/index | 12.16 | 80.57 | 9.74 | 8.16 |

V3 improves on v2 across the measured workloads; Python remains faster on most
compute cases. Times include process startup and JIT compilation. LLVM build
costs are separate. These are workstation results, not universal language speed claims.

[All results and findings](benchmarks/RESULTS.md) · [Chart](benchmarks/comparison/results.svg) ·
[Raw samples](benchmarks/comparison/samples.csv) · [Reproduce](benchmarks/comparison/README.md)

`cargo bench` runs the separate in-process Criterion suite. See
[benchmarks/README.md](benchmarks/README.md) for both suites and the
[historical Criterion report](benchmarks/CRITERION_RESULTS.md).

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
