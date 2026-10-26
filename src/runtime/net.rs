//! DNS lookup, TCP sockets, and UDP datagrams.

use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream, ToSocketAddrs, UdpSocket};
use std::sync::atomic::{AtomicI64, Ordering};
use std::time::{Duration, Instant};

use super::context::{RuntimeLocal, RuntimeState};
use super::value::Value;
use crate::error::{codes, runtime_error_ex, ErrorMeta, SoplangError};

const STATE: RuntimeLocal<RuntimeState> = RuntimeLocal::new(|state| state);

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

static NEXT_ID: AtomicI64 = AtomicI64::new(1);

pub(super) fn error(message: impl Into<String>) -> SoplangError {
    runtime_error_ex(
        message,
        0,
        0,
        ErrorMeta::default().with_code(codes::E055_NETWORK),
    )
}

pub(super) fn error_with_details(
    message: impl Into<String>,
    details: impl Into<String>,
) -> SoplangError {
    runtime_error_ex(
        message,
        0,
        0,
        ErrorMeta::default()
            .with_code(codes::E055_NETWORK)
            .with_details(details),
    )
}

pub(super) fn new_id() -> i64 {
    NEXT_ID.fetch_add(1, Ordering::Relaxed)
}

pub fn lookup(name: &str) -> Result<Value, SoplangError> {
    if name.is_empty() || name.contains([' ', '\0']) {
        return Err(error("Magaca DNS ma saxna."));
    }
    let addresses = (name, 0)
        .to_socket_addrs()
        .map_err(|_| error(format!("Magaca '{name}' lama helin.")))?;
    let mut ips: Vec<String> = addresses.map(|address| address.ip().to_string()).collect();
    ips.sort();
    ips.dedup();
    if ips.is_empty() {
        return Err(error(format!("Magaca '{name}' lama helin.")));
    }
    Ok(Value::List(std::rc::Rc::new(std::cell::RefCell::new(
        ips.into_iter().map(Value::Str).collect(),
    ))))
}

pub fn connect(host: &str, port: i64, timeout: Duration) -> Result<Value, SoplangError> {
    let port = check_port(port)?;
    let mut last = None;
    for address in resolve(host, port)? {
        match TcpStream::connect_timeout(&address, timeout) {
            Ok(stream) => {
                let _ = stream.set_read_timeout(Some(timeout));
                let _ = stream.set_write_timeout(Some(timeout));
                return Ok(store_connection(stream));
            }
            Err(failure) => last = Some(failure),
        }
    }
    Err(error(match last {
        Some(failure) => format!(
            "Xidhiidhka '{host}' lama furin. {}",
            crate::error::io_reason(&failure)
        ),
        None => format!("Magaca '{host}' lama helin."),
    }))
}

pub fn listen(host: &str, port: i64) -> Result<Value, SoplangError> {
    STATE.with(|state| {
        if host.is_empty() {
            return Err(error("Martigeliyaha TCP ma madhnaan karo."));
        }
        let port = check_port(port)?;
        let listener = TcpListener::bind(format!("{host}:{port}")).map_err(|failure| {
            error(format!(
                "Dhegeysiga TCP lama furin. {}",
                crate::error::io_reason(&failure)
            ))
        })?;
        let bound = listener.local_addr().map_err(|failure| {
            error(format!(
                "Dekedda TCP lama aqoonin. {}",
                crate::error::io_reason(&failure)
            ))
        })?;
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        state
            .tcp_listeners
            .borrow_mut()
            .get_or_insert_with(HashMap::new)
            .insert(id, listener);
        Ok(endpoint(id, "dhegeysi", host, bound.port()))
    })
}

pub fn accept(listener: &Value, timeout: Duration) -> Result<Value, SoplangError> {
    STATE.with(|state| {
        let _ = std::io::stdout().flush();
        let id = endpoint_id(listener, "dhegeysi")?;
        let listener = {
            let listeners = state.tcp_listeners.borrow_mut();
            listeners
                .as_ref()
                .and_then(|listeners| listeners.get(&id))
                .ok_or_else(|| error("Dhegeysiga TCP waa la xidhay."))?
                .try_clone()
                .map_err(|failure| {
                    error(format!(
                        "Dhegeysiga TCP lama akhriyi karo. {}",
                        crate::error::io_reason(&failure)
                    ))
                })?
        };
        listener.set_nonblocking(true).map_err(|failure| {
            error(format!(
                "Dhegeysiga TCP lama diyaarin. {}",
                crate::error::io_reason(&failure)
            ))
        })?;
        let started = Instant::now();
        let stream = loop {
            match listener.accept() {
                Ok((stream, _)) => break stream,
                Err(failure) if failure.kind() == std::io::ErrorKind::WouldBlock => {
                    if started.elapsed() >= timeout {
                        return Err(error("Waqtigii dhegeysiga TCP wuu dhammaaday."));
                    }
                    std::thread::sleep(Duration::from_millis(5));
                }
                Err(failure) => {
                    return Err(error(format!(
                        "Xidhiidhka TCP lama aqbalin. {}",
                        crate::error::io_reason(&failure)
                    )));
                }
            }
        };
        let _ = stream.set_read_timeout(Some(timeout));
        let _ = stream.set_write_timeout(Some(timeout));
        let _ = stream.set_nonblocking(false);
        Ok(store_connection(stream))
    })
}

