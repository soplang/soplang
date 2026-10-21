//! WebSocket client.

use std::collections::HashMap;
use std::net::TcpStream;
use std::sync::atomic::{AtomicI64, Ordering};
use std::time::Duration;

use tungstenite::stream::MaybeTlsStream;
use tungstenite::{Message, WebSocket};

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
        let (mut socket, _) = tungstenite::connect(url)
            .map_err(|failure| error_with_details(explain(&failure), failure))?;
        if let MaybeTlsStream::Plain(stream) = socket.get_mut() {
            let _ = stream.set_read_timeout(Some(timeout));
            let _ = stream.set_write_timeout(Some(timeout));
        }
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
    let object = object.try_borrow().map_err(|_| error("Walaxda hadda lama akhriyi karo."))?;
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
