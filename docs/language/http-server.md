# HTTP server

`server_fur`, `server_aqbal`, and `server_xidh` serve HTTP requests one at a
time. The server speaks HTTP and answers JSON APIs. TLS is optional and
serves one connection at a time. Bind, route, and shutdown failures are
catchable and use error code `E054`.

`server_fur(host, port)` binds a listener. A third argument `{shahaado, fure}`
loads a PEM certificate and private key and serves that listener with TLS.
Port `0` chooses an open port. The result contains `aqoonsi`, `martigeliye`,
and `deked`.

`server_aqbal(server, routes, options?)` waits for one connection, calls the
matching handler, and returns `maran` after the response is sent. A handler
failure becomes status 500 and does not stop the listener. An unknown path
returns status 404. A known path with another method returns status 405.

`server_xidh(server)` closes a listener opened with `server_fur`. Another accept
then fails.

`server_aqbal_shaqo(server, routes, options?)` accepts and serves one request
on a listener from `sug(tcp_dhegeysi_shaqo(host, port))`. It returns a task;
several such tasks can wait for connections and run handlers concurrently on
the owning runtime thread. Use `sug` to collect each task result and
`tcp_xidh(server)` to close the asynchronous listener. Closing it cancels
pending accepts. The options and route behavior match `server_aqbal`. The
exchange timeout begins when a connection is accepted, and covers all reads
and writes. This task-based path currently supports plain TCP.

`server_adeeg_shaqo(server, routes, options?)` keeps accepting and serving
connections until `tcp_xidh(server)` closes the asynchronous listener. It
returns a task that completes after accepted requests finish. The optional
`shaqaale` setting bounds simultaneous exchanges from 1 to 32; the default is
8. Other accepted connections wait in the operating-system queue. A client
disconnect or exchange timeout does not stop the listener. Closing the
listener cancels pending accepts while active exchanges drain. Use `sug` on
the serving task to wait for that drain.

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

Methods are `GET`, `POST`, `PUT`, `PATCH`, `DELETE`, and `OPTIONS`. A path segment that
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

The sequential listener serves one request at a time. Other connections wait
in the operating-system queue until the next `server_aqbal`. Starting multiple
`server_aqbal_shaqo` tasks permits overlapping exchanges on an asynchronous
listener.

Open listeners and connections belong to the current runtime context. Another
session cannot use or close them. Destroying the context closes its remaining
TCP, UDP, HTTP, and WebSocket resources. Close them explicitly when finished
to release resources during a long-running session.

## Message validation

HTTP/1.1 requests need one nonempty `Host` field. Duplicate `Host`,
`Content-Length`, or `Transfer-Encoding` fields are rejected. A request that
combines transfer encoding and content length receives 400. Lengths must be
unsigned decimal digits. The 8192-byte header limit includes the terminating
blank line. This server requires known-length bodies and still rejects chunked
transfer encoding.

Response header names must be HTTP tokens. Values cannot contain carriage
returns, newlines, NUL, or other control characters except tab. The server owns
`Content-Length`, `Transfer-Encoding`, and `Connection`; handlers cannot supply
them. Extra headers are limited to 64 fields and 8192 bytes. An invalid handler
response becomes a 500 response, and the listener remains available.

Final response statuses range from 200 to 599. Status 204 and 304 responses
cannot contain a body and omit `Content-Length`. A supplied text or byte body
must have the matching Soplang type.

The framing rules follow [RFC 9112, sections 3.2 and 6.3](https://www.rfc-editor.org/rfc/rfc9112#section-6.3).
The server remains an HTTP subset. Keep-alive, chunked streaming, and upgrades
need later server work. TLS is available through sequential `server_fur` and
`server_aqbal`; asynchronous TLS serving remains later work.
