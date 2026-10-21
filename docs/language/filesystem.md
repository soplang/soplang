# Paths and filesystem management

This 3.3 slice adds path helpers, metadata queries, and file and directory
management. Paths resolve from the process working directory unless noted.
Errors remain catchable with Somali messages.

UTF-8 text convenience calls remain in [text files](text-files.md).

## Metadata

| Function | Meaning |
| --- | --- |
| `fayl_jiraa(path)` | Returns `run` when the path exists. |
| `fayl_xog(path)` | Returns a `walax` describing the path. |

When the path is missing, `fayl_xog` returns `{ jir: been }`. Otherwise it
includes:

| Field | Meaning |
| --- | --- |
| `jir` | Always `run` when present. |
| `nooc` | `"fayl"`, `"gal"`, or `"xidhiidh"` for symlinks. |
| `cabbir` | Size in bytes (`abn`). |
| `wax_laga_beddelay` | Modification time as whole seconds since the Unix epoch. |

## Paths

| Function | Meaning |
| --- | --- |
| `isku_xidh(part, ...)` | Join one or more path segments. |
| `baabi_jid(path)` | Resolve an absolute path from the working directory. |

## Files

| Function | Meaning |
| --- | --- |
| `nuqul_fayl(laga, u, beddel?)` | Copy a file. The optional third argument allows overwriting an existing destination. |
| `wareeji_fayl(laga, u)` | Move or rename a file or directory. |
| `tirtir_fayl(path)` | Delete a file. |

## Directories

| Function | Meaning |
| --- | --- |
| `abuur_gal(path)` | Create a directory and any missing parents. |
| `tir_gal(path)` | List entry names in a directory as a `teed`. |
| `tirtir_gal(path, qoto_dheer?)` | Remove a directory. Pass `run` as the second argument to remove recursively. |

Cross-filesystem moves, symbolic-link policies, and streaming handles remain
later 3.3 work. See [ROADMAP.md](../../ROADMAP.md).
