# DNS and TCP

`dns_raadi` looks up a host name. `tcp_fur`, `tcp_dhegeysi`, `tcp_aqbal`,
`tcp_qor`, `tcp_akhris`, and `tcp_xidh` open TCP connections. Failures are
catchable and use error code `E055`. UDP and WebSocket remain later work.

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
