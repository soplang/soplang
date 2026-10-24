# CLI commands and diagnostics

The current development CLI accepts English and Somali names for the same
actions. Each top-level name also accepts a `--` prefix: `build`, `--build`,
`dhis`, and `--dhis` all use one build path. These command aliases do not
change Soplang source-language keywords.

## Commands

| English | Somali | Action |
| --- | --- | --- |
| `init` | `abuur` | Create a project. |
| `run` | `fuli` | Run one source file or a project. |
| `build` | `dhis` | Build executable programs. |
| `check` | `hubi` | Check source without running it. |
| `fmt` | `habee` | Format source. |
| `test` | `tijaabi` | Run tests. |
| `doc` | `duk` | Generate Markdown API documentation. |
| `debug` | `baar` | Debug one program. |
| `profile` | `cabbir` | Measure execution. |
| `add` | `kudar` | Add a dependency. |
| `remove` | `saar` | Remove a dependency. |
| `update` | `cusboonaysii` | Update locked dependencies. |
| `publish` | `daabac` | Publish the current package. |
| `data generate` | `xog abuur` | Generate or check a schema client. |
| `repl` | `isdhexgal` | Open the interactive shell. |
| `eval` | `qiimee` | Run a supplied source snippet. |
| `example` | `tusaale` | Run a numbered bundled example. |
| `inspect` | `eeg` | Display AST or HIR without executing. |
| `help` | `caawimo` | Show general or command-specific help. |
| `version` | `nooc` | Show the Soplang software version. |
| `lsp` | `lsp` | Start the editor protocol server. |

Only `doc` and `duk` name the documentation command; `docs` is not an alias.
The `--` prefix applies to a **top-level action**, not to ordinary arguments:
`soplang --dhis hello.sop` builds just like `soplang dhis hello.sop`.
The separate `--` token before arguments passes them to a running program:
`soplang fuli app.sop -- first second`.

```sh
soplang                         # interactive command browser in a terminal
soplang repl                    # interactive Soplang shell
soplang program.sop             # quick file execution
soplang fuli program.sop        # equivalent explicit action
soplang qiimee 'qor("Salaan!")'
soplang dhis program.sop        # writes dhis/program (program.exe on Windows)
soplang hubi program.sop
soplang eeg program.sop --hir
soplang caawimo dhis
```

`build` with no path builds the current project. One or more `.sop` paths build
one executable per file in `dhis/`; each output takes its source stem. If two
input files have the same stem, the command fails before building. A project
build uses the `magac` in `soplang.toml`. Use `--name` or `--magac` for a single
build's executable name. `-o`/`--output` remains available when an exact output
path is needed. `keen` imports combine several source files into **one**
program; passing several files to `build` makes separate programs.

With no arguments in a terminal, Soplang opens a command browser. Up and Down
browse commands or options, Tab completes them, and the current choice shows
its description. Select a command to see its arguments and options, then press
Enter to run it. `soplang repl` or `soplang isdhexgal` starts the Soplang shell
directly. When input is piped, bare `soplang` keeps opening the REPL.
`-i`/`--interactive`/`--isdhexgal` continues in the REPL after running a file
or example. Use `/caawi` for REPL commands and `/bixi` to exit. AST and HIR
dumps retain compiler notation.

### Options

| English | Somali | Applies to |
| --- | --- | --- |
| `--help`, `-h` | `--caawimo` | General and command help. |
| `--version`, `-V` | `--nooc` | Software version. |
| `--strict` | `--adag` | Run, build, check, debug and source evaluation. |
| `--offline` | `--kayd-keliya` | Locked, verified cached **package dependencies** only; does not disable network APIs in a running program. |
| `--no-color` | `--midab-dami` | Diagnostic color. |
| `--name` | `--magac` | Project creation and a single build's executable name. |
| `--opt-level`, `-O` | `--heer-hagaajin` | Build optimization level `0..3`. |
| `--quiet`, `-q` | `--aamus` | Build success messages. |
| `--output`, `-o` | `--natiijo` | Exact build output path; documentation and data generation also accept an output path. |
| `--lib` | `--maktabad` | Create a library project. |
| `--write` | `--qor` | Format files in place. |
| `--private` | `--gaar` | Include private names in generated documentation. |
| `--break` | `--hakad` | Debugger breakpoint at `[FILE:]LINE`; existing `--jebi` remains. |
| `--continue` | `--soco` | Continue to a breakpoint when debugging. |
| `--parallel` | `--barbar` | Run tests with an automatic worker count; existing `--wada` remains. |
| `--filter`, `--index` | `--shaandhe`, `--lambarka` | Select tests by name or zero-based index. |
| `--workers`, `--coverage`, `--report` | `--shaqaale`, `--dabool`, `--warbixin` | Set test workers, coverage, and report path. |

`init`, `debug`, and `data generate` have further options under their own
`--help` screens. Existing invocations such as `soplang -c CODE`, `-f FILE`,
`-e NUMBER`, `--ast FILE`, `--hir FILE`, `--ku-dar ... --qeexid ...`,
`--ka-saar`, `--cusbooneysii`, and `--daabac` remain supported.

Help and version output go to stdout and exit successfully. Invalid CLI
arguments go to stderr with exit status 2. Compilation and unhandled runtime
failures retain exit status 1. The localization does not change program output
or the language's keywords and compatibility names.

## Error explanations

A missing source file produces a contextual message such as:

```text
Khalad fulin: Faylka 'magac.sop' lama helin.
```

Permission, invalid UTF-8, directory, storage, and tool failures receive their
own explanations where the operating system provides a recognizable cause.
Unknown causes receive a general Somali explanation. Infrastructure errors
without a source location omit line and column numbers.

Source errors still retain their original file, one-based line and column,
error code when available, source text, and caret. Tabs and Unicode display
width continue to determine caret spacing. These locations describe source
characters rather than UTF-8 byte offsets.

## Technical details

Set `SOPLANG_ERROR_DETAILS=1` to include original OS, compiler, or linker
messages under `Faahfaahin farsamo:` on stderr:

```sh
SOPLANG_ERROR_DETAILS=1 soplang --build program.sop -o program
```

In PowerShell:

```powershell
$env:SOPLANG_ERROR_DETAILS = '1'
soplang --build program.sop -o program
Remove-Item Env:SOPLANG_ERROR_DETAILS
```

The additional text can be English. Tool names, paths, versions, platform
identifiers, and error codes remain literal in both modes. Keep these details
when reporting a compiler or package problem. Unset the variable or give it
another value to return to normal explanations.

Caught errors retain their Somali primary message regardless of this setting;
the troubleshooting details do not become part of the value supplied to
`qabo`.

`NO_COLOR=1` or `--no-color` disables diagnostic color. Neither setting
translates text printed by the Soplang program itself. Third-party panics and
failures before Soplang starts are outside normal diagnostic handling.

## Create a project

`soplang init [directory] [--lib] [--name name]` creates a manifest and a starter.
Applications start in `main.sop`, libraries in `lib.sop`. See
[packages](packages.md#create-a-project). `soplang init --help` explains the
options in Somali. Existing manifests and starters are preserved.

## Git packages and offline builds

Add a repository dependency with `soplang --ku-dar name --qeexid source`.
`source` may be `github.com/team/project@tag`, a `git+URL#revision`, or a local
path. `--cusbooneysii` updates the lock explicitly; `--ka-saar name` removes a
dependency. `--offline` uses an existing lock and verified cache without a fetch.
See [packages](packages.md) for selectors, graph constraints and cache behavior.
