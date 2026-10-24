//! WebSocket client and context-owned asynchronous server connections.

use std::cell::RefCell;
use std::collections::HashMap;
use std::net::{TcpStream, ToSocketAddrs};
use std::rc::Rc;
use std::sync::atomic::{AtomicI64, Ordering};
use std::time::Duration;

use tungstenite::protocol::WebSocketConfig;
use tungstenite::stream::MaybeTlsStream;
use tungstenite::{client::IntoClientRequest, HandshakeError, Message, WebSocket};

use super::context::{RuntimeLocal, RuntimeState};
use super::http_server::TaskStream;
use super::value::Value;
use crate::error::{codes, runtime_error_ex, ErrorMeta, SoplangError};

const STATE: RuntimeLocal<RuntimeState> = RuntimeLocal::new(|state| state);

pub(super) type Socket = WebSocket<MaybeTlsStream<TcpStream>>;
pub(super) type ServerSocket = WebSocket<TaskStream>;

const MAX_SERVER_MESSAGE: usize = 1024 * 1024;
const MAX_SERVER_SOCKETS: usize = 64;
const DEFAULT_SERVER_WORKERS: usize = 8;
const MAX_SERVER_WORKERS: usize = 32;

pub(super) struct ServerConnection {
    socket: RefCell<ServerSocket>,
    endpoint_id: i64,
}

static NEXT_ID: AtomicI64 = AtomicI64::new(1);

fn error(message: impl Into<String>) -> SoplangError {
    runtime_error_ex(
        message,
        0,
        0,
        ErrorMeta::default().with_code(codes::E056_WEBSOCKET),
    )
}

fn error_with_details(message: impl Into<String>, details: impl std::fmt::Display) -> SoplangError {
    runtime_error_ex(
        message,
        0,
        0,
        ErrorMeta::default()
            .with_code(codes::E056_WEBSOCKET)
            .with_details(details.to_string()),
    )
}

pub fn connect(url: &str, timeout: Duration) -> Result<Value, SoplangError> {
    STATE.with(|state| {
        if !(url.starts_with("ws://") || url.starts_with("wss://")) {
            return Err(error(
                "Cinwaanka WebSocket waa inuu ka bilaabmaa ws:// ama wss://.",
            ));
        }
        let socket = connect_with_timeout(url, timeout)?;
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        state
            .websockets
            .borrow_mut()
            .get_or_insert_with(HashMap::new)
            .insert(id, socket);
        let mut fields = HashMap::new();
        fields.insert("aqoonsi".to_string(), Value::Int(id));
        fields.insert("nooc".to_string(), Value::Str("websocket".to_string()));
        Ok(Value::Object(std::rc::Rc::new(std::cell::RefCell::new(
            fields,
        ))))
    })
}

fn connect_with_timeout(url: &str, timeout: Duration) -> Result<Socket, SoplangError> {
    // Other TLS clients may enable a second provider through Cargo features.
    let _ = rustls::crypto::ring::default_provider().install_default();
    let mut current = url.to_string();
    for attempt in 0..=3 {
        let request = current
            .as_str()
            .into_client_request()
            .map_err(|failure| error_with_details(explain(&failure), failure))?;
        let uri = request.uri();
        let secure = match uri.scheme_str() {
            Some("ws") => false,
            Some("wss") => true,
            _ => {
                return Err(error(
                    "Cinwaanka WebSocket waa inuu ka bilaabmaa ws:// ama wss://.",
                ))
            }
        };
        let host = uri
            .host()
            .ok_or_else(|| error("Martigeliyaha WebSocket lama helin."))?
            .trim_start_matches('[')
            .trim_end_matches(']');
        let port = uri.port_u16().unwrap_or(if secure { 443 } else { 80 });
        let addresses = (host, port).to_socket_addrs().map_err(|failure| {
            error_with_details("Magaca martigeliyaha WebSocket lama helin.", failure)
        })?;
        let mut connected = None;
        let mut last = None;
        for address in addresses {
            match TcpStream::connect_timeout(&address, timeout) {
                Ok(stream) => {
                    connected = Some(stream);
                    break;
                }
                Err(failure) => last = Some(failure),
            }
        }
        let stream = connected.ok_or_else(|| match last {
            Some(failure) => error_with_details("Xidhiidhka WebSocket lama furin.", failure),
            None => error("Magaca martigeliyaha WebSocket lama helin."),
        })?;
        // Configure the transport before either TLS or the HTTP upgrade. The
        // timeouts stay on the underlying TCP socket for encrypted I/O too.
        stream
            .set_read_timeout(Some(timeout))
            .and_then(|_| stream.set_write_timeout(Some(timeout)))
            .and_then(|_| stream.set_nodelay(true))
            .map_err(|failure| {
                error_with_details("Xidhiidhka WebSocket lama diyaarin.", failure)
            })?;
        match tungstenite::client_tls(request, stream) {
            Ok((socket, _)) => return Ok(socket),
            Err(HandshakeError::Failure(tungstenite::Error::Http(response)))
                if response.status().is_redirection() && attempt < 3 =>
            {
                let next = response
                    .headers()
                    .get("Location")
                    .and_then(|value| value.to_str().ok())
                    .ok_or_else(|| error("Leexinta WebSocket ma laha cinwaan sax ah."))?;
                if secure && !next.starts_with("wss://") {
                    return Err(error(
                        "Leexinta WebSocket ma joojin karto sirgelinta xidhiidhka.",
                    ));
                }
                current = next.to_string();
            }
            Err(HandshakeError::Failure(failure)) => {
                return Err(error_with_details(explain(&failure), failure))
            }
            Err(HandshakeError::Interrupted(_)) => {
                return Err(error("Waqtigii WebSocket wuu dhammaaday."))
            }
        }
    }
    Err(error("Leexinta WebSocket way ka badatay xadka."))
}

