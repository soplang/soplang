//! WebSocket client.

use std::collections::HashMap;
use std::net::{TcpStream, ToSocketAddrs};
use std::sync::atomic::{AtomicI64, Ordering};
use std::time::Duration;

use tungstenite::stream::MaybeTlsStream;
use tungstenite::{client::IntoClientRequest, HandshakeError, Message, WebSocket};

use super::context::{RuntimeLocal, RuntimeState};
use super::value::Value;
use crate::error::{codes, runtime_error_ex, ErrorMeta, SoplangError};

const STATE: RuntimeLocal<RuntimeState> = RuntimeLocal::new(|state| state);

pub(super) type Socket = WebSocket<MaybeTlsStream<TcpStream>>;

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
