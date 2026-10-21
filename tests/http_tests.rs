//! Synchronous HTTP client against a local server (3.3 stage).

use std::io::{Read, Write};
use std::net::TcpListener;
use std::process::Command;
use std::thread;
use std::time::Duration;

fn run(source: &str) -> String {
    let exe = env!("CARGO_BIN_EXE_soplang");
    let out = Command::new(exe)
        .arg("-c")
        .arg(source)
        .output()
        .expect("run soplang");
    assert!(
        out.status.success(),
        "program failed:\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).unwrap()
}

fn response(status: &str, content_type: &str, extra: &str, body: &str) -> Vec<u8> {
    format!(
        "HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\n{extra}Content-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    )
    .into_bytes()
}

fn serve<F>(accepts: usize, handler: F) -> String
where
    F: Fn(&str) -> Vec<u8> + Send + 'static,
{
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    thread::spawn(move || {
        for _ in 0..accepts {
            let Ok((mut socket, _)) = listener.accept() else {
                break;
            };
            let _ = socket.set_read_timeout(Some(Duration::from_secs(1)));
            let mut collected = Vec::new();
            let mut buf = [0u8; 2048];
            loop {
                match socket.read(&mut buf) {
                    Ok(0) => break,
                    Ok(n) => {
                        collected.extend_from_slice(&buf[..n]);
                        if let Some(end) = collected.windows(4).position(|mark| mark == b"\r\n\r\n") {
                            let header = String::from_utf8_lossy(&collected[..end]);
                            let length = header
                                .lines()
                                .find_map(|line| line.split_once(':').and_then(|(name, value)| {
                                    name.eq_ignore_ascii_case("content-length").then(|| value.trim().parse::<usize>().ok())
                                }))
                                .flatten()
                                .unwrap_or(0);
                            if collected.len() >= end + 4 + length {
                                break;
                            }
                        }
                    }
                    Err(_) => break,
                }
            }
            let request = String::from_utf8_lossy(&collected).into_owned();
            let _ = socket.write_all(&handler(&request));
        }
    });
    format!("http://{addr}/item")
}

#[test]
fn http_get_post_json_and_status() {
    let url = serve(2, |request| {
        if request.starts_with("POST") {
            assert!(request.contains("\"n\":7") || request.contains("\"n\": 7"));
            assert!(request.to_ascii_lowercase().contains("content-type: application/json"));
            return response("201 Created", "application/json", "", r#"{"n":7}"#);
        }
        assert!(request.to_ascii_lowercase().contains("summad: haa"));
        response("200 OK", "text/plain", "", "Salaan")
    });
    let source = format!(
        r#"
hawl hubi(v, name) {{ haddii (!v) {{ tuur name }} }}
door got = http_codso("GET", "{url}", {{cinwaan: {{Summad: "haa"}}}})
hubi(got.heer == 200 && got.qoraal == "Salaan" && got.json == maran, "get")
door posted = http_codso("POST", "{url}", {{json: {{n: 7}}}})
hubi(posted.heer == 201 && posted.json.n == 7, "post")
qor("http ok")
"#
    );
    assert_eq!(run(&source), "http ok\n");
}

#[test]
fn http_limits_redirects_size_and_timeout() {
    let redirect = serve(1, |_| {
        response("302 Found", "text/plain", "Location: http://127.0.0.1:1/next\r\n", "")
    });
    let large = serve(1, |_| response("200 OK", "text/plain", "", "0123456789abcdef"));
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let slow_addr = listener.local_addr().unwrap();
    thread::spawn(move || {
        if let Ok((socket, _)) = listener.accept() {
            thread::sleep(Duration::from_secs(2));
            drop(socket);
        }
    });
    let slow = format!("http://{slow_addr}/slow");
    let source = format!(
        r#"
hawl hubi(v, name) {{ haddii (!v) {{ tuur name }} }}
door moved = http_codso("GET", "{redirect}", {{leexasho: 0}})
hubi(moved.heer == 302, "redirect")
door too_big = been
tijaabi {{ http_codso("GET", "{large}", {{cabbir: 4}}) }} qabo (e) {{ too_big = run }}
hubi(too_big, "size")
door timed = been
tijaabi {{ http_codso("GET", "{slow}", {{waqti: 0.3}}) }} qabo (e) {{ timed = run }}
hubi(timed, "timeout")
qor("http limits ok")
"#
    );
    assert_eq!(run(&source), "http limits ok\n");
}

#[test]
fn http_connection_and_method_errors_are_catchable() {
    let source = r#"
hawl hubi(v, name) { haddii (!v) { tuur name } }
door failed = been
tijaabi { http_codso("GET", "http://127.0.0.1:1/") } qabo (e) { failed = run }
hubi(failed, "connect")
failed = been
tijaabi { http_codso("TRACE", "http://127.0.0.1:1/") } qabo (e) { failed = run }
hubi(failed, "method")
failed = been
tijaabi { http_codso("GET", "https://127.0.0.1:1/") } qabo (e) { failed = run }
hubi(failed, "tls")
qor("http errors ok")
"#;
    assert_eq!(run(source), "http errors ok\n");
}

#[test]
fn http_get_runs_through_llvm() {
    let url = serve(1, |_| response("200 OK", "text/plain", "", "LLVM"));
    let source = format!(
        r#"
hawl hubi(v, name) {{ haddii (!v) {{ tuur name }} }}
door got = http_codso("GET", "{url}")
hubi(got.heer == 200 && got.qoraal == "LLVM", "llvm")
qor("llvm http ok")
"#
    );
    let stmts = soplang::frontend::parse_source(&source, None).expect("parse");
    let sym = soplang::analyze(&stmts).expect("analyze");
    let hir = soplang::HirLowering::lower(&sym, &stmts);
    let ir = soplang::backend::llvm_ir::emit_llvm_ir(&hir, &sym).unwrap();
    let mut bin = std::env::temp_dir();
    bin.push(format!("soplang_http_llvm_{}", std::process::id()));
    soplang::backend::llvm_ir::link_llvm_binary(&ir, &bin).unwrap();
    let out = Command::new(&bin).output().expect("run");
    let stdout = String::from_utf8(out.stdout).unwrap();
    let stderr = String::from_utf8_lossy(&out.stderr);
    let _ = std::fs::remove_file(&bin);
    assert!(out.status.success(), "{stderr}\n{stdout}");
    assert_eq!(stdout, "llvm http ok\n");
}
