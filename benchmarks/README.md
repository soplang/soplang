# Soplang benchmarks

## Python, Soplang 2, and Soplang 3

The current comparison measures ten equivalent programs in four modes: Python,
the original v2 interpreter, v3 Cranelift JIT, and v3 LLVM executables.

- [Results and interpretation](comparison/RESULTS.md)
- [Comparison chart](comparison/results.svg)
- [Reproduction and methodology](comparison/README.md)
- [Raw observations](comparison/samples.csv) and [metadata](comparison/results.json)

Every warmup and timed process must produce the expected result. The suite
reports medians, interquartile ranges, ratios, and separate LLVM build costs.
Times include fresh-process startup and are specific to the recorded machine.

## Internal compiler benchmarks

The Criterion suite in `benches/soplang_bench.rs` measures the v3 compiler inside
one Rust process. Its full-program groups read a file, lex, parse, analyze,
lower, JIT-compile, and execute it. The pipeline groups measure lexing alone
and lexing plus parsing. These measurements differ from the fresh-process
comparison above.

```sh
cargo bench
cargo bench -- fibonacci
cargo bench -- loops
```

Criterion writes reports under `target/criterion/`. The historical wrapper
`bash benchmarks/run_benchmarks.sh` regenerates [RESULTS.md](RESULTS.md), which
contains older compiler-only measurements. That wrapper does not run Python
or the v2 interpreter and does not regenerate the current comparison.

The original Criterion fixtures remain unchanged. Several names do not match
the inclusive loop endpoints in their files:

| Fixture | Actual workload |
| --- | --- |
| `fib_recursive.sop` | `fib(25)`; 242,785 calls; result 75,025 |
| `loop_sum.sop` | Sum 1 through 100,001 inclusive |
| `nested_loops.sop` | 201 × 201 iterations |
| `string_concat.sop` | 1,001 concatenations |
| `list_ops.sop` | 5,001 appends and 5,001 indexed reads |
| `object_create.sop` | 2,002 object allocations |

The comparison suite has its own paired `.sop` / `.py` fixtures with exact
counts and checked outputs. Historical Criterion timings must not be presented
as measurements from the current release or compared directly to Python/v2
process timings.
