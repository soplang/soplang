# Projects and packages

A project has a `soplang.toml` manifest. Packages come from Git repositories or
local paths; no central package host is required. When running or building a
source file, Soplang resolves its dependency graph before execution.

## Create a project

```sh
soplang init my-app
cd my-app
soplang main.sop
soplang --build main.sop -o my-app
```

`init` creates `soplang.toml` and a root `main.sop`. Without a directory, it uses
the current directory. `--name` sets the package name; otherwise the name is
derived from the directory using lowercase ASCII letters, digits, `-`, and `_`.
The initial package version is `0.1.0`; its language requirement records the
installed compiler version.

`soplang init my-library --lib --name xisaab` creates a root `lib.sop` with an
exported `salaan` greeting function. Existing manifests and starters, including
dangling symlinks, are preserved. Failed file creation removes the files created
by that attempt.

## Manifest

```toml
magac = "app"
nusxa = "0.1.0"
soplang = "3"

[ku-tiirsan]
xisaab = { git = "https://github.com/koox/xisaab.git", rev = "v1.2.3" }
```

The repository name above is illustrative.

| Field | Required | Meaning |
| --- | --- | --- |
| `magac` | yes | Package name: a lowercase ASCII letter followed by lowercase letters, digits, `-`, or `_`, at most 64 characters. |
| `nusxa` | yes | Package version, such as `1.2.3` or `1.2.3-alpha`. |
| `soplang` | yes | Minimum compiler version, with the same major version: `3`, `3.1`, or `3.1.0`. |
| `gelid` | no | Source directory inside the package; defaults to `.`. Absolute paths and `..` components are rejected. |
| `[ku-tiirsan]` | no | Dependencies, keyed by their manifest's `magac`. |

TOML syntax includes quoted keys, literal/basic strings, escapes, multiline
strings, dotted tables, and inline comments. The manifest schema requires string
identity fields and correctly typed dependency declarations. Duplicate keys or
tables, unknown fields, and wrong value types fail with `E057`. Syntax errors
retain the manifest's file, line and column. Original parser/tool diagnostics
appear only with `SOPLANG_ERROR_DETAILS=1`.

The source file's nearest manifest defines its project. Files inside the current
working directory do not search above that directory for another manifest.
`-c` does not implicitly search for a project; an explicit file import can
establish that file's project context.

## Git dependencies

A structured Git declaration has `git` and an optional `rev`. The revision is an
exact branch, tag, or full commit identifier; its default is `HEAD`.

```toml
[ku-tiirsan]
xisaab = { git = "https://github.com/koox/xisaab.git", rev = "v1.2.3" }
```

Equivalent string forms are supported:

```toml
[ku-tiirsan]
xisaab = "github.com/koox/xisaab@v1.2.3"
```

Or use `git+https://github.com/koox/xisaab.git#v1.2.3`. Full `.git` URLs with
`#revision`, `ssh://` URLs, and `git@host:path.git` spellings are also accepted.
Short forms support `github.com`, `gitlab.com`, and `codeberg.org`. Other hosts
work through full URLs. `file://` repositories are useful for local mirrors and
testing. A full URL without a `.git` suffix can always use the structured `git`
form or the `git+` prefix.

Git must be installed for an uncached fetch. Existing SSH keys and Git
credential helpers provide authentication; interactive credential prompts are
disabled. URL query/fragment components and embedded passwords are not accepted
as repository addresses. Put the revision in `rev` or the supported selector
form instead.

