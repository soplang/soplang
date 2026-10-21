//! WebSocket client against a local server (3.3 stage).

use std::net::TcpListener;
use std::process::Command;
use std::thread;
use tungstenite::{accept, Message};

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
