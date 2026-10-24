//! JSON HTTP server (3.3 stage).

use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

const PROGRAM: &str = r#"
hawl hello(c) { celi {json: {id: c.qayb.id, x: c.weydiimo.x}} }
hawl create(c) { celi {heer: 201, json: c.json} }
hawl boom(c) { tuur "xumo" }
hawl injection(c) { celi {cinwaan: {"X-Test": e"ok\r\nX-Evil: yes"}} }
hawl go(c) { celi {heer: 302, cinwaan: {Location: "/next"}} }
hawl stream(c) {
    door state = {i: 0}
    hawl next() {
        state.i = state.i + 1
        haddii (state.i == 1) { celi "ab" }
        haddii (state.i == 2) { celi "cd" }
        celi maran
    }
    celi {socod: next, cinwaan: {"content-type": "text/plain"}}
}
door s = server_fur("127.0.0.1", 0)
qor(s.deked)
door routes = [["GET", "/users/:id", hello], ["POST", "/items", create], ["GET", "/fail", boom], ["GET", "/go", go], ["GET", "/injection", injection], ["GET", "/stream", stream]]
server_aqbal(s, routes)
server_aqbal(s, routes)
server_aqbal(s, routes)
server_aqbal(s, routes)
server_aqbal(s, routes)
server_aqbal(s, routes)
server_aqbal(s, routes)
server_aqbal(s, routes)
server_xidh(s)
door failed = been
tijaabi { server_aqbal(s, routes) } qabo (e) { failed = run }
haddii (!failed) { tuur "closed" }
qor("server done")
"#;

fn exchange(port: &str, request: &str) -> (u16, String, String) {
    let mut stream = TcpStream::connect(format!("127.0.0.1:{port}")).expect("connect");
    stream
        .set_read_timeout(Some(Duration::from_secs(3)))
        .unwrap();
    stream.write_all(request.as_bytes()).unwrap();
    let mut buffer = Vec::new();
    let mut chunk = [0u8; 1024];
    loop {
        match stream.read(&mut chunk) {
            Ok(0) | Err(_) => break,
            Ok(n) => buffer.extend_from_slice(&chunk[..n]),
        }
    }
    let text = String::from_utf8_lossy(&buffer).into_owned();
    let status = text
        .split_whitespace()
        .nth(1)
        .unwrap_or("0")
        .parse()
        .unwrap_or(0);
    (status, text.to_ascii_lowercase(), text)
}

fn drive(command: &mut Command) {
    let mut child = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut stdout = child.stdout.take().unwrap();
    let mut port = String::new();
    let mut byte = [0u8; 1];
    loop {
        let n = stdout.read(&mut byte).unwrap();
        if n == 0 {
            break;
        }
        if byte[0] == b'\n' {
            break;
        }
        port.push(byte[0] as char);
    }
    let port = port.trim();
    assert!(!port.is_empty(), "server did not publish a port");
    let (status, _, body) = exchange(
        port,
        "GET /users/9?x=1 HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
    );
    assert_eq!(status, 200, "{body}");
    assert!(
        body.contains("\"id\":\"9\"") && body.contains("\"x\":\"1\""),
        "{body}"
    );
    let (status, _, body) = exchange(
        port,
        "POST /items HTTP/1.1\r\nHost: localhost\r\nContent-Type: application/json\r\nContent-Length: 17\r\nConnection: close\r\n\r\n{\"magac\":\"Ayaan\"}",
    );
    assert_eq!(status, 201, "{body}");
    assert!(body.contains("Ayaan"), "{body}");
    let (status, _, body) = exchange(
        port,
        "GET /missing HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
    );
    assert_eq!(status, 404, "{body}");
    let (status, _, body) = exchange(
        port,
        "GET /fail HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
    );
    assert_eq!(status, 500, "{body}");
    assert!(body.contains("xumo"), "{body}");
    let (status, headers, body) =
        exchange(port, "GET /injection HTTP/1.1\r\nHost: localhost\r\n\r\n");
    assert_eq!(status, 500, "{body}");
    assert!(!headers.contains("x-evil"), "{headers}");
    let (status, _, body) = exchange(
        port,
        "POST /items HTTP/1.1\r\nHost: localhost\r\nContent-Length: 0\r\nContent-Length: 1\r\n\r\n",
    );
    assert_eq!(status, 400, "{body}");
    let (status, headers, _) = exchange(
        port,
        "GET /go HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
    );
    assert_eq!(status, 302, "{headers}");
    assert!(headers.contains("location: /next"), "{headers}");
    let (status, headers, body) = exchange(
        port,
        "GET /stream HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
    );
    assert_eq!(status, 200, "{body}");
    assert!(headers.contains("transfer-encoding: chunked"), "{body}");
    assert!(!headers.contains("content-length:"), "{body}");
    assert!(
        body.ends_with("\r\n\r\n2\r\nab\r\n2\r\ncd\r\n0\r\n\r\n"),
        "{body}"
    );
    let mut rest = String::new();
    stdout.read_to_string(&mut rest).unwrap();
    let output = child.wait_with_output().unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "status {:?}\n{stderr}\n{rest}",
        output.status.code()
    );
    assert!(rest.contains("server done"), "{rest}");
}

