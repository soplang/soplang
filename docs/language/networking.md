# DNS and TCP

`dns_raadi` looks up a host name. TCP sockets, UDP datagrams, and a WebSocket
client exchange data on this machine. Failures are catchable and use error
code `E055` for DNS, TCP, and UDP, and `E056` for WebSocket. A WebSocket server
remains later work.

## DNS

`dns_raadi(name)` returns a list of IP addresses. The list is sorted and
contains each address once. An IP literal does not ask the network. An empty
name is rejected.

```soplang
qor(dns_raadi("localhost"))
```

## TCP

The task forms `tcp_fur_shaqo`, `tcp_dhegeysi_shaqo`, `tcp_aqbal_shaqo`,
`tcp_akhris_shaqo`, and `tcp_qor_shaqo` run TCP I/O without blocking the
owning runtime thread. Pass each returned task to `sug` to get its result.
They create separate asynchronous endpoints; close them with `tcp_xidh`.
Closing an endpoint cancels its pending read, write, and accept tasks. Reads
return up to the requested limit and at most 64 KiB per operation; repeat
them to consume a stream. Writes accept at most 16 MiB. A cancelled write may
have sent a prefix, so application protocols should frame their messages.

```soplang
door connection = sug(tcp_fur_shaqo("127.0.0.1", 9000))
sug(tcp_qor_shaqo(connection, "salaan"))
door answer = sug(tcp_akhris_shaqo(connection, 1024))
tcp_xidh(connection)
```

`tcp_dhegeysi(host, port)` binds a listener. Port `0` chooses an open port.
The result includes `deked` and `nooc` `"dhegeysi"`.

`tcp_aqbal(listener, options?)` waits for one connection. `tcp_fur(host, port, options?)`
connects to one. Both return `nooc` `"xidhiidh"`.

`tcp_qor` writes a `qoraal` or `baytyo`. `tcp_akhris(connection, limit)` reads
the next available bytes, at most `limit`, and returns a `baytyo`. A closed
peer returns an empty `baytyo`. `tcp_xidh` closes a connection or a listener.

`options` may contain `waqti`, a timeout in seconds. The default is 30.
Connect, accept, read, and write use that timeout.

```soplang
door listener = tcp_dhegeysi("127.0.0.1", 0)
door incoming = tcp_aqbal(listener)
door message = tcp_akhris(incoming, 1024)
tcp_qor(incoming, "nabad")
tcp_xidh(incoming)
tcp_xidh(listener)
```

The listener accepts one connection at a time.

## UDP

`udp_fur_shaqo`, `udp_qor_shaqo`, and `udp_akhris_shaqo` are the task forms of
the UDP operations below. `sug` returns the socket, `maran`, or packet object,
respectively. Close asynchronous sockets with `udp_xidh`; closing one cancels
its pending send and receive tasks. A send accepts one datagram of at most
65507 bytes. A receive returns up to the requested limit, capped at 65536
bytes.

`udp_fur(host, port, options?)` binds a datagram socket. Port `0` chooses an
open port. The result has `nooc` `"udp"`, `martigeliye`, and `deked`.

`udp_qor(socket, host, port, data)` sends one datagram. `data` is a `qoraal`
or `baytyo`. `udp_akhris(socket, limit)` waits for one datagram and returns
`baytyo`, `martigeliye`, and `deked` for the sender. `udp_xidh` closes the
socket. `options` may contain `waqti`, with the same default of 30 seconds.

```soplang
door inbox = udp_fur("127.0.0.1", 0)
udp_qor(outbox, "127.0.0.1", inbox.deked, "salaan")
door packet = udp_akhris(inbox, 1024)
qor(packet.baytyo.qoraal())
```

## WebSocket

`ws_fur(url, options?)` connects to a `ws://` or `wss://` server. `wss` checks
the certificate with the operating system's trusted certificates. `options`
may contain `waqti` for the connection attempt and read/write timeout. The
transport is configured before TLS and the HTTP upgrade, so stalled handshakes
and encrypted reads/writes also honor it. The default is 30 seconds. DNS lookup
uses the operating system resolver; `waqti` is not an overall DNS or multi-I/O
deadline. Up to three redirects are followed. A secure `wss` connection cannot
redirect to an unencrypted `ws` connection.

`ws_aqbal_shaqo(listener, handler, options?)` accepts one WebSocket upgrade on
an asynchronous TCP listener. `sug` waits for its handler to finish.
`ws_adeeg_shaqo(listener, handler, options?)` keeps accepting connections with
8 workers by default; `shaqaale` selects 1 to 32 workers. Each handler runs
on the owning runtime thread and receives one WebSocket handle. Closing the
listener with `tcp_xidh` cancels pending accepts; `sug` on the serving task
waits for active handlers to finish. Invalid upgrades and client disconnects
affect only their connection. These server functions currently use plain TCP
listeners and do not share an HTTP listener or route WebSocket upgrades through
the bundled `web` module.

```soplang
door listener = sug(tcp_dhegeysi_shaqo("127.0.0.1", 9000))
hawl echo(socket) {
    door message = ws_akhris(socket)
    ws_qor(socket, message)
}
door serving = ws_adeeg_shaqo(listener, echo, {shaqaale: 8, waqti: 30})
sug(serving)
```

Server messages and frames are limited to 1 MiB; at most 64 server WebSocket
handles may be open in one runtime context. The `waqti` option applies to each
handshake, read, and write operation. The handle closes when its handler
returns, even if the handler raises an error. `ws_xidh` closes it early.

`ws_qor(socket, data)` sends a text frame for a `qoraal` and a binary frame for
a `baytyo`. `ws_akhris(socket)` returns the next data frame as `qoraal` or
`baytyo`. Ping frames are answered automatically. `ws_xidh` closes the socket.

```soplang
door socket = ws_fur("ws://127.0.0.1:9000/chat")
ws_qor(socket, "salaan")
qor(ws_akhris(socket))
ws_xidh(socket)
```

Open listeners and connections belong to the current runtime context. Another
session cannot use or close them. Destroying the context closes its remaining
TCP, UDP, HTTP, and WebSocket resources. Close them explicitly when finished
to release resources during a long-running session.
