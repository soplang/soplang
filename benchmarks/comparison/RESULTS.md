# Python vs Soplang 2 vs Soplang 3

Measured 2026-10-17T03:22:22.092333+00:00 on 11th Gen Intel(R) Core(TM) i7-1185G7 @ 3.00GHz.

- OS: `Linux-7.2.6-arch2-1-x86_64-with-glibc2.44`.
- Python baseline and v2 host: `3.14.7 (main, Aug 10 2026, 07:46:56) [GCC 16.1.1 20260728]` (CPython).
- Python build: debug=False, GIL enabled=True, JIT enabled=False.
- v2 source: `v2.0.0` at `b9f6457715e90028237239065f22ad7daa674d0b`; its internal version is `2.0.0-beta`.
- v3: `soplang 3.0.0` at `0a3d36e44dd65a0add6f6dac285d774e439d37ff`; Cargo release build; LLVM optimization level 2.
- 25 measured runs and 5 warmups per cell; interleaved engine order, random seed 1729.
- CPU affinity: `[2]`; initial load average: `(1.54833984375, 1.349609375, 1.25244140625)`.

## Median process wall time

Milliseconds; lower is better. Each cell is a fresh process, including startup, source reading, parsing, execution, and output capture. v3 JIT includes compilation. LLVM executable timing excludes its separate build step.

| Workload | Python | Soplang 2 | Soplang 3 JIT | Soplang 3 LLVM |
| --- | ---: | ---: | ---: | ---: |
| startup | 11.194 | 52.342 | 2.846 | 1.972 |
| integer_sum | 13.960 | 222.053 | 42.774 | 41.791 |
| integer_sum_typed | 13.791 | 280.631 | 83.952 | 83.268 |
| float_sum_typed | 15.390 | 321.467 | 100.254 | 102.938 |
| nested_loops | 13.115 | 129.016 | 22.305 | 21.196 |
| fibonacci | 20.877 | 1265.457 | 171.612 | 142.999 |
| function_calls | 15.821 | 205.235 | 26.083 | 23.973 |
| list_push_index | 12.164 | 80.566 | 9.740 | 8.161 |
| object_create | 15.321 | 98.817 | 25.334 | 23.677 |
| string_concat | 12.075 | 66.099 | 13.142 | 11.718 |

## Ratios of median times

The numerator is the reference language's median. Values above 1 mean v3 finished sooner; values below 1 mean v3 finished later. These ratios include startup and must not be read as execution-only speedups.

| Workload | v2 / v3 JIT | v2 / v3 LLVM | Python / v3 JIT | Python / v3 LLVM |
| --- | ---: | ---: | ---: | ---: |
| startup | 18.39× | 26.54× | 3.93× | 5.68× |
| integer_sum | 5.19× | 5.31× | 0.33× | 0.33× |
| integer_sum_typed | 3.34× | 3.37× | 0.16× | 0.17× |
| float_sum_typed | 3.21× | 3.12× | 0.15× | 0.15× |
| nested_loops | 5.78× | 6.09× | 0.59× | 0.62× |
| fibonacci | 7.37× | 8.85× | 0.12× | 0.15× |
| function_calls | 7.87× | 8.56× | 0.61× | 0.66× |
| list_push_index | 8.27× | 9.87× | 1.25× | 1.49× |
| object_create | 3.90× | 4.17× | 0.60× | 0.65× |
| string_concat | 5.03× | 5.64× | 0.92× | 1.03× |

## Sample spread

Interquartile range (25th–75th percentile) in milliseconds. No outliers removed.

| Workload | Python | Soplang 2 | Soplang 3 JIT | Soplang 3 LLVM |
| --- | ---: | ---: | ---: | ---: |
| startup | 10.886–11.557 | 50.889–53.656 | 2.705–3.216 | 1.787–2.100 |
| integer_sum | 13.413–14.379 | 219.260–228.935 | 42.298–43.477 | 41.255–42.647 |
| integer_sum_typed | 13.556–14.892 | 279.085–294.849 | 83.631–88.363 | 82.590–88.413 |
| float_sum_typed | 14.860–16.576 | 317.685–328.968 | 98.852–104.888 | 99.950–108.119 |
| nested_loops | 12.817–13.362 | 124.305–131.717 | 21.884–22.699 | 20.583–21.955 |
| fibonacci | 19.967–22.111 | 1214.775–1430.840 | 167.436–177.068 | 140.514–152.110 |
| function_calls | 13.883–19.302 | 165.928–256.473 | 22.550–28.110 | 21.076–26.781 |
| list_push_index | 11.571–13.172 | 78.254–83.877 | 9.177–9.986 | 7.963–8.534 |
| object_create | 14.133–16.615 | 91.657–108.529 | 23.105–27.711 | 20.975–26.940 |
| string_concat | 11.629–12.314 | 64.833–67.897 | 12.889–13.496 | 11.482–12.145 |

## LLVM build costs

One observed build per workload, in seconds. Not a statistical benchmark; toolchain startup, linking, and existing build caches affect these values.

| Workload | Build seconds |
| --- | ---: |
| startup | 0.113 |
| integer_sum | 0.126 |
| integer_sum_typed | 0.112 |
| float_sum_typed | 0.116 |
| nested_loops | 0.128 |
| fibonacci | 0.113 |
| function_calls | 0.149 |
| list_push_index | 0.131 |
| object_create | 0.117 |
| string_concat | 0.154 |

## Workloads and expected output

| Workload | Work performed | Output |
| --- | --- | ---: |
| startup | Minimal program; process and language startup | `1` |
| integer_sum | 100,000 integer additions with dynamic Soplang locals | `5000050000` |
| integer_sum_typed | 100,000 integer additions with an abn accumulator | `5000050000` |
| float_sum_typed | 100,000 exact binary-float additions with a jajab accumulator | `50000` |
| nested_loops | 200 by 200 iterations; 40,000 increments | `40000` |
| fibonacci | Recursive fib(25); 242,785 function invocations | `75025` |
| function_calls | 20,000 calls to a one-argument increment function | `20000` |
| list_push_index | Append 5,000 integers, then sum 5,000 indexed reads | `12502500` |
| object_create | Create 5,000 two-field objects/dicts and sum both fields | `37492500` |
| string_concat | 5,000 single-character concatenations; print final length | `5000` |

## Interpretation and limits

- The same `.sop` files run in v2 and v3. Python uses explicit equivalent loops, recursion, append/index operations, and dictionaries; no closed-form replacements.
- The v2 launcher calls the tag's original `run_soplang_file` pipeline, including normal package imports. It avoids initializing the interactive shell and writing history. It does not modify or port the interpreter.
- `integer_sum` uses dynamic `door`; `integer_sum_typed` uses `abn`. The Python programs are identical for these two rows. `float_sum_typed` uses `jajab` and exact halves, converting the final result to an integer for validation.
- Short programs are sensitive to process and interpreter startup. Warmups warm filesystem/bytecode caches; each measured process starts a fresh VM/JIT. No startup subtraction, timing inside a persistent VM, or manual disabling of GC.
- Results describe these workloads on one shared workstation, not all programs. CPU frequency, background work, allocation strategies, garbage collection, and the chosen Python build affect results. The suite does not measure peak memory or sustained server workloads; unreclaimed runtime allocations can favor short runs.
- Every warmup and measured process must exit successfully with exactly the expected stdout and no stderr. Failures abort the comparison.

Raw samples, hashes, commands, tool versions, and summary statistics are in [results.json](results.json) and [samples.csv](samples.csv). See [README.md](README.md) to reproduce the run.
