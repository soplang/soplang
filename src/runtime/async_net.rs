//! Context-owned TCP endpoints with cancellable operations on Tokio.

use std::cell::RefCell;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};

use super::context::{RuntimeLocal, RuntimeState};
use super::net;
use super::stdlib::{expect_arity, expect_arity_range, network_bytes};
use super::tasks::{self, ExternalKind, OwnedFailure, OwnedResult};
use super::value::Value;
use crate::error::SoplangError;

const MAX_ENDPOINTS: usize = 64;
const MAX_TRANSFER: usize = 16 * 1024 * 1024;
const STATE: RuntimeLocal<RuntimeState> = RuntimeLocal::new(|state| state);

pub(super) struct Connection {
    reader: tokio::sync::Mutex<OwnedReadHalf>,
    writer: tokio::sync::Mutex<OwnedWriteHalf>,
    timeout: Duration,
}

pub(super) struct UdpEndpoint {
    socket: tokio::net::UdpSocket,
    timeout: Duration,
}

fn failure(action: &str, error: std::io::Error) -> OwnedFailure {
    OwnedFailure::Network(
        format!("{action} {}", crate::error::io_reason(&error)),
        error.to_string(),
    )
}

fn timeout_failure(action: &str) -> OwnedFailure {
    OwnedFailure::Network(
        format!("Waqtigii {action} wuu dhammaaday."),
        "TCP timeout".into(),
    )
}

fn port(value: &Value, name: &str) -> Result<u16, SoplangError> {
    let Value::Int(port) = value else {
        return Err(net::error(format!("Dekedda {name} waa inay noqotaa abn.")));
    };
    u16::try_from(*port).map_err(|_| net::error("Dekedda TCP ma saxna."))
}

fn host(value: &Value, name: &str) -> Result<String, SoplangError> {
    let Value::Str(host) = value else {
        return Err(net::error(format!(
            "Martigeliyaha {name} waa inuu noqdaa qoraal."
        )));
    };
    if host.is_empty() {
        return Err(net::error("Martigeliyaha TCP ma madhnaan karo."));
    }
    Ok(host.clone())
}

pub(super) fn store_connection(
    state: &RuntimeState,
    stream: tokio::net::TcpStream,
    timeout: Duration,
) -> Result<Value, SoplangError> {
    let mut connections = state.async_tcp_connections.borrow_mut();
    if connections.len() >= MAX_ENDPOINTS {
        return Err(net::error("Xidhiidho TCP badan ayaa furan."));
    }
    let id = net::new_id();
    let (reader, writer) = stream.into_split();
    connections.insert(
        id,
        Arc::new(Connection {
            reader: tokio::sync::Mutex::new(reader),
            writer: tokio::sync::Mutex::new(writer),
            timeout,
        }),
    );
    Ok(net::endpoint(id, "xidhiidh_async", "", 0))
}

pub(super) fn store_listener(
    state: &RuntimeState,
    listener: tokio::net::TcpListener,
    host: String,
) -> Result<Value, SoplangError> {
    let mut listeners = state.async_tcp_listeners.borrow_mut();
    if listeners.len() >= MAX_ENDPOINTS {
        return Err(net::error("Dhegeysiyo TCP badan ayaa furan."));
    }
    let port = listener
        .local_addr()
        .map_err(|error| net::error_with_details("Dekedda TCP lama aqoonin.", error.to_string()))?
        .port();
    let id = net::new_id();
    listeners.insert(id, Arc::new(listener));
    Ok(net::endpoint(id, "dhegeysi_async", &host, port))
}

pub(super) fn close_connection(state: &RuntimeState, id: i64) {
    tasks::cancel_resource(id);
    state.async_tcp_connections.borrow_mut().remove(&id);
}

pub(super) fn close_listener(state: &RuntimeState, id: i64) {
    tasks::cancel_resource(id);
    state.async_tcp_listeners.borrow_mut().remove(&id);
}

pub(super) fn store_udp(
    state: &RuntimeState,
    socket: tokio::net::UdpSocket,
    host: String,
    timeout: Duration,
) -> Result<Value, SoplangError> {
    let mut sockets = state.async_udp_sockets.borrow_mut();
    if sockets.len() >= MAX_ENDPOINTS {
        return Err(net::error("Soketyo UDP badan ayaa furan."));
    }
    let port = socket
        .local_addr()
        .map_err(|error| net::error_with_details("Dekedda UDP lama aqoonin.", error.to_string()))?
        .port();
    let id = net::new_id();
    sockets.insert(id, Arc::new(UdpEndpoint { socket, timeout }));
    Ok(net::endpoint(id, "udp_async", &host, port))
}

pub(super) fn close_udp(state: &RuntimeState, id: i64) {
    tasks::cancel_resource(id);
    state.async_udp_sockets.borrow_mut().remove(&id);
}

