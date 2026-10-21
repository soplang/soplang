//! One-request-at-a-time HTTP server for JSON APIs.

use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicI64, Ordering};
use std::time::Duration;

use super::context::{RuntimeLocal, RuntimeState};
use super::value::Value;
use crate::error::{codes, runtime_error_ex, ErrorMeta, SoplangError};

const STATE: RuntimeLocal<RuntimeState> = RuntimeLocal::new(|state| state);

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);
const DEFAULT_LIMIT: usize = 1024 * 1024;
const MAX_LIMIT: usize = 16 * 1024 * 1024;
const MAX_HEADERS: usize = 8 * 1024;

pub(super) struct Listener {
    socket: TcpListener,
}

static NEXT_ID: AtomicI64 = AtomicI64::new(1);

fn error(message: impl Into<String>) -> SoplangError {
    runtime_error_ex(
        message,
        0,
        0,
        ErrorMeta::default().with_code(codes::E054_HTTP_SERVER),
    )
}

struct Route {
    method: String,
    parts: Vec<Part>,
    handler: Value,
}

enum Part {
    Fixed(String),
    Param(String),
}

struct RawRequest {
    method: String,
    path: String,
    query: HashMap<String, String>,
    headers: HashMap<String, String>,
    body: Vec<u8>,
}

pub fn open(host: &str, port: i64) -> Result<Value, SoplangError> {
    STATE.with(|state| {
        if host.is_empty() {
            return Err(error("Martigeliyaha serverka ma madhnaan karo."));
        }
        let port = u16::try_from(port).map_err(|_| error("Dekedda serverka ma saxna."))?;
        let listener = TcpListener::bind(format!("{host}:{port}")).map_err(|failure| {
            error(format!(
                "Serverka lama furi karo. {}",
                crate::error::io_reason(&failure)
            ))
        })?;
        let bound = listener.local_addr().map_err(|failure| {
            error(format!(
                "Dekedda serverka lama aqoonin. {}",
                crate::error::io_reason(&failure)
            ))
        })?;
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        let mut listeners = state.http_listeners.borrow_mut();
        listeners
            .get_or_insert_with(HashMap::new)
            .insert(id, Listener { socket: listener });
        Ok(object(&[
            ("aqoonsi", Value::Int(id)),
            ("martigeliye", Value::Str(host.to_string())),
            ("deked", Value::Int(i64::from(bound.port()))),
        ]))
    })
}

pub fn close(server: &Value) -> Result<Value, SoplangError> {
    STATE.with(|state| {
        let id = server_id(server)?;
        state
            .http_listeners
            .borrow_mut()
            .get_or_insert_with(HashMap::new)
            .remove(&id);
        Ok(Value::Null)
    })
}

pub fn accept(
    server: &Value,
    routes: &Value,
    options: Option<&Value>,
) -> Result<Value, SoplangError> {
    STATE.with(|state| {
        let _ = std::io::stdout().flush();
        let id = server_id(server)?;
        let routes = compile_routes(routes)?;
        let (timeout, limit) = limits(options)?;
        let mut stream = {
            let listeners = state.http_listeners.borrow_mut();
            let listener = listeners
                .as_ref()
                .and_then(|listeners| listeners.get(&id))
                .ok_or_else(|| error("Serverka waa la xidhay."))?;
            listener
                .socket
                .accept()
                .map_err(|failure| {
                    error(format!(
                        "Codsiga lama aqbalin. {}",
                        crate::error::io_reason(&failure)
                    ))
                })?
                .0
        };
        let _ = stream.set_read_timeout(Some(timeout));
        let _ = stream.set_write_timeout(Some(timeout));
        match read_request(&mut stream, limit) {
            Ok(request) => dispatch(&mut stream, &routes, request),
            Err(ResponseFault::Reply(status, message)) => {
                let _ = write_response(
                    &mut stream,
                    status,
                    &[],
                    &error_body(&message),
                    "application/json",
                );
                Ok(Value::Null)
            }
            Err(ResponseFault::Stop(message)) => Err(error(message)),
        }
    })
}

