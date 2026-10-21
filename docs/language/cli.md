# CLI help and diagnostics

Soplang 3.0.1 localizes CLI help and argument errors as well as compiler and
runtime explanations. Help, invalid arguments, file access, and tool failures
use Somali explanations while retaining the existing command interface.

## Commands

Run `soplang --help` for Somali descriptions of the supported options.
Option spellings remain unchanged:

```sh
soplang program.sop
soplang -c 'qor("Salaan!")'
soplang --build program.sop -o program
soplang --strict program.sop
soplang --hir program.sop
soplang --version
```

With no arguments, Soplang opens the REPL. `-i` continues in the REPL after
running a file or example. Use `/caawi` for commands and `/bixi` to exit.
AST and HIR dumps retain compiler instruction notation; their surrounding
help and section headings use Somali.

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