pub fn send(socket: &Value, data: &[u8], text: bool) -> Result<Value, SoplangError> {
    if let Some(id) = server_socket_id(socket) {
        return send_server(id, data, text);
    }
    STATE.with(|state| {
        let id = socket_id(socket)?;
        let mut sockets = state.websockets.borrow_mut();
        let socket = sockets
            .as_mut()
            .and_then(|sockets| sockets.get_mut(&id))
            .ok_or_else(|| error("Xidhiidhka WebSocket waa la xidhay."))?;
        let message = if text {
            let text =
                std::str::from_utf8(data).map_err(|_| error("Fariinta qoraalka ma aha UTF-8."))?;
            Message::text(text)
        } else {
            Message::binary(data.to_vec())
        };
        socket
            .send(message)
            .map_err(|failure| error_with_details(explain(&failure), failure))?;
        Ok(Value::Null)
    })
}

pub fn receive(socket: &Value) -> Result<Value, SoplangError> {
    if let Some(id) = server_socket_id(socket) {
        return receive_server(id);
    }
    STATE.with(|state| {
        let id = socket_id(socket)?;
        let mut sockets = state.websockets.borrow_mut();
        let socket = sockets
            .as_mut()
            .and_then(|sockets| sockets.get_mut(&id))
            .ok_or_else(|| error("Xidhiidhka WebSocket waa la xidhay."))?;
        loop {
            let message = socket
                .read()
                .map_err(|failure| error_with_details(explain(&failure), failure))?;
            match message {
                Message::Text(text) => return Ok(Value::Str(text.to_string())),
                Message::Binary(bytes) => return Ok(super::bytes::from_raw_vec(bytes.to_vec())),
                Message::Ping(_) | Message::Pong(_) | Message::Frame(_) => continue,
                Message::Close(_) => return Err(error("Xidhiidhka WebSocket waa la xidhay.")),
            }
        }
    })
}

pub fn close(socket: &Value) -> Result<Value, SoplangError> {
    if let Some(id) = server_socket_id(socket) {
        return close_server(id);
    }
    STATE.with(|state| {
        let id = socket_id(socket)?;
        let mut sockets = state.websockets.borrow_mut();
        if let Some(mut socket) = sockets.get_or_insert_with(HashMap::new).remove(&id) {
            let _ = socket.close(None);
            let _ = socket.read();
        }
        Ok(Value::Null)
    })
}

/// Accept one WebSocket connection on an asynchronous TCP listener and invoke
/// its handler on the owning runtime thread.
pub fn accept_async(
    listener: &Value,
    handler: &Value,
    options: Option<&Value>,
) -> Result<Value, SoplangError> {
    super::net::endpoint_id(listener, "dhegeysi_async")?;
    if !matches!(
        handler,
        Value::Function(_) | Value::Closure(_) | Value::NativeFunction(_)
    ) {
        return Err(error("Qabtaha WebSocket waa inuu noqdaa hawl."));
    }
    super::tasks::spawn(
        Value::NativeFunction(accept_async_task),
        vec![
            listener.clone(),
            handler.clone(),
            options.cloned().unwrap_or(Value::Null),
        ],
    )
}

fn accept_async_task(args: Vec<Value>) -> Result<Value, SoplangError> {
    let timeout = super::net::timeout(args.get(2).filter(|value| !matches!(value, Value::Null)))?;
    let connection = accept_connection(&args[0], timeout)?;
    handle_connection(&connection, &args[1], timeout)
}