fn dispatch(
    stream: &mut TcpStream,
    routes: &[Route],
    request: RawRequest,
) -> Result<Value, SoplangError> {
    let mut path_match = None;
    let mut method_mismatch = false;
    for route in routes {
        if let Some(params) = match_path(&route.parts, &request.path) {
            if route.method == request.method {
                path_match = Some((route, params));
                break;
            }
            method_mismatch = true;
        }
    }
    let Some((route, params)) = path_match else {
        let status = if method_mismatch { 405 } else { 404 };
        let message = if method_mismatch {
            "Habka lama oggola."
        } else {
            "Lama helin."
        };
        let _ = write_response(
            stream,
            status,
            &[],
            &error_body(message),
            "application/json",
        );
        return Ok(Value::Null);
    };
    let request_value = match request_object(&request, params) {
        Ok(value) => value,
        Err(message) => {
            let _ = write_response(stream, 400, &[], &error_body(&message), "application/json");
            return Ok(Value::Null);
        }
    };
    match super::abi::invoke(&route.handler, &[request_value]) {
        Ok(value) => {
            let response = match response_from(&value) {
                Ok(response) => response,
                Err(message) => (
                    500,
                    Vec::new(),
                    error_body(&message),
                    "application/json".to_string(),
                ),
            };
            let _ = write_response(stream, response.0, &response.1, &response.2, &response.3);
        }
        Err(failure) => {
            let _ = write_response(
                stream,
                500,
                &[],
                &error_body(failure.message()),
                "application/json",
            );
        }
    }
    Ok(Value::Null)
}

fn request_object(request: &RawRequest, params: HashMap<String, String>) -> Result<Value, String> {
    let text = std::str::from_utf8(&request.body).ok().map(str::to_owned);
    let content_type = request
        .headers
        .get("content-type")
        .cloned()
        .unwrap_or_default();
    let json = if content_type.to_ascii_lowercase().contains("json") {
        match &text {
            Some(text) if !text.is_empty() => Some(
                super::json::parse(text).map_err(|_| "JSON-ka codsiga ma saxna.".to_string())?,
            ),
            _ => None,
        }
    } else {
        None
    };
    Ok(object(&[
        ("hab", Value::Str(request.method.clone())),
        ("jid", Value::Str(request.path.clone())),
        ("weydiimo", string_object(&request.query)),
        ("cinwaan", string_object(&request.headers)),
        ("qoraal", text.map(Value::Str).unwrap_or(Value::Null)),
        ("baytyo", super::bytes::from_raw_vec(request.body.clone())),
        ("json", json.unwrap_or(Value::Null)),
        ("qayb", string_object(&params)),
    ]))
}

enum ResponseFault {
    Reply(u16, String),
    Stop(String),
}