pub fn write(connection: &Value, bytes: &[u8]) -> Result<Value, SoplangError> {
    STATE.with(|state| {
        let id = endpoint_id(connection, "xidhiidh")?;
        let mut connections = state.tcp_connections.borrow_mut();
        let stream = connections
            .as_mut()
            .and_then(|connections| connections.get_mut(&id))
            .ok_or_else(|| error("Xidhiidhka TCP waa la xidhay."))?;
        stream.write_all(bytes).map_err(|failure| {
            error(format!(
                "Qorista TCP way fashilantay. {}",
                crate::error::io_reason(&failure)
            ))
        })?;
        Ok(Value::Null)
    })
}

pub fn read(connection: &Value, limit: usize) -> Result<Value, SoplangError> {
    STATE.with(|state| {
        if limit == 0 || limit > 16 * 1024 * 1024 {
            return Err(error(
                "Cabbirka akhriska TCP waa inuu u dhaxeeyaa 1 iyo 16777216.",
            ));
        }
        let id = endpoint_id(connection, "xidhiidh")?;
        let mut connections = state.tcp_connections.borrow_mut();
        let stream = connections
            .as_mut()
            .and_then(|connections| connections.get_mut(&id))
            .ok_or_else(|| error("Xidhiidhka TCP waa la xidhay."))?;
        let mut buffer = vec![0u8; limit];
        let read = stream.read(&mut buffer).map_err(|failure| {
            error(format!(
                "Akhriska TCP wuu fashilmay. {}",
                crate::error::io_reason(&failure)
            ))
        })?;
        buffer.truncate(read);
        Ok(super::bytes::from_raw_vec(buffer))
    })
}

pub fn close(endpoint: &Value) -> Result<Value, SoplangError> {
    STATE.with(|state| {
        let Value::Object(object) = endpoint else {
            return Err(error("Xidhiidhka TCP waa inuu noqdaa walax."));
        };
        let object = object
            .try_borrow()
            .map_err(|_| error("Walaxda hadda lama akhriyi karo."))?;
        let id = match object.get("aqoonsi") {
            Some(Value::Int(id)) => *id,
            _ => return Err(error("Aqoonsiga TCP ma saxna.")),
        };
        match object.get("nooc").and_then(|value| match value {
            Value::Str(kind) => Some(kind.as_str()),
            _ => None,
        }) {
            Some("xidhiidh") => {
                state
                    .tcp_connections
                    .borrow_mut()
                    .get_or_insert_with(HashMap::new)
                    .remove(&id);
            }
            Some("dhegeysi") => {
                state
                    .tcp_listeners
                    .borrow_mut()
                    .get_or_insert_with(HashMap::new)
                    .remove(&id);
            }
            Some("udp") => {
                state
                    .udp_sockets
                    .borrow_mut()
                    .get_or_insert_with(HashMap::new)
                    .remove(&id);
            }
            Some("xidhiidh_async") => {
                super::async_net::close_connection(state, id);
            }
            Some("dhegeysi_async") => {
                super::async_net::close_listener(state, id);
            }
            Some("udp_async") => {
                super::async_net::close_udp(state, id);
            }
            _ => return Err(error("Nooca soketka ma saxna.")),
        }
        Ok(Value::Null)
    })
}

pub fn timeout(value: Option<&Value>) -> Result<Duration, SoplangError> {
    let Some(value) = value else {
        return Ok(DEFAULT_TIMEOUT);
    };
    let Value::Object(object) = value else {
        return Err(error("Xulashada TCP waa inay noqotaa walax."));
    };
    let object = object
        .try_borrow()
        .map_err(|_| error("Walaxda hadda lama akhriyi karo."))?;
    match object.get("waqti") {
        None => Ok(DEFAULT_TIMEOUT),
        Some(Value::Int(seconds)) if *seconds > 0 => Ok(Duration::from_secs(*seconds as u64)),
        Some(Value::Float(seconds)) if seconds.is_finite() && *seconds > 0.0 => {
            Duration::try_from_secs_f64(*seconds).map_err(|_| error("Waqtiga TCP ma saxna."))
        }
        _ => Err(error("Waqtiga TCP waa inuu noqdaa tiro ka weyn eber.")),
    }
}

fn resolve(host: &str, port: u16) -> Result<Vec<SocketAddr>, SoplangError> {
    if host.is_empty() {
        return Err(error("Martigeliyaha TCP ma madhnaan karo."));
    }
    let addresses = (host, port)
        .to_socket_addrs()
        .map_err(|_| error(format!("Magaca '{host}' lama helin.")))?
        .collect::<Vec<_>>();
    if addresses.is_empty() {
        return Err(error(format!("Magaca '{host}' lama helin.")));
    }
    Ok(addresses)
}