fn accept_connection(listener: &Value, timeout: Duration) -> Result<Value, SoplangError> {
    let accept = super::async_net::accept_server(listener, timeout)?;
    let Value::Task(accept) = accept else {
        unreachable!("asynchronous accept returns a task")
    };
    super::tasks::await_task(&accept)
}

fn handle_connection(
    connection: &Value,
    handler: &Value,
    timeout: Duration,
) -> Result<Value, SoplangError> {
    let stream = TaskStream::for_websocket(connection.clone(), timeout);
    let config = WebSocketConfig::default()
        .read_buffer_size(4096)
        .write_buffer_size(0)
        .max_write_buffer_size(MAX_SERVER_MESSAGE + 1024)
        .max_message_size(Some(MAX_SERVER_MESSAGE))
        .max_frame_size(Some(MAX_SERVER_MESSAGE));
    let websocket = match tungstenite::accept_with_config(stream, Some(config)) {
        Ok(websocket) => websocket,
        Err(HandshakeError::Failure(failure)) => {
            let _ = super::net::close(&connection);
            return Err(error_with_details(explain(&failure), failure));
        }
        Err(HandshakeError::Interrupted(_)) => {
            let _ = super::net::close(&connection);
            return Err(error("Waqtigii WebSocket wuu dhammaaday."));
        }
    };
    let handle = match store_server_socket(websocket, &connection) {
        Ok(handle) => handle,
        Err(failure) => {
            let _ = super::net::close(&connection);
            return Err(failure);
        }
    };
    let result = super::abi::invoke(handler, &[handle.clone()]);
    let _ = close(&handle);
    result
}

/// Keep a bounded number of WebSocket handlers alive until the listener closes.
pub fn serve_async(
    listener: &Value,
    handler: &Value,
    options: Option<&Value>,
) -> Result<Value, SoplangError> {
    super::net::endpoint_id(listener, "dhegeysi_async")?;
    if !matches!(
        handler,
        Value::Function(_) | Value::Closure(_) | Value::NativeFunction(_)
    ) {
        return Err(error("Qabtaha WebSocket waa inuu noqdaa hawl."));
    }
    super::tasks::spawn(
        Value::NativeFunction(serve_async_task),
        vec![
            listener.clone(),
            handler.clone(),
            options.cloned().unwrap_or(Value::Null),
        ],
    )
}

fn serve_async_task(args: Vec<Value>) -> Result<Value, SoplangError> {
    let options = args.get(2).filter(|value| !matches!(value, Value::Null));
    let _ = super::net::timeout(options)?;
    let workers = worker_count(options)?;
    let mut pending = Vec::with_capacity(workers);
    for _ in 0..workers {
        let worker = super::tasks::spawn(Value::NativeFunction(serve_worker), args.clone())?;
        let Value::Task(worker) = worker else {
            unreachable!("WebSocket worker returns a task")
        };
        pending.push(worker);
    }
    let mut first_failure = None;
    for worker in pending {
        if let Err(failure) = super::tasks::await_task(&worker) {
            first_failure.get_or_insert(failure);
        }
    }
    match first_failure {
        Some(failure) => Err(failure),
        None => Ok(Value::Null),
    }
}

fn serve_worker(args: Vec<Value>) -> Result<Value, SoplangError> {
    let timeout = super::net::timeout(args.get(2).filter(|value| !matches!(value, Value::Null)))?;
    let listener = &args[0];
    loop {
        if !super::async_net::listener_is_open(listener) {
            return Ok(Value::Null);
        }
        let connection = match accept_connection(listener, timeout) {
            Ok(connection) => connection,
            Err(_) if !super::async_net::listener_is_open(listener) => return Ok(Value::Null),
            Err(failure) => {
                let _ = super::net::close(listener);
                return Err(failure);
            }
        };
        // Invalid upgrades, disconnects, and callback failures stay local to a peer.
        let _ = handle_connection(&connection, &args[1], timeout);
    }
}

fn worker_count(options: Option<&Value>) -> Result<usize, SoplangError> {
    let Some(Value::Object(options)) = options else {
        return Ok(DEFAULT_SERVER_WORKERS);
    };
    let options = options
        .try_borrow()
        .map_err(|_| error("Walaxda hadda lama akhriyi karo."))?;
    match options.get("shaqaale") {
        None => Ok(DEFAULT_SERVER_WORKERS),
        Some(Value::Int(workers)) => usize::try_from(*workers)
            .ok()
            .filter(|workers| (1..=MAX_SERVER_WORKERS).contains(workers))
            .ok_or_else(|| error("Tirada shaqaalaha WebSocket waa inay u dhaxaysaa 1 iyo 32.")),
        _ => Err(error("Tirada shaqaalaha WebSocket waa inay noqotaa abn.")),
    }
}

