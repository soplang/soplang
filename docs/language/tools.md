# Tools

These commands are separate from `soplang file.sop`. Their help text is Somali.

| Command | Purpose |
| --- | --- |
| `soplang fmt FILE [--qor]` | Print the canonical form. `--qor` replaces the file. Formatting the result again does not change it. |
| `soplang check FILE [--adag]` | Analyze without running. `--adag` requires strict types. A `schema.sop` in the same directory is validated. |
| `soplang test [FILE or DIR] [--shaandhe TEXT] [--wada] [--dabool]` | Run top-level tests. |
| `soplang doc FILE [--qor OUT]` | Write Markdown for functions, classes, exports, and tests. |
| `soplang lsp` | Speak Language Server Protocol on standard input. |
| `soplang debug FILE` | Run a file. `jebi()` prints the stack and waits. |
| `soplang profile FILE` | Print function call counts, inclusive/exclusive time, and separate compilation/execution time. |
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
tijaabo "eber" qalad("E030") { qor(1 / 0) }
```

`hubi(run)` passes. `hubi(left, right)` passes when the values are equal.
`qalad` passes only when the test body raises a runtime error. An optional
error code, such as `qalad("E030")`, must match the structured error's code.
Compilation, top-level initialization, setup, and cleanup failures always fail
the test. `diyaar` runs before each test. `nadiifi` runs after setup begins,
including when setup or the body fails. Body errors retain their diagnostic
metadata, source location, and stack trace.

With no path, the runner searches the current directory. Directory discovery
selects `*_test.sop`, `*.test.sop`, and `.sop` files inside `tests` directories.
It skips hidden/build/dependency directories and symbolic links, with explicit
limits on traversal. An explicit file may have any `.sop` name.

`--shaandhe` keeps tests whose names contain the text. `--lambarka N` selects
one zero-based test in one file. Invalid options and indexes are errors.
Each test has its own runtime, including when execution is serial. `--wada`
uses up to eight worker processes, bounded by available CPUs;
`--shaqaale N` explicitly selects 1 to 32 workers. Results are printed in
discovery order. Workers are joined and their pipes drained before returning.

`--dabool` reports each source file separately: reached/executable lines,
a percentage, and missed line numbers. Executable statements in uncalled
functions contribute to the denominator. Repeated loop hits consume no extra
report storage. Parallel workers merge their reports by source identity;
application output is never parsed as coverage data.

`hubi` is also the field validator inside `keen "web" sida web`, available as
`web.hubi`. A flat `keen "web"` defines its own `hubi` and hides the assertion.

## Editor

`check` and the language server use the same import expansion as run/build,
including bundled modules, class exports and package namespaces. Analysis reads
unsaved source overlays, enforces file/depth/size limits, and honors cancellation.
It does not execute imported statements, download dependencies or rewrite a
lockfile. Remote dependencies must already have a valid lock and verified cache;
local path dependencies are analyzed directly.

`editors/vscode` contains a VS Code extension that launches the configured
`soplang lsp` executable over stdio, supports untitled buffers, restarts on
configuration changes, and stops on deactivation. Untrusted workspaces retain
syntax highlighting and snippets; starting the executable requires trust.
Its grammar follows the compiler's comments, literal prefixes, triple strings
and interpolation. See the [extension README](../../editors/vscode/README.md)
for executable settings, packaging and integration tests.

`soplang lsp` answers
`initialize`, document open and change, diagnostics, completion, hover,
definition, references, rename, formatting, semantic tokens, and the code
action `Hagaaji qaabka`.

The server uses UTF-16 positions and accepts incremental document updates.
Definitions and references use the target file's current text; rename groups
edits by document URI and preserves untitled-buffer URIs. Open imported files
use their unsaved contents. Stale document versions and changes that split a
surrogate pair are ignored, and closing a document clears stale diagnostics.
Semantic token lengths include source delimiters and prefixes, and multiline
strings and comments are split into valid per-line ranges. Stdio headers and
message sizes are bounded.

Files have separate symbol scopes. Namespace and selective imports follow
public exports; explicit aliases retain their own rename identity while go to
definition follows the target. Same-named classes in different modules have
distinct member identities. Completion and navigation cover namespace exports,
inherited methods, declared return types, typed collection elements, chained
calls and known instance fields. Inferred receiver types become uncertain after
reassignment. Unsafe renames that could capture another binding, rename a
constructor, or split an override family are refused.

References and rename include open documents and source files under initialized
workspace roots, including closed importers. Discovery skips hidden/build/vendor
directories and links and has explicit file, depth and byte limits. A global
rename is refused when analysis is incomplete instead of returning a partial
edit. Source reads are bounded before allocation; overlays take precedence over
disk. File watchers refresh diagnostics for changed imports and manifests.

Bundled-library definitions open `soplang-stdlib:` read-only documents supplied
by the server. Their source matches the installed compiler and cannot be renamed;
local import aliases can still be renamed. User files named `web.sop` or
`xog.sop` continue to take precedence over the bundled imports.


## Breakpoints and profile

`jebi()` prints each stack frame as `name line:column` and reads one line.
`soco` continues. `jooji` stops the program. The frame does not include local
values.

`soplang profile` reports calls and inclusive/exclusive execution seconds per
compiled function. Inclusive time includes compiled callees; exclusive time
subtracts them. `ururin` is semantic analysis, lowering and JIT compilation;
`waqti` measures execution alone. Imports are resolved before these timers.
Profiles and coverage belong to one runtime context and do not leak between
independent executions. Native builtins contribute to their compiled caller's
time rather than receiving separate rows.