fn check_port(port: i64) -> Result<u16, SoplangError> {
    u16::try_from(port).map_err(|_| error("Dekedda TCP ma saxna."))
}

fn store_connection(stream: TcpStream) -> Value {
    STATE.with(|state| {
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        state
            .tcp_connections
            .borrow_mut()
            .get_or_insert_with(HashMap::new)
            .insert(id, stream);
        endpoint(id, "xidhiidh", "", 0)
    })
}

pub fn bind_udp(host: &str, port: i64, timeout: Duration) -> Result<Value, SoplangError> {
    STATE.with(|state| {
        if host.is_empty() {
            return Err(error("Martigeliyaha UDP ma madhnaan karo."));
        }
        let port = check_port(port)?;
        let socket = UdpSocket::bind(format!("{host}:{port}")).map_err(|failure| {
            error(format!(
                "Soketka UDP lama furin. {}",
                crate::error::io_reason(&failure)
            ))
        })?;
        let bound = socket.local_addr().map_err(|failure| {
            error(format!(
                "Dekedda UDP lama aqoonin. {}",
                crate::error::io_reason(&failure)
            ))
        })?;
        let _ = socket.set_read_timeout(Some(timeout));
        let _ = socket.set_write_timeout(Some(timeout));
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        state
            .udp_sockets
            .borrow_mut()
            .get_or_insert_with(HashMap::new)
            .insert(id, socket);
        Ok(endpoint(id, "udp", host, bound.port()))
    })
}

pub fn send_udp(
    socket: &Value,
    host: &str,
    port: i64,
    bytes: &[u8],
) -> Result<Value, SoplangError> {
    STATE.with(|state| {
        let id = endpoint_id(socket, "udp")?;
        let port = check_port(port)?;
        let address = resolve(host, port)?.into_iter().next().unwrap();
        let sockets = state.udp_sockets.borrow_mut();
        let socket = sockets
            .as_ref()
            .and_then(|sockets| sockets.get(&id))
            .ok_or_else(|| error("Soketka UDP waa la xidhay."))?;
        socket.send_to(bytes, address).map_err(|failure| {
            error(format!(
                "Dirista UDP way fashilantay. {}",
                crate::error::io_reason(&failure)
            ))
        })?;
        Ok(Value::Null)
    })
}

pub fn receive_udp(socket: &Value, limit: usize) -> Result<Value, SoplangError> {
    STATE.with(|state| {
        if limit == 0 || limit > 16 * 1024 * 1024 {
            return Err(error(
                "Cabbirka akhriska UDP waa inuu u dhaxeeyaa 1 iyo 16777216.",
            ));
        }
        let id = endpoint_id(socket, "udp")?;
        let sockets = state.udp_sockets.borrow_mut();
        let socket = sockets
            .as_ref()
            .and_then(|sockets| sockets.get(&id))
            .ok_or_else(|| error("Soketka UDP waa la xidhay."))?;
        let mut buffer = vec![0u8; limit];
        let (read, source) = socket.recv_from(&mut buffer).map_err(|failure| {
            error(format!(
                "Akhriska UDP wuu fashilmay. {}",
                crate::error::io_reason(&failure)
            ))
        })?;
        buffer.truncate(read);
        let mut fields = HashMap::new();
        fields.insert("baytyo".to_string(), super::bytes::from_raw_vec(buffer));
        fields.insert(
            "martigeliye".to_string(),
            Value::Str(source.ip().to_string()),
        );
        fields.insert("deked".to_string(), Value::Int(i64::from(source.port())));
        Ok(Value::Object(std::rc::Rc::new(std::cell::RefCell::new(
            fields,
        ))))
    })
}

pub(super) fn endpoint(id: i64, kind: &str, host: &str, port: u16) -> Value {
    let mut fields = HashMap::new();
    fields.insert("aqoonsi".to_string(), Value::Int(id));
    fields.insert("nooc".to_string(), Value::Str(kind.to_string()));
    if matches!(kind, "dhegeysi" | "udp" | "dhegeysi_async" | "udp_async") {
        fields.insert("martigeliye".to_string(), Value::Str(host.to_string()));
        fields.insert("deked".to_string(), Value::Int(i64::from(port)));
    }
    Value::Object(std::rc::Rc::new(std::cell::RefCell::new(fields)))
}

pub(super) fn endpoint_id(value: &Value, expected: &str) -> Result<i64, SoplangError> {
    let Value::Object(object) = value else {
        return Err(error("Xidhiidhka TCP waa inuu noqdaa walax."));
    };
    let object = object
        .try_borrow()
        .map_err(|_| error("Walaxda hadda lama akhriyi karo."))?;
    match object.get("nooc") {
        Some(Value::Str(kind)) if kind == expected => {}
        _ => return Err(error("Nooca soketka ma saxna.")),
    }
    match object.get("aqoonsi") {
        Some(Value::Int(id)) => Ok(*id),
        _ => Err(error("Aqoonsiga TCP ma saxna.")),
    }
}