pub(super) fn packet_value(bytes: Vec<u8>, source: SocketAddr) -> Value {
    let mut fields = HashMap::new();
    fields.insert("baytyo".to_string(), super::bytes::from_raw_vec(bytes));
    fields.insert(
        "martigeliye".to_string(),
        Value::Str(source.ip().to_string()),
    );
    fields.insert("deked".to_string(), Value::Int(i64::from(source.port())));
    Value::Object(Rc::new(RefCell::new(fields)))
}

pub(super) fn connect(args: Vec<Value>) -> Result<Value, SoplangError> {
    expect_arity_range(&args, "tcp_fur_shaqo", 2, 3)?;
    let host = host(&args[0], "tcp_fur_shaqo()")?;
    let port = port(&args[1], "tcp_fur_shaqo()")?;
    let timeout = net::timeout(args.get(2))?;
    tasks::spawn_external(
        async move {
            let stream = tokio::time::timeout(
                timeout,
                tokio::net::TcpStream::connect((host.as_str(), port)),
            )
            .await
            .map_err(|_| timeout_failure("xidhiidhka TCP"))?
            .map_err(|error| failure("Xidhiidhka TCP lama furin.", error))?;
            Ok(OwnedResult::TcpConnection { stream, timeout })
        },
        ExternalKind::Tcp,
    )
}

pub(super) fn listen(args: Vec<Value>) -> Result<Value, SoplangError> {
    expect_arity(&args, "tcp_dhegeysi_shaqo", 2)?;
    let host = host(&args[0], "tcp_dhegeysi_shaqo()")?;
    let port = port(&args[1], "tcp_dhegeysi_shaqo()")?;
    tasks::spawn_external(
        async move {
            let listener = tokio::net::TcpListener::bind((host.as_str(), port))
                .await
                .map_err(|error| failure("Dhegeysiga TCP lama furin.", error))?;
            Ok(OwnedResult::TcpListener { listener, host })
        },
        ExternalKind::Tcp,
    )
}

pub(super) fn accept(args: Vec<Value>) -> Result<Value, SoplangError> {
    expect_arity_range(&args, "tcp_aqbal_shaqo", 1, 2)?;
    let id = net::endpoint_id(&args[0], "dhegeysi_async")?;
    let timeout = net::timeout(args.get(1))?;
    let listener = STATE.with(|state| {
        state
            .async_tcp_listeners
            .borrow()
            .get(&id)
            .cloned()
            .ok_or_else(|| net::error("Dhegeysiga TCP waa la xidhay."))
    })?;
    tasks::spawn_external_on(
        async move {
            let (stream, _) = tokio::time::timeout(timeout, listener.accept())
                .await
                .map_err(|_| timeout_failure("dhegeysiga TCP"))?
                .map_err(|error| failure("Xidhiidhka TCP lama aqbalin.", error))?;
            Ok(OwnedResult::TcpConnection { stream, timeout })
        },
        ExternalKind::Tcp,
        Some(id),
    )
}

pub(super) fn read(args: Vec<Value>) -> Result<Value, SoplangError> {
    expect_arity(&args, "tcp_akhris_shaqo", 2)?;
    let id = net::endpoint_id(&args[0], "xidhiidh_async")?;
    let Value::Int(limit) = args[1] else {
        return Err(net::error(
            "Cabbirka tcp_akhris_shaqo() waa inuu noqdaa abn.",
        ));
    };
    let limit = usize::try_from(limit)
        .ok()
        .filter(|limit| (1..=MAX_TRANSFER).contains(limit))
        .ok_or_else(|| net::error("Cabbirka akhriska TCP waa inuu u dhaxeeyaa 1 iyo 16777216."))?;
    let connection = STATE.with(|state| {
        state
            .async_tcp_connections
            .borrow()
            .get(&id)
            .cloned()
            .ok_or_else(|| net::error("Xidhiidhka TCP waa la xidhay."))
    })?;
    tasks::spawn_external_on(
        async move {
            let mut buffer = vec![0u8; limit.min(64 * 1024)];
            let read = tokio::time::timeout(connection.timeout, async {
                let mut reader = connection.reader.lock().await;
                reader.read(&mut buffer).await
            })
            .await
            .map_err(|_| timeout_failure("akhriska TCP"))?
            .map_err(|error| failure("Akhriska TCP wuu fashilmay.", error))?;
            buffer.truncate(read);
            Ok(OwnedResult::Bytes(buffer))
        },
        ExternalKind::Tcp,
        Some(id),
    )
}

pub(super) fn write(args: Vec<Value>) -> Result<Value, SoplangError> {
    expect_arity(&args, "tcp_qor_shaqo", 2)?;
    let id = net::endpoint_id(&args[0], "xidhiidh_async")?;
    let bytes = network_bytes(&args[1], "tcp_qor_shaqo")?;
    if bytes.len() > MAX_TRANSFER {
        return Err(net::error("Xogta TCP waxay ka weyn tahay 16 MiB."));
    }
    let connection = STATE.with(|state| {
        state
            .async_tcp_connections
            .borrow()
            .get(&id)
            .cloned()
            .ok_or_else(|| net::error("Xidhiidhka TCP waa la xidhay."))
    })?;
    tasks::spawn_external_on(
        async move {
            tokio::time::timeout(connection.timeout, async {
                let mut writer = connection.writer.lock().await;
                writer.write_all(&bytes).await
            })
            .await
            .map_err(|_| timeout_failure("qorista TCP"))?
            .map_err(|error| failure("Qorista TCP way fashilantay.", error))?;
            Ok(OwnedResult::Unit)
        },
        ExternalKind::Tcp,
        Some(id),
    )
}