fn store_server_socket(socket: ServerSocket, connection: &Value) -> Result<Value, SoplangError> {
    STATE.with(|state| {
        let mut sockets = state.server_websockets.borrow_mut();
        if sockets.len() >= MAX_SERVER_SOCKETS {
            return Err(error("Xidhiidhyo WebSocket badan ayaa furan."));
        }
        let endpoint_id = super::net::endpoint_id(connection, "xidhiidh_async")?;
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        sockets.insert(
            id,
            Rc::new(ServerConnection {
                socket: RefCell::new(socket),
                endpoint_id,
            }),
        );
        let mut fields = HashMap::new();
        fields.insert("aqoonsi".to_string(), Value::Int(id));
        fields.insert(
            "nooc".to_string(),
            Value::Str("websocket_server".to_string()),
        );
        Ok(Value::Object(Rc::new(RefCell::new(fields))))
    })
}

fn server_socket_id(value: &Value) -> Option<i64> {
    let Value::Object(object) = value else {
        return None;
    };
    let object = object.try_borrow().ok()?;
    match (object.get("nooc"), object.get("aqoonsi")) {
        (Some(Value::Str(kind)), Some(Value::Int(id))) if kind == "websocket_server" => Some(*id),
        _ => None,
    }
}

fn server_socket(id: i64) -> Result<Rc<ServerConnection>, SoplangError> {
    STATE.with(|state| {
        state
            .server_websockets
            .borrow()
            .get(&id)
            .cloned()
            .ok_or_else(|| error("Xidhiidhka WebSocket waa la xidhay."))
    })
}

fn send_server(id: i64, data: &[u8], text: bool) -> Result<Value, SoplangError> {
    if data.len() > MAX_SERVER_MESSAGE {
        return Err(error("Fariinta WebSocket way ka weyn tahay 1 MiB."));
    }
    let socket = server_socket(id)?;
    let mut socket = socket
        .socket
        .try_borrow_mut()
        .map_err(|_| error("Xidhiidhka WebSocket hadda waa la adeegsanayaa."))?;
    let message = if text {
        Message::text(
            std::str::from_utf8(data).map_err(|_| error("Fariinta qoraalka ma aha UTF-8."))?,
        )
    } else {
        Message::binary(data.to_vec())
    };
    let result = socket
        .send(message)
        .map_err(|failure| error_with_details(explain(&failure), failure));
    if let Some(failure) = socket.get_mut().take_failure() {
        return Err(failure);
    }
    result.map(|_| Value::Null)
}

fn receive_server(id: i64) -> Result<Value, SoplangError> {
    let socket = server_socket(id)?;
    let mut socket = socket
        .socket
        .try_borrow_mut()
        .map_err(|_| error("Xidhiidhka WebSocket hadda waa la adeegsanayaa."))?;
    loop {
        let result = socket.read();
        if let Some(failure) = socket.get_mut().take_failure() {
            return Err(failure);
        }
        match result.map_err(|failure| error_with_details(explain(&failure), failure))? {
            Message::Text(text) => return Ok(Value::Str(text.to_string())),
            Message::Binary(bytes) => return Ok(super::bytes::from_raw_vec(bytes.to_vec())),
            Message::Ping(_) | Message::Pong(_) | Message::Frame(_) => continue,
            Message::Close(_) => return Err(error("Xidhiidhka WebSocket waa la xidhay.")),
        }
    }
}

fn close_server(id: i64) -> Result<Value, SoplangError> {
    STATE.with(|state| {
        let socket = state.server_websockets.borrow_mut().remove(&id);
        if let Some(socket) = socket {
            if let Ok(mut owned) = socket.socket.try_borrow_mut() {
                let _ = owned.close(None);
            }
            super::async_net::close_connection(state, socket.endpoint_id);
        }
        Ok(Value::Null)
    })
}

fn socket_id(value: &Value) -> Result<i64, SoplangError> {
    let Value::Object(object) = value else {
        return Err(error("Xidhiidhka WebSocket waa inuu noqdaa walax."));
    };
    let object = object
        .try_borrow()
        .map_err(|_| error("Walaxda hadda lama akhriyi karo."))?;
    match object.get("nooc") {
        Some(Value::Str(kind)) if kind == "websocket" => {}
        _ => return Err(error("Nooca WebSocket ma saxna.")),
    }
    match object.get("aqoonsi") {
        Some(Value::Int(id)) => Ok(*id),
        _ => Err(error("Aqoonsiga WebSocket ma saxna.")),
    }
}