fn read_request(stream: &mut TcpStream, limit: usize) -> Result<RawRequest, ResponseFault> {
    let mut buffer = Vec::new();
    let mut chunk = [0u8; 2048];
    let header_end;
    loop {
        if buffer.len() > MAX_HEADERS {
            return Err(ResponseFault::Reply(
                431,
                "Cinwaannada HTTP way ka weyn yihiin.".into(),
            ));
        }
        let read = stream
            .read(&mut chunk)
            .map_err(|_| ResponseFault::Stop("Akhriska codsiga wuu fashilmay.".into()))?;
        if read == 0 {
            return Err(ResponseFault::Stop(
                "Xidhiidhkii codsiga wuu xirmay.".into(),
            ));
        }
        buffer.extend_from_slice(&chunk[..read]);
        if let Some(position) = buffer.windows(4).position(|mark| mark == b"\r\n\r\n") {
            header_end = position + 4;
            break;
        }
    }
    if header_end > MAX_HEADERS {
        return Err(ResponseFault::Reply(
            431,
            "Cinwaannada HTTP way ka weyn yihiin.".into(),
        ));
    }
    let mut headers = [httparse::EMPTY_HEADER; 64];
    let mut parsed = httparse::Request::new(&mut headers);
    let status = parsed
        .parse(&buffer[..header_end])
        .map_err(|_| ResponseFault::Reply(400, "Codsiga HTTP ma saxna.".into()))?;
    if status.is_partial() {
        return Err(ResponseFault::Reply(
            400,
            "Codsiga HTTP ma dhammaystirna.".into(),
        ));
    }
    let method = parsed.method.unwrap_or("").to_ascii_uppercase();
    if !matches!(method.as_str(), "GET" | "POST" | "PUT" | "PATCH" | "DELETE") {
        return Err(ResponseFault::Reply(405, "Habka lama oggola.".into()));
    }
    let target = parsed.path.unwrap_or("/");
    let (path, query) = split_target(target)?;
    let mut header_map = HashMap::new();
    for header in parsed.headers {
        let name = header.name.to_ascii_lowercase();
        let value = std::str::from_utf8(header.value)
            .map_err(|_| ResponseFault::Reply(400, "Cinwaanka HTTP ma aha UTF-8.".into()))?;
        if matches!(
            name.as_str(),
            "host" | "content-length" | "transfer-encoding"
        ) && header_map.contains_key(&name)
        {
            return Err(ResponseFault::Reply(
                400,
                "Cinwaan muhiim ah ayaa lagu celiyey codsiga HTTP.".into(),
            ));
        }
        header_map.insert(name, value.to_string());
    }
    if parsed.version == Some(1)
        && header_map.get("host").is_none_or(|host| {
            host.is_empty()
                || host
                    .bytes()
                    .any(|byte| byte.is_ascii_whitespace() || byte == b',')
        })
    {
        return Err(ResponseFault::Reply(
            400,
            "Codsiga HTTP wuxuu u baahan yahay hal martigeliye oo sax ah.".into(),
        ));
    }
    if header_map.contains_key("transfer-encoding") {
        let (status, message) = if header_map.contains_key("content-length") {
            (
                400,
                "Dhererka iyo habka gudbinta codsiga HTTP isma waafaqaan.",
            )
        } else {
            (411, "Jidhka chunked lama taageero.")
        };
        return Err(ResponseFault::Reply(status, message.into()));
    }
    let length = match header_map.get("content-length") {
        Some(value) if !value.is_empty() && value.bytes().all(|byte| byte.is_ascii_digit()) => {
            value
                .parse::<usize>()
                .map_err(|_| ResponseFault::Reply(400, "Dhererka jidhka ma saxna.".into()))?
        }
        Some(_) => {
            return Err(ResponseFault::Reply(
                400,
                "Dhererka jidhka ma saxna.".into(),
            ))
        }
        None => 0,
    };
    if length > limit {
        return Err(ResponseFault::Reply(
            413,
            "Jidhka codsiga wuu ka weyn yahay xadka.".into(),
        ));
    }
    let mut body = buffer[header_end..].to_vec();
    while body.len() < length {
        let read = stream
            .read(&mut chunk)
            .map_err(|_| ResponseFault::Stop("Akhriska jidhka wuu fashilmay.".into()))?;
        if read == 0 {
            return Err(ResponseFault::Reply(
                400,
                "Jidhka codsiga ma dhammaystirna.".into(),
            ));
        }
        body.extend_from_slice(&chunk[..read]);
    }
    body.truncate(length);
    Ok(RawRequest {
        method,
        path,
        query,
        headers: header_map,
        body,
    })
}

fn split_target(target: &str) -> Result<(String, HashMap<String, String>), ResponseFault> {
    let (path, query) = target.split_once('?').unwrap_or((target, ""));
    if !path.starts_with('/') {
        return Err(ResponseFault::Reply(400, "Jidka codsiga ma saxna.".into()));
    }
    let path = super::url::decode(path, false)
        .map_err(|_| ResponseFault::Reply(400, "Jidka codsiga ma saxna.".into()))?;
    let mut fields = HashMap::new();
    if !query.is_empty() {
        for pair in query.split('&') {
            if pair.is_empty() {
                continue;
            }
            let (name, value) = pair.split_once('=').unwrap_or((pair, ""));
            let name = super::url::decode(name, true)
                .map_err(|_| ResponseFault::Reply(400, "Weydiinta codsiga ma saxna.".into()))?;
            let value = super::url::decode(value, true)
                .map_err(|_| ResponseFault::Reply(400, "Weydiinta codsiga ma saxna.".into()))?;
            fields.insert(name, value);
        }
    }
    Ok((path, fields))
}

