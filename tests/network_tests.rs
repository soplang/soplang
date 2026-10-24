//! DNS lookup and TCP sockets (3.3 stage).

use std::io::{Read, Write};
use std::net::TcpStream;
use std::process::{Command, Stdio};
use std::time::Duration;

fn run(source: &str) -> String {
    let exe = env!("CARGO_BIN_EXE_soplang");
    let out = Command::new(exe).arg("-c").arg(source).output().unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).unwrap()
}

#[test]
fn dns_lookup_accepts_literals_and_local_names() {
    let source = r#"
hawl hubi(v, name) { haddii (!v) { tuur name } }
door ips = dns_raadi("127.0.0.1")
hubi(dherer(ips) == 1 && ips[0] == "127.0.0.1", "literal")
door local = dns_raadi("localhost")
hubi(dherer(local) >= 1, "localhost")
door failed = been
tijaabi { dns_raadi("") } qabo (e) { failed = run }
hubi(failed, "empty")
qor("dns ok")
"#;
    assert_eq!(run(source), "dns ok\n");
}

const TCP_PROGRAM: &str = r#"
hawl hubi(v, name) { haddii (!v) { tuur name } }
door ear = tcp_dhegeysi("127.0.0.1", 0)
qor(ear.deked)
door conn = tcp_aqbal(ear, {waqti: 3})
door data = tcp_akhris(conn, 16)
tcp_qor(conn, "nabad")
tcp_xidh(conn)
tcp_xidh(ear)
door failed = been
tijaabi { tcp_akhris(conn, 4) } qabo (e) { failed = run }
hubi(failed, "closed")
qor(data.qoraal())
"#;

const ASYNC_TCP_PROGRAM: &str = r#"
hawl hubi(v, name) { haddii (!v) { tuur name } }
door ear = sug(tcp_dhegeysi_shaqo("127.0.0.1", 0))
qor(ear.deked)
door conn = sug(tcp_aqbal_shaqo(ear, {waqti: 3}))
door data = sug(tcp_akhris_shaqo(conn, 16))
sug(tcp_qor_shaqo(conn, "nabad"))
tcp_xidh(conn)
tcp_xidh(ear)
door failed = been
tijaabi { tcp_akhris_shaqo(conn, 4) } qabo (e) { failed = run }
hubi(failed, "closed")
qor(data.qoraal())
"#;

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
        if stdout.read(&mut byte).unwrap() == 0 {
            break;
        }
        if byte[0] == b'\n' {
            break;
        }
        port.push(byte[0] as char);
    }
    let port = port.trim();
    assert!(!port.is_empty(), "listener did not publish a port");
    let mut stream = TcpStream::connect(format!("127.0.0.1:{port}")).unwrap();
    stream
        .set_read_timeout(Some(Duration::from_secs(3)))
        .unwrap();
    stream.write_all(b"salaan").unwrap();
    let mut reply = [0u8; 8];
    let read = stream.read(&mut reply).unwrap();
    assert_eq!(&reply[..read], b"nabad");
    let mut rest = String::new();
    stdout.read_to_string(&mut rest).unwrap();
    let output = child.wait_with_output().unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "status {:?}\n{stderr}\n{rest}",
        output.status.code()
    );
    assert!(rest.contains("salaan"), "{rest}");
}

const UDP_PROGRAM: &str = r#"
hawl hubi(v, name) { haddii (!v) { tuur name } }
door a = udp_fur("127.0.0.1", 0, {waqti: 2})
door b = udp_fur("127.0.0.1", 0, {waqti: 2})
udp_qor(a, "127.0.0.1", b.deked, "salaan")
door packet = udp_akhris(b, 32)
hubi(packet.baytyo.qoraal() == "salaan", "payload")
hubi(packet.martigeliye == "127.0.0.1" && packet.deked == a.deked, "source")
udp_xidh(a)
udp_xidh(b)
door failed = been
tijaabi { udp_akhris(b, 8) } qabo (e) { failed = run }
hubi(failed, "closed")
qor("udp ok")
"#;

#[test]
fn udp_sockets_exchange_one_datagram() {
    assert_eq!(run(UDP_PROGRAM), "udp ok\n");
}