#[test]
fn json_server_handles_routes_errors_and_shutdown() {
    let mut command = Command::new(env!("CARGO_BIN_EXE_soplang"));
    command.arg("-c").arg(PROGRAM);
    drive(&mut command);
}

#[test]
fn json_server_runs_through_llvm() {
    let stmts = soplang::frontend::parse_source(PROGRAM, None).unwrap();
    let symbols = soplang::analyze(&stmts).unwrap();
    let hir = soplang::HirLowering::lower(&symbols, &stmts);
    let ir = soplang::backend::llvm_ir::emit_llvm_ir(&hir, &symbols).unwrap();
    let mut path = std::env::temp_dir();
    path.push(format!("soplang_http_server_{}", std::process::id()));
    soplang::backend::llvm_ir::link_llvm_binary(&ir, &path).unwrap();
    let mut command = Command::new(&path);
    drive(&mut command);
    let _ = std::fs::remove_file(path);
}

const CONCURRENT_PROGRAM: &str = r#"
keen "web" sida web
door app = cusub web.Web()
app.hel("GET", "/slow", hawl(codsi) {
    qor("slow-started")
    sug(hurdada(1000))
    celi web.json({kind: "slow"})
})
app.hel("GET", "/fast", hawl(codsi) { celi web.json({kind: "fast"}) })
door server = sug(tcp_dhegeysi_shaqo("127.0.0.1", 0))
qor(server.deked)
door slow = app.aqbal_shaqo(server)
door fast = app.aqbal_shaqo(server)
sug(slow)
sug(fast)
tcp_xidh(server)
qor("server done")
"#;

