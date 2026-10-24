//! HTTP server for JSON APIs, with sequential and task-based exchanges.

use std::cell::RefCell;
use std::collections::HashMap;
use std::io::{self, Read, Write};
#[cfg_attr(not(test), allow(unused_imports))]
use std::net::{TcpListener, TcpStream};
use std::rc::Rc;
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use rustls::ServerConfig;

use super::context::{RuntimeLocal, RuntimeState};
use super::value::Value;
use crate::error::{codes, runtime_error_ex, ErrorMeta, SoplangError};

const STATE: RuntimeLocal<RuntimeState> = RuntimeLocal::new(|state| state);

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);
const DEFAULT_LIMIT: usize = 1024 * 1024;
const MAX_LIMIT: usize = 16 * 1024 * 1024;
const MAX_STREAM_BODY: usize = 1024 * 1024 * 1024;
const MAX_HEADERS: usize = 8 * 1024;
const MAX_STREAM_CHUNK: usize = 64 * 1024;
const DEFAULT_WORKERS: usize = 8;
const MAX_WORKERS: usize = 32;

pub(super) struct Listener {
    socket: TcpListener,
    tls: Option<Arc<ServerConfig>>,
}

static NEXT_ID: AtomicI64 = AtomicI64::new(1);
static NEXT_BODY_ID: AtomicI64 = AtomicI64::new(1);

/// An absolute exchange deadline also bounds TLS's internal read/write loops.
struct DeadlineStream {
    socket: TcpStream,
    expires: Instant,
}

impl DeadlineStream {
    fn remaining(&self) -> std::io::Result<Duration> {
        self.expires
            .checked_duration_since(Instant::now())
            .filter(|time| !time.is_zero())
            .ok_or_else(|| {
                std::io::Error::new(
                    std::io::ErrorKind::TimedOut,
                    "Waqtigii codsiga wuu dhammaaday.",
                )
            })
    }
}

impl Read for DeadlineStream {
    fn read(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
        if buffer.is_empty() {
            return Ok(0);
        }
        self.socket.set_read_timeout(Some(self.remaining()?))?;
        self.socket.read(buffer)
    }
}

impl Write for DeadlineStream {
    fn write(&mut self, buffer: &[u8]) -> std::io::Result<usize> {
        if buffer.is_empty() {
            return Ok(0);
        }
        self.socket.set_write_timeout(Some(self.remaining()?))?;
        self.socket.write(buffer)
    }
    fn flush(&mut self) -> std::io::Result<()> {
        self.remaining()?;
        self.socket.flush()
    }
}

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
    streaming: bool,
}

pub(super) struct RequestBody {
    connection_id: i64,
    prefetched: Vec<u8>,
    offset: usize,
    remaining: usize,
    expires: Instant,
    reading: bool,
    active: bool,
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
    open_with(host, port, None)
}

pub fn open_tls(host: &str, port: i64, cert: &str, key: &str) -> Result<Value, SoplangError> {
    open_with(host, port, Some((cert, key)))
}

fn open_with(host: &str, port: i64, tls: Option<(&str, &str)>) -> Result<Value, SoplangError> {
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
        let tls = match tls {
            Some((cert, key)) => Some(load_tls(cert, key)?),
            None => None,
        };
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        let mut listeners = state.http_listeners.borrow_mut();
        listeners.get_or_insert_with(HashMap::new).insert(
            id,
            Listener {
                socket: listener,
                tls,
            },
        );
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
        let (stream, tls) = {
            let listeners = state.http_listeners.borrow_mut();
            let listener = listeners
                .as_ref()
                .and_then(|listeners| listeners.get(&id))
                .ok_or_else(|| error("Serverka waa la xidhay."))?;
            let stream = listener
                .socket
                .accept()
                .map_err(|failure| {
                    error(format!(
                        "Codsiga lama aqbalin. {}",
                        crate::error::io_reason(&failure)
                    ))
                })?
                .0;
            (stream, listener.tls.clone())
        };
        let expires = Instant::now()
            .checked_add(timeout)
            .ok_or_else(|| error("Waqtiga serverka aad buu u dheer yahay."))?;
        let mut stream = DeadlineStream {
            socket: stream,
            expires,
        };
        if let Some(config) = tls {
            let connection = rustls::ServerConnection::new(config)
                .map_err(|failure| error(format!("TLS-ka lama bilaabin. {failure}")))?;
            let mut stream = rustls::StreamOwned::new(connection, stream);
            serve(&mut stream, &routes, limit)
        } else {
            serve(&mut stream, &routes, limit)
        }
    })
}

