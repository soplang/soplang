//! JSON HTTP server (3.3 stage).

use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::process::{Command, Stdio};
use std::time::Duration;

const PROGRAM: &str = r#"
hawl hello(c) { celi {json: {id: c.qayb.id, x: c.weydiimo.x}} }
hawl create(c) { celi {heer: 201, json: c.json} }
hawl boom(c) { tuur "xumo" }
hawl injection(c) { celi {cinwaan: {"X-Test": e"ok\r\nX-Evil: yes"}} }
hawl go(c) { celi {heer: 302, cinwaan: {Location: "/next"}} }
door s = server_fur("127.0.0.1", 0)
qor(s.deked)
door routes = [["GET", "/users/:id", hello], ["POST", "/items", create], ["GET", "/fail", boom], ["GET", "/go", go], ["GET", "/injection", injection]]
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
