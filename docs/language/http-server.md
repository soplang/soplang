# HTTP server

`server_fur`, `server_aqbal`, and `server_xidh` serve HTTP requests one at a
time. The server speaks HTTP and answers JSON APIs. It does not use TLS.
Bind, route, and shutdown failures are catchable and use error code `E054`.

`server_fur(host, port)` binds a listener. Port `0` chooses an open port. The
result contains `aqoonsi`, `martigeliye`, and `deked`.

`server_aqbal(server, routes, options?)` waits for one connection, calls the
matching handler, and returns `maran` after the response is sent. A handler
failure becomes status 500 and does not stop the listener. An unknown path
returns status 404. A known path with another method returns status 405.

`server_xidh(server)` closes the listener. Another accept then fails.

## Routes

Each route is a list of method, path, and handler:

```soplang
hawl user(codsi) {
    celi {json: {id: codsi.qayb.id}}
}
door server = server_fur("127.0.0.1", 8080)
server_aqbal(server, [
    ["GET", "/users/:id", user]
])
```

Methods are `GET`, `POST`, `PUT`, `PATCH`, and `DELETE`. A path segment that
starts with `:` captures that part of the path.

## Request

The handler receives one `walax`:

| Field | Meaning |
| --- | --- |
| `hab` | Method. |
| `jid` | Path without the query string. |
| `qayb` | Named path parameters. |
| `weydiimo` | Query parameters. `+` is read as a space. |
| `cinwaan` | Headers, with lowercase names. |
| `qoraal` | Body text, or `maran` when it is not UTF-8. |
| `baytyo` | Raw body. |
| `json` | Parsed JSON when the content type contains `json`. Invalid JSON returns status 400 without calling the handler. |

The body must use `Content-Length`. Chunked bodies are rejected. The default
limit is 1048576 bytes and the default read and write timeout is 30 seconds.
Pass `{cabbir: n, waqti: seconds}` as the third argument of `server_aqbal` to
change them.

## Response

The handler returns a `walax`. `heer` defaults to 200. Use one of `qoraal`,
`baytyo`, or `json` for the body. `cinwaan` supplies extra headers. A redirect
is a 301 or 302 response with a `Location` header.

The listener serves one request at a time. Other connections wait in the
operating-system queue until the next `server_aqbal`.

Open listeners and connections belong to the current runtime context. Another
session cannot use or close them. Destroying the context closes its remaining
TCP, UDP, HTTP, and WebSocket resources. Close them explicitly when finished
to release resources during a long-running session.
