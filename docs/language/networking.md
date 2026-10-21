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
may contain `waqti` for the read and write timeout. The default is 30 seconds.

`ws_qor(socket, data)` sends a text frame for a `qoraal` and a binary frame for
a `baytyo`. `ws_akhris(socket)` returns the next data frame as `qoraal` or
`baytyo`. Ping frames are answered automatically. `ws_xidh` closes the socket.

```soplang
door socket = ws_fur("ws://127.0.0.1:9000/chat")
ws_qor(socket, "salaan")
qor(ws_akhris(socket))
ws_xidh(socket)
```
