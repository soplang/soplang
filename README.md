# Soplang

> The Somali Programming Language

Soplang is a programming language with syntax inspired by Somali, making programming more accessible to Somali speakers. It combines static and dynamic typing in one language with a focus on clarity and ease of use.

**The implementation is in Rust.** Soplang runs through a compiled pipeline: Cranelift JIT for `run` and an AOT build path for standalone binaries. See [COMPILER_PLAN.md](COMPILER_PLAN.md).

## Project structure

| Path | Description |
|------|-------------|
| **src/** | Rust implementation: lexer, parser, semantic, HIR, runtime, Cranelift JIT, and AOT build backend. |
| **examples/** | Soplang example programs (`.sop` files). |
| **benchmarks/** | Compiler benchmark programs, [RESULTS.md](benchmarks/RESULTS.md), [README](benchmarks/README.md), and `run_benchmarks.sh` to regenerate results. |
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

./target/release/soplang examples/hello.sop        # JIT run
./target/release/soplang -i    # REPL
./target/release/soplang -c 'qor("Salaan!")'
./target/release/soplang --build examples/hello.sop -o hello_aot
./hello_aot
```

Or: `make build`, `make run FILE=examples/hello.sop`, `make shell`, `make test`, `make bench`.

### CLI notes

- `soplang <file.sop>`: run via Cranelift JIT.
- `soplang --build <file.sop> -o <out>`: build a standalone native binary.
- `soplang --build ... --opt-level 0..3`: tune AOT optimization level.
- `soplang --strict`: enable stricter static typing checks.

### How a program runs

`soplang file.sop` and the REPL share one pipeline: lex, parse, resolve `keen`, analyze, lower to HIR, then Cranelift. Static `abn` and `jajab` arithmetic is native integer and float code. `door` and other dynamic values, including string concatenation, go through the tagged runtime.

`soplang --build file.sop` inlines `keen` from the real source path, then embeds that text in a small Cargo runner. `--opt-level` selects the Cargo profile. The binary calls the same JIT when it starts, and it has no `keen` left to resolve. A direct LLVM backend is later work; `inkwell` is not a dependency.

## Benchmark results

The **compiler** (Cranelift JIT) is benchmarked with [Criterion](https://github.com/bheisler/criterion.rs). Summary (release build, in-process):

| Benchmark | Mean time |
|-----------|-----------|
| fib(25) recursive | ~25.6 ms |
| loop sum 1..100k | ~9.9 ms |
| nested loops 200×200 | ~4.2 ms |
| string concat 1k | ~0.67 ms |
| list ops 5k elements | ~1.3 ms |
| object create 2k | ~252 µs |

**How to run:** `cargo bench` or `bash benchmarks/run_benchmarks.sh` (regenerates RESULTS.md).  
**Details:** [benchmarks/RESULTS.md](benchmarks/RESULTS.md). **Benchmark names and performance notes:** [benchmarks/README.md](benchmarks/README.md).

## Legacy interpreter

The legacy tree‑walking interpreter has been moved to a dedicated repository:

- [`soplang/soplang-interpreter`](https://github.com/soplang/soplang-interpreter)

All new development (compiler, JIT, AOT, CLI) happens here in the Rust implementation.

## Documentation

- [Getting started](docs/index.md)
- [Build guide](docs/BUILD_GUIDE.md) — Build from source on Windows, macOS, and Linux
- [Language reference](docs/language/keywords.md)
- [Installation](docs/installation.md)
- [Contributing](docs/CONTRIBUTING.md)
- [Rust implementation plan (complete)](IMPLEMENTATION_PLAN.md) — Phases 1–7 done
- [Compiler plan](COMPILER_PLAN.md) — Cranelift run path and Cargo `--build` are in place; real LLVM is phase 5

## License

This project is licensed under the [MIT License](LICENSE).