/// Start one concurrently scheduled HTTP exchange on an asynchronous TCP
/// listener. Parsing and user handlers stay on the owning runtime thread.
pub fn accept_async(
    server: &Value,
    routes: &Value,
    options: Option<&Value>,
) -> Result<Value, SoplangError> {
    super::net::endpoint_id(server, "dhegeysi_async")?;
    super::tasks::spawn(
        Value::NativeFunction(serve_async_task),
        vec![
            server.clone(),
            routes.clone(),
            options.cloned().unwrap_or(Value::Null),
        ],
    )
}

fn serve_async_task(args: Vec<Value>) -> Result<Value, SoplangError> {
    let (timeout, limit) = limits(args.get(2).filter(|value| !matches!(value, Value::Null)))?;
    let routes = compile_routes(&args[1])?;
    let connection = accept_async_connection(&args[0], timeout)?;
    serve_async_connection(&connection, &routes, timeout, limit)
}

fn accept_async_connection(server: &Value, timeout: Duration) -> Result<Value, SoplangError> {
    let accept = super::async_net::accept_server(server, timeout)?;
    let Value::Task(accept) = accept else {
        unreachable!("asynchronous accept returns a task")
    };
    super::tasks::await_task(&accept)
}

fn serve_async_connection(
    connection: &Value,
    routes: &[Route],
    timeout: Duration,
    limit: usize,
) -> Result<Value, SoplangError> {
    let expires = Instant::now()
        .checked_add(timeout)
        .ok_or_else(|| error("Waqtiga serverka aad buu u dheer yahay."))?;
    let mut stream = TaskStream::for_http(connection.clone(), expires);
    let result = serve_async_request(&mut stream, connection, routes, limit, expires);
    let bridge_failure = stream.take_failure();
    let _ = super::net::close(connection);
    match bridge_failure {
        Some(failure) => Err(failure),
        None => result,
    }
}

/// Serve until the asynchronous listener closes. A fixed number of workers
/// bounds accepted connections and lets in-flight exchanges drain on close.
pub fn serve_async(
    server: &Value,
    routes: &Value,
    options: Option<&Value>,
) -> Result<Value, SoplangError> {
    super::net::endpoint_id(server, "dhegeysi_async")?;
    super::tasks::spawn(
        Value::NativeFunction(serve_async_loop),
        vec![
            server.clone(),
            routes.clone(),
            options.cloned().unwrap_or(Value::Null),
        ],
    )
}