#[test]
fn udp_exchange_runs_through_llvm() {
    let stmts = soplang::frontend::parse_source(UDP_PROGRAM, None).unwrap();
    let symbols = soplang::analyze(&stmts).unwrap();
    let hir = soplang::HirLowering::lower(&symbols, &stmts);
    let ir = soplang::backend::llvm_ir::emit_llvm_ir(&hir, &symbols).unwrap();
    let mut path = std::env::temp_dir();
    path.push(format!("soplang_udp_{}", std::process::id()));
    soplang::backend::llvm_ir::link_llvm_binary(&ir, &path).unwrap();
    let out = Command::new(&path).output().unwrap();
    let stdout = String::from_utf8(out.stdout).unwrap();
    let stderr = String::from_utf8_lossy(&out.stderr);
    let _ = std::fs::remove_file(path);
    assert!(out.status.success(), "{stderr}\n{stdout}");
    assert_eq!(stdout, "udp ok\n");
}

#[test]
fn tcp_listener_and_client_exchange_bytes() {
    let mut command = Command::new(env!("CARGO_BIN_EXE_soplang"));
    command.arg("-c").arg(TCP_PROGRAM);
    drive(&mut command);
}

#[test]
fn tcp_exchange_runs_through_llvm() {
    let stmts = soplang::frontend::parse_source(TCP_PROGRAM, None).unwrap();
    let symbols = soplang::analyze(&stmts).unwrap();
    let hir = soplang::HirLowering::lower(&symbols, &stmts);
    let ir = soplang::backend::llvm_ir::emit_llvm_ir(&hir, &symbols).unwrap();
    let mut path = std::env::temp_dir();
    path.push(format!("soplang_tcp_{}", std::process::id()));
    soplang::backend::llvm_ir::link_llvm_binary(&ir, &path).unwrap();
    let mut command = Command::new(&path);
    drive(&mut command);
    let _ = std::fs::remove_file(path);
}

#[test]
fn async_tcp_listener_and_client_exchange_bytes() {
    let mut command = Command::new(env!("CARGO_BIN_EXE_soplang"));
    command.arg("-c").arg(ASYNC_TCP_PROGRAM);
    drive(&mut command);
}

#[test]
fn closing_async_listener_cancels_pending_accept() {
    soplang::run_source(
        r#"
        door ear = sug(tcp_dhegeysi_shaqo("127.0.0.1", 0))
        door pending = tcp_aqbal_shaqo(ear, {waqti: 10})
        tcp_xidh(ear)
        door cancelled = been
        tijaabi { sug(pending) } qabo (e) { cancelled = run }
        hubi(cancelled)
        "#,
        None,
        false,
        false,
        false,
    )
    .unwrap();
}

#[test]
fn async_tcp_client_connects_and_close_cancels_pending_read() {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let server = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        stream.set_read_timeout(Some(Duration::from_secs(2))).unwrap();
        let mut bytes = [0u8; 6];
        stream.read_exact(&mut bytes).unwrap();
        assert_eq!(&bytes, b"salaan");
        std::thread::sleep(Duration::from_millis(100));
    });
    let source = format!(
        r#"
        door conn = sug(tcp_fur_shaqo("127.0.0.1", {port}, {{waqti: 2}}))
        sug(tcp_qor_shaqo(conn, "salaan"))
        door pending = tcp_akhris_shaqo(conn, 8)
        tcp_xidh(conn)
        door cancelled = been
        tijaabi {{ sug(pending) }} qabo (e) {{ cancelled = run }}
        hubi(cancelled)
        "#
    );
    soplang::run_source(&source, None, false, false, false).unwrap();
    server.join().unwrap();
}

#[test]
fn async_tcp_exchange_runs_through_llvm() {
    let stmts = soplang::frontend::parse_source(ASYNC_TCP_PROGRAM, None).unwrap();
    let symbols = soplang::analyze(&stmts).unwrap();
    let hir = soplang::HirLowering::lower(&symbols, &stmts);
    let ir = soplang::backend::llvm_ir::emit_llvm_ir(&hir, &symbols).unwrap();
    let mut path = std::env::temp_dir();
    path.push(format!("soplang_tcp_async_{}", std::process::id()));
    soplang::backend::llvm_ir::link_llvm_binary(&ir, &path).unwrap();
    let mut command = Command::new(&path);
    drive(&mut command);
    let _ = std::fs::remove_file(path);
}
