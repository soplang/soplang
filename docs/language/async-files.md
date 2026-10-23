# Asynchronous file operations

The whole-file task functions return a handle immediately. Pass the handle to
`sug` to receive the result or a catchable file error. Existing synchronous
file functions remain available.

```sop
door write = qor_fayl_shaqo("notes.txt", "Salaan")
sug(write)
door contents = sug(akhrifayl_shaqo("notes.txt"))
qor(contents)
```

| Text | Bytes | Result of `sug` |
| --- | --- | --- |
| `akhrifayl_shaqo(jid)` | `akhrif_baytyo_shaqo(jid)` | `qoraal` or `baytyo` |
| `qor_fayl_shaqo(jid, xog)` | `qor_baytyo_shaqo(jid, xog)` | `maran` |
| `kudar_fayl_shaqo(jid, xog)` | `kudar_baytyo_shaqo(jid, xog)` | `maran` |

Each operation accepts at most 16 MiB of data. A context may have at most 16
file tasks running at once. The text reader requires valid UTF-8. `baaji(task)`
cancels waiting for an operation, but an already started write may have written
some data; cancellation does not roll back filesystem changes. Await a write
before depending on its contents or starting another write to the same path.