fn compile_routes(value: &Value) -> Result<Vec<Route>, SoplangError> {
    let Value::List(routes) = value else {
        return Err(error("Waddooyinka serverka waa inay noqdaan teed."));
    };
    let routes = routes
        .try_borrow()
        .map_err(|_| error("Teedka hadda lama akhriyi karo."))?;
    routes.iter().map(compile_route).collect()
}

fn compile_route(value: &Value) -> Result<Route, SoplangError> {
    let Value::List(route) = value else {
        return Err(error("Waddo kasta waa inay noqotaa teed saddex qayb ah."));
    };
    let route = route
        .try_borrow()
        .map_err(|_| error("Teedka hadda lama akhriyi karo."))?;
    if route.len() != 3 {
        return Err(error("Waddo kasta waa inay noqotaa teed saddex qayb ah."));
    }
    let Value::Str(method) = &route[0] else {
        return Err(error("Habka waddada waa inuu noqdaa qoraal."));
    };
    let method = method.to_ascii_uppercase();
    if !matches!(method.as_str(), "GET" | "POST" | "PUT" | "PATCH" | "DELETE") {
        return Err(error(
            "Habka waddada waa inuu noqdaa GET, POST, PUT, PATCH, ama DELETE.",
        ));
    }
    let Value::Str(path) = &route[1] else {
        return Err(error("Jidka waddada waa inuu noqdaa qoraal."));
    };
    if !path.starts_with('/') {
        return Err(error("Jidka waddada waa inuu ka bilaabmaa /."));
    }
    let handler = match &route[2] {
        handler @ (Value::Function(_) | Value::NativeFunction(_) | Value::Closure(_)) => {
            handler.clone()
        }
        _ => return Err(error("Qabta-hawsha waddada waa inuu noqdaa hawl.")),
    };
    Ok(Route {
        method,
        parts: parse_pattern(path)?,
        handler,
    })
}

fn parse_pattern(path: &str) -> Result<Vec<Part>, SoplangError> {
    let mut parts = Vec::new();
    for part in path.split('/').skip(1) {
        if part.is_empty() {
            continue;
        }
        if let Some(name) = part.strip_prefix(':') {
            if name.is_empty()
                || parts
                    .iter()
                    .any(|item| matches!(item, Part::Param(found) if found == name))
            {
                return Err(error("Magaca qaybta jidka ma saxna."));
            }
            parts.push(Part::Param(name.to_string()));
        } else {
            parts.push(Part::Fixed(part.to_string()));
        }
    }
    Ok(parts)
}

fn match_path(pattern: &[Part], path: &str) -> Option<HashMap<String, String>> {
    let mut actual = path.split('/').skip(1).filter(|part| !part.is_empty());
    let mut params = HashMap::new();
    for part in pattern {
        let Some(value) = actual.next() else {
            return None;
        };
        match part {
            Part::Fixed(expected) if expected == value => {}
            Part::Param(name) => {
                params.insert(name.clone(), value.to_string());
            }
            _ => return None,
        }
    }
    actual.next().is_none().then_some(params)
}

