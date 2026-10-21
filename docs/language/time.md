# Date and time

UTC timestamps are whole seconds since `1970-01-01T00:00:00Z`. Parsing,
formatting, calendar parts, and durations are catchable and use error code
`E050`. Years outside `0001` through `9999` are rejected. Offsets other than
`Z` are not accepted.

| Function | Meaning |
| --- | --- |
| `waqti()` | Current UTC time as `abn` seconds. |
| `waqti_akhris(text)` | Parse `YYYY-MM-DD` or `YYYY-MM-DDTHH:MM:SSZ`. A date alone is midnight UTC. |
| `waqti_qor(seconds)` | Format `YYYY-MM-DDTHH:MM:SSZ`. |
| `waqti_qayb(seconds)` | A `walax` with `sanad`, `bil`, `maalin`, `saac`, `daqiiqad`, and `ilbiriqsi`. |
| `waqti_ka_qayb(parts)` | Build seconds from that `walax`. |
| `waqti_kudar(seconds, delta)` | Add a duration in seconds. A negative delta subtracts. |

```soplang
door instant = waqti_akhris("2026-10-21T08:12:00Z")
qor(waqti_qor(waqti_kudar(instant, 3600)))
qor(waqti_qayb(instant).maalin)
```