Resolution fetches committed objects rather than copying the repository's dirty
working tree. Annotated tags are peeled to commits. Both SHA-1 and SHA-256 Git
repositories retain their full commit identifiers. No working-tree checkout, package filters or package-supplied hooks,
or submodule initialization runs during installation. The implementation reads
[Git tree records](https://git-scm.com/docs/git-ls-tree) and
[blob contents](https://git-scm.com/docs/git-cat-file).

Snapshots contain regular UTF-8-named files, including committed hidden files.
Symlinks and submodules are rejected. Files are stored as data; executable mode
bits are not part of this source-package format. A snapshot is limited to 32 MiB
and 16384 files, with paths up to 4096 bytes and 64 components. Downloaded
packages cannot depend on mutable local paths or import files outside their
snapshot. The graph is limited to 1024 packages and 64 levels.

## Local dependencies

```toml
[ku-tiirsan]
xisaab = "../xisaab"
# Equivalent: xisaab = { jid = "../xisaab" }
```

The path is relative to the containing manifest and may use `..` to reach a
sibling project. That directory must contain a manifest with the same package
name. Local source remains editable; its bytes are not content-locked.

## Lockfile and one version per package

The first successful Git resolution creates `soplang.lock`. Add/remove/update
commands also write it. A later run or build uses the recorded commit even when
a branch or tag moves. It does not silently select newer repository contents.

```toml
# soplang.lock
[[xirmo]]
magac = "xisaab"
nusxa = "1.2.3"
jid = "git+https://github.com/koox/xisaab.git"
sumad = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
commit = "0123456789abcdef0123456789abcdef01234567"
codsiyo = ["v1.2.3"]
```

`nusxa` comes from the dependency manifest. `commit` pins the complete Git
commit identifier. `sumad` is the SHA-256 of sorted paths, byte lengths and file
contents. `codsiyo` records the revisions requested by the graph. Equivalent URL
spellings are normalized. Several selectors may share one lock entry when they
resolve to the same repository and commit. Conflicting sources or commits for
one package name are rejected; versions are not installed side by side.

One root lock covers direct and transitive dependencies. A dependency's own
lockfile is ordinary snapshot data and cannot override that root selection.
Imports use a validated graph snapshot throughout compilation. A REPL session
keeps its selection until restarted, including across collection and subsequent
aliases. Restart after changing dependency declarations or updating the lock.

Lock comparison uses TOML values, so harmless formatting, comments and entry
ordering are accepted. Duplicate entries, unknown fields, malformed hashes or
versions, missing sources and changed selectors are errors. Malformed or dangling
locks fail before fetching dependencies. A lock with changed declarations must
be updated explicitly.

## Cache and offline builds

`SOPLANG_CACHE` sets the cache directory; the default is `~/.soplang/cache`.
Content trees are stored under their SHA-256. Each use verifies their contents.
Concurrent installers stage separate trees and serialize publication with
operating-system file locks. Failed fetches remove their staging directories.

```sh
soplang --offline main.sop
soplang --offline --build main.sop -o my-app
```

`SOPLANG_OFFLINE=1` is equivalent. Downloaded dependencies require a valid lock
and verified cached contents; a missing or corrupt snapshot fails without
fetching or executing it. Local paths remain usable without a cache. This flag
controls dependency acquisition; application networking keeps its usual behavior. A valid locked Git cache works without Git installed or access
to the original repository. Online operation can restore a corrupt tree from
the pinned commit after verifying its expected content hash.

A compiled executable contains the imported Soplang code. It does not need the
source project, package cache, Git, or compiler at runtime.

## Commands

| Command | Effect |
| --- | --- |
| `soplang --ku-dar xisaab --qeexid github.com/koox/xisaab@v1.2.3` | Add a Git dependency and resolve the graph. |
| `soplang --ku-dar xisaab --qeexid ../xisaab` | Add a local dependency. |
| `soplang --cusbooneysii` | Resolve the current selectors again and rewrite the lock. |
| `soplang --ka-saar xisaab` | Remove a dependency and rewrite the lock. |

Commands run in the current project directory. Resolution failures leave the
manifest and lock unchanged. Successful metadata writes replace each file
atomically and serialize strings as valid TOML. `--cusbooneysii` is the explicit
operation for following a newer branch/tag target; it requires access to the
sources. Normal runs consume the existing selection.

Git commands have a 60-second timeout each. `SOPLANG_GIT_TIMEOUT_MS` can select
1 to 300000 milliseconds. Tool output is bounded, and raw stderr is retained as
technical detail rather than included in the ordinary Somali error message.

## Imports

```soplang
keen "xisaab" sida math
qor(math.wad(1, 2))
```

A bare package import selects `<gelid>/lib.sop` when present, with
`<gelid>/<package-name>.sop` as a compatibility fallback.
`keen "xisaab/util.sop"` selects `<gelid>/util.sop`. Existing relative file paths
retain precedence. See [modules](modules.md) for exports and selective imports.

## Existing registry and archive sources

The earlier directory registry and HTTP `.sopkg` archive providers remain
supported. They are optional compatibility sources; Git packages do not need
that registry or a separate package-hosting service.

`kayd:^1.2.3`, `kayd:~1.2.3`, and exact versions resolve through `SOPLANG_KAYD`
(default `~/.soplang/kayd`). `--daabac` publishes into that local registry.
HTTP/HTTPS `.sopkg` URLs resolve an archive directly. Their locked `sumad` fields
pin content. In online mode, a registry version is still checked against its
published bytes; offline mode can use its verified cache.
