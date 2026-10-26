# Regular expressions

`regex_raadi`, `regex_dhammaan`, and `regex_beddel` search and replace text.
Invalid patterns are catchable and use error code `E049`. The original pattern
compiler message is kept as technical detail and is not part of the caught
message.

Patterns use the Rust `regex` dialect. Matching is Unicode-aware. The dialect
does not include backreferences or look-around.

| Function | Meaning |
| --- | --- |
| `regex_raadi(pattern, text)` | First match as `qoraal`, or `maran` when nothing matches. |
| `regex_dhammaan(pattern, text)` | Every non-overlapping match as a `teed` of strings. |
| `regex_beddel(pattern, text, replacement)` | Replace every match. `$1`, `$2`, and `$$` follow the usual replacement syntax. |

```soplang
qor(regex_raadi("a+", "baaac"))
qor(regex_dhammaan(e"\\d+", "a1b22"))
qor(regex_beddel("(a)(b)", "ab", "$2$1"))
```
