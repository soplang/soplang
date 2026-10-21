# Text file access

The initial 3.3 stage adds convenience UTF-8 text file operations. Paths are
resolved from the process working directory. Import paths for `keen` keep their
existing rule relative to the importing source file.

These operations open and close resources automatically for each call. Missing
files, permission failures, invalid UTF-8, and other I/O errors are catchable
Somali runtime errors with optional technical details behind
`SOPLANG_ERROR_DETAILS=1`.

## Builtins

| Function | Meaning |
| --- | --- |
| `akhrifayl(path)` | Read an entire UTF-8 text file into a `qoraal`. |
| `qor_fayl(path, text)` | Create or overwrite a text file. Returns `maran`. |
| `kudar_fayl(path, text)` | Append text, creating the file when absent. Returns `maran`. |

The path argument must be a string. The written text uses the normal Soplang
string conversion for the content argument.

```soplang
qor_fayl("magac.txt", "Salaan")
door text = akhrifayl("magac.txt")
kudar_fayl("magac.txt", "\nNabadgelyo")
qor(text)
```

Path helpers, directory management, and file metadata are documented in
[filesystem](filesystem.md). Binary files, streaming handles, and JSON remain
later 3.3 work. See [ROADMAP.md](../../ROADMAP.md).
