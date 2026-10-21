# Web framework

`keen "web" sida web` loads the bundled sequential web framework. It builds on
`server_fur`, `server_aqbal`, and `server_xidh`. The listener still handles one
request at a time. TLS, multipart uploads, and a template engine are later work.

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
open.

`app.waqti` and `app.cabbir` are the accept timeout in seconds and the body
limit. The defaults are 30 and 1048576.

## Requests and responses

The request object is the [HTTP server](http-server.md) request. `web.kuki(codsi, name)`
reads one cookie. `web.foom(codsi)` parses a URL-encoded body into a `walax`.
`web.hubi(xog, ["magac"])` rejects a missing or empty field.

| Helper | Response |
| --- | --- |
| `web.json(value, status?)` | JSON body. Status defaults to 200. |
| `web.html(text, status?)` | HTML text. |
| `web.leex(path, status?)` | Redirect. Status defaults to 302. |
| `web.fayl(root, path)` | Bytes from a file inside `root`, 404 when it is missing, or 400 when the path escapes. |

`fayl_gudaha(root, path)` is the containment check. It follows a final symlink
only when the destination is still inside the root.

## Sessions and authentication

`fur_kalfadhi(codsi)` returns `{id, xog, cusub_ah}`. A `kalfadhi` cookie reuses the
same `xog` until `dherer_kalfadhi` seconds have passed. The default is 3600.
`web.kuki_qoraal(id)` is the `Set-Cookie` value.

`ilaali(check)` adds middleware. `check(codsi)` returns `maran` to stop with
status 401. Any other value is stored on `codsi.isticmaale`.

`diiwaan_gelin(codsi, response)` appends `"METHOD path"` to `diiwaan` and returns
the response. Status 404 is produced before middleware, so it is not recorded.

```sh
cargo test --locked --test web_tests --test http_server_tests
```
