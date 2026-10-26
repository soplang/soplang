# Debugging Soplang

Soplang reports errors in Somali with the original file, one-based line and
column, source line, and a caret at the failing token. This works in Cranelift
JIT and LLVM native executables, including code loaded with `keen`.

## Reading a diagnostic

For `examples/errors/division_zero.sop`:

```text
Khalad fulin: Tiro looma qaybin karo eber. [E030] safka 2, tiirka 14
     --> examples/errors/division_zero.sop:2:14
   2 │       qor(tiro / 0)
     │                ^
```

The arrow names the original source file. `safka` means line and `tiirka` means
column. The underline selects the token associated with the error. Optional
`[E...]` codes and hints provide more detail.

| Heading | Phase |
| --- | --- |
| `Khalad ereyayn` | Lexing |
| `Khalad naxwe` | Parsing |
| `Khalad nooc` | Name, type, or semantic validation |
| `Khalad fulin` | Runtime execution |
| `Khalad soo-dejin` | File importing |

Columns count Unicode scalar values. The displayed caret accounts for tabs,
combining marks, emoji, and wide characters; tabs expand to four-column stops.
An unclosed string/comment points to its opening delimiter. An error at EOF
can point just past the final character or at an empty final line.

Use `--no-color` or `NO_COLOR=1` for plain output. Diagnostics go to stderr and
uncaught errors exit with status 1. A native executable retains relevant source
lines for errors even when the source files have been removed.

## Catch a runtime failure

```soplang
tijaabi {
    qor(1 / 0)
} qabo (qalad) {
    qor(qalad)
}
qor("Fulintu way sii socotaa")
```

Caught errors remain Somali message strings. Static errors are reported before
execution and cannot be caught by a runtime `qabo`. Compiled call stack traces
and structured user-raised errors are not implemented yet.

## Inspect the compiler

```bash
cargo build --locked
./target/debug/soplang --ast examples/18_closures.sop
./target/debug/soplang --hir examples/18_closures.sop
cargo test --locked --test diagnostic_tests
cargo test --locked --test interpreter_tests test_global_updates
```

`--ast` shows the parse tree; `--hir` shows the flat instructions and source
locations. Neither executes the program. Add `qor` calls to inspect values when
following a runtime failure. Function parameters and local declarations shadow
globals; top-level reads observe writes completed by functions and callbacks.

For Rust-level debugging:

```bash
rust-gdb --args target/debug/soplang examples/18_closures.sop
```

Use `rust-lldb` on macOS. See [the error examples](../../examples/errors/README.md)
for the checked failure catalog and [test methodology](TEST_METHODOLOGY.md) for
the remaining suites.
