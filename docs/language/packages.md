# Package manifest

A directory can contain `soplang.toml`. When Soplang runs or builds a `.sop`
file in that directory, it reads the manifest first. A directory without the
file behaves as before. `-c` programs do not look for a manifest.

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

Blank lines and `#` comments are ignored. Keys cannot be repeated. Unknown keys
and unknown `[tables]` are rejected. Failures use error code `E057`.

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
resolved dependencies is an error. Download and version constraints on
dependencies remain later 3.4 work.
