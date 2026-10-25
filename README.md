# Soplang

> The Somali Programming Language

Soplang is a programming language with Somali keywords. It supports static and dynamic typing, functions, closures, and classes.

**Soplang 3.6.0 is available** for Linux x86_64, Windows x64, macOS Apple Silicon, and Docker. Get the [release packages](https://github.com/soplang/soplang/releases/tag/v3.6.0) or follow the [installation guide](docs/installation.md).

The 3.6.0 release includes the 3.1 through 3.6 implementation milestones:

- [3.1 runtime foundations](docs/releases/3.1.0.md): long-running contexts,
  structured errors, cleanup, strings, and callable expressions.
- [3.2 language foundations](docs/releases/3.2.0.md): iteration, function
  arguments, byte arrays, typed collections, tuples, sets, and numeric methods.
- [3.3 files and networking](docs/releases/3.3.0.md): file handles, structured
  data, HTTP, DNS, TCP, UDP, and a WebSocket client.
- [3.4 modules and web applications](docs/releases/3.4.0.md): Git packages,
  project initialization, the sequential web framework, and a typed data layer.
- [3.5 developer tooling](docs/releases/3.5.0.md): project commands, native
  tests, coverage, language server, VS Code extension, debugging, and profiling.
- [3.6 concurrency and servers](docs/architecture/CONCURRENCY.md): tasks,
  channels, async I/O, streaming HTTP bodies, and bounded HTTP and WebSocket
  servers.

The [roadmap](ROADMAP.md) records the completed milestones and later 3.x work.

The implementation is in Rust. Files and the REPL use Cranelift JIT; `--build` produces standalone LLVM executables. See [COMPILER_PLAN.md](COMPILER_PLAN.md).

## Create a project

```sh
soplang init my-app
cd my-app
soplang main.sop
```

Use `soplang init my-library --lib` for a library with a root `lib.sop`.
Git repositories and local paths provide packages. A root lock pins Git commits
and content hashes, and `--offline` builds from a verified cache.
See the [package reference](docs/language/packages.md) for manifests and imports.

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

- Static typing with `abn`, `qoraal`, and other type annotations; dynamic typing with `door`.
- Somali keywords and diagnostic messages with source locations.
- Functions, closures, classes, and inheritance.
- A persistent REPL that keeps functions, classes, and globals across lines.
- Cranelift JIT for running programs and LLVM for building native executables.

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
./target/release/soplang       # interactive command browser
./target/release/soplang repl  # Soplang REPL
./target/release/soplang -i    # REPL
./target/release/soplang -c 'qor("Salaan!")'
./target/release/soplang --build examples/01_hello.sop -o hello_aot
./hello_aot
```

Or: `make build`, `make run FILE=examples/01_hello.sop`, `make shell`, `make test`, `make bench`.

### Docker

The Linux amd64 image includes the compiler and its native build runtime:

```sh
docker run --platform linux/amd64 --rm soplang/soplang:3.6.0 --version
docker run --platform linux/amd64 --rm -it soplang/soplang:3.6.0
docker run --platform linux/amd64 --rm -v "$PWD:/scripts:ro" \
  soplang/soplang:3.6.0 examples/01_hello.sop
```

The file example assumes you are in this repository. See the
[Docker guide](docs/docker.md) for mounting your own programs and building
executables.

### CLI notes

Soplang accepts English and Somali command names. Each top-level action also
accepts a `--` prefix, and existing flag forms remain supported. See
[CLI commands and diagnostics](docs/language/cli.md) for the full command list,
exit statuses, source locations, and `SOPLANG_ERROR_DETAILS=1` troubleshooting
setting.

- `soplang <file.sop>`: run via Cranelift JIT.
- `soplang build <file.sop>` or `soplang dhis <file.sop>`: build a standalone
  native binary in `dhis/`, named after the source file.
- `soplang build`: build the current project using its manifest name.
- `soplang build a.sop b.sop`: build one executable per file.
- `soplang build ... --opt-level 0..3`: tune AOT optimization level.
- `soplang --strict`: enable stricter static typing checks.

### How a program runs

`soplang file.sop` and the REPL share one pipeline: lex, parse, resolve `keen`, analyze, lower to HIR, then Cranelift. Numeric operations with known static types can use native integer and float instructions. Dynamic values and remaining validation/conversion steps go through the tagged runtime; a type annotation alone does not guarantee a faster program.

`soplang --build file.sop` resolves `keen`, lowers the program to HIR, and emits textual LLVM IR. The 3.6 packages generate object code with bundled LLVM and link it with the packaged runtime, without requiring Rust or Clang on the user's machine. Keep the complete package intact; see [package installation](docs/INSTALLING_PACKAGES.md). Source builds without the `embedded-llvm` feature retain the system Clang and matching Rust toolchain path. `--opt-level` selects optimization level 0 to 3. The resulting binary runs without Soplang or its source files beside it.

## Benchmark results

The V3 results below measure Soplang 3.0.0 at the revision recorded in the report.
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

The legacy tree-walking interpreter has been moved to a dedicated repository:

- [`soplang/soplang-interpreter`](https://github.com/soplang/soplang-interpreter)

All new development (compiler, JIT, AOT, CLI) happens here in the Rust implementation.

## Documentation

- [Getting started](docs/index.md)
- [Build guide](docs/BUILD_GUIDE.md): build from source on Windows, macOS, and Linux
- [Docker guide](docs/docker.md): run programs and build executables in a container
- [Language reference](docs/language/keywords.md)
- [Migrating to Soplang 3](docs/MIGRATION_V3.md)
- [Soplang 3 roadmap](ROADMAP.md): planned `3.x` language, runtime, library, and tooling work
- [Installation](docs/installation.md)
- [Contributing](docs/CONTRIBUTING.md)
- [Rust implementation plan (complete)](IMPLEMENTATION_PLAN.md): phases 1 through 7
- [Compiler plan](COMPILER_PLAN.md): Cranelift execution and LLVM builds

## License

This project is licensed under the [MIT License](LICENSE).
