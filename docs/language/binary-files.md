# Binary file access

Whole-file binary operations complement the UTF-8 [text file](text-files.md) API.
Paths resolve from the process working directory. Reads return `baytyo`; writes
require a `baytyo` argument so arbitrary byte sequences are preserved.

| Function | Meaning |
| --- | --- |
| `akhrif_baytyo(path)` | Read an entire file into a `baytyo`. |
| `qor_baytyo(path, data)` | Create or overwrite a file. Returns `maran`. |
| `kudar_baytyo(path, data)` | Append bytes, creating the file when absent. Returns `maran`. |

```soplang
door signature = baytyo([0x89, 0x50, 0x4E, 0x47])
qor_baytyo("image.png", signature)
door loaded = akhrif_baytyo("image.png")
qor(dherer(loaded))
```

Streaming handles, seeking, and atomic replacement remain later 3.3 work.
See [ROADMAP.md](../../ROADMAP.md).
