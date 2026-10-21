//! Bundled web framework: routes, middleware, and response helpers.

use std::fs;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::Duration;

fn temp_dir(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "soplang_{name}_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn exchange(port: &str, request: &str) -> (u16, String) {
    let mut stream = TcpStream::connect(format!("127.0.0.1:{port}")).expect("connect");
    stream
        .set_read_timeout(Some(Duration::from_secs(3)))
        .unwrap();
    stream.write_all(request.as_bytes()).unwrap();
    stream.flush().unwrap();
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
    (status, text)
}

fn program(root: &str) -> String {
    format!(
        r#"
keen "web" sida web
door app = cusub web.Web()
app.dhexdhexa(hawl(codsi, xiga) {{
    door jawaab = xiga(codsi)
    app.diiwaan.geli(codsi.jid)
    celi jawaab
}})
app.dhexdhexa(web.cors("*"))
app.dhexdhexa(hawl(codsi, xiga) {{
    codsi.calan = "a"
    celi xiga(codsi)
}})
app.dhexdhexa(hawl(codsi, xiga) {{
    codsi.calan = codsi.calan + "b"
    celi xiga(codsi)
}})
app.hel("GET", "/ping", hawl(codsi) {{ celi web.json({{ calan: codsi.calan, q: codsi.weydiimo.q }}) }})
app.koox("/api", hawl(a) {{
    a.hel("POST", "/foom", hawl(codsi) {{
        door xog = web.foom(codsi)
        celi web.json({{ magac: xog.magac }})
    }})
}})
app.hel("GET", "/html", hawl(codsi) {{ celi web.html("<p>salaan</p>") }})
app.hel("GET", "/leex", hawl(codsi) {{ celi web.leex("/ping") }})
app.hel("GET", "/fayl", hawl(codsi) {{ celi web.fayl("{root}", "note.txt") }})
app.hel("GET", "/bax", hawl(codsi) {{ celi web.fayl("{root}", "../secret.txt") }})
app.hel("GET", "/gal", hawl(codsi) {{
    door fadhi = app.fur_kalfadhi(codsi)
    fadhi.xog.magac = "Amina"
    celi {{ json: {{ id: fadhi.id }}, cinwaan: {{ "set-cookie": web.kuki_qoraal(fadhi.id) }} }}
}})
app.hel("GET", "/magac", hawl(codsi) {{
    door fadhi = app.fur_kalfadhi(codsi)
    celi web.json({{ magac: fadhi.xog.magac }})
}})
app.ilaali(hawl(codsi) {{ celi web.kuki(codsi, "user") }})
app.hel("GET", "/sir", hawl(codsi) {{ celi web.json({{ user: codsi.isticmaale }}) }})
door server = server_fur("127.0.0.1", 0)
qor(server.deked)
kuceli (i ku taxane(12)) {{ app.aqbal(server) }}
server_xidh(server)
qor(dherer(app.diiwaan))
qor("web done")
"#
    )
}

fn serve(dir: &Path, source: &str) -> (std::process::Child, String, std::process::ChildStdout) {
    let main = dir.join("main.sop");
    fs::write(&main, source).unwrap();
    let mut child = Command::new(env!("CARGO_BIN_EXE_soplang"))
        .arg(&main)
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
            let output = child.wait_with_output().unwrap();
            panic!(
                "server exited before the port\n{}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
        if byte[0] == b'\n' {
            break;
        }
        port.push(byte[0] as char);
    }
    (child, port.trim().to_string(), stdout)
}

