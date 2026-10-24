//! WebSocket client and server integration tests.

use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;
use tungstenite::{accept, client, Message};

fn serve() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    thread::spawn(move || {
        let (stream, _) = listener.accept().unwrap();
        let mut socket = accept(stream).unwrap();
        match socket.read().unwrap() {
            Message::Text(text) => assert_eq!(text.as_str(), "salaan"),
            other => panic!("expected text, got {other:?}"),
        }
        socket.send(Message::text("nabad")).unwrap();
        match socket.read().unwrap() {
            Message::Binary(bytes) => assert_eq!(&bytes[..], &[9, 8]),
            other => panic!("expected bytes, got {other:?}"),
        }
        socket.send(Message::binary(vec![1, 2, 3])).unwrap();
        let _ = socket.read();
    });
    port
}

fn program(port: u16) -> String {
    format!(
        r#"
hawl hubi(v, name) {{ haddii (!v) {{ tuur name }} }}
door socket = ws_fur("ws://127.0.0.1:{port}", {{waqti: 3}})
ws_qor(socket, "salaan")
hubi(ws_akhris(socket) == "nabad", "text")
ws_qor(socket, baytyo([9, 8]))
door raw = ws_akhris(socket)
hubi(nooc(raw) == "baytyo" && dherer(raw) == 3 && raw[0] == 1 && raw[2] == 3, "binary")
ws_xidh(socket)
door failed = been
tijaabi {{ ws_akhris(socket) }} qabo (e) {{ failed = run }}
hubi(failed, "closed")
qor("ws ok")
"#
    )
}

fn assert_ok(output: std::process::Output) {
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "{stderr}\n{stdout}");
    assert_eq!(stdout, "ws ok\n");
}

#[test]
fn websocket_exchanges_text_and_bytes() {
    let port = serve();
    let output = Command::new(env!("CARGO_BIN_EXE_soplang"))
        .arg("-c")
        .arg(program(port))
        .output()
        .unwrap();
    assert_ok(output);
}

#[test]
fn websocket_exchange_runs_through_llvm() {
    let port = serve();
    let source = program(port);
    let stmts = soplang::frontend::parse_source(&source, None).unwrap();
    let symbols = soplang::analyze(&stmts).unwrap();
    let hir = soplang::HirLowering::lower(&symbols, &stmts);
    let ir = soplang::backend::llvm_ir::emit_llvm_ir(&hir, &symbols).unwrap();
    let mut path = std::env::temp_dir();
    path.push(format!("soplang_ws_{port}_{}", std::process::id()));
    soplang::backend::llvm_ir::link_llvm_binary(&ir, &path).unwrap();
    let output = Command::new(&path).output().unwrap();
    let _ = std::fs::remove_file(&path);
    assert_ok(output);
}

#[test]
fn websocket_bad_urls_are_catchable() {
    let source = r#"
hawl hubi(v, name) { haddii (!v) { tuur name } }
door failed = been
tijaabi { ws_fur("http://127.0.0.1:1/") } qabo (e) { failed = run }
hubi(failed, "scheme")
failed = been
tijaabi { ws_fur("wss://127.0.0.1:1/", {waqti: 1}) } qabo (e) { failed = run }
hubi(failed, "tls")
qor("ws errors ok")
"#;
    let output = Command::new(env!("CARGO_BIN_EXE_soplang"))
        .arg("-c")
        .arg(source)
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "{stderr}\n{stdout}");
    assert_eq!(stdout, "ws errors ok\n");
}

const SERVER_PROGRAM: &str = r#"
hawl echo(socket) {
    door first = ws_akhris(socket)
    ws_qor(socket, first)
    door second = ws_akhris(socket)
    ws_qor(socket, second)
}
door listener = sug(tcp_dhegeysi_shaqo("127.0.0.1", 0))
qor(listener.deked)
sug(ws_aqbal_shaqo(listener, echo, {waqti: 3}))
tcp_xidh(listener)
qor("ws server done")
"#;

