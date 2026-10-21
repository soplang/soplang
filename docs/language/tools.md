# Tools

These commands are separate from `soplang file.sop`. Their help text is Somali.

| Command | Purpose |
| --- | --- |
| `soplang fmt FILE [--qor]` | Print the canonical form. `--qor` replaces the file. Formatting the result again does not change it. |
| `soplang check FILE [--adag]` | Analyze without running. `--adag` requires strict types. A `schema.sop` in the same directory is validated. |
| `soplang test FILE [--shaandhe TEXT] [--wada] [--dabool]` | Run top-level tests. |
| `soplang doc FILE [--qor OUT]` | Write Markdown for functions, classes, exports, and tests. |
| `soplang lsp` | Speak Language Server Protocol on standard input. |
| `soplang debug FILE` | Run a file. `jebi()` prints the stack and waits. |
| `soplang profile FILE` | Print function call counts and elapsed seconds. |
| `soplang run [DIR]` | Run the package entry, `main.sop` or `lib.sop` inside `gelid`. |
| `soplang dhis [DIR]` | Build that entry to `dhis/<magac>`. |

## Formatting

The formatter uses four-space indentation and consistent spacing. It preserves
comments, anonymous function bodies, quoted object keys, type annotations and
the exact contents of strings, including raw strings and interpolation.
Existing line boundaries are retained because some call syntax depends on them.

`fmt --qor` validates the formatted source before replacing the file atomically.
It follows an existing symbolic link to its target, preserves file permissions,
and refuses to replace a read-only file. A syntax error leaves the source intact.

## Tests

A test file keeps its normal program. `soplang file.sop` skips tests. `soplang test` runs them.

```soplang
door n = 1
diyaar { n = n + 1 }
nadiifi { n = 0 }
tijaabo "kordhi" { hubi(n > 1) }
tijaabo "khalad" qalad { hubi(been) }
```

`hubi(run)` passes. `hubi(left, right)` passes when the values are equal.
`qalad` passes when the body fails. `diyaar` runs before each test, and
`nadiifi` runs afterwards even when the body fails.

`--shaandhe` keeps tests whose names contain the text. `--wada` runs each
selected test in a separate process. `--dabool` prints the source lines that
ran.

`hubi` is also the field validator inside `keen "web" sida web`, available as
`web.hubi`. A flat `keen "web"` defines its own `hubi` and hides the assertion.

## Editor

`editors/vscode` provides TextMate syntax for `.sop`. `soplang lsp` answers
`initialize`, document open and change, diagnostics, completion, hover,
definition, references, rename, formatting, semantic tokens, and the code
action `Hagaaji qaabka`.

## Breakpoints and profile

`jebi()` prints each stack frame as `name line:column` and reads one line.
`soco` continues. `jooji` stops the program. The frame does not include local
values. `soplang profile` counts entries made while the file runs and prints
`waqti` in seconds.
