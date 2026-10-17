# Historical Soplang Compiler Benchmarks

These are older, compiler-only, in-process results. They are not the v3.0.0
comparison with Python and v2. See the [current comparison](comparison/RESULTS.md).
Some historical workload labels undercount inclusive loop bounds; the current
comparison uses separate, explicitly checked workloads.

> Auto-generated on **2026-02-20 11:07:46 UTC**
> System: `Linux 6.18.7-arch1-1 x86_64` | `rustc 1.91.1 (ed61e7d7e 2025-11-07)` | `cargo 1.91.1 (ea2d97820 2025-10-10)`

## Benchmark Programs

| Benchmark | Description | Workload |
|-----------|-------------|----------|
| fib_recursive | Recursive Fibonacci | fib(25) = 75025 |
| loop_sum | Tight for-loop | Sum 1..100,000 |
| nested_loops | Nested loop dispatch | 200 x 200 iterations |
| string_concat | String concatenation | 1,000 appends |
| list_ops | List push + traversal | 5,000 elements |
| object_create | Object allocation | 2,000 objects |

## Criterion Results (compiler — Cranelift JIT)

In-process: Lex → Parse → HIR → Cranelift JIT → execute (same path as `soplang file.sop`).


| Benchmark | Mean | Low | High |
|-----------|------|-----|------|
| fibonacci/fib_25_full | 25.62 ms | 24.93 ms | 26.27 ms |
| loops/loop_sum_100k | 9.90 ms | 9.62 ms | 10.08 ms |
| loops/nested_loops_200x20 | 4.20 ms | 4.07 ms | 4.35 ms |
| strings/string_concat_1 | 0.67 ms | 0.65 ms | 0.69 ms |
| lists/list_ops_5k | 1.30 ms | 1.28 ms | 1.34 ms |
| objects/object_create_2 | 251.69 µs | 240.42 µs | 259.04 µs |

### Pipeline stage breakdown

| Stage | Mean | Low | High |
|-------|------|-----|------|
| pipeline_stages/lex_onl | 1.60 µs | 1.58 µs | 1.61 µs |
| pipeline_stages/parse_onl | 2.73 µs | 2.71 µs | 2.75 µs |

## Quick timings (wall-clock)

| Benchmark | Time |
|-----------|------|
| fib_recursive | 0m0.041s |
| list_ops | 0m0.003s |
| loop_sum | 0m0.016s |
| nested_loops | 0m0.007s |
| object_create | 0m0.002s |
| string_concat | 0m0.002s |

## How to run

```bash
# Full criterion benchmarks (HTML reports in target/criterion/)
cargo bench

# This script: benchmarks + formatted results
bash benchmarks/run_benchmarks.sh

# Single group
cargo bench -- fibonacci
cargo bench -- loops
```

## Notes

- **What we benchmark:** The compiler (Cranelift JIT). Same path as `./target/release/soplang file.sop`.
- **Criterion** = in-process, statistical (mean, confidence intervals).
- **HTML reports:** `target/criterion/report/index.html`.
