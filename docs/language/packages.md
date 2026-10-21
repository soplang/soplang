# Package manifest

A directory can contain `soplang.toml`. When Soplang runs or builds a `.sop`
file, it reads the nearest manifest in that file's directory or a parent
directory. A file inside the working directory is not matched to a manifest
above that directory. A tree without the file behaves as before. `-c` programs
do not look for a manifest.

```toml
magac = "xisaab"
nusxa = "0.1.0"
soplang = "3.1"
gelid = "src"
```

| Field | Required | Meaning |
| --- | --- | --- |
| `magac` | yes | Package name. It starts with a lowercase letter and contains only lowercase letters, digits, `-`, and `_`. |
| `nusxa` | yes | Package version, such as `1.2.3` or `1.2.3-alpha`. |
| `soplang` | yes | Minimum Soplang version: `3`, `3.1`, or `3.1.0`. The compiler's major version must match, and it must be at least the requested minor and patch. |
| `gelid` | no | Source directory inside the package. The default is `.`. It cannot be absolute or contain `..`. |

The file uses TOML syntax, including quoted keys, literal/basic strings,
escapes, multiline strings, dotted tables, and inline comments. Its schema still
requires string identity fields and a dependency table. Duplicate keys/tables,
unknown fields and incorrectly typed values are rejected. Failures use `E057`.
TOML syntax errors retain their source file, line and column; the parser's
original diagnostic is available only in technical details.

## Create a project

```sh
soplang init my-app
cd my-app
soplang main.sop
soplang --build main.sop -o my-app
```

`init` creates `soplang.toml` and a root-level `main.sop`. Run `soplang init`
without a directory to initialize the current directory. `--name` sets the
package name; otherwise it is derived from the directory name using lowercase
ASCII letters, digits, `-`, and `_`. A new package starts at `0.1.0` and records
the installed compiler version as its minimum Soplang version.

`soplang init my-library --lib --name xisaab` creates `lib.sop` with an exported
`salaan` greeting function. A bare dependency import
loads `lib.sop` when present, or the older `<package-name>.sop` layout otherwise.
Explicit subpaths keep their existing meaning. Neither command replaces an
existing manifest or starter, including dangling symlinks. File creation failures
remove the files created by that attempt.

## Local dependencies

`[ku-tiirsan]` maps a package name to a relative path. The path is resolved from
the directory that contains the manifest. It may use `..` to reach a sibling
package. The directory must contain `soplang.toml`, and that file's `magac`
must be the same name. Dependencies are checked recursively, and a cycle is an
error.

```toml
[ku-tiirsan]
xisaab = "../xisaab"
```

A local dependency may also be written as `xisaab = { jid = "../xisaab" }` or
as a `[ku-tiirsan.xisaab]` table with a `jid` string. Package commands rewrite
these forms into canonical TOML and preserve the actual string values.

A `soplang.lock` file, when present, must list the same packages. Its
entries are sorted by name:

```toml
# soplang.lock

[[xirmo]]
magac = "xisaab"
nusxa = "0.1.0"
jid = "../xisaab"
```

`nusxa` is the version from that package's manifest. `jid` is the path from the
root package. A missing lockfile is allowed. A lockfile that does not match the
resolved dependencies is an error. Valid TOML formatting, comments, and entry
ordering do not change the lock's meaning. Duplicate package entries, unknown
fields, malformed versions and invalid SHA-256 strings are errors. A malformed
or dangling lockfile is rejected before dependency resolution.

## Registry and archives

The package source is a directory registry, not Git. `SOPLANG_KAYD` is that
directory. When it is unset, Soplang uses `~/.soplang/kayd`. Installed copies
live in `SOPLANG_CACHE`, or `~/.soplang/cache`.

```toml
[ku-tiirsan]
xisaab = "kayd:^0.1.0"
web = "https://example.com/web-1.2.3.sopkg"
```

`kayd:` is followed by an exact version (`0.1.0` or `0.1.0-alpha`), a caret
bound (`^1.2.3`), or a tilde bound (`~1.2.3`). A pre-release matches only an
exact request. `^1.2.3` accepts `1.2.3` up to, but not including, `2.0.0`.
`^0.1.2` stays inside `0.1`. `~1.2.3` stays inside `1.2`. The highest matching
version in the registry is chosen when no lock entry pins it.

An `http://` or `https://` value downloads one `.sopkg` archive. The archive
starts with `SOPKG1\n`. Each file is a relative UTF-8 path, a newline, a
decimal byte length, a newline, and the bytes. Paths cannot be absolute or
contain `..`. The total size limit is 32 MiB.

A published package cannot depend on a relative path. Use `kayd:` or an archive
URL for those dependencies. The registry layout is
`<kayd>/<magac>/<nusxa>/` plus `<kayd>/<magac>/<magac>-<nusxa>.sopkg`.

## Cache and integrity

Registry and archive dependencies are copied into the cache. The lock records
them with `jid = "kayd"` or the archive URL, plus `sumad`, a SHA-256 of the
sorted file paths and bytes. A present lock pins that version and hash. A
mismatch is `E057`. Path dependencies stay as relative `jid` values and have
no `sumad`.

```toml
[[xirmo]]
magac = "xisaab"
nusxa = "0.1.0"
jid = "kayd"
sumad = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
```

## Commands

These commands run in the current directory and rewrite `soplang.toml` in
canonical field order. They also write `soplang.lock`.

| Command | Effect |
| --- | --- |
| `soplang --ku-dar <magac> --qeexid <jid ama nusxa>` | Add one dependency and resolve it. |
| `soplang --cusbooneysii` | Resolve every constraint to its current highest match and rewrite the lock. |
| `soplang --ka-saar <magac>` | Remove one dependency and rewrite the lock. |
| `soplang --daabac` | Copy this package into the registry. An existing version is rejected. |

`keen "xisaab"` loads `<gelid>/lib.sop` when present, or `<gelid>/xisaab.sop` from that dependency. `keen
"xisaab/util.sop"` loads `<gelid>/util.sop`. A relative file that already
exists is still a file import.
