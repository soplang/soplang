# Namespaced imports

`keen "file.sop"` still copies the imported file into the current program, as
before. `keen "file.sop" sida name` keeps that file in its own namespace.

`sida` is contextual. It is an ordinary name except immediately after an import
path.

The namespace is initialized when the import is reached. Its top-level
functions and variables are fields of `name`. Names that start with `_` stay
inside the file. Functions and classes defined in the file are not copied into
the caller's global scope.

```soplang
keen "math.sop" sida xisaab
door add = xisaab.wad
qor(add(xisaab.base, 2))
```

A file can be imported this way once. Importing it again, or a cycle back to a
file already loaded, is a catchable import error. Flat `keen` imports keep
their existing cycle behavior.

Explicit export lists, selective imports, and package imports remain later 3.4
work.