#[test]
fn the_web_framework_routes_middleware_and_helpers() {
    let dir = temp_dir("web");
    let public = dir.join("public");
    fs::create_dir_all(&public).unwrap();
    fs::write(public.join("note.txt"), "salaan").unwrap();
    fs::write(dir.join("secret.txt"), "sir").unwrap();
    let root = public.display().to_string().replace('\\', "/");
    let (child, port, mut stdout) = serve(&dir, &program(&root));

    let (status, body) = exchange(
        &port,
        "GET /ping?q=1 HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
    );
    assert_eq!(status, 200, "{body}");
    assert!(
        body.contains("\"calan\":\"ab\"") && body.contains("\"q\":\"1\""),
        "{body}"
    );
    assert!(
        body.to_ascii_lowercase()
            .contains("access-control-allow-origin: *"),
        "{body}"
    );

    let form = "magac=Amina";
    let (status, body) = exchange(
        &port,
        &format!(
            "POST /api/foom HTTP/1.1\r\nHost: localhost\r\nContent-Type: application/x-www-form-urlencoded\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{form}",
            form.len()
        ),
    );
    assert_eq!(status, 200, "{body}");
    assert!(body.contains("Amina"), "{body}");

    let (status, body) = exchange(
        &port,
        "GET /html HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
    );
    assert_eq!(status, 200, "{body}");
    assert!(body.contains("<p>salaan</p>"), "{body}");
    assert!(body.to_ascii_lowercase().contains("text/html"), "{body}");

    let (status, body) = exchange(
        &port,
        "GET /leex HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
    );
    assert_eq!(status, 302, "{body}");
    assert!(
        body.to_ascii_lowercase().contains("location: /ping"),
        "{body}"
    );

    let (status, body) = exchange(
        &port,
        "GET /gal HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
    );
    assert_eq!(status, 200, "{body}");
    let cookie = body
        .lines()
        .find(|line| line.to_ascii_lowercase().starts_with("set-cookie:"))
        .expect("set-cookie")
        .split_once(':')
        .unwrap()
        .1
        .trim()
        .split(';')
        .next()
        .unwrap()
        .trim()
        .to_string();
    assert!(cookie.starts_with("kalfadhi="), "{cookie}");

    let (status, body) = exchange(
        &port,
        &format!("GET /magac HTTP/1.1\r\nHost: localhost\r\nCookie: {cookie}\r\nConnection: close\r\n\r\n"),
    );
    assert_eq!(status, 200, "{body}");
    assert!(body.contains("Amina"), "{body}");

    let (status, body) = exchange(
        &port,
        "GET /sir HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
    );
    assert_eq!(status, 401, "{body}");

    let (status, body) = exchange(
        &port,
        "GET /sir HTTP/1.1\r\nHost: localhost\r\nCookie: user=Amina\r\nConnection: close\r\n\r\n",
    );
    assert_eq!(status, 200, "{body}");
    assert!(body.contains("Amina"), "{body}");

    let (status, body) = exchange(
        &port,
        "GET /fayl HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
    );
    assert_eq!(status, 200, "{body}");
    assert!(body.contains("salaan"), "{body}");

    let (status, body) = exchange(
        &port,
        "GET /bax HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
    );
    assert_eq!(status, 400, "{body}");

    let (status, body) = exchange(
        &port,
        "OPTIONS /ping HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
    );
    assert_eq!(status, 204, "{body}");
    assert!(
        body.to_ascii_lowercase()
            .contains("access-control-allow-methods"),
        "{body}"
    );

    let (status, _) = exchange(
        &port,
        "GET /majoogto HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
    );
    assert_eq!(status, 404);

    let mut rest = String::new();
    stdout.read_to_string(&mut rest).unwrap();
    let output = child.wait_with_output().unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "{stderr}\n{rest}");
    assert!(rest.contains("11"), "{rest}");
    assert!(rest.contains("web done"), "{rest}");
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn the_web_framework_builds_with_llvm() {
    let dir = temp_dir("web_llvm");
    fs::write(
        dir.join("main.sop"),
        "keen \"web\" sida web\ndoor app = cusub web.Web()\napp.hel(\"GET\", \"/ping\", hawl(c) { celi web.json({n: 1}) })\ndoor server = server_fur(\"127.0.0.1\", 0)\nqor(server.deked)\napp.aqbal(server)\nserver_xidh(server)\n",
    )
    .unwrap();
    let binary = dir.join("web-bin");
    let built = Command::new(env!("CARGO_BIN_EXE_soplang"))
        .arg("--build")
        .arg(dir.join("main.sop"))
        .arg("-o")
        .arg(&binary)
        .arg("-q")
        .output()
        .unwrap();
    assert!(
        built.status.success(),
        "{}",
        String::from_utf8_lossy(&built.stderr)
    );
    let mut child = Command::new(&binary)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut stdout = child.stdout.take().unwrap();
    let mut port = String::new();
    let mut byte = [0u8; 1];
    loop {
        if stdout.read(&mut byte).unwrap() == 0 {
            let output = child.wait_with_output().unwrap();
            panic!("{}", String::from_utf8_lossy(&output.stderr));
        }
        if byte[0] == b'\n' {
            break;
        }
        port.push(byte[0] as char);
    }
    let (status, body) = exchange(
        port.trim(),
        "GET /ping HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
    );
    assert_eq!(status, 200, "{body}");
    assert!(body.contains("\"n\":1"), "{body}");
    let _ = child.wait();
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn templates_escape_html_and_multipart_uploads_stay_bounded() {
    let dir = temp_dir("web_more");
    let source = r#"
keen "web" sida web
door app = cusub web.Web()
app.hel("GET", "/habee", hawl(codsi) { celi web.habee("<p>{{ magac }}</p>{{!html}}", { magac: "<Amina>", html: "<b>ok</b>" }) })
app.hel("POST", "/qayb", hawl(codsi) {
    door qaybaha = web.qayb(codsi)
    celi web.json({ magac: qaybaha[0].qoraal })
})
door server = server_fur("127.0.0.1", 0)
qor(server.deked)
kuceli (i ku taxane(2)) { app.aqbal(server) }
server_xidh(server)
"#;
    let (child, port, mut stdout) = serve(&dir, source);
    let (status, body) = exchange(
        &port,
        "GET /habee HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
    );
    assert_eq!(status, 200, "{body}");
    assert!(body.contains("&lt;Amina&gt;"), "{body}");
    assert!(body.contains("<b>ok</b>"), "{body}");
    let form = "\
--Aa\r\n\
Content-Disposition: form-data; name=\"magac\"\r\n\
\r\n\
Amina\r\n\
--Aa--\r\n";
    let request = format!(
        "POST /qayb HTTP/1.1\r\nHost: localhost\r\nContent-Type: multipart/form-data; boundary=Aa\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{form}",
        form.len()
    );
    let (status, body) = exchange(&port, &request);
    assert_eq!(status, 200, "{body}");
    assert!(body.contains("Amina"), "{body}");
    let mut rest = String::new();
    stdout.read_to_string(&mut rest).unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(
        output.status.success(),
        "{}\n{rest}",
        String::from_utf8_lossy(&output.stderr)
    );
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn tls_serves_one_request() {
    let dir = temp_dir("web_tls");
    let cert = dir.join("cert.pem");
    let key = dir.join("key.pem");
    let made = Command::new("openssl")
        .args([
            "req",
            "-x509",
            "-newkey",
            "rsa:2048",
            "-keyout",
            key.to_str().unwrap(),
            "-out",
            cert.to_str().unwrap(),
            "-days",
            "1",
            "-nodes",
            "-subj",
            "/CN=localhost",
        ])
        .output()
        .unwrap();
    assert!(
        made.status.success(),
        "{}",
        String::from_utf8_lossy(&made.stderr)
    );
    let source = format!(
        "keen \"web\" sida web\ndoor app = cusub web.Web()\napp.shahaado(\"{}\", \"{}\")\napp.hel(\"GET\", \"/ping\", hawl(codsi) {{ celi web.json({{ n: 1 }}) }})\ndoor server = app.fur(\"127.0.0.1\", 0)\nqor(server.deked)\napp.aqbal(server)\nserver_xidh(server)\n",
        cert.display().to_string().replace('\\', "/"),
        key.display().to_string().replace('\\', "/")
    );
    let (child, port, mut stdout) = serve(&dir, &source);
    let mut client = Command::new("openssl")
        .args([
            "s_client",
            "-connect",
            &format!("127.0.0.1:{port}"),
            "-servername",
            "localhost",
            "-quiet",
            "-CAfile",
            cert.to_str().unwrap(),
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    client
        .stdin
        .as_mut()
        .unwrap()
        .write_all(b"GET /ping HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n")
        .unwrap();
    let output = client.wait_with_output().unwrap();
    let body = String::from_utf8_lossy(&output.stdout);
    assert!(
        body.contains("\"n\":1"),
        "{body}\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let mut rest = String::new();
    stdout.read_to_string(&mut rest).unwrap();
    let server = child.wait_with_output().unwrap();
    assert!(
        server.status.success(),
        "{}\n{rest}",
        String::from_utf8_lossy(&server.stderr)
    );
    let _ = fs::remove_dir_all(dir);
}