fn drive_concurrent(command: &mut Command) {
    let mut child = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut stdout = BufReader::new(child.stdout.take().unwrap());
    let mut port = String::new();
    stdout.read_line(&mut port).unwrap();
    let port = port.trim();
    assert!(!port.is_empty(), "server did not publish a port");

    let mut slow = TcpStream::connect(format!("127.0.0.1:{port}")).unwrap();
    slow.set_read_timeout(Some(Duration::from_secs(3))).unwrap();
    slow.write_all(b"GET /slow HTTP/1.1\r\nHost: localhost\r\n\r\n")
        .unwrap();
    let mut started = String::new();
    stdout.read_line(&mut started).unwrap();
    assert_eq!(started.trim(), "slow-started");

    let mut fast = TcpStream::connect(format!("127.0.0.1:{port}")).unwrap();
    fast.set_read_timeout(Some(Duration::from_millis(500)))
        .unwrap();
    fast.write_all(b"GET /fast HTTP/1.1\r\nHost: localhost\r\n\r\n")
        .unwrap();
    let mut fast_response = String::new();
    fast.read_to_string(&mut fast_response)
        .expect("fast request should finish while slow handler waits");
    assert!(fast_response.starts_with("HTTP/1.1 200"), "{fast_response}");
    assert!(
        fast_response.contains("\"kind\":\"fast\""),
        "{fast_response}"
    );

    let mut slow_response = String::new();
    slow.read_to_string(&mut slow_response).unwrap();
    assert!(slow_response.starts_with("HTTP/1.1 200"), "{slow_response}");
    assert!(
        slow_response.contains("\"kind\":\"slow\""),
        "{slow_response}"
    );

    let mut rest = String::new();
    stdout.read_to_string(&mut rest).unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(
        output.status.success(),
        "status {:?}\n{}\n{rest}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(rest.contains("server done"), "{rest}");
}

#[test]
fn concurrent_http_requests_run_through_jit_and_llvm() {
    let mut jit = Command::new(env!("CARGO_BIN_EXE_soplang"));
    jit.arg("-c").arg(CONCURRENT_PROGRAM);
    drive_concurrent(&mut jit);

    let dir = std::env::temp_dir().join(format!(
        "soplang_concurrent_http_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let source = dir.join("main.sop");
    let binary = dir.join("server");
    std::fs::write(&source, CONCURRENT_PROGRAM).unwrap();
    soplang::build_source(CONCURRENT_PROGRAM, Some(&source), &binary, 2, false).unwrap();
    drive_concurrent(&mut Command::new(&binary));
    std::fs::remove_dir_all(dir).unwrap();
}

const SERVING_PROGRAM: &str = r#"
keen "web" sida web
door server = sug(tcp_dhegeysi_shaqo("127.0.0.1", 0))
door app = cusub web.Web()
door active = 0
door peak = 0
app.hel("GET", "/work", hawl(codsi) {
    active = active + 1
    haddii (active > peak) { peak = active }
    haddii (codsi.weydiimo.phase == "shutdown") {
        qor("draining")
        sug(hurdada(600))
    } kale {
        sug(hurdada(150))
    }
    active = active - 1
    celi web.json({ok: run})
})
app.hel("GET", "/stop", hawl(codsi) {
    tcp_xidh(server)
    celi web.json({stopped: run})
})
qor(server.deked)
door serving = app.adeeg_shaqo(server, 2)
sug(serving)
qor("peak=" + qoraal(peak))
"#;

fn drive_serving(command: &mut Command) {
    let mut child = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut stdout = BufReader::new(child.stdout.take().unwrap());
    let mut port = String::new();
    stdout.read_line(&mut port).unwrap();
    let port = port.trim();
    assert!(!port.is_empty(), "server did not publish a port");

    let connect = |path: &str| {
        let mut stream = TcpStream::connect(format!("127.0.0.1:{port}")).unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(3)))
            .unwrap();
        stream
            .write_all(format!("GET {path} HTTP/1.1\r\nHost: localhost\r\n\r\n").as_bytes())
            .unwrap();
        stream
    };
    let mut work = (0..4).map(|_| connect("/work")).collect::<Vec<_>>();
    for stream in &mut work {
        let mut response = String::new();
        stream.read_to_string(&mut response).unwrap();
        assert!(response.starts_with("HTTP/1.1 200"), "{response}");
        assert!(response.contains("\"ok\":true"), "{response}");
    }

    let mut draining = connect("/work?phase=shutdown");
    let mut started = String::new();
    stdout.read_line(&mut started).unwrap();
    assert_eq!(started.trim(), "draining");
    let mut stopping = connect("/stop");
    stopping
        .set_read_timeout(Some(Duration::from_millis(400)))
        .unwrap();
    let mut stopped = String::new();
    stopping.read_to_string(&mut stopped).unwrap();
    assert!(stopped.contains("\"stopped\":true"), "{stopped}");
    let mut drained = String::new();
    draining.read_to_string(&mut drained).unwrap();
    assert!(drained.contains("\"ok\":true"), "{drained}");

    let mut rest = String::new();
    stdout.read_to_string(&mut rest).unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(
        output.status.success(),
        "status {:?}\n{}\n{rest}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(rest.contains("peak=2"), "{rest}");
}

#[test]
fn bounded_http_serving_drains_active_requests_in_jit_and_llvm() {
    let mut jit = Command::new(env!("CARGO_BIN_EXE_soplang"));
    jit.arg("-c").arg(SERVING_PROGRAM);
    drive_serving(&mut jit);

    let dir = std::env::temp_dir().join(format!(
        "soplang_bounded_http_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let source = dir.join("main.sop");
    let binary = dir.join("server");
    std::fs::write(&source, SERVING_PROGRAM).unwrap();
    soplang::build_source(SERVING_PROGRAM, Some(&source), &binary, 2, false).unwrap();
    drive_serving(&mut Command::new(&binary));
    std::fs::remove_dir_all(dir).unwrap();
}

const STREAMING_REQUEST_PROGRAM: &str = r#"
keen "web" sida web
door app = cusub web.Web()
app.cabbir = 17 * 1024 * 1024
door saved = {jidh: maran}
app.hel_socod("POST", "/upload", hawl(codsi) {
    saved.jidh = codsi.jidh
    door total = 0
    door parts = 0
    intay (run) {
        door chunk = web.jidh_akhris(codsi, 32768)
        haddii (dherer(chunk) == 0) { jooji }
        total = total + dherer(chunk)
        parts = parts + 1
        haddii (parts == 1) { qor("first-chunk") }
    }
    celi web.json({total: total, parts: parts})
})
app.hel("POST", "/buffered", hawl(codsi) { celi web.json({accepted: run}) })
door server = sug(tcp_dhegeysi_shaqo("127.0.0.1", 0))
qor(server.deked)
sug(app.aqbal_shaqo(server))
sug(app.aqbal_shaqo(server))
tcp_xidh(server)
door closed = been
tijaabi { server_jidh_akhris(saved.jidh, 1) } qabo (e) { closed = run }
haddii (!closed) { tuur "body handle escaped" }
qor("stream done")
"#;

fn drive_streaming_request(command: &mut Command) {
    const LENGTH: usize = 16 * 1024 * 1024 + 1;
    let mut child = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut stdout = BufReader::new(child.stdout.take().unwrap());
    let mut port = String::new();
    stdout.read_line(&mut port).unwrap();
    let mut stream = TcpStream::connect(format!("127.0.0.1:{}", port.trim())).unwrap();
    stream
        .set_read_timeout(Some(Duration::from_secs(10)))
        .unwrap();
    stream
        .write_all(
            format!("POST /upload HTTP/1.1\r\nHost: localhost\r\nContent-Length: {LENGTH}\r\n\r\n")
                .as_bytes(),
        )
        .unwrap();
    let chunk = vec![b'x'; 32768];
    stream.write_all(&chunk).unwrap();
    let mut started = String::new();
    stdout.read_line(&mut started).unwrap();
    assert_eq!(started.trim(), "first-chunk");
    let mut remaining = LENGTH - chunk.len();
    while remaining > 0 {
        let count = remaining.min(chunk.len());
        stream.write_all(&chunk[..count]).unwrap();
        remaining -= count;
    }
    let mut response = String::new();
    stream.read_to_string(&mut response).unwrap();
    assert!(response.starts_with("HTTP/1.1 200"), "{response}");
    assert!(
        response.contains(&format!("\"total\":{LENGTH}")),
        "{response}"
    );
    let mut buffered = TcpStream::connect(format!("127.0.0.1:{}", port.trim())).unwrap();
    buffered
        .set_read_timeout(Some(Duration::from_secs(3)))
        .unwrap();
    buffered
        .write_all(
            format!(
                "POST /buffered HTTP/1.1\r\nHost: localhost\r\nContent-Length: {LENGTH}\r\n\r\n"
            )
            .as_bytes(),
        )
        .unwrap();
    let mut rejected = String::new();
    buffered.read_to_string(&mut rejected).unwrap();
    assert!(rejected.starts_with("HTTP/1.1 413"), "{rejected}");
    let mut rest = String::new();
    stdout.read_to_string(&mut rest).unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(
        output.status.success(),
        "status {:?}\n{}\n{rest}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(rest.contains("stream done"), "{rest}");
}

#[test]
fn streaming_request_body_exceeds_buffered_limit_in_jit_and_llvm() {
    let mut jit = Command::new(env!("CARGO_BIN_EXE_soplang"));
    jit.arg("-c").arg(STREAMING_REQUEST_PROGRAM);
    drive_streaming_request(&mut jit);

    let dir = std::env::temp_dir().join(format!(
        "soplang_streaming_request_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let source = dir.join("main.sop");
    let binary = dir.join("server");
    std::fs::write(&source, STREAMING_REQUEST_PROGRAM).unwrap();
    soplang::build_source(STREAMING_REQUEST_PROGRAM, Some(&source), &binary, 2, false).unwrap();
    drive_streaming_request(&mut Command::new(&binary));
    std::fs::remove_dir_all(dir).unwrap();
}

const CHUNKED_REQUEST_PROGRAM: &str = r#"
keen "web" sida web
door app = cusub web.Web()
app.hel_socod("POST", "/stream", hawl(codsi) {
    haddii (codsi.dherer_jidh != maran) { tuur "chunked length should be unknown" }
    door total = 0
    intay (run) {
        door part = web.jidh_akhris(codsi, 2)
        haddii (dherer(part) == 0) { jooji }
        total = total + dherer(part)
        haddii (total == 3) { qor("first-chunk") }
    }
    celi web.json({total: total})
})
app.hel("POST", "/buffered", hawl(codsi) {
    celi web.json({body: codsi.qoraal})
})
door server = sug(tcp_dhegeysi_shaqo("127.0.0.1", 0))
qor(server.deked)
sug(app.aqbal_shaqo(server))
sug(app.aqbal_shaqo(server))
tcp_xidh(server)
qor("chunked done")
"#;

fn drive_chunked_request(command: &mut Command) {
    let mut child = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut stdout = BufReader::new(child.stdout.take().unwrap());
    let mut port = String::new();
    stdout.read_line(&mut port).unwrap();
    let address = format!("127.0.0.1:{}", port.trim());
    let mut stream = TcpStream::connect(&address).unwrap();
    stream
        .set_read_timeout(Some(Duration::from_secs(10)))
        .unwrap();
    stream.write_all(b"POST /stream HTTP/1.1\r\nHost: localhost\r\nTransfer-Encoding: chunked\r\n\r\n3\r\nabc\r\n").unwrap();
    let mut started = String::new();
    stdout.read_line(&mut started).unwrap();
    assert_eq!(started.trim(), "first-chunk");
    stream
        .write_all(b"2\r\nde\r\n0\r\nX-Final: yes\r\n\r\n")
        .unwrap();
    let mut response = String::new();
    stream.read_to_string(&mut response).unwrap();
    assert!(response.starts_with("HTTP/1.1 200"), "{response}");
    assert!(response.contains("\"total\":5"), "{response}");

    let mut buffered = TcpStream::connect(&address).unwrap();
    buffered
        .set_read_timeout(Some(Duration::from_secs(10)))
        .unwrap();
    buffered.write_all(b"POST /buffered HTTP/1.1\r\nHost: localhost\r\nTransfer-Encoding: chunked\r\n\r\n3\r\nabc\r\n2\r\nde\r\n0\r\n\r\n").unwrap();
    let mut response = String::new();
    buffered.read_to_string(&mut response).unwrap();
    assert!(response.starts_with("HTTP/1.1 200"), "{response}");
    assert!(response.contains("\"body\":\"abcde\""), "{response}");
    let mut rest = String::new();
    stdout.read_to_string(&mut rest).unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(
        output.status.success(),
        "status {:?}\n{}\n{rest}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(rest.contains("chunked done"), "{rest}");
}

#[test]
fn chunked_request_bodies_stream_and_buffer_in_jit_and_llvm() {
    let mut jit = Command::new(env!("CARGO_BIN_EXE_soplang"));
    jit.arg("-c").arg(CHUNKED_REQUEST_PROGRAM);
    drive_chunked_request(&mut jit);

    let dir = std::env::temp_dir().join(format!(
        "soplang_chunked_request_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let source = dir.join("main.sop");
    let binary = dir.join("server");
    std::fs::write(&source, CHUNKED_REQUEST_PROGRAM).unwrap();
    soplang::build_source(CHUNKED_REQUEST_PROGRAM, Some(&source), &binary, 2, false).unwrap();
    drive_chunked_request(&mut Command::new(&binary));
    std::fs::remove_dir_all(dir).unwrap();
}

const HTTP_LOAD_PROGRAM: &str = r#"
keen "web" sida web
door server = sug(tcp_dhegeysi_shaqo("127.0.0.1", 0))
door app = cusub web.Web()
app.waqti = 0.45
door active = 0
door peak = 0
door completed = 0
app.hel("GET", "/ok", hawl(codsi) {
    active = active + 1
    haddii (active > peak) { peak = active }
    sug(hurdada(10))
    active = active - 1
    completed = completed + 1
    celi web.json({ok: run})
})
app.hel("GET", "/fail", hawl(codsi) { tuur "intentional failure" })
app.hel("GET", "/stop", hawl(codsi) {
    tcp_xidh(server)
    celi web.json({stopped: run})
})
qor(server.deked)
door serving = app.adeeg_shaqo(server, 4)
sug(serving)
qor("peak=" + qoraal(peak) + " completed=" + qoraal(completed))
"#;

#[cfg(target_os = "linux")]
fn resident_kib(pid: u32) -> Option<usize> {
    std::fs::read_to_string(format!("/proc/{pid}/status"))
        .ok()?
        .lines()
        .find_map(|line| {
            line.strip_prefix("VmRSS:")?
                .split_whitespace()
                .next()?
                .parse()
                .ok()
        })
}

fn drive_http_load(command: &mut Command) {
    let mut child = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut stdout = BufReader::new(child.stdout.take().unwrap());
    let mut port = String::new();
    stdout.read_line(&mut port).unwrap();
    let address = format!("127.0.0.1:{}", port.trim());
    let connect = || {
        let stream = TcpStream::connect(&address).unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(4)))
            .unwrap();
        stream
            .set_write_timeout(Some(Duration::from_secs(4)))
            .unwrap();
        stream
    };

    let mut stalled = Vec::new();
    for _ in 0..4 {
        let mut stream = connect();
        stream.write_all(b"GET /ok HTTP/1.1\r\nHost:").unwrap();
        stalled.push(stream);
    }
    for mut stream in stalled {
        let mut one = [0];
        assert_eq!(
            stream.read(&mut one).unwrap(),
            0,
            "stalled client stayed open"
        );
    }

    #[cfg(target_os = "linux")]
    let mut initial_rss = None;
    for wave in 0..12 {
        #[cfg(not(target_os = "linux"))]
        let _ = wave;
        let mut requests = Vec::new();
        for index in 0..8 {
            let address = address.clone();
            requests.push(thread::spawn(move || {
                let mut stream = TcpStream::connect(address).unwrap();
                stream
                    .set_read_timeout(Some(Duration::from_secs(4)))
                    .unwrap();
                let path = if index < 6 { "/ok" } else { "/fail" };
                stream
                    .write_all(format!("GET {path} HTTP/1.1\r\nHost: localhost\r\n\r\n").as_bytes())
                    .unwrap();
                let mut response = String::new();
                stream.read_to_string(&mut response).unwrap();
                (path, response)
            }));
        }
        let mut dropped = connect();
        dropped
            .write_all(b"GET /ok HTTP/1.1\r\nHost: localhost\r\n\r\n")
            .unwrap();
        drop(dropped);
        for request in requests {
            let (path, response) = request.join().unwrap();
            let expected = if path == "/ok" { 200 } else { 500 };
            assert!(
                response.starts_with(&format!("HTTP/1.1 {expected}")),
                "{response}"
            );
        }
        #[cfg(target_os = "linux")]
        {
            if wave == 1 {
                initial_rss = resident_kib(child.id());
            }
            if wave == 11 {
                if let (Some(start), Some(end)) = (initial_rss, resident_kib(child.id())) {
                    assert!(
                        end <= start + 64 * 1024,
                        "HTTP resident memory grew from {start} to {end} KiB"
                    );
                }
            }
        }
    }
    let mut stop = connect();
    stop.write_all(b"GET /stop HTTP/1.1\r\nHost: localhost\r\n\r\n")
        .unwrap();
    let mut stopped = String::new();
    stop.read_to_string(&mut stopped).unwrap();
    assert!(stopped.starts_with("HTTP/1.1 200"), "{stopped}");
    let deadline = Instant::now() + Duration::from_secs(5);
    while child.try_wait().unwrap().is_none() {
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            panic!("HTTP server did not drain after shutdown");
        }
        thread::sleep(Duration::from_millis(10));
    }
    let mut rest = String::new();
    stdout.read_to_string(&mut rest).unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(
        output.status.success(),
        "{}\n{rest}",
        String::from_utf8_lossy(&output.stderr)
    );
    let line = rest.lines().find(|line| line.starts_with("peak=")).unwrap();
    let mut fields = line.split_whitespace();
    let peak: usize = fields
        .next()
        .unwrap()
        .trim_start_matches("peak=")
        .parse()
        .unwrap();
    let completed: usize = fields
        .next()
        .unwrap()
        .trim_start_matches("completed=")
        .parse()
        .unwrap();
    assert!((1..=4).contains(&peak), "{rest}");
    assert!(completed >= 72, "{rest}");
}

#[test]
fn bounded_http_server_survives_stalled_dropped_and_failing_clients() {
    let mut jit = Command::new(env!("CARGO_BIN_EXE_soplang"));
    jit.arg("-c").arg(HTTP_LOAD_PROGRAM);
    drive_http_load(&mut jit);

    let dir = std::env::temp_dir().join(format!(
        "soplang_http_load_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let source = dir.join("main.sop");
    let binary = dir.join("server");
    std::fs::write(&source, HTTP_LOAD_PROGRAM).unwrap();
    soplang::build_source(HTTP_LOAD_PROGRAM, Some(&source), &binary, 2, false).unwrap();
    drive_http_load(&mut Command::new(&binary));
    std::fs::remove_dir_all(dir).unwrap();
}

const HANDLER_DEADLINE_PROGRAM: &str = r#"
keen "web" sida web
door app = cusub web.Web()
app.waqti = 0.25
app.hel("GET", "/slow", hawl(codsi) {
    tijaabi {
        sug(hurdada(3000))
        qor("too late")
    } dhammee { qor("cleanup") }
    celi web.json({ok: run})
})
app.hel("GET", "/stream", hawl(codsi) {
    hawl next() {
        tijaabi {
            sug(hurdada(3000))
            qor("too late")
        } dhammee { qor("stream cleanup") }
        celi "chunk"
    }
    celi web.socod(next)
})
app.hel("GET", "/fast", hawl(codsi) { celi web.json({ok: run}) })
door server = sug(tcp_dhegeysi_shaqo("127.0.0.1", 0))
qor(server.deked)
tijaabi { sug(app.aqbal_shaqo(server)) } qabo (e) { qor("timed out") }
tijaabi { sug(app.aqbal_shaqo(server)) } qabo (e) { qor("producer timed out") }
sug(app.aqbal_shaqo(server))
tcp_xidh(server)
qor("deadline done")
"#;

fn drive_handler_deadline(command: &mut Command) {
    let mut child = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut stdout = BufReader::new(child.stdout.take().unwrap());
    let mut port = String::new();
    stdout.read_line(&mut port).unwrap();
    let address = format!("127.0.0.1:{}", port.trim());
    let mut slow = TcpStream::connect(&address).unwrap();
    slow.set_read_timeout(Some(Duration::from_secs(3))).unwrap();
    slow.write_all(b"GET /slow HTTP/1.1\r\nHost: localhost\r\n\r\n")
        .unwrap();
    let started = Instant::now();
    let mut ignored = Vec::new();
    slow.read_to_end(&mut ignored).unwrap();
    assert!(
        started.elapsed() < Duration::from_secs(2),
        "handler exceeded its exchange deadline"
    );
    let mut cleanup = String::new();
    stdout.read_line(&mut cleanup).unwrap();
    assert_eq!(cleanup.trim(), "cleanup");
    let mut timeout = String::new();
    stdout.read_line(&mut timeout).unwrap();
    assert_eq!(timeout.trim(), "timed out");
    let mut stream = TcpStream::connect(&address).unwrap();
    stream
        .set_read_timeout(Some(Duration::from_secs(3)))
        .unwrap();
    stream
        .write_all(b"GET /stream HTTP/1.1\r\nHost: localhost\r\n\r\n")
        .unwrap();
    let started = Instant::now();
    let mut partial = Vec::new();
    stream.read_to_end(&mut partial).unwrap();
    assert!(
        started.elapsed() < Duration::from_secs(2),
        "producer exceeded its exchange deadline"
    );
    let mut cleanup = String::new();
    stdout.read_line(&mut cleanup).unwrap();
    assert_eq!(cleanup.trim(), "stream cleanup");
    let mut timeout = String::new();
    stdout.read_line(&mut timeout).unwrap();
    assert_eq!(timeout.trim(), "producer timed out");
    let mut fast = TcpStream::connect(&address).unwrap();
    fast.set_read_timeout(Some(Duration::from_secs(3))).unwrap();
    fast.write_all(b"GET /fast HTTP/1.1\r\nHost: localhost\r\n\r\n")
        .unwrap();
    let mut response = String::new();
    fast.read_to_string(&mut response).unwrap();
    assert!(response.starts_with("HTTP/1.1 200"), "{response}");
    let mut rest = String::new();
    stdout.read_to_string(&mut rest).unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(
        output.status.success(),
        "{}\n{rest}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        rest.contains("deadline done") && !rest.contains("too late"),
        "{rest}"
    );
}

#[test]
fn asynchronous_http_handler_deadline_runs_cleanup_in_jit_and_llvm() {
    let mut jit = Command::new(env!("CARGO_BIN_EXE_soplang"));
    jit.arg("-c").arg(HANDLER_DEADLINE_PROGRAM);
    drive_handler_deadline(&mut jit);

    let dir = std::env::temp_dir().join(format!(
        "soplang_handler_deadline_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let source = dir.join("main.sop");
    let binary = dir.join("server");
    std::fs::write(&source, HANDLER_DEADLINE_PROGRAM).unwrap();
    soplang::build_source(HANDLER_DEADLINE_PROGRAM, Some(&source), &binary, 2, false).unwrap();
    drive_handler_deadline(&mut Command::new(&binary));
    std::fs::remove_dir_all(dir).unwrap();
}
