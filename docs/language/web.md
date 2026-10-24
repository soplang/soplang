# Web framework

`keen "web" sida web` loads the bundled web framework. For sequential serving,
it builds on `server_fur`, `server_aqbal`, and `server_xidh`. An asynchronous TCP
listener can dispatch several requests at once through `app.aqbal_shaqo`.
Chunked uploads and WebSocket routes remain 3.6 work.

```soplang
keen "web" sida web
door app = cusub web.Web()
app.hel("GET", "/ping", hawl(codsi) { celi web.json({ n: 1 }) })
door server = sug(tcp_dhegeysi_shaqo("127.0.0.1", 8080))
door first = app.aqbal_shaqo(server)
door second = app.aqbal_shaqo(server)
sug(first)
sug(second)
tcp_xidh(server)
```

Each `aqbal_shaqo` call handles one connection. Multiple task handles allow
handlers to overlap while Soplang callbacks still run on one owning thread.
Each connection has the same absolute exchange deadline and request size limit
as `app.aqbal`. This path currently uses plain TCP.

For a long-running plain TCP server, `app.adeeg_shaqo(server, shaqaale?)`
keeps a bounded set of workers accepting requests. The default is 8 workers;
the supported range is 1 to 32. Closing the listener stops new accepts, then
`sug(serving)` waits for active requests to finish:

```soplang
door server = sug(tcp_dhegeysi_shaqo("127.0.0.1", 8080))
door serving = app.adeeg_shaqo(server, 8)
sug(hurdada(60000))
tcp_xidh(server)
sug(serving)
```

Connection timeouts or client disconnects affect only those connections.
Pending connections beyond the worker count wait in the operating-system
queue. This serving path currently uses plain TCP; TLS serving remains a
separate sequential operation.

```soplang
keen "web" sida web
door app = cusub web.Web()
app.dhexdhexa(web.cors("*"))
app.hel("GET", "/ping", hawl(codsi) { celi web.json({ n: 1 }) })
app.koox("/api", hawl(a) {
    a.hel("POST", "/foom", hawl(codsi) { celi web.json(web.foom(codsi)) })
})
door server = server_fur("127.0.0.1", 0)
app.aqbal(server)
```

## Routes and middleware

`hel(method, path, handler)` records one route. `koox(prefix, builder)` prepends
the prefix while the builder adds routes. `dhexdhexa(middleware)` applies that
middleware to routes added afterwards. Middleware runs in registration order.
Each one is `hawl(codsi, xiga)`. It calls `xiga(codsi)` to continue, or returns
a response to stop. An uncaught error becomes status 500 and the listener stays
open. `web.diid(status, message)` raises a structured `HTTP400` to `HTTP599`
error that becomes the requested status and a JSON `khalad` message.
`web.hubi` uses status 400 for missing fields. `app.khalad_qabta(callback)`
installs an error handler receiving `(request, structured_error)` and returning
a response. A failing error handler falls back to the normal error response.
Route groups restore both the prefix and middleware after their builder returns
or raises an error.

`app.waqti` and `app.cabbir` are the connection exchange deadline in seconds
and the request-body limit. The defaults are 30 and 1048576. All reads and
writes, including TLS handshakes, share that absolute deadline. Incoming bytes
do not restart it. Waiting for the next connection has no deadline; synchronous
handler code cannot be forcibly interrupted.

## Requests and responses

The request object is the [HTTP server](http-server.md) request. `web.kuki(codsi, name)`
reads one raw cookie value without decoding percent escapes or discarding `=` characters.
`web.foom(codsi)` parses a URL-encoded body into a `walax`: `+` becomes a space,
percent escapes are decoded, the first `=` separates key from value, and the
last occurrence wins for repeated keys. Missing `=` means an empty value.
`web.hubi(xog, ["magac"])` rejects a missing or empty field.

| Helper | Response |
| --- | --- |
| `web.json(value, status?)` | JSON body. Status defaults to 200. |
| `web.html(text, status?)` | HTML text. |
| `web.socod(next, status?, content_type?)` | Chunked response from a zero-argument producer. The producer returns text or bytes, then `maran`. |
| `web.leex(path, status?)` | Redirect. Status defaults to 302. |
| `web.fayl(root, path)` | Bytes from a file inside `root`, 404 when it is missing, or 400 when the path escapes. |
| `web.habee(template, data)` | HTML from `{{name}}` slots. `{{!name}}` is not escaped. |
| `web.qayb(codsi)` | Multipart parts as `{magac, fayl, qoraal, baytyo}`. The body stays inside `cabbir`. |

