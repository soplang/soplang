# HTTP client

`http_codso_shaqo(hab, url, xulasho?)` performs a request on the I/O executor
and returns a task. `sug(task)` returns the normal response object;
`baaji(task)` or `sug_waqti(task, millisekan)` cancels an in-flight request.
The async request body is limited to 16 MiB, and a runtime may have at most 64
HTTP requests in flight. The `cabbir` response limit applies while chunks are
read, so a larger body is not retained in memory.

`http_codso(method, url, options?)` sends one synchronous HTTP request and
waits for the response. Methods are `GET`, `POST`, `PUT`, `PATCH`, and
`DELETE`. URLs must start with `http://` or `https://`. `https` verifies the
server certificate with the operating system's trusted certificates.

Failures are catchable and use error code `E053`. A response status such as
404 is a result, not an error. The original client message is kept as a
technical detail.

## Options

| Field | Meaning |
| --- | --- |
| `cinwaan` | Request headers as a `walax` of strings. |
| `qoraal` | Text body. |
| `baytyo` | Byte body. |
| `json` | JSON body. Sets `content-type` to `application/json` when that header is absent. |
| `waqti` | Connection and total timeout in seconds. The default is 30. |
| `leexasho` | How many redirects to follow, from 0 through 10. The default is 5. Extra redirects return the last response. |
| `cabbir` | Maximum response body size in bytes, from 1 through 16777216. The default is 1048576. |

Only one of `qoraal`, `baytyo`, and `json` may be set.

## Response

| Field | Meaning |
| --- | --- |
| `heer` | HTTP status code. |
| `cinwaan` | Response headers. Names are lowercase. |
| `qoraal` | Body text, or `maran` when the body is not UTF-8. |
| `baytyo` | Raw body. |
| `json` | Parsed JSON when the content type contains `json`; otherwise `maran`. |

```soplang
door page = http_codso("GET", "https://example.com/")
qor(page.heer)
door created = http_codso("POST", "https://example.com/items", {
    json: {magac: "Ayaan"}
})
qor(created.json)
```

A user-defined module namespace remains part of the later package work. This
function is the builtin surface for the HTTP client.
