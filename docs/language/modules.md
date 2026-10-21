# Namespaced imports

`keen "file.sop"` still copies the imported file into the current program, as
before. `keen "file.sop" sida name` keeps that file in its own namespace.

`sida` is contextual. It is an ordinary name except immediately after an import
path.

The namespace is initialized when the import is reached. Without `soosaar`, its
top-level functions and variables are fields of `name`, except names that
start with `_`. A `soosaar` list exports only the names it mentions, and those
names must already be defined in the file. Functions and classes defined in
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

A file can be imported this way once. Importing it again, or a cycle back to a
file already loaded, is an import error. Flat `keen` imports keep their
existing cycle behavior.

Selective imports and package imports remain later 3.4 work.
