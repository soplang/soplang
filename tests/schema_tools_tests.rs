//! Schema diagnostics and editor services use the schema grammar and locations.
use serde_json::{json, Value};
use soplang::cli::lsp::Server;
use std::fs;
use std::process::Command;

fn open(server: &mut Server, uri: &str, text: &str) -> Vec<Value> {
    server.handle(&json!({"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"textDocument":{"uri":uri,"languageId":"soplang-schema","version":1,"text":text}}}).to_string())
}
fn request(server: &mut Server, uri: &str, method: &str, line: usize, character: usize) -> Value {
    server.handle(&json!({"jsonrpc":"2.0","id":1,"method":method,"params":{"textDocument":{"uri":uri},"position":{"line":line,"character":character}}}).to_string())[0]["result"].clone()
}
#[test]
fn schema_errors_preserve_tokens_lines_columns_and_codes() {
    let path = std::path::Path::new("/tmp/schema.sop");
    for (source, line, col, end) in [
        (
            "# 😀\njadwal qof {\n id abn furaha\n magac integer\n}\n",
            4,
            8,
            15,
        ),
        ("jadwal qof {\n id abn furaha\n id qoraal\n}\n", 3, 2, 4),
        (
            "jadwal qof {\n id abn furaha\n other abn furaha\n}\n",
            3,
            12,
            18,
        ),
        ("jadwal qof {\n id abn? furaha\n}\n", 2, 10, 16),
        ("jadwal qof {\n id abn furaha furaha\n}\n", 2, 16, 22),
    ] {
        let error = soplang::data::parse_schema_file(source, path).unwrap_err();
        assert_eq!(error.location(), (line, col), "{}", error.message());
        assert_eq!(error.meta().end_col, Some(end));
        assert_eq!(error.code(), Some("E059"));
        assert_eq!(error.meta().file.as_deref(), Some(path));
        assert_eq!(
            error.meta().source_line.as_deref(),
            source.lines().nth(line - 1)
        );
        let mut server = Server::default();
        let notices = open(&mut server, "untitled:schema.sop", source);
        let diagnostic = &notices[0]["params"]["diagnostics"][0];
        assert_eq!(diagnostic["code"], "E059");
        assert_eq!(
            diagnostic["range"]["start"],
            json!({"line":line-1,"character":col-1})
        );
        assert_eq!(
            diagnostic["range"]["end"],
            json!({"line":line-1,"character":end-1})
        );
    }
    assert!(soplang::data::parse_schema("jadwal qof {{{\n id abn furaha\n}\n").is_err());
    assert!(soplang::data::parse_schema("jadwal\tqof{\n id\tabn\tfuraha\n}\n").is_ok());
}
#[test]
fn schema_completion_hover_formatting_and_comment_syntax_are_distinct_from_code() {
    let mut server = Server::default();
    let uri = "untitled:Schema-1";
    let source = "# People\njadwal qof{\nid abn furaha\nmagac qoraal? # optional\n}\n";
    let notices = open(&mut server, uri, source);
    assert!(
        notices[0]["params"]["diagnostics"]
            .as_array()
            .unwrap()
            .is_empty(),
        "{notices:?}"
    );
    let completion = request(&mut server, uri, "textDocument/completion", 3, 6);
    assert!(completion
        .as_array()
        .unwrap()
        .iter()
        .any(|item| item["label"] == "qoraal"));
    let hover = request(&mut server, uri, "textDocument/hover", 3, 9);
    assert!(
        hover["contents"]["value"]
            .as_str()
            .unwrap()
            .contains("Optional"),
        "{hover}"
    );
    let formatted = request(&mut server, uri, "textDocument/formatting", 0, 0);
    let output = formatted[0]["newText"].as_str().unwrap();
    assert_eq!(
        output,
        "# People\njadwal qof {\n    id abn furaha\n    magac qoraal? # optional\n}\n"
    );
    open(&mut server, uri, output);
    assert_eq!(
        request(&mut server, uri, "textDocument/formatting", 0, 0)[0]["newText"],
        output
    );
    assert!(request(&mut server, uri, "textDocument/completion", 3, 33)
        .as_array()
        .unwrap()
        .is_empty());
    assert!(request(&mut server, uri, "textDocument/definition", 2, 1).is_null());
    assert_eq!(
        request(&mut server, uri, "textDocument/semanticTokens/full", 0, 0),
        json!({"data":[]})
    );
}
#[test]
fn cli_schema_check_and_generation_show_the_actual_schema_location() {
    let dir = std::env::temp_dir().join(format!(
        "soplang_schema_diagnostics_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(&dir).unwrap();
    let path = dir.join("schema.sop");
    let source = "jadwal qof {\n id abn furaha\n magac integer\n}\n";
    fs::write(&path, source).unwrap();
    for prefix in [vec!["check"], vec!["xog", "abuur"], vec!["fmt", "--qor"]] {
        let output = Command::new(env!("CARGO_BIN_EXE_soplang"))
            .args(prefix)
            .arg(&path)
            .env("NO_COLOR", "1")
            .output()
            .unwrap();
        assert!(!output.status.success());
        let error = String::from_utf8(output.stderr).unwrap();
        assert!(
            error.contains("schema.sop:3:8") && error.contains("E059") && error.contains("integer"),
            "{error}"
        );
    }
    fs::write(dir.join("main.sop"), "qor(1)\n").unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_soplang"))
        .arg("check")
        .arg(dir.join("main.sop"))
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("schema.sop:3:8"));
    assert_eq!(fs::read_to_string(&path).unwrap(), source);
    fs::write(&path, "# People\njadwal qof{\nid abn furaha\n}\n").unwrap();
    let formatted = Command::new(env!("CARGO_BIN_EXE_soplang"))
        .args(["fmt", "--qor"])
        .arg(&path)
        .output()
        .unwrap();
    assert!(formatted.status.success());
    assert_eq!(
        fs::read_to_string(&path).unwrap(),
        "# People\njadwal qof {\n    id abn furaha\n}\n"
    );
    fs::remove_dir_all(dir).unwrap();
}