fn drive_server(command: &mut Command) {
    let mut child = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut stdout = BufReader::new(child.stdout.take().unwrap());
    let mut port = String::new();
    stdout.read_line(&mut port).unwrap();
    let port = port.trim();
    assert!(!port.is_empty(), "WebSocket server did not publish a port");
    let stream = TcpStream::connect(format!("127.0.0.1:{port}")).unwrap();
    stream
        .set_read_timeout(Some(Duration::from_secs(3)))
        .unwrap();
    stream
        .set_write_timeout(Some(Duration::from_secs(3)))
        .unwrap();
    let (mut socket, _) = client(format!("ws://127.0.0.1:{port}/"), stream).unwrap();
    socket.send(Message::text("salaan")).unwrap();
    assert_eq!(socket.read().unwrap().to_text().unwrap(), "salaan");
    socket.send(Message::binary(vec![9, 8, 7])).unwrap();
    assert_eq!(socket.read().unwrap().into_data(), vec![9, 8, 7]);
    let _ = socket.close(None);
    let mut rest = String::new();
    stdout.read_to_string(&mut rest).unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(
        output.status.success(),
        "status {:?}\n{}\n{rest}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(rest.contains("ws server done"), "{rest}");
}

#[test]
fn websocket_server_exchanges_text_and_bytes_in_jit_and_llvm() {
    let mut jit = Command::new(env!("CARGO_BIN_EXE_soplang"));
    jit.arg("-c").arg(SERVER_PROGRAM);
    drive_server(&mut jit);

    let stmts = soplang::frontend::parse_source(SERVER_PROGRAM, None).unwrap();
    let symbols = soplang::analyze(&stmts).unwrap();
    let hir = soplang::HirLowering::lower(&symbols, &stmts);
    let ir = soplang::backend::llvm_ir::emit_llvm_ir(&hir, &symbols).unwrap();
    let path = std::env::temp_dir().join(format!(
        "soplang_ws_server_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    soplang::backend::llvm_ir::link_llvm_binary(&ir, &path).unwrap();
    drive_server(&mut Command::new(&path));
    std::fs::remove_file(path).unwrap();
}

const SERVING_PROGRAM: &str = r#"
door listener = sug(tcp_dhegeysi_shaqo("127.0.0.1", 0))
door active = 0
door peak = 0
hawl handle(socket) {
    active = active + 1
    haddii (active > peak) { peak = active }
    door message = ws_akhris(socket)
    haddii (message == "stop") {
        tcp_xidh(listener)
        ws_qor(socket, "stopped")
    } kale {
        haddii (message == "drain") {
            qor("draining")
            sug(hurdada(600))
        } kale {
            sug(hurdada(150))
        }
        ws_qor(socket, message)
    }
    active = active - 1
}
qor(listener.deked)
sug(ws_adeeg_shaqo(listener, handle, {shaqaale: 2, waqti: 3}))
qor("peak=" + qoraal(peak))
"#;

fn drive_bounded_server(command: &mut Command) {
    let mut child = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut stdout = BufReader::new(child.stdout.take().unwrap());
    let mut port = String::new();
    stdout.read_line(&mut port).unwrap();
    let port = port.trim();
    assert!(!port.is_empty(), "WebSocket server did not publish a port");
    let connect = || {
        let stream = TcpStream::connect(format!("127.0.0.1:{port}")).unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(3)))
            .unwrap();
        stream
            .set_write_timeout(Some(Duration::from_secs(3)))
            .unwrap();
        client(format!("ws://127.0.0.1:{port}/"), stream).unwrap().0
    };
    let mut malformed = TcpStream::connect(format!("127.0.0.1:{port}")).unwrap();
    malformed
        .write_all(b"GET / HTTP/1.1\r\nHost: localhost\r\n\r\n")
        .unwrap();
    drop(malformed);
    let mut clients = Vec::new();
    for _ in 0..4 {
        let mut socket = connect();
        socket.send(Message::text("wait")).unwrap();
        clients.push(socket);
    }
    for socket in &mut clients {
        assert_eq!(socket.read().unwrap().to_text().unwrap(), "wait");
        let _ = socket.close(None);
    }

    let mut draining = connect();
    draining.send(Message::text("drain")).unwrap();
    let mut started = String::new();
    stdout.read_line(&mut started).unwrap();
    assert_eq!(started.trim(), "draining");
    let mut stopping = connect();
    stopping.send(Message::text("stop")).unwrap();
    assert_eq!(stopping.read().unwrap().to_text().unwrap(), "stopped");
    assert_eq!(draining.read().unwrap().to_text().unwrap(), "drain");
    let _ = stopping.close(None);
    let _ = draining.close(None);

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
fn bounded_websocket_server_drains_peers_in_jit_and_llvm() {
    let mut jit = Command::new(env!("CARGO_BIN_EXE_soplang"));
    jit.arg("-c").arg(SERVING_PROGRAM);
    drive_bounded_server(&mut jit);

    let stmts = soplang::frontend::parse_source(SERVING_PROGRAM, None).unwrap();
    let symbols = soplang::analyze(&stmts).unwrap();
    let hir = soplang::HirLowering::lower(&symbols, &stmts);
    let ir = soplang::backend::llvm_ir::emit_llvm_ir(&hir, &symbols).unwrap();
    let path = std::env::temp_dir().join(format!(
        "soplang_ws_bounded_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    soplang::backend::llvm_ir::link_llvm_binary(&ir, &path).unwrap();
    drive_bounded_server(&mut Command::new(&path));
    std::fs::remove_file(path).unwrap();
}