fn explain(error: &tungstenite::Error) -> &'static str {
    match error {
        tungstenite::Error::ConnectionClosed | tungstenite::Error::AlreadyClosed => {
            "Xidhiidhka WebSocket waa la xidhay."
        }
        tungstenite::Error::Io(failure)
            if failure.kind() == std::io::ErrorKind::TimedOut
                || failure.kind() == std::io::ErrorKind::WouldBlock =>
        {
            "Waqtigii WebSocket wuu dhammaaday."
        }
        tungstenite::Error::Io(_) | tungstenite::Error::Url(_) => {
            "Xidhiidhka WebSocket wuu fashilmay."
        }
        tungstenite::Error::Protocol(_) | tungstenite::Error::Utf8(_) => {
            "Fariinta WebSocket ma saxna."
        }
        _ => "Codsiga WebSocket wuu fashilmay.",
    }
}

#[cfg(test)]
mod timeout_tests {
    use super::*;
    use std::io::Read;
    use std::net::TcpListener;
    use std::time::Instant;

    #[test]
    fn redirects_keep_timeout_handling_and_a_three_hop_limit() {
        let target = TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let target_port = target.local_addr().unwrap().port();
        let echo = std::thread::spawn(move || {
            let (stream, _) = target.accept().unwrap();
            let mut socket = tungstenite::accept(stream).unwrap();
            socket.send(Message::text("redirected")).unwrap();
        });
        let redirect = TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let port = redirect.local_addr().unwrap().port();
        let first = std::thread::spawn(move || {
            let (mut stream, _) = redirect.accept().unwrap();
            stream
                .set_read_timeout(Some(Duration::from_secs(2)))
                .unwrap();
            let mut request = Vec::new();
            let mut byte = [0u8; 1];
            while !request.ends_with(b"\r\n\r\n") {
                std::io::Read::read_exact(&mut stream, &mut byte).unwrap();
                request.push(byte[0]);
            }
            std::io::Write::write_all(&mut stream,format!("HTTP/1.1 302 Found\r\nLocation: ws://127.0.0.1:{target_port}/\r\nContent-Length: 0\r\n\r\n").as_bytes()).unwrap();
        });
        let mut socket =
            connect_with_timeout(&format!("ws://127.0.0.1:{port}/"), Duration::from_secs(2))
                .unwrap();
        assert_eq!(socket.read().unwrap(), Message::text("redirected"));
        first.join().unwrap();
        echo.join().unwrap();

        let redirect = TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let port = redirect.local_addr().unwrap().port();
        let loop_server = std::thread::spawn(move || {
            for _ in 0..4 {
                let (mut stream, _) = redirect.accept().unwrap();
                stream
                    .set_read_timeout(Some(Duration::from_secs(2)))
                    .unwrap();
                let mut request = Vec::new();
                let mut byte = [0u8; 1];
                while !request.ends_with(b"\r\n\r\n") {
                    std::io::Read::read_exact(&mut stream, &mut byte).unwrap();
                    request.push(byte[0]);
                }
                std::io::Write::write_all(&mut stream,format!("HTTP/1.1 302 Found\r\nLocation: ws://127.0.0.1:{port}/\r\nContent-Length: 0\r\n\r\n").as_bytes()).unwrap();
            }
        });
        assert_eq!(
            connect_with_timeout(&format!("ws://127.0.0.1:{port}/"), Duration::from_secs(2))
                .unwrap_err()
                .code(),
            Some("E056")
        );
        loop_server.join().unwrap();
    }

    #[test]
    fn plain_and_tls_handshakes_honor_transport_timeouts() {
        for scheme in ["ws", "wss"] {
            let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
            let port = listener.local_addr().unwrap().port();
            let server = std::thread::spawn(move || {
                let (mut stream, _) = listener.accept().unwrap();
                stream
                    .set_read_timeout(Some(Duration::from_secs(3)))
                    .unwrap();
                let mut data = [0u8; 4096];
                while matches!(stream.read(&mut data), Ok(n) if n > 0) {}
            });
            let started = Instant::now();
            let error = connect_with_timeout(
                &format!("{scheme}://127.0.0.1:{port}/"),
                Duration::from_millis(100),
            )
            .unwrap_err();
            assert_eq!(error.code(), Some("E056"));
            assert!(
                started.elapsed() < Duration::from_secs(2),
                "{scheme}: {error}"
            );
            server.join().unwrap();
        }
    }
}
