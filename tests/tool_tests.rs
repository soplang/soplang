//! Formatter, checker, test runner, docs, debugger, profiler, and LSP.

use std::fs;
use std::io::Write;
use std::process::{Command, Stdio};

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

fn soplang(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_soplang"))
        .args(args)
        .output()
        .unwrap()
}

#[test]
fn fmt_is_stable_and_check_accepts_it() {
    let dir = temp_dir("fmt");
    let file = dir.join("main.sop");
    fs::write(&file, "door n = 1 + 2\nqor(n)\n").unwrap();
    let first = soplang(&["fmt", file.to_str().unwrap()]);
    assert!(
        first.status.success(),
        "{}",
        String::from_utf8_lossy(&first.stderr)
    );
    let text = String::from_utf8(first.stdout).unwrap();
    fs::write(&file, &text).unwrap();
    let second = soplang(&["fmt", file.to_str().unwrap()]);
    assert_eq!(String::from_utf8(second.stdout).unwrap(), text);
    let checked = soplang(&["check", file.to_str().unwrap()]);
    assert!(
        checked.status.success(),
        "{}",
        String::from_utf8_lossy(&checked.stderr)
    );
}

#[test]
fn tests_filter_expected_errors_cleanup_and_coverage() {
    let dir = temp_dir("tests");
    let file = dir.join("main.sop");
    fs::write(
        &file,
        r#"
door n = 1
diyaar { n = n + 1 }
nadiifi { n = 0 }
tijaabo "kordhi" { hubi(n > 1) }
tijaabo "khalad" qalad { hubi(been) }
"#,
    )
    .unwrap();
    let all = soplang(&["test", file.to_str().unwrap(), "--dabool"]);
    let stdout = String::from_utf8_lossy(&all.stdout);
    assert!(
        all.status.success(),
        "{stdout} {}",
        String::from_utf8_lossy(&all.stderr)
    );
    assert!(stdout.contains("waa: kordhi"));
    assert!(stdout.contains("waa: khalad"));
    assert!(stdout.contains("2 waa, 0 fashil"));
    assert!(stdout.contains("dabool:"));

    let one = soplang(&["test", file.to_str().unwrap(), "--shaandhe", "kordhi"]);
    let stdout = String::from_utf8_lossy(&one.stdout);
    assert!(one.status.success(), "{stdout}");
    assert!(stdout.contains("waa: kordhi"));
    assert!(!stdout.contains("waa: khalad"));

    let parallel = soplang(&["test", file.to_str().unwrap(), "--wada"]);
    assert!(
        parallel.status.success(),
        "{}",
        String::from_utf8_lossy(&parallel.stderr)
    );
}

#[test]
fn doc_debug_profile_and_project_run() {
    let dir = temp_dir("project");
    fs::write(
        dir.join("soplang.toml"),
        "magac = \"tijaabo\"\nnusxa = \"0.1.0\"\nsoplang = \"3.1.0\"\n",
    )
    .unwrap();
    let main = dir.join("main.sop");
    fs::write(
        &main,
        "hawl salaan(magac) { jebi() celi magac }\nqor(salaan(\"Amina\"))\n",
    )
    .unwrap();
    let docs = soplang(&["doc", main.to_str().unwrap()]);
    let text = String::from_utf8_lossy(&docs.stdout);
    assert!(
        docs.status.success(),
        "{}",
        String::from_utf8_lossy(&docs.stderr)
    );
    assert!(text.contains("hawl salaan"));

    let mut child = Command::new(env!("CARGO_BIN_EXE_soplang"))
        .arg("debug")
        .arg(&main)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.as_mut().unwrap().write_all(b"soco\n").unwrap();
    let output = child.wait_with_output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        output.status.success(),
        "{stdout} {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(stdout.contains("Amina"));

    let profile = soplang(&["profile", main.to_str().unwrap()]);
    let stdout = String::from_utf8_lossy(&profile.stdout);
    assert!(
        profile.status.success(),
        "{stdout} {}",
        String::from_utf8_lossy(&profile.stderr)
    );
    assert!(stdout.contains("waqti"));

    let run = Command::new(env!("CARGO_BIN_EXE_soplang"))
        .arg("run")
        .arg(&dir)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut run = run;
    run.stdin.as_mut().unwrap().write_all(b"soco\n").unwrap();
    let output = run.wait_with_output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn lsp_answers_definition_hover_and_formatting() {
    let mut server = soplang::cli::lsp::Server::default();
    let opened = server.handle(
        r#"{"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"textDocument":{"uri":"file:///tmp/a.sop","languageId":"soplang","version":1,"text":"hawl salaan() { celi 1 }\n"}}}"#,
    );
    assert_eq!(opened[0]["method"], "textDocument/publishDiagnostics");

    let hover = server.handle(
        r#"{"jsonrpc":"2.0","id":2,"method":"textDocument/hover","params":{"textDocument":{"uri":"file:///tmp/a.sop"},"position":{"line":0,"character":5}}}"#,
    );
    let value = hover[0]["result"]["contents"]["value"]
        .as_str()
        .unwrap_or("");
    assert!(value.contains("salaan"), "{hover:?}");

    let definition = server.handle(
        r#"{"jsonrpc":"2.0","id":3,"method":"textDocument/definition","params":{"textDocument":{"uri":"file:///tmp/a.sop"},"position":{"line":0,"character":5}}}"#,
    );
    assert!(definition[0]["result"]["uri"]
        .as_str()
        .unwrap_or("")
        .contains("a.sop"));

    let formatted = server.handle(
        r#"{"jsonrpc":"2.0","id":4,"method":"textDocument/formatting","params":{"textDocument":{"uri":"file:///tmp/a.sop"}}}"#,
    );
    assert!(formatted[0]["result"][0]["newText"]
        .as_str()
        .unwrap_or("")
        .contains("hawl salaan"));

    let tokens = server.handle(
        r#"{"jsonrpc":"2.0","id":5,"method":"textDocument/semanticTokens/full","params":{"textDocument":{"uri":"file:///tmp/a.sop"}}}"#,
    );
    assert!(tokens[0]["result"]["data"].as_array().unwrap().len() > 4);

    let action = server.handle(
        r#"{"jsonrpc":"2.0","id":6,"method":"textDocument/codeAction","params":{"textDocument":{"uri":"file:///tmp/a.sop"},"range":{"start":{"line":0,"character":0},"end":{"line":0,"character":1}},"context":{"diagnostics":[]}}}"#,
    );
    assert_eq!(action[0]["result"][0]["title"], "Hagaaji qaabka");

    let renamed = server.handle(
        r#"{"jsonrpc":"2.0","id":7,"method":"textDocument/rename","params":{"textDocument":{"uri":"file:///tmp/a.sop"},"position":{"line":0,"character":5},"newName":"hello"}}"#,
    );
    let edits = &renamed[0]["result"]["changes"]["file:///tmp/a.sop"];
    assert!(
        edits.as_array().is_some_and(|edits| !edits.is_empty()),
        "{renamed:?}"
    );
}

#[test]
fn schema_check_rejects_two_keys() {
    let dir = temp_dir("schema");
    let file = dir.join("schema.sop");
    fs::write(&file, "jadwal qof {\n  a abn furaha\n  b abn furaha\n}\n").unwrap();
    let output = soplang(&["check", file.to_str().unwrap()]);
    assert!(!output.status.success());
}