fn udp_host(value: &Value, name: &str) -> Result<String, SoplangError> {
    let Value::Str(host) = value else {
        return Err(net::error(format!(
            "Martigeliyaha {name} waa inuu noqdaa qoraal."
        )));
    };
    if host.is_empty() {
        return Err(net::error("Martigeliyaha UDP ma madhnaan karo."));
    }
    Ok(host.clone())
}

fn udp_port(value: &Value, name: &str) -> Result<u16, SoplangError> {
    let Value::Int(port) = value else {
        return Err(net::error(format!("Dekedda {name} waa inay noqotaa abn.")));
    };
    u16::try_from(*port).map_err(|_| net::error("Dekedda UDP ma saxna."))
}

pub(super) fn bind_udp(args: Vec<Value>) -> Result<Value, SoplangError> {
    expect_arity_range(&args, "udp_fur_shaqo", 2, 3)?;
    let host = udp_host(&args[0], "udp_fur_shaqo()")?;
    let port = udp_port(&args[1], "udp_fur_shaqo()")?;
    let timeout = net::timeout(args.get(2))?;
    tasks::spawn_external(
        async move {
            let socket =
                tokio::time::timeout(timeout, tokio::net::UdpSocket::bind((host.as_str(), port)))
                    .await
                    .map_err(|_| timeout_failure("furitaanka UDP"))?
                    .map_err(|error| failure("Soketka UDP lama furin.", error))?;
            Ok(OwnedResult::UdpSocket {
                socket,
                host,
                timeout,
            })
        },
        ExternalKind::Udp,
    )
}

pub(super) fn send_udp(args: Vec<Value>) -> Result<Value, SoplangError> {
    expect_arity(&args, "udp_qor_shaqo", 4)?;
    let id = net::endpoint_id(&args[0], "udp_async")?;
    let host = udp_host(&args[1], "udp_qor_shaqo()")?;
    let port = udp_port(&args[2], "udp_qor_shaqo()")?;
    let bytes = network_bytes(&args[3], "udp_qor_shaqo")?;
    if bytes.len() > 65_507 {
        return Err(net::error("Xogta UDP waxay ka weyn tahay hal xirmo."));
    }
    let endpoint = STATE.with(|state| {
        state
            .async_udp_sockets
            .borrow()
            .get(&id)
            .cloned()
            .ok_or_else(|| net::error("Soketka UDP waa la xidhay."))
    })?;
    tasks::spawn_external_on(
        async move {
            tokio::time::timeout(
                endpoint.timeout,
                endpoint.socket.send_to(&bytes, (host.as_str(), port)),
            )
            .await
            .map_err(|_| timeout_failure("dirista UDP"))?
            .map_err(|error| failure("Dirista UDP way fashilantay.", error))?;
            Ok(OwnedResult::Unit)
        },
        ExternalKind::Udp,
        Some(id),
    )
}

pub(super) fn receive_udp(args: Vec<Value>) -> Result<Value, SoplangError> {
    expect_arity(&args, "udp_akhris_shaqo", 2)?;
    let id = net::endpoint_id(&args[0], "udp_async")?;
    let Value::Int(limit) = args[1] else {
        return Err(net::error(
            "Cabbirka udp_akhris_shaqo() waa inuu noqdaa abn.",
        ));
    };
    let limit = usize::try_from(limit)
        .ok()
        .filter(|limit| (1..=MAX_TRANSFER).contains(limit))
        .ok_or_else(|| net::error("Cabbirka akhriska UDP waa inuu u dhaxeeyaa 1 iyo 16777216."))?;
    let endpoint = STATE.with(|state| {
        state
            .async_udp_sockets
            .borrow()
            .get(&id)
            .cloned()
            .ok_or_else(|| net::error("Soketka UDP waa la xidhay."))
    })?;
    tasks::spawn_external_on(
        async move {
            let mut buffer = vec![0u8; limit.min(65_536)];
            let (read, source) =
                tokio::time::timeout(endpoint.timeout, endpoint.socket.recv_from(&mut buffer))
                    .await
                    .map_err(|_| timeout_failure("akhriska UDP"))?
                    .map_err(|error| failure("Akhriska UDP wuu fashilmay.", error))?;
            buffer.truncate(read);
            Ok(OwnedResult::UdpPacket {
                bytes: buffer,
                source,
            })
        },
        ExternalKind::Udp,
        Some(id),
    )
}