fn serve_async_loop(args: Vec<Value>) -> Result<Value, SoplangError> {
    let options = args.get(2).filter(|value| !matches!(value, Value::Null));
    let _ = limits(options)?;
    let _ = compile_routes(&args[1])?;
    let workers = worker_count(options)?;
    let mut pending = Vec::with_capacity(workers);
    for _ in 0..workers {
        let worker = super::tasks::spawn(Value::NativeFunction(serve_async_worker), args.clone())?;
        let Value::Task(worker) = worker else {
            unreachable!("server worker returns a task")
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

fn serve_async_worker(args: Vec<Value>) -> Result<Value, SoplangError> {
    let (timeout, limit) = limits(args.get(2).filter(|value| !matches!(value, Value::Null)))?;
    let routes = compile_routes(&args[1])?;
    let server = &args[0];
    loop {
        if !super::async_net::listener_is_open(server) {
            return Ok(Value::Null);
        }
        let connection = match accept_async_connection(server, timeout) {
            Ok(connection) => connection,
            Err(_) if !super::async_net::listener_is_open(server) => return Ok(Value::Null),
            Err(failure) => {
                let _ = super::net::close(server);
                return Err(failure);
            }
        };
        // A client disconnect or exchange timeout affects only its connection.
        let _ = serve_async_connection(&connection, &routes, timeout, limit);
    }
}

fn worker_count(options: Option<&Value>) -> Result<usize, SoplangError> {
    let Some(Value::Object(options)) = options else {
        return Ok(DEFAULT_WORKERS);
    };
    let options = options
        .try_borrow()
        .map_err(|_| error("Walaxda hadda lama akhriyi karo."))?;
    match options.get("shaqaale") {
        None => Ok(DEFAULT_WORKERS),
        Some(Value::Int(workers)) => usize::try_from(*workers)
            .ok()
            .filter(|workers| (1..=MAX_WORKERS).contains(workers))
            .ok_or_else(|| error("Tirada shaqaalaha serverka waa inay u dhaxaysaa 1 iyo 32.")),
        _ => Err(error("Tirada shaqaalaha serverka waa inay noqotaa abn.")),
    }
}

pub(super) struct TaskStream {
    connection: Value,
    timeout: StreamTimeout,
    failure: Option<SoplangError>,
}

#[derive(Clone, Copy)]
enum StreamTimeout {
    Absolute(Instant),
    PerOperation(Duration),
}

impl TaskStream {
    fn for_http(connection: Value, expires: Instant) -> Self {
        Self {
            connection,
            timeout: StreamTimeout::Absolute(expires),
            failure: None,
        }
    }

    pub(super) fn for_websocket(connection: Value, timeout: Duration) -> Self {
        Self {
            connection,
            timeout: StreamTimeout::PerOperation(timeout),
            failure: None,
        }
    }

    pub(super) fn take_failure(&mut self) -> Option<SoplangError> {
        self.failure.take()
    }

    fn wait(&mut self, task: Result<Value, SoplangError>) -> io::Result<Value> {
        if self.failure.is_some() {
            return Err(io::Error::other("Hawsha shabakadda way fashilantay."));
        }
        let remaining = match self.timeout {
            StreamTimeout::Absolute(expires) => expires
                .checked_duration_since(Instant::now())
                .filter(|time| !time.is_zero())
                .ok_or_else(|| self.fail(error("Waqtigii codsiga wuu dhammaaday.")))?,
            StreamTimeout::PerOperation(timeout) => timeout,
        };
        let task = task.map_err(|failure| self.fail(failure))?;
        let Value::Task(task) = task else {
            unreachable!("asynchronous TCP operation returns a task")
        };
        let millis = remaining.as_millis().min(u64::MAX as u128) as u64;
        super::tasks::await_with_timeout(&task, millis).map_err(|failure| self.fail(failure))
    }

    fn fail(&mut self, failure: SoplangError) -> io::Error {
        self.failure = Some(failure);
        io::Error::other("Hawsha shabakadda way fashilantay.")
    }
}

impl Read for TaskStream {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        if buffer.is_empty() {
            return Ok(0);
        }
        let operation = super::async_net::read(vec![
            self.connection.clone(),
            Value::Int(buffer.len().min(64 * 1024) as i64),
        ]);
        let result = self.wait(operation)?;
        let Value::Bytes(bytes) = result else {
            unreachable!("TCP read returns bytes")
        };
        let bytes = bytes
            .try_borrow()
            .map_err(|_| self.fail(error("Baytyada la akhriyey lama heli karo.")))?;
        buffer[..bytes.len()].copy_from_slice(&bytes);
        Ok(bytes.len())
    }
}

impl Write for TaskStream {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        if buffer.is_empty() {
            return Ok(0);
        }
        let operation = super::async_net::write(vec![
            self.connection.clone(),
            super::bytes::from_raw_vec(buffer.to_vec()),
        ]);
        self.wait(operation)?;
        Ok(buffer.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// Exercise the exact network parser, router, handler and response encoder,
/// without opening a listening socket or requiring a second process.
pub fn test(
    routes: &Value,
    method: &str,
    target: &str,
    options: Option<&Value>,
    server_options: Option<&Value>,
) -> Result<Value, SoplangError> {
    let method = method.to_ascii_uppercase();
    if method.is_empty()
        || !method.bytes().all(|b| b.is_ascii_uppercase())
        || !target.starts_with('/')
        || target.bytes().any(|b| b <= b' ' || b == 0x7f)
    {
        return Err(error("Habka ama jidka codsiga tijaabadu ma saxna."));
    }
    let routes = compile_routes(routes)?;
    let (_, limit) = limits(server_options)?;
    let (headers, body, response_limit) = super::http::request_parts(options)?;
    if body.len() > MAX_LIMIT {
        return Err(error("Jidhka codsiga tijaabadu aad buu u weyn yahay."));
    }
    let mut request = format!("{method} {target} HTTP/1.1\r\n");
    if !headers
        .iter()
        .any(|(name, _)| name.eq_ignore_ascii_case("host"))
    {
        request.push_str("Host: localhost\r\n");
    }
    for (name, value) in &headers {
        if !valid_response_header(name, value) {
            return Err(error("Cinwaanka codsiga tijaabadu ma saxna."));
        }
        request.push_str(&format!("{name}: {value}\r\n"));
    }
    request.push_str(&format!("Content-Length: {}\r\n\r\n", body.len()));
    if request.len() > MAX_HEADERS {
        return Err(error(
            "Cinwaannada codsiga tijaabadu aad bay u weyn yihiin.",
        ));
    }
    let mut request = request.into_bytes();
    request.extend_from_slice(&body);
    let mut stream = MemoryExchange {
        input: std::io::Cursor::new(request),
        output: Vec::new(),
        response_limit,
    };
    serve(&mut stream, &routes, limit)?;
    let mut headers = [httparse::EMPTY_HEADER; 80];
    let mut response = httparse::Response::new(&mut headers);
    let parsed = response
        .parse(&stream.output)
        .map_err(|_| error("Jawaabta tijaabadu ma saxna."))?;
    let httparse::Status::Complete(offset) = parsed else {
        return Err(error("Jawaabta tijaabadu ma dhammaystirna."));
    };
    let mut fields = HashMap::new();
    for header in response.headers.iter() {
        let text = std::str::from_utf8(header.value)
            .map_err(|_| error("Cinwaanka jawaabtu ma aha UTF-8."))?;
        fields.insert(header.name.to_ascii_lowercase(), text.to_owned());
    }
    let body = if fields
        .get("transfer-encoding")
        .is_some_and(|value| value.eq_ignore_ascii_case("chunked"))
    {
        decode_chunked_response(&stream.output[offset..], response_limit)?
    } else {
        stream.output[offset..].to_vec()
    };
    if body.len() > response_limit {
        return Err(error("Jawaabta tijaabadu way ka weyn tahay xadka."));
    }
    if fields
        .get("content-length")
        .is_some_and(|length| length.parse::<usize>().ok() != Some(body.len()))
    {
        return Err(error("Jidhka jawaabta tijaabadu ma dhammaystirna."));
    }
    super::http::decoded_response(i64::from(response.code.unwrap_or(500)), fields, body)
}

struct MemoryExchange {
    input: std::io::Cursor<Vec<u8>>,
    output: Vec<u8>,
    response_limit: usize,
}

impl Read for MemoryExchange {
    fn read(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
        self.input.read(buffer)
    }
}

impl Write for MemoryExchange {
    fn write(&mut self, buffer: &[u8]) -> std::io::Result<usize> {
        if self.output.len().saturating_add(buffer.len()) > self.response_limit + MAX_HEADERS {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Jawaabtu aad bay u weyn tahay.",
            ));
        }
        self.output.extend_from_slice(buffer);
        Ok(buffer.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

fn serve<S: Read + Write>(
    stream: &mut S,
    routes: &[Route],
    limit: usize,
) -> Result<Value, SoplangError> {
    match read_request(stream, limit.min(MAX_LIMIT)) {
        Ok(request) => dispatch(stream, routes, request),
        Err(ResponseFault::Reply(status, message)) => {
            let _ = write_response(
                stream,
                status,
                &[],
                &error_body(&message),
                "application/json",
            );
            Ok(Value::Null)
        }
        Err(ResponseFault::Stop(message)) => Err(error(message)),
    }
}

fn serve_async_request(
    stream: &mut TaskStream,
    connection: &Value,
    routes: &[Route],
    limit: usize,
    expires: Instant,
) -> Result<Value, SoplangError> {
    let (mut request, length) = match read_request_head(stream, limit) {
        Ok(head) => head,
        Err(ResponseFault::Reply(status, message)) => {
            let _ = write_response(
                stream,
                status,
                &[],
                &error_body(&message),
                "application/json",
            );
            return Ok(Value::Null);
        }
        Err(ResponseFault::Stop(message)) => return Err(error(message)),
    };
    let (route, _) = select_route(routes, &request);
    if let Some((route, params)) = route {
        if route.streaming {
            let handle = register_request_body(connection, &request.body, length, expires)?;
            let request_value = streaming_request_object(&request, params, handle.clone(), length);
            let result = dispatch_handler(stream, route, request_value);
            close_request_body(&handle);
            return result;
        }
    }
    if length > limit.min(MAX_LIMIT) {
        let _ = write_response(
            stream,
            413,
            &[],
            &error_body("Jidhka codsiga wuu ka weyn yahay xadka."),
            "application/json",
        );
        return Ok(Value::Null);
    }
    match fill_request_body(stream, &mut request, length) {
        Ok(()) => dispatch(stream, routes, request),
        Err(ResponseFault::Reply(status, message)) => {
            let _ = write_response(
                stream,
                status,
                &[],
                &error_body(&message),
                "application/json",
            );
            Ok(Value::Null)
        }
        Err(ResponseFault::Stop(message)) => Err(error(message)),
    }
}

fn load_tls(cert: &str, key: &str) -> Result<Arc<ServerConfig>, SoplangError> {
    let _ = rustls::crypto::ring::default_provider().install_default();
    let cert_file = std::fs::File::open(cert).map_err(|failure| {
        error(format!(
            "Shahaadada lama furin. {}",
            crate::error::io_reason(&failure)
        ))
    })?;
    let key_file = std::fs::File::open(key).map_err(|failure| {
        error(format!(
            "Furaha lama furin. {}",
            crate::error::io_reason(&failure)
        ))
    })?;
    let certs = rustls_pemfile::certs(&mut std::io::BufReader::new(cert_file))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|failure| error(format!("Shahaadada ma saxna. {failure}")))?;
    let key = rustls_pemfile::private_key(&mut std::io::BufReader::new(key_file))
        .map_err(|failure| error(format!("Furaha ma saxna. {failure}")))?
        .ok_or_else(|| error("Furaha shahaadada lama helin."))?;
    let config = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, key)
        .map_err(|failure| error(format!("TLS-ka lama diyaarin. {failure}")))?;
    Ok(Arc::new(config))
}

fn dispatch<S: Write>(
    stream: &mut S,
    routes: &[Route],
    request: RawRequest,
) -> Result<Value, SoplangError> {
    let (path_match, method_mismatch) = select_route(routes, &request);
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
    if route.streaming {
        let _ = write_response(
            stream,
            500,
            &[],
            &error_body("Socodka jidhka codsigu wuxuu u baahan yahay server async ah."),
            "application/json",
        );
        return Ok(Value::Null);
    }
    let request_value = match request_object(&request, params) {
        Ok(value) => value,
        Err(message) => {
            let _ = write_response(stream, 400, &[], &error_body(&message), "application/json");
            return Ok(Value::Null);
        }
    };
    dispatch_handler(stream, route, request_value)
}

fn select_route<'a>(
    routes: &'a [Route],
    request: &RawRequest,
) -> (Option<(&'a Route, HashMap<String, String>)>, bool) {
    let mut method_mismatch = false;
    for route in routes {
        if let Some(params) = match_path(&route.parts, &request.path) {
            if route.method == request.method {
                return (Some((route, params)), method_mismatch);
            }
            method_mismatch = true;
        }
    }
    (None, method_mismatch)
}

fn dispatch_handler<S: Write>(
    stream: &mut S,
    route: &Route,
    request_value: Value,
) -> Result<Value, SoplangError> {
    match super::abi::invoke(&route.handler, &[request_value]) {
        Ok(value) => {
            let response = match response_from(&value) {
                Ok(response) => response,
                Err(message) => (
                    500,
                    Vec::new(),
                    ResponseBody::Fixed(error_body(&message)),
                    "application/json".to_string(),
                ),
            };
            match response.2 {
                ResponseBody::Fixed(body) => {
                    let _ = write_response(stream, response.0, &response.1, &body, &response.3);
                }
                ResponseBody::Stream(next) => {
                    write_streaming_response(stream, response.0, &response.1, &next, &response.3)?;
                }
            }
        }
        Err(failure) => {
            let status = failure
                .code()
                .and_then(|code| code.strip_prefix("HTTP"))
                .filter(|code| code.len() == 3 && code.bytes().all(|b| b.is_ascii_digit()))
                .and_then(|code| code.parse::<u16>().ok())
                .filter(|status| (400..=599).contains(status))
                .unwrap_or(500);
            let _ = write_response(
                stream,
                status,
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
    let json = if super::http::is_json_type(&content_type) {
        match &text {
            Some(text) if !text.is_empty() => Some(
                super::json::parse(text).map_err(|_| "JSON-ka codsiga ma saxna.".to_string())?,
            ),
            None if !request.body.is_empty() => {
                return Err("JSON-ka codsigu ma aha qoraal UTF-8 ah.".into())
            }
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

fn streaming_request_object(
    request: &RawRequest,
    params: HashMap<String, String>,
    body: Value,
    length: usize,
) -> Value {
    object(&[
        ("hab", Value::Str(request.method.clone())),
        ("jid", Value::Str(request.path.clone())),
        ("weydiimo", string_object(&request.query)),
        ("cinwaan", string_object(&request.headers)),
        ("qoraal", Value::Null),
        ("baytyo", Value::Null),
        ("json", Value::Null),
        ("qayb", string_object(&params)),
        ("jidh", body),
        ("dherer_jidh", Value::Int(length as i64)),
    ])
}

fn register_request_body(
    connection: &Value,
    prefetched: &[u8],
    length: usize,
    expires: Instant,
) -> Result<Value, SoplangError> {
    let connection_id = super::net::endpoint_id(connection, "xidhiidh_async")?;
    STATE.with(|state| {
        let id = NEXT_BODY_ID.fetch_add(1, Ordering::Relaxed);
        state.http_request_bodies.borrow_mut().insert(
            id,
            Rc::new(RefCell::new(RequestBody {
                connection_id,
                prefetched: prefetched.to_vec(),
                offset: 0,
                remaining: length - prefetched.len(),
                expires,
                reading: false,
                active: true,
            })),
        );
        Ok(object(&[
            ("aqoonsi", Value::Int(id)),
            ("nooc", Value::Str("jidh_http".into())),
        ]))
    })
}

fn request_body_id(handle: &Value) -> Result<i64, SoplangError> {
    let Value::Object(handle) = handle else {
        return Err(error("Jidhka codsigu waa inuu noqdaa walax."));
    };
    let handle = handle
        .try_borrow()
        .map_err(|_| error("Walaxda hadda lama akhriyi karo."))?;
    match (handle.get("nooc"), handle.get("aqoonsi")) {
        (Some(Value::Str(kind)), Some(Value::Int(id))) if kind == "jidh_http" => Ok(*id),
        _ => Err(error("Aqoonsiga jidhka codsigu ma saxna.")),
    }
}

fn close_request_body(handle: &Value) {
    if let Ok(id) = request_body_id(handle) {
        STATE.with(|state| {
            if let Some(body) = state.http_request_bodies.borrow_mut().remove(&id) {
                body.borrow_mut().active = false;
            }
        });
    }
}

pub fn read_request_body(handle: &Value, limit: i64) -> Result<Value, SoplangError> {
    let limit = usize::try_from(limit)
        .ok()
        .filter(|limit| (1..=MAX_STREAM_CHUNK).contains(limit))
        .ok_or_else(|| error("Cabbirka akhriska jidhku waa inuu u dhaxaysaa 1 iyo 65536."))?;
    let id = request_body_id(handle)?;
    let body = STATE.with(|state| {
        state
            .http_request_bodies
            .borrow()
            .get(&id)
            .cloned()
            .ok_or_else(|| error("Jidhka codsigu wuu xirmay."))
    })?;
    let (connection_id, read_size, millis) = {
        let mut body = body
            .try_borrow_mut()
            .map_err(|_| error("Jidhka codsiga hadda waa la akhriyayaa."))?;
        if !body.active {
            return Err(error("Jidhka codsigu wuu xirmay."));
        }
        if body.offset < body.prefetched.len() {
            let end = (body.offset + limit).min(body.prefetched.len());
            let bytes = body.prefetched[body.offset..end].to_vec();
            body.offset = end;
            return Ok(super::bytes::from_raw_vec(bytes));
        }
        if body.remaining == 0 {
            return Ok(super::bytes::from_raw_vec(Vec::new()));
        }
        if body.reading {
            return Err(error("Jidhka codsiga hadda waa la akhriyayaa."));
        }
        let remaining = body
            .expires
            .checked_duration_since(Instant::now())
            .filter(|time| !time.is_zero())
            .ok_or_else(|| error("Waqtigii codsiga wuu dhammaaday."))?;
        body.reading = true;
        (
            body.connection_id,
            limit.min(body.remaining),
            remaining.as_millis().min(u64::MAX as u128) as u64,
        )
    };
    let result = (|| {
        let operation = super::async_net::read(vec![
            super::net::endpoint(connection_id, "xidhiidh_async", "", 0),
            Value::Int(read_size as i64),
        ])?;
        let Value::Task(operation) = operation else {
            unreachable!("asynchronous TCP read returns a task")
        };
        super::tasks::await_with_timeout(&operation, millis)
    })();
    let mut body = body
        .try_borrow_mut()
        .map_err(|_| error("Jidhka codsiga hadda waa la akhriyayaa."))?;
    body.reading = false;
    if !body.active {
        return Err(error("Jidhka codsigu wuu xirmay."));
    }
    let Value::Bytes(bytes) = result? else {
        unreachable!("asynchronous TCP read returns bytes")
    };
    let count = bytes
        .try_borrow()
        .map_err(|_| error("Baytyada jidhka codsiga lama akhriyi karo."))?
        .len();
    if count == 0 {
        return Err(error("Jidhka codsiga ma dhammaystirna."));
    }
    body.remaining -= count;
    Ok(Value::Bytes(bytes))
}

enum ResponseFault {
    Reply(u16, String),
    Stop(String),
}

fn read_request<S: Read>(stream: &mut S, limit: usize) -> Result<RawRequest, ResponseFault> {
    let (mut request, length) = read_request_head(stream, limit)?;
    fill_request_body(stream, &mut request, length)?;
    Ok(request)
}

fn read_request_head<S: Read>(
    stream: &mut S,
    limit: usize,
) -> Result<(RawRequest, usize), ResponseFault> {
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
    if !matches!(
        method.as_str(),
        "GET" | "POST" | "PUT" | "PATCH" | "DELETE" | "OPTIONS"
    ) {
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
    body.truncate(length);
    Ok((
        RawRequest {
            method,
            path,
            query,
            headers: header_map,
            body,
        },
        length,
    ))
}

fn fill_request_body<S: Read>(
    stream: &mut S,
    request: &mut RawRequest,
    length: usize,
) -> Result<(), ResponseFault> {
    let mut chunk = [0u8; 2048];
    while request.body.len() < length {
        let read = stream
            .read(&mut chunk)
            .map_err(|_| ResponseFault::Stop("Akhriska jidhka wuu fashilmay.".into()))?;
        if read == 0 {
            return Err(ResponseFault::Reply(
                400,
                "Jidhka codsiga ma dhammaystirna.".into(),
            ));
        }
        request.body.extend_from_slice(&chunk[..read]);
    }
    request.body.truncate(length);
    Ok(())
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
    if !matches!(route.len(), 3 | 4) {
        return Err(error(
            "Waddo kasta waa inay noqotaa teed saddex ama afar qayb ah.",
        ));
    }
    let Value::Str(method) = &route[0] else {
        return Err(error("Habka waddada waa inuu noqdaa qoraal."));
    };
    let method = method.to_ascii_uppercase();
    if !matches!(
        method.as_str(),
        "GET" | "POST" | "PUT" | "PATCH" | "DELETE" | "OPTIONS"
    ) {
        return Err(error(
            "Habka waddada waa inuu noqdaa GET, POST, PUT, PATCH, DELETE, ama OPTIONS.",
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
    let streaming = match route.get(3) {
        None => false,
        Some(Value::Str(mode)) if mode == "socod" => true,
        _ => return Err(error("Nooca waddada afraad waa inuu noqdaa 'socod'.")),
    };
    Ok(Route {
        method,
        parts: parse_pattern(path)?,
        handler,
        streaming,
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

enum ResponseBody {
    Fixed(Vec<u8>),
    Stream(Value),
}

fn response_from(
    value: &Value,
) -> Result<(u16, Vec<(String, String)>, ResponseBody, String), String> {
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
    let mut stream = None;
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
    if let Some(value) = object.get("socod") {
        if !matches!(
            value,
            Value::Function(_) | Value::Closure(_) | Value::NativeFunction(_)
        ) {
            return Err("Soo-saarka socodka jawaabtu waa inuu noqdaa hawl.".into());
        }
        kinds += 1;
        stream = Some(value.clone());
        content_type = "application/octet-stream".into();
    }
    if kinds > 1 {
        return Err("Jawaabtu waxay qaadan kartaa hal nooc oo jidh ah.".into());
    }
    if matches!(status, 204 | 304) && (!body.is_empty() || stream.is_some()) {
        return Err("Heerkan jawaabtu ma qaadan karo jidh.".into());
    }
    if !headers
        .iter()
        .any(|(name, _)| name.eq_ignore_ascii_case("content-type"))
        && !content_type.is_empty()
    {
        headers.push(("content-type".into(), content_type.clone()));
    }
    Ok((
        status,
        headers,
        stream.map_or(ResponseBody::Fixed(body), ResponseBody::Stream),
        content_type,
    ))
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
            .filter(|size| (1..=MAX_STREAM_BODY).contains(size))
            .ok_or_else(|| error("Cabbirka jidhka waa inuu u dhaxeeyaa 1 iyo 1073741824."))?,
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

fn write_response<S: Write>(
    stream: &mut S,
    status: u16,
    headers: &[(String, String)],
    body: &[u8],
    content_type: &str,
) -> std::io::Result<()> {
    write_response_head(stream, status, headers, content_type, Some(body.len()))?;
    stream.write_all(body)?;
    stream.flush()
}

fn write_response_head<S: Write>(
    stream: &mut S,
    status: u16,
    headers: &[(String, String)],
    content_type: &str,
    length: Option<usize>,
) -> std::io::Result<()> {
    let mut message = format!(
        "HTTP/1.1 {status} {}\r\nConnection: close\r\n",
        reason(status)
    );
    if !matches!(status, 204 | 304) {
        match length {
            Some(length) => message.push_str(&format!("Content-Length: {length}\r\n")),
            None => message.push_str("Transfer-Encoding: chunked\r\n"),
        }
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
    stream.write_all(message.as_bytes())
}

fn write_streaming_response<S: Write>(
    stream: &mut S,
    status: u16,
    headers: &[(String, String)],
    next: &Value,
    content_type: &str,
) -> Result<(), SoplangError> {
    write_response_head(stream, status, headers, content_type, None).map_err(|failure| {
        error(format!(
            "Cinwaanka jawaabta lama qorin. {}",
            crate::error::io_reason(&failure)
        ))
    })?;
    loop {
        let chunk = super::abi::invoke(next, &[])?;
        let bytes = match chunk {
            Value::Null => break,
            Value::Str(text) => text.into_bytes(),
            Value::Bytes(bytes) => super::bytes::read(&bytes)?.clone(),
            _ => {
                return Err(error(
                    "Qaybta socodka jawaabtu waa inay noqotaa qoraal, baytyo, ama maran.",
                ))
            }
        };
        if bytes.is_empty() || bytes.len() > MAX_STREAM_CHUNK {
            return Err(error(
                "Qaybta socodka jawaabtu waa inay u dhaxaysaa 1 iyo 65536 bayt.",
            ));
        }
        stream
            .write_all(format!("{:X}\r\n", bytes.len()).as_bytes())
            .and_then(|_| stream.write_all(&bytes))
            .and_then(|_| stream.write_all(b"\r\n"))
            .map_err(|failure| {
                error(format!(
                    "Jidhka jawaabta lama qorin. {}",
                    crate::error::io_reason(&failure)
                ))
            })?;
    }
    stream
        .write_all(b"0\r\n\r\n")
        .and_then(|_| stream.flush())
        .map_err(|failure| {
            error(format!(
                "Jawaabta lama dhammaystirin. {}",
                crate::error::io_reason(&failure)
            ))
        })
}

fn decode_chunked_response(body: &[u8], limit: usize) -> Result<Vec<u8>, SoplangError> {
    let mut cursor = 0;
    let mut decoded = Vec::new();
    loop {
        let end = body[cursor..]
            .windows(2)
            .position(|mark| mark == b"\r\n")
            .map(|offset| cursor + offset)
            .ok_or_else(|| error("Jawaabta socodku ma dhammaystirna."))?;
        let length = std::str::from_utf8(&body[cursor..end])
            .ok()
            .and_then(|text| usize::from_str_radix(text, 16).ok())
            .ok_or_else(|| error("Dhererka qaybta jawaabtu ma saxna."))?;
        cursor = end + 2;
        if length == 0 {
            if body.get(cursor..) != Some(b"\r\n".as_slice()) {
                return Err(error("Jawaabta socodku ma dhammaystirna."));
            }
            return Ok(decoded);
        }
        if decoded.len().saturating_add(length) > limit {
            return Err(error("Jawaabta tijaabadu way ka weyn tahay xadka."));
        }
        let end = cursor
            .checked_add(length)
            .ok_or_else(|| error("Dhererka qaybta jawaabtu ma saxna."))?;
        if body.get(end..end.saturating_add(2)) != Some(b"\r\n".as_slice()) {
            return Err(error("Jawaabta socodku ma dhammaystirna."));
        }
        decoded.extend_from_slice(&body[cursor..end]);
        cursor = end + 2;
    }
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
        fn done(_: Vec<Value>) -> Result<Value, SoplangError> {
            Ok(Value::Null)
        }
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
        assert!(response_from(&object(&[("socod", Value::Str("wrong".into()))])).is_err());
        assert!(response_from(&object(&[
            ("socod", Value::NativeFunction(done)),
            ("json", Value::Int(1)),
        ]))
        .is_err());
        assert!(response_from(&object(&[
            ("heer", Value::Int(204)),
            ("socod", Value::NativeFunction(done)),
        ]))
        .is_err());
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

    #[test]
    fn a_trickled_plain_or_tls_header_cannot_extend_the_exchange_deadline() {
        #[derive(Debug)]
        struct NoCertificate;
        impl rustls::server::ResolvesServerCert for NoCertificate {
            fn resolve(
                &self,
                _: rustls::server::ClientHello<'_>,
            ) -> Option<Arc<rustls::sign::CertifiedKey>> {
                None
            }
        }
        for tls in [false, true] {
            let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
            let address = listener.local_addr().unwrap();
            let client = std::thread::spawn(move || {
                let mut stream = TcpStream::connect(address).unwrap();
                if tls {
                    stream.write_all(&[0x16, 0x03, 0x03, 0x10, 0x00]).unwrap();
                }
                for _ in 0..100 {
                    if stream.write_all(if tls { b"\0" } else { b"G" }).is_err() {
                        break;
                    }
                    std::thread::sleep(Duration::from_millis(25));
                }
            });
            let (socket, _) = listener.accept().unwrap();
            let started = Instant::now();
            let deadline = DeadlineStream {
                socket,
                expires: started + Duration::from_millis(100),
            };
            if tls {
                let config = ServerConfig::builder_with_provider(Arc::new(
                    rustls::crypto::ring::default_provider(),
                ))
                .with_safe_default_protocol_versions()
                .unwrap()
                .with_no_client_auth()
                .with_cert_resolver(Arc::new(NoCertificate));
                let connection = rustls::ServerConnection::new(Arc::new(config)).unwrap();
                let mut stream = rustls::StreamOwned::new(connection, deadline);
                assert!(matches!(
                    read_request(&mut stream, 32),
                    Err(ResponseFault::Stop(_))
                ));
            } else {
                let mut stream = deadline;
                assert!(matches!(
                    read_request(&mut stream, 32),
                    Err(ResponseFault::Stop(_))
                ));
            }
            let elapsed = started.elapsed();
            assert!(
                elapsed >= Duration::from_millis(70) && elapsed < Duration::from_secs(1),
                "trickle extended deadline: {elapsed:?}"
            );
            client.join().unwrap();
        }
    }
}