fn response_from(value: &Value) -> Result<(u16, Vec<(String, String)>, Vec<u8>, String), String> {
    let Value::Object(object) = value else {
        return Err("Jawaabtu waa inay noqotaa walax.".into());
    };
    let object = object
        .try_borrow()
        .map_err(|_| "Walaxda jawaabta lama akhriyi karo.".to_string())?;
    let status = match object.get("heer") {
        None => 200,
        Some(Value::Int(status)) if (200..=599).contains(status) => *status as u16,
        _ => return Err("Heerka jawaabta ma saxna.".into()),
    };
    let mut headers = Vec::new();
    if let Some(value) = object.get("cinwaan") {
        let Value::Object(fields) = value else {
            return Err("Cinwaannada jawaabta waa inay noqdaan walax.".into());
        };
        let fields = fields
            .try_borrow()
            .map_err(|_| "Cinwaannada jawaabta lama akhriyi karo.".to_string())?;
        for (name, value) in fields.iter() {
            let Value::Str(text) = value else {
                return Err("Qiimaha cinwaanka jawaabta waa inuu noqdaa qoraal.".into());
            };
            if !valid_response_header(name, text) {
                return Err("Magaca ama qiimaha cinwaanka jawaabta ma saxna.".into());
            }
            headers.push((name.clone(), text.clone()));
        }
    }
    if headers.len() > 64
        || headers
            .iter()
            .map(|(name, value)| name.len() + value.len() + 4)
            .sum::<usize>()
            > MAX_HEADERS
    {
        return Err("Cinwaannada jawaabta way ka weyn yihiin xadka.".into());
    }
    if object
        .get("qoraal")
        .is_some_and(|value| !matches!(value, Value::Str(_)))
        || object
            .get("baytyo")
            .is_some_and(|value| !matches!(value, Value::Bytes(_)))
    {
        return Err("Nooca jidhka jawaabta ma saxna.".into());
    }
    let mut kinds = 0;
    let mut body = Vec::new();
    let mut content_type = String::new();
    if let Some(Value::Str(text)) = object.get("qoraal") {
        kinds += 1;
        body = text.as_bytes().to_vec();
        content_type = "text/plain; charset=utf-8".into();
    }
    if let Some(Value::Bytes(bytes)) = object.get("baytyo") {
        kinds += 1;
        body = super::bytes::read(bytes)
            .map_err(|_| "Baytyada jawaabta lama akhriyi karo.".to_string())?
            .clone();
        content_type = "application/octet-stream".into();
    }
    if let Some(value) = object.get("json") {
        kinds += 1;
        body = super::json::stringify(value)
            .map_err(|failure| failure.message().to_string())?
            .into_bytes();
        content_type = "application/json".into();
    }
    if kinds > 1 {
        return Err("Jawaabtu waxay qaadan kartaa hal nooc oo jidh ah.".into());
    }
    if matches!(status, 204 | 304) && !body.is_empty() {
        return Err("Heerkan jawaabtu ma qaadan karo jidh.".into());
    }
    if !headers
        .iter()
        .any(|(name, _)| name.eq_ignore_ascii_case("content-type"))
        && !content_type.is_empty()
    {
        headers.push(("content-type".into(), content_type.clone()));
    }
    Ok((status, headers, body, content_type))
}

fn limits(options: Option<&Value>) -> Result<(Duration, usize), SoplangError> {
    let Some(value) = options else {
        return Ok((DEFAULT_TIMEOUT, DEFAULT_LIMIT));
    };
    let Value::Object(object) = value else {
        return Err(error("Xulashada serverka waa inay noqotaa walax."));
    };
    let object = object
        .try_borrow()
        .map_err(|_| error("Walaxda hadda lama akhriyi karo."))?;
    let timeout = match object.get("waqti") {
        None => DEFAULT_TIMEOUT,
        Some(Value::Int(seconds)) if *seconds > 0 => Duration::from_secs(*seconds as u64),
        Some(Value::Float(seconds)) if seconds.is_finite() && *seconds > 0.0 => {
            Duration::try_from_secs_f64(*seconds)
                .map_err(|_| error("Waqtiga serverka ma saxna."))?
        }
        _ => return Err(error("Waqtiga serverka waa inuu noqdaa tiro ka weyn eber.")),
    };
    let limit = match object.get("cabbir") {
        None => DEFAULT_LIMIT,
        Some(Value::Int(size)) => usize::try_from(*size)
            .ok()
            .filter(|size| (1..=MAX_LIMIT).contains(size))
            .ok_or_else(|| error("Cabbirka jidhka waa inuu u dhaxeeyaa 1 iyo 16777216."))?,
        _ => return Err(error("Cabbirka jidhka waa inuu noqdaa abn.")),
    };
    Ok((timeout, limit))
}

