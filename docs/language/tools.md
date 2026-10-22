# Tools

These commands are separate from `soplang file.sop`. Their help text is Somali.

| Command | Purpose |
| --- | --- |
| `soplang fmt FILE [--qor]` | Print the canonical form. `--qor` replaces the file. Formatting the result again does not change it. |
| `soplang check FILE [--adag]` | Analyze without running. `--adag` requires strict types. A `schema.sop` in the same directory is validated. |
| `soplang test [FILE or DIR] [--shaandhe TEXT] [--wada] [--dabool]` | Run top-level tests. |
| `soplang doc FILE [--qor OUT] [--gaar]` | Write public API signatures, documentation comments and tests as Markdown. |
| `soplang xog abuur [SCHEMA] [--qor CLIENT or --hubi CLIENT]` | Generate or verify a typed database client without connecting. |
| `soplang lsp` | Speak Language Server Protocol on standard input. |
| `soplang debug FILE [--jebi [FILE:]LINE] [--soco] [--adag] [--dhis OUT]` | Debug source statements, inspect frames and locals, or build an instrumented LLVM executable. |
| `soplang profile FILE` | Print function call counts, inclusive/exclusive time, and separate compilation/execution time. |
| `soplang run [DIR] [--adag] [--offline] [-- ARGS]` | Run the package entry with application arguments. |
| `soplang dhis [DIR] [-o OUT] [-O 0..3] [--adag] [--offline] [-q]` | Build that entry to a native executable. `build` is an alias. |
| `soplang add NAME SOURCE [DIR] [--offline]` | Add a dependency and update the root lock. |
| `soplang remove NAME [DIR] [--offline]` | Remove a dependency and update the root lock. |
| `soplang update [DIR] [--offline]` | Resolve dependencies again and update the root lock. |

## Project commands

Project commands search for the nearest `soplang.toml` from the given directory
or the current directory, including when invoked inside a nested source folder.
`run` and `dhis` choose `main.sop`, then `lib.sop`, inside the manifest's `gelid`.
Application paths remain relative to the caller's working directory. Arguments
after `run --` are available through `amarro()`.

`--adag` and `--strict` are equivalent. `--offline` verifies existing locked
caches without fetching; it does not make a missing remote dependency available.
The same option is accepted by `check` and the dependency commands. Source
analysis itself never downloads packages. Run/build may prepare dependencies
unless offline mode is selected.

`dhis` defaults to LLVM optimization level 2 and writes `dhis/<magac>` under the
project root, with `.exe` on Windows. `-o`, `--output` and `--qor` select an
output path relative to the caller; `-O` and `--opt-level` select 0 through 3.
`-q` or `--quiet` suppresses the success message. Conflicting aliases, missing
option values, unknown options and extra positional arguments are errors.
Existing package flags remain supported alongside `add`, `remove` and `update`.
See [packages](packages.md) for supported source forms and lock behavior.

## API documentation

`doc` parses the source without executing it or fetching dependencies. It keeps
function parameter types, collector types, default expressions, return types,
class parents and constructor/method signatures exactly as written. Contiguous
`///` comments immediately above a declaration become its documentation.

The same `soosaar` and private-name rules select public declarations. Public
reexports are listed by name; their declarations are documented at their source.
Class members starting with `_` are omitted. `--gaar` includes private declarations
and methods. Test names and expected error codes are included separately.

`--qor` writes atomically, follows existing symbolic links, preserves permissions,
and refuses read-only files or replacement of the source itself. Invalid syntax
leaves the existing documentation intact. Without `--qor`, output goes to
standard output. Generated prose is English; Soplang signatures and comments
retain their original language.

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

`soplang lsp` accepts the language client's `--stdio` flag and answers
`initialize`, document open and change, diagnostics, completion, hover,
definition, references, rename, formatting, semantic tokens, and the code
action `Hagaaji qaabka`.

Incomplete editor input retains earlier tokens, imports, parameters and local
scopes, including inside an unclosed function or unfinished string/comment.
This recovery is only for navigation and completion; compiler acceptance and
syntax diagnostics are unchanged. Token/nesting limits still apply when
lexing fails. Synthetic recovery tokens never become source edits. Rename is refused when
the defining file has a syntax error, including for local variables.

When multiple open URIs refer to one file, navigation uses the requesting
buffer's text and URI. Imports choose a deterministic overlay. Renames that
could touch an ambiguous document are refused until the extra buffer is closed;
workspace renames require an unambiguous analysis.

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

`soplang debug file.sop` pauses before the first executable statement. A pause
shows the current source line and stack on standard error; application output
stays on standard output. `--soco` runs until a breakpoint or `jebi()`.
Repeat `--jebi LINE` or `--jebi FILE:LINE` to set up to 128 breakpoints, including
in imported files. CLI breakpoints must name an executable source line; blank
lines and comments are rejected. Loop-header breakpoints stop at every check.
`--adag` enables strict analysis.

| Console command | Action |
| --- | --- |
| `soco` | Continue to the next breakpoint. |
| `tallaabo` | Stop at the next statement, entering compiled calls. |
| `xiga` | Stop at the next statement at the current or an outer call depth. |
| `bax` | Continue until the current call returns. |
| `raad` | Print the stack with source files, lines and columns. |
| `jir N` | Select a frame for inspection; zero is the current frame. |
| `deegaan` | List that frame's local and captured values. |
| `eeg NAME` | Inspect one variable without evaluating an expression. |
| `jebi FILE:LINE` / `tir FILE:LINE` | Add or remove a breakpoint. |
| `jooji` | End execution with an error. |

Frames read the compiler's rooted value storage. Variables not yet initialized
show `<lama qeexin>`, distinct from `maran`. Inspection does not call methods or
execute source. Strings, collection entries and recursion are bounded; tuples,
sets and errors currently show a kind summary. Console commands are limited to
4096 bytes. Closing debugger input ends execution with an error.

`jebi()` also works during ordinary execution, printing the available call stack
and reading console commands. Local inspection and stepping require the
instrumentation produced by `debug`; ordinary run/build does not add it. For
compatibility, an ordinary `jebi()` continues when standard input is closed.

```sh
soplang debug main.sop --jebi 12 --soco
soplang debug main.sop --jebi 12 --soco --dhis app-debug
./app-debug
```

`--dhis` produces a standalone LLVM executable containing the same source
console, without running it during the build. Debug builds use optimization
level zero. This is Soplang source instrumentation, not DWARF or a debug adapter
for a native debugger. Stepping follows executable statements; it does not
enter Rust builtins. The console shares standard input with the application.
Frame selection affects inspection; stepping resumes the executing frame.

`soplang profile` reports calls and inclusive/exclusive execution seconds per
compiled function. Inclusive time includes compiled callees; exclusive time
subtracts them. `ururin` is semantic analysis, lowering and JIT compilation;
`waqti` measures execution alone. Imports are resolved before these timers.
Profiles and coverage belong to one runtime context and do not leak between
independent executions. Native builtins contribute to their compiled caller's
time rather than receiving separate rows.
