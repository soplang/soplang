# File handles

These calls open one file and keep a position in it. Paths resolve from the
process working directory. Close a handle with `fayl_xidh`. A handle that was
not closed stays open until its runtime context is destroyed. Handles belong
to the context that opened them; another session cannot read or close them.
Use `dhammee` to close handles promptly, including after errors. Failures use `E058` and are
catchable.

| Function | Meaning |
| --- | --- |
| `fayl_fur(path, hab)` | Open a file. `hab` is `"akhris"`, `"qor"`, `"kudar"`, or `"akhris_qor"`. |
| `fayl_akhris(fayl, cabbir)` | Read at most `cabbir` bytes, from 1 to 16777216. The end of the file returns empty `baytyo`. |
| `fayl_xariiq(fayl)` | Read one UTF-8 line without its newline. The end of the file returns `maran`. |
| `fayl_qor(fayl, xog)` | Write `qoraal` or `baytyo`. `"akhris"` rejects writes. |
| `fayl_nadiifi(fayl)` | Flush buffered writes. |
| `fayl_xidh(fayl)` | Close the handle. |
| `fayl_raad(fayl, meel, asal)` | Move the position. `asal` is `"bilow"`, `"hadda"`, or `"dhamaad"`. |
| `fayl_meel(fayl)` | Return the position as `abn`. |
| `fayl_beddel(path, xog)` | Replace a file with `qoraal` or `baytyo`. |

`"qor"` creates or truncates the file. `"kudar"` creates it and appends. Writes
in `"kudar"` stay at the end even after `fayl_raad`. `"akhris_qor"` creates the
file when it is missing and does not truncate it.

The handle is a `walax` with `nooc` `"fayl"`, `aqoonsi`, and `hab`.

## Atomic replacement

`fayl_beddel` creates a unique temporary file in the destination directory, flushes it,
then renames it onto the destination. On POSIX, that rename replaces an existing
file in the same directory atomically. On Windows, the replacement uses
`MoveFileExW` with replace-existing and write-through. A crash before the rename
leaves the temporary file and the previous destination. The rename replaces a
symlink itself and does not overwrite the symlink's target.

Temporary files are created exclusively. Existing neighboring files and symlinks
are never used as scratch space. Concurrent replacements use separate temporary
files; each successful rename installs one complete value. Failure removes only
the temporary file created by that operation.