fn valid_response_header(name: &str, value: &str) -> bool {
    !name.is_empty()
        && name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"!#$%&'*+-.^_`|~".contains(&byte))
        && !matches!(
            name.to_ascii_lowercase().as_str(),
            "content-length" | "transfer-encoding" | "connection"
        )
        && value
            .bytes()
            .all(|byte| byte == b'\t' || (byte >= 0x20 && byte != 0x7f))
}

fn write_response(
    stream: &mut TcpStream,
    status: u16,
    headers: &[(String, String)],
    body: &[u8],
    content_type: &str,
) -> std::io::Result<()> {
    let mut message = format!(
        "HTTP/1.1 {status} {}\r\nConnection: close\r\n",
        reason(status)
    );
    if !matches!(status, 204 | 304) {
        message.push_str(&format!("Content-Length: {}\r\n", body.len()));
    }
    if !content_type.is_empty()
        && !headers
            .iter()
            .any(|(name, _)| name.eq_ignore_ascii_case("content-type"))
    {
        message.push_str(&format!("Content-Type: {content_type}\r\n"));
    }
    for (name, value) in headers {
        message.push_str(name);
        message.push_str(": ");
        message.push_str(value);
        message.push_str("\r\n");
    }
    message.push_str("\r\n");
    stream.write_all(message.as_bytes())?;
    stream.write_all(body)?;
    stream.flush()
}

fn reason(status: u16) -> &'static str {
    match status {
        200 => "OK",
        201 => "Created",
        204 => "No Content",
        301 => "Moved Permanently",
        302 => "Found",
        400 => "Bad Request",
        404 => "Not Found",
        405 => "Method Not Allowed",
        411 => "Length Required",
        413 => "Payload Too Large",
        431 => "Request Header Fields Too Large",
        500 => "Internal Server Error",
        _ => "Status",
    }
}

fn error_body(message: &str) -> Vec<u8> {
    let mut fields = HashMap::new();
    fields.insert("khalad".to_string(), Value::Str(message.to_string()));
    super::json::stringify(&Value::Object(std::rc::Rc::new(std::cell::RefCell::new(
        fields,
    ))))
    .unwrap_or_else(|_| "{\"khalad\":\"Khalad\"}".to_string())
    .into_bytes()
}

fn server_id(value: &Value) -> Result<i64, SoplangError> {
    let Value::Object(object) = value else {
        return Err(error("Serverka waa inuu noqdaa walax."));
    };
    let object = object
        .try_borrow()
        .map_err(|_| error("Walaxda hadda lama akhriyi karo."))?;
    match object.get("aqoonsi") {
        Some(Value::Int(id)) => Ok(*id),
        _ => Err(error("Aqoonsiga serverka ma saxna.")),
    }
}

fn object(fields: &[(&str, Value)]) -> Value {
    let mut map = HashMap::new();
    for (name, value) in fields {
        map.insert((*name).to_string(), value.clone());
    }
    Value::Object(std::rc::Rc::new(std::cell::RefCell::new(map)))
}

fn string_object(fields: &HashMap<String, String>) -> Value {
    let mut map = HashMap::new();
    for (name, value) in fields {
        map.insert(name.clone(), Value::Str(value.clone()));
    }
    Value::Object(std::rc::Rc::new(std::cell::RefCell::new(map)))
}

#[cfg(test)]
mod protocol_tests {
    use super::*;

