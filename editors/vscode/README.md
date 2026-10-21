# Soplang for VS Code

Install Soplang and make `soplang` available on `PATH`, then open a `.sop` file.
The extension starts `soplang lsp` over standard input/output in trusted
workspaces. It uses the compiler's language server for diagnostics, completion,
hover, definitions, references, rename, formatting, code actions, and semantic
highlighting. File documents and untitled buffers are supported.

The extension runs the installed compiler. It does not download or bundle one.
Use a compiler built from the same Soplang milestone as this extension.

## Settings

| Setting | Default | Meaning |
| --- | --- | --- |
| `soplang.server.path` | `soplang` | Executable path or command on `PATH`. Relative paths use the first workspace folder. |
| `soplang.server.enabled` | `true` | Enable the language server. |
| `soplang.trace.server` | `off` | Protocol tracing: `off`, `messages`, or `verbose`. |

Use **Soplang: Restart Language Server** after replacing the compiler binary.
Changing server settings restarts it automatically. A window uses one server;
the first folder controls executable configuration in a multi-root workspace.
Imports are resolved relative to each source file.

Syntax highlighting, comments, brackets, and snippets work without a compiler
and in untrusted workspaces. Starting the executable requires workspace trust.
Failure details appear in the **Soplang** output channel; the error notification
offers the executable setting. No shell is used to launch the server.

The grammar supports `//` and `/* ... */` comments, both quote styles, triple
strings, escaped `e` strings, raw/default strings, and `f` interpolation.
Ordinary strings keep backslashes literally. Snippets include functions,
classes, loops, error cleanup, tests, and imports.

## Integration checks

Build the compiler with `cargo build --locked --bin soplang` at the repository
root, then run `npm run test:integration` here. The harness checks every snippet
with the compiler and starts a separate VS Code extension host. It verifies
live diagnostics, hover, definition, rename, formatting and restart.

By default the official test runner downloads VS Code stable. Set
`SOPLANG_VSCODE_EXECUTABLE` to an installed Electron executable (for example,
`/usr/share/code/code`) to use that installation. `SOPLANG_VSCODE_VERSION` selects
a downloadable version; `SOPLANG_TEST_BINARY` selects the compiler. The test uses
a temporary workspace, editor data and extension directory, and removes them
when finished. Linux runs with the headless Ozone backend.

The editor workflow tests and packages on Linux, Windows and macOS and runs the
compiler/client integration check on Linux. It does not publish the extension.

## Develop and package

```sh
npm ci --ignore-scripts
npm run check
npm run package
code --install-extension soplang.vsix
```

Node.js 22 or newer is required for the packaging tools. The extension itself
supports VS Code 1.85 or newer. `npm test` checks client startup/lifecycle with
the VS Code API mocked and tokenizes source using the real TextMate/Oniguruma
engine. The Rust language-server tests exercise framed protocol traffic.
