# Soplang 3.1 runtime verification

Local verification on 2026-10-19, Linux x86_64, Rust 1.94.0, and clang 22.1.8.
This records implementation checks, not published package or platform results.

## Commands and results

```bash
cargo test --locked
cargo test --locked --release \
  --test interpreter_tests \
  --test llvm_ir_tests \
  --test jit_input_ownership \
  --test gc_tests
```

| Check | Result |
| --- | --- |
| Full debug suite, including AOT tests | 172 passed, none failed or ignored |
| Optimized JIT, LLVM, embedding, and GC suites | 69 passed, none failed or ignored |
| LLVM integration suite in each profile | 24 passed |
| Diagnostic corpus through JIT and LLVM | All 73 intentional failures matched their snapshots |
| Milestone note examples | Both snippets executed successfully |
| Source compiler version | `soplang 3.1.0` |

For faster development checks, `cargo test --locked -- --skip test_aot` omits
some native executable tests. The full milestone run above included them.

## Coverage

| Area | Evidence |
| --- | --- |
| Context ownership and recovery | `runtime_context_tests.rs`: isolation, nested activation, dropped owners, stale handles, continued REPL execution |
| Garbage collection | `gc_tests.rs` and `fixtures/garbage_collection.sop`: cycles, live captures, native snapshots, Rust roots, repeated successful and failing snippets |
| JIT input lifetime | `jit_input_ownership.rs`: caller mutation and input destruction, incremental storage growth, preserved literals/descriptors/trace names/source text |
| Error propagation | `fixtures/error_boundaries.sop`: failed expressions stop before later side effects |
| Raising and cleanup | `fixtures/raise_cleanup.sop`: catches, rethrows, returns, loop exits, cleanup failures, closure captures |
| Error metadata | `fixtures/structured_errors.sop`: immutable details, causes, traces, and compatibility with text catches |
| Callable ABI | `fixtures/callable_expressions.sop`: wide arguments, direct and indirect calls, returned/indexed/grouped functions, closures, constructors, callable fields |
| Strings | `string_literal_tests.rs`, `interpolation_tests.rs`, and shared execution fixtures: raw/escaped/triple literals, Unicode, nested fields, formatting, evaluation order, capture and GC behavior |
| Numeric boundaries | `fixtures/numeric_edges.sop`: dynamic/native overflow, minimum integer remainder, signed zero, extreme float remainder, exact range bounds, overflow termination, non-advancing steps |
| Diagnostics | `examples/errors/cases.tsv`: Somali messages, original paths, exact lines/columns, Unicode display width, source text, carets, imported files, retained source after removal |

Execution fixtures run in both compiler backends. The numeric boundary fixture
also runs with LLVM optimization levels 0 and 2. LLVM loop tests verify fixed
argument buffers without relying on optimization to bound stack use.

## Remaining release gates

Windows x64, macOS Apple Silicon, relocatable packages, installers, Docker,
installation tests, and release artifact verification have not been rerun for
3.1. Earlier 3.0.x results do not certify these changes. Those checks and public
release publication remain separate from this local implementation milestone.