    fn parse(request: &[u8], limit: usize) -> Result<RawRequest, ResponseFault> {
        let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let address = listener.local_addr().unwrap();
        let request = request.to_vec();
        let client = std::thread::spawn(move || {
            let mut stream = TcpStream::connect(address).unwrap();
            let _ = stream.write_all(&request);
            let _ = stream.shutdown(std::net::Shutdown::Write);
        });
        let (mut stream, _) = listener.accept().unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(2)))
            .unwrap();
        let result = read_request(&mut stream, limit);
        drop(stream);
        client.join().unwrap();
        result
    }

    #[test]
    fn ambiguous_request_framing_and_invalid_hosts_are_rejected() {
        for (headers, status) in [
            ("Host: a\r\nContent-Length: 0\r\nContent-Length: 1\r\n", 400),
            ("Host: a\r\nContent-Length: 0\r\ncontent-length: 0\r\n", 400),
            (
                "Host: a\r\nContent-Length: 0\r\nTransfer-Encoding: chunked\r\n",
                400,
            ),
            ("Host: a\r\nTransfer-Encoding: chunked\r\n", 411),
            ("Host: a\r\nHost: b\r\n", 400),
            ("Host: \r\n", 400),
            ("Host: a b\r\n", 400),
            ("", 400),
            ("Host: a\r\nContent-Length: +0\r\n", 400),
            ("Host: a\r\nContent-Length: 1, 1\r\n", 400),
        ] {
            let request = format!("POST / HTTP/1.1\r\n{headers}\r\n");
            assert!(
                matches!(parse(request.as_bytes(), 32), Err(ResponseFault::Reply(found, _)) if found == status),
                "{request:?}"
            );
        }
        let request = parse(
            b"POST / HTTP/1.1\r\nHost: localhost\r\nContent-Length: 3\r\n\r\nabc",
            3,
        );
        assert!(matches!(request, Ok(request) if request.body == b"abc"));
        assert!(parse(b"GET / HTTP/1.0\r\n\r\n", 32).is_ok());
    }

    #[test]
    fn header_terminator_cannot_bypass_the_size_limit() {
        let prefix = "GET / HTTP/1.1\r\nHost: localhost\r\nX: ";
        let suffix = "\r\n\r\n";
        let request = format!(
            "{prefix}{}{suffix}",
            "a".repeat(MAX_HEADERS + 1 - prefix.len() - suffix.len())
        );
        assert!(matches!(
            parse(request.as_bytes(), 32),
            Err(ResponseFault::Reply(431, _))
        ));
        assert!(matches!(
            parse(
                b"POST / HTTP/1.1\r\nHost: a\r\nContent-Length: 33\r\n\r\n",
                32
            ),
            Err(ResponseFault::Reply(413, _))
        ));
    }

    #[test]
    fn responses_cannot_inject_headers_or_override_framing() {
        for (name, value) in [
            ("X-Test", "ok\r\nX-Evil: yes"),
            ("X-Test", "nul\0"),
            ("X-Test", "del\x7f"),
            ("X Test", "ok"),
            ("X\r\nBad", "ok"),
            ("", "ok"),
            ("Content-Length", "0"),
            ("TRANSFER-ENCODING", "chunked"),
            ("Connection", "keep-alive"),
        ] {
            let response = object(&[("cinwaan", object(&[(name, Value::Str(value.into()))]))]);
            assert!(response_from(&response).is_err(), "{name:?}: {value:?}");
        }
        assert!(response_from(&object(&[(
            "cinwaan",
            object(&[("X-Test", Value::Str("ok\tvalue".into()))])
        )]))
        .is_ok());
        assert!(response_from(&object(&[("qoraal", Value::Int(1))])).is_err());
        assert!(response_from(&object(&[("baytyo", Value::Str("wrong".into()))])).is_err());
        assert!(response_from(&object(&[(
            "cinwaan",
            object(&[("X-Test", Value::Str("a".repeat(MAX_HEADERS)))])
        )]))
        .is_err());
    }

    #[test]
    fn no_body_responses_omit_framing_and_reject_payloads() {
        assert!(response_from(&object(&[("heer", Value::Int(101))])).is_err());
        for status in [204, 304] {
            assert!(response_from(&object(&[
                ("heer", Value::Int(status)),
                ("qoraal", Value::Str("body".into()))
            ]))
            .is_err());
            let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
            let address = listener.local_addr().unwrap();
            let client = std::thread::spawn(move || {
                let mut stream = TcpStream::connect(address).unwrap();
                let mut message = String::new();
                stream.read_to_string(&mut message).unwrap();
                message
            });
            let (mut stream, _) = listener.accept().unwrap();
            write_response(&mut stream, status as u16, &[], &[], "").unwrap();
            drop(stream);
            let message = client.join().unwrap().to_ascii_lowercase();
            assert!(!message.contains("content-length") && message.ends_with("\r\n\r\n"));
        }
    }
}
