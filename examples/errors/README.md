# Intentional error examples

These programs are expected to fail. They are separate from the numbered,
successful examples and are never selected by `--example N`.

The [manifest](cases.tsv) lists each case's phase, entry file, expected one-based
line and column, and an optional Somali message fragment. A `.stderr` file beside
each program records its complete diagnostic, including the original source
line, exact caret spacing, optional error code, and hint. Files in `support/`
are imported helpers, not separate entry points.

## Coverage

| Area | Examples |
| --- | --- |
| Lexer | Unexpected characters, unclosed strings/comments, invalid escapes and Unicode scalars, triple quotes, Unicode columns |
| Parser | Missing names, values, delimiters, invalid numbers, invalid class contents |
| Semantics | Constants, duplicate names, invalid assignments and types, control flow, function/constructor arguments |
| Classes | Missing parents/classes, cycles, invalid parent calls, incompatible instances |
| Strict mode | Untyped variables and function signatures |
| Imports | Missing files and lexer, type, and runtime failures originating in imported files |
| Arithmetic | Integer/float division and modulo by zero, incompatible operands |
| Calls | Unknown names/methods, non-callable values, closure/native arity, typed arguments and returns |
| Collections | Invalid index types/writes, empty removal, invalid slice bounds, repeat/count/direction arguments |
| Callbacks | Failure inside an anonymous callback or nested function; invalid callbacks on an empty list |
| Positions | Indentation, EOF, tabs, accents, emoji, wide characters, combining-character display width, and large line-number gutters |

## Run and inspect

```bash
cargo build --locked
./target/debug/soplang --no-color examples/errors/division_zero.sop
./target/debug/soplang --no-color --strict examples/errors/strict_variable.sop
cargo test --locked --test diagnostic_tests
```

An intentional failure exits with status 1 and writes the diagnostic to stderr.
It must not produce stdout. The native test compiles runtime-failure cases into
executables and checks their diagnostics; frontend failures are rejected during
`--build`. Both paths must agree with the same snapshot.

Columns count Unicode scalar values, matching the lexer. The caret follows
terminal display width: tabs expand to four-column stops, combining marks take
no extra display column, and wide characters occupy two. The failing token is
underlined where its span is known. An unclosed string or comment points to its
opening delimiter. EOF can point just after the final character or at the start
of an empty final line.

Runtime diagnostics in native binaries retain the relevant source line, so they
also work when the original source file is unavailable. This includes code
from `keen` imports. Caught errors remain message strings for compatibility;
an optional second catch binding exposes structured details. Compiled function
traces retain call sites alongside the failing operation's original location.

## Add a case

1. Add a failing `.sop` program in this directory.
2. Record the intended phase and token position in `cases.tsv` before checking
   the emitted diagnostic. Use `strict` when the case requires `--strict`.
3. Inspect the Somali text and caret, then save the approved `.stderr` output.
4. Run the diagnostic suite against both backends. A manifest test rejects
   missing or duplicate case entries.

Do not refresh snapshots merely to make a failing test green: inspect whether
the source, message, and location represent the intended failure.
