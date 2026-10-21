# Namespaced imports

`keen "file.sop"` still copies the imported file into the current program, as
before. `keen "file.sop" sida name` keeps that file in its own namespace.

`sida` is contextual. It is an ordinary name except immediately after an import
path.

The namespace is initialized when the import is reached. Without `soosaar`, its
top-level functions and variables are fields of `name`, except names that
start with `_`. A `soosaar` list exports only the names it mentions, and those
names must be defined in the file. Functions and classes defined in
the file are not copied into the caller's global scope.

```soplang
keen "math.sop" sida xisaab
door add = xisaab.wad
qor(add(xisaab.base, 2))
```

```soplang
soosaar wad, base
door base = 10
hawl wad(a, b) { celi a + b + base }
hawl kale() { celi 1 }
```

`kale` is omitted from the namespace. `soosaar` is contextual: it starts an
export list only when the next word is a name. Elsewhere it can still be an
ordinary name.

`keen "file.sop" ka name` binds that public name in the caller. `name sida local`
binds it under another name. A name that was not exported is an import error.
`ka` is contextual after an import path. It cannot be combined with a namespace
`sida` on the same import.

```soplang
keen "math.sop" ka wad sida add, base
qor(add(base, 2))
```

Each canonical file has one namespace and one initializer per program or REPL
session. Multiple aliases and selective imports reuse the same exports, including
when several modules import the same dependency. Initialization runs at the first
import reached, in source order. A cycle through a module still being loaded is
an import error with the location of the import that closes the cycle. Flat
`keen` imports keep their existing once-per-program expansion behavior.

Private functions retain their lexical names. Parameters may shadow them;
default expressions and interpolated strings resolve names in the same scope as
ordinary expressions. Classes used internally have distinct identities across
modules, including inside nested type annotations.

A public class is exported with the same rules as a function. `cusub m.Qof(...)`
constructs it, `m.Qof q = cusub m.Qof(...)` names its type, and `qaab Arday dhaxal m.Qof`
extends it. `keen "people.sop" ka Qof sida Student` binds that same class under
`Student`. A class omitted from `soosaar`, or one whose name starts with `_`
when `soosaar` is absent, cannot be constructed or named from the caller.

Class aliases belong to the module that imports them. Two modules can each use
`sida m` or `ka Qof sida Local` for different classes without changing each
other's constructors or type contracts. A selective class import can be exported
again with `soosaar Local`, retaining its original identity.

Methods, constructor defaults, and nested method functions can capture private
module values and functions. An instance retains its lexical method environment
when another call to a factory creates an instance with different captures.
Inherited methods and `waalid` use the same environment, including named calls.
These environments remain valid across garbage collection and REPL commands.

REPL imports survive garbage collection and later commands. An analysis failure
does not reserve a new module or discard the initializer needed by a later
successful import. A failed runtime initializer remains eligible for retry.

`keen "xisaab"` loads a package named in the nearest `soplang.toml`. The file is
`<gelid>/lib.sop`, or `<gelid>/xisaab.sop` for older packages. `keen "xisaab/util.sop"` loads `<gelid>/util.sop`. A
relative path that already names a file stays a file import. See
[packages](packages.md).

Package imports share the consuming project's validated dependency graph and
root lock. A cached dependency's own manifest identifies its direct dependencies;
its own lock cannot select different transitive versions. Git snapshot-relative
imports must stay inside that snapshot. REPL dependency selections remain stable
until the session restarts. See [packages](packages.md).
