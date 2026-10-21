# Program arguments and environment

`amarro`, `deegaan`, `deegaan_qor`, and `deegaan_tirtir` expose the process
arguments and environment. Invalid environment names are catchable and use
error code `E051`.

`amarro()` returns the arguments meant for the Soplang program. It does not
include the `soplang` executable, compiler options, or the source filename.

```sh
soplang tool.sop one --two
soplang -c 'qor(amarro())' one --two
```

Both invocations see `["one", "--two"]`. A built binary sees the arguments
after its own program name.

| Function | Meaning |
| --- | --- |
| `amarro()` | Program arguments as a `teed` of strings. |
| `deegaan(name)` | The environment value, or `maran` when it is absent. |
| `deegaan_qor(name, value)` | Set an environment variable for this process. |
| `deegaan_tirtir(name)` | Remove an environment variable. Removing a missing name succeeds. |

Names must be non-empty and cannot contain `=` or a zero byte.

```soplang
door args = amarro()
door mode = deegaan("SOPLANG_MODE")
```
