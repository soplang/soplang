# Contributing to Soplang

Soplang 3 is a Rust compiler with Cranelift JIT and LLVM native builds.
Language and library changes should include examples, diagnostics, and tests
for both execution paths.

## Development setup

Install Git, the Rust toolchain pinned by `rust-toolchain.toml` (1.94.0), clang,
and a native linker. Windows also needs Visual Studio C++ Build Tools and the
Windows SDK; macOS needs Xcode Command Line Tools. See the
[build guide](BUILD_GUIDE.md) for platform and packaging requirements.

```sh
git clone https://github.com/soplang/soplang.git
cd soplang
cargo build --release --locked
cargo test --locked
```

Python is used for packaging and benchmark tooling. The comparison suite needs
Python 3.12+; the packaging scripts require Python 3.9+.

## Development checks

```sh
# Faster iteration, skipping the test_aot cases.
cargo test --locked -- --skip test_aot

# LLVM integration coverage.
cargo test --locked --test llvm_ir_tests

# Complete debug and optimized suites before a release.
cargo test --locked
cargo test --release --locked
```

The [test methodology](testing/TEST_METHODOLOGY.md) describes the suites. Public
examples have `.expected` stdout; [diagnostic examples](../examples/errors/README.md)
check Somali messages, files, line/column positions, and caret spacing.

For reversible edits, prefer focused checks that cover the behavior changed.
Format only changed Rust files with the repository's edition, for example:

```sh
rustfmt --edition 2021 --config skip_children=true src/backend/llvm_ir.rs
```

Repository-wide formatting can introduce unrelated changes. Review the diff
before submitting it. Pre-commit is optional; `.pre-commit-config.yaml` contains
basic file checks and a Rust formatting hook. Install it in an isolated Python
environment if you use it, then run `pre-commit install` and `make precommit`.

## Benchmarks

The [Python/v2/v3 comparison](../benchmarks/comparison/README.md) measures whole
programs in fresh processes, validates every output, and records raw samples.
The internal `cargo bench` / `make bench` suite measures the compiler in-process.
Their timings use different boundaries.

When publishing new measurements:

1. Record the source revisions, toolchain, host, and measurement options.
2. Run the comparison with correct equivalent workloads and no concurrent
   build/test jobs. Preserve every sample, including outliers.
3. Review `benchmarks/comparison/RESULTS.md`, `results.json`, and `samples.csv`.
4. Regenerate the chart with `benchmarks/comparison/plot.py`.
5. Refresh the summary tables in `README.md` and `benchmarks/RESULTS.md` from
   the recorded medians. Keep startup and LLVM build costs explicit.
6. Keep internal Criterion results in `benchmarks/CRITERION_RESULTS.md`.

Do not replace missing measurements with estimates or present a workload win
as a universal language speed claim.

## Useful commands

| Command | Purpose |
| --- | --- |
| `make build` | Release compiler build |
| `make run FILE=examples/01_hello.sop` | Run a file |
| `make shell` | Persistent REPL |
| `make test` | Rust test suites |
| `make bench` | Internal Criterion suite |
| `bash benchmarks/run_benchmarks.sh` | Regenerate the Criterion report |

## Changes and pull requests

- Explain the behavior and reason for the change.
- Add or update examples and behavior tests when language semantics change.
- Keep errors in Somali and check exact locations for new diagnostics.
- Update the language reference, migration notes, and changelog when relevant.
- Include the checks run and their results in the pull request description.
- Use clear commit messages and address review feedback.

Package and release changes follow the [release process](RELEASE_PROCESS.md)
and [native verification record](testing/RELEASE_VERIFICATION.md). Releases
continue on the [3.x roadmap](../ROADMAP.md).

Follow the project's [Code of Conduct](../CODE_OF_CONDUCT.md).