`web.socod` pulls one chunk only after the previous chunk has been written.
Chunks must be 1 to 65536 bytes. It uses HTTP/1.1 chunked transfer encoding,
and defaults to `application/octet-stream`. See [HTTP server](http-server.md)
for framing, errors, and the in-memory test client limit. Buffered routes
still load their full request bodies. For a streaming upload,
register the handler with `app.hel_socod(method, path, handler)` and call
`web.jidh_akhris(codsi, size?)` inside it. The size defaults to 65536 and
cannot exceed 65536. The handler runs after headers arrive, and each read
returns up to that many bytes. An empty byte array marks the declared end.
`codsi.qoraal`, `codsi.baytyo`, and `codsi.json` are `maran` on these routes.
Streaming uploads require an asynchronous TCP listener and a total body no
larger than the configured `app.cabbir` (up to 1 GiB). Send `Content-Length`
for a nonempty body; omitting it means an empty body. The
body handle closes when the handler returns. Chunked upload framing remains
later 3.6 work.

`app.shahaado(cert, key)` stores PEM paths. `app.fur(host, port)` then calls
`server_fur` with those files. Without them, `fur` opens a plain listener.

Multipart parsing accepts quoted boundary/name/filename parameters and escaped
quotes. Only complete delimiter lines split an upload; similar byte sequences
inside a file are preserved. A closing boundary is required. The parser caps
bodies at 16 MiB, parts at 128, and each part's headers at 8 KiB/32 fields,
in addition to `app.cabbir`. Duplicate header parameters and headers are errors.
`filename*` and `Content-Transfer-Encoding` are explicitly unsupported; use a
UTF-8 `filename`. File names are data, never paths written automatically.

`fayl_gudaha(root, path)` is the containment check. It follows a final symlink
only when the destination is still inside the root.

## Sessions and authentication

`fur_kalfadhi(codsi)` returns `{id, xog, cusub_ah}`. IDs contain 32 bytes from
the operating system's cryptographic random generator, encoded as unpadded
URL-safe Base64. They do not depend on `xul` / `abuur` or the wall clock.
A `kalfadhi` cookie reuses the same `xog` until `dherer_kalfadhi` seconds after
creation (3600 by default). Expired entries are removed on session operations;
a clock moving backwards invalidates affected sessions. `xad_kalfadhi` bounds
the in-memory store (1024 by default, configurable from 1 to 65536). Creating a
session at capacity evicts the oldest entry. Sessions belong to that `Web`
instance and are not persistent or shared between processes.

`beddel_kalfadhi(codsi)` rotates the ID, preserves data, and invalidates the old
ID; use it after login or privilege changes. `xidh_kalfadhi(codsi)` invalidates
the supplied cookie's session. `nadiifi_kalfadhi()` explicitly sweeps expiration.

`web.kuki_qoraal(id, options?)` creates a `Set-Cookie` value with `Path=/`,
`SameSite=Lax`, and `HttpOnly` by default. Options are `jid` (path), `sugan`
(Secure), `http_keliya` (HttpOnly), `isla_goob` (`Lax`, `Strict`, `None`), and
`cimri` (nonnegative Max-Age seconds). `None` requires `sugan: run`.
Header injection and invalid options are rejected. Use `sugan: run` over HTTPS.
For logout, send `web.kuki_qoraal("", {cimri: 0})` as well as invalidating the
server-side session.

`ilaali(check)` adds middleware. `check(codsi)` returns `maran` to stop with
status 401. Any other value is stored on `codsi.isticmaale`.

`diiwaan_gelin(codsi, response)` appends `"METHOD path"` to `diiwaan` and returns
the response. It retains at most `xad_diiwaan` entries (1024 by default, 1 to 65536). Status 404 is produced before middleware, so it is not recorded.

## Test client

`app.tijaabi(method, path, options?)` calls the same request parser, routes,
middleware, validation and response encoder as a network request, without
opening a socket. It works in JIT and LLVM-built applications. The returned
object has `heer`, `cinwaan`, `qoraal`, `baytyo`, and `json`, matching
`http_codso`. Options accept `cinwaan` and one of `json`, `qoraal`, `baytyo`;
`cabbir` bounds the response (1 MiB by default). `app.cabbir` still bounds the
request. Network timeout and redirect options do not schedule network work in
this client. Cookies can be supplied explicitly via `cinwaan.cookie`.

```soplang
door response = app.tijaabi("GET", "/ping")
hubi(response.heer == 200)
hubi(response.json.n == 1)
```

Malformed JSON returns 400, method mismatches 405, missing routes 404, oversized
bodies 413, and unhandled failures 500. Header injection and invalid response
objects pass through the same validation as real responses.

```sh
cargo test --locked --test web_tests --test http_server_tests
```
