//! Editor coordinates, overlays, edits and the actual stdio protocol.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use serde_json::{json, Value};
use soplang::cli::lsp::Server;

struct Tree(PathBuf);
impl Tree {
    fn new(name: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "soplang_lsp_{name}_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&path).unwrap();
        Self(path)
    }
}
impl Drop for Tree {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn uri(path: &Path) -> String {
    url::Url::from_file_path(path).unwrap().to_string()
}
fn open(server: &mut Server, uri: &str, source: &str) -> Vec<Value> {
    server.handle(
        &json!({"jsonrpc":"2.0", "method":"textDocument/didOpen", "params":{
        "textDocument":{"uri":uri,"version":1,"languageId":"soplang","text":source}}})
        .to_string(),
    )
}
fn request(server: &mut Server, method: &str, uri: &str, position: Value, extra: Value) -> Value {
    let mut params = json!({"textDocument":{"uri":uri},"position":position});
    for (key, value) in extra.as_object().unwrap() {
        params[key] = value.clone();
    }
    server.handle(&json!({"jsonrpc":"2.0","id":1,"method":method,"params":params}).to_string())[0]
        ["result"]
        .clone()
}

#[test]
fn definitions_and_rename_use_each_files_unsaved_text_and_encoded_uri() {
    let tree = Tree::new("Af Soomaali 😀");
    let lib = tree.0.join("lib.sop");
    let main = tree.0.join("main.sop");
    fs::write(&lib, "hawl sal() { celi 1 }\n").unwrap();
    let unsaved = "// first\n// second\n// third\nhawl sal() { celi 1 }\n";
    let source = "keen \"lib.sop\"\nqor(sal())\n";
    fs::write(&main, source).unwrap();
    let (lib_uri, main_uri) = (uri(&lib), uri(&main));
    let mut server = Server::default();
    open(&mut server, &lib_uri, unsaved);
    open(&mut server, &main_uri, source);
    let position = json!({"line":1,"character":5});
    let definition = request(
        &mut server,
        "textDocument/definition",
        &main_uri,
        position.clone(),
        json!({}),
    );
    assert_eq!(definition["uri"], lib_uri);
    assert_eq!(
        definition["range"],
        json!({"start":{"line":3,"character":5},"end":{"line":3,"character":8}})
    );
    let edits = request(
        &mut server,
        "textDocument/rename",
        &main_uri,
        position.clone(),
        json!({"newName":"hello"}),
    );
    assert_eq!(edits["changes"].as_object().unwrap().len(), 2);
    assert_eq!(edits["changes"][&lib_uri][0]["range"], definition["range"]);
    assert_eq!(
        edits["changes"][&main_uri][0]["range"],
        json!({"start":{"line":1,"character":4},"end":{"line":1,"character":7}})
    );
    let refs = request(
        &mut server,
        "textDocument/references",
        &main_uri,
        position.clone(),
        json!({"context":{"includeDeclaration":false}}),
    );
    assert_eq!(refs.as_array().unwrap().len(), 1);
    let refs = request(
        &mut server,
        "textDocument/references",
        &main_uri,
        position.clone(),
        json!({"context":{"includeDeclaration":true}}),
    );
    assert_eq!(refs.as_array().unwrap().len(), 2);
    assert!(request(
        &mut server,
        "textDocument/rename",
        &main_uri,
        position,
        json!({"newName":"door"})
    )
    .is_null());
}

#[test]
fn utf16_incremental_changes_and_versions_preserve_document_state() {
    let tree = Tree::new("utf16");
    let main_uri = uri(&tree.0.join("main.sop"));
    let source = "qor(\"😀\"); abn magac=1; qor(magac)\n";
    let mut server = Server::default();
    open(&mut server, &main_uri, source);
    let usage = source[..source.rfind("magac").unwrap()]
        .encode_utf16()
        .count();
    let hover = request(
        &mut server,
        "textDocument/hover",
        &main_uri,
        json!({"line":0,"character":usage}),
        json!({}),
    );
    assert!(hover["contents"]["value"]
        .as_str()
        .unwrap()
        .contains("magac"));
    let number = source[..source.find("=1").unwrap() + 1]
        .encode_utf16()
        .count();
    let change = |version, edits| {
        json!({"jsonrpc":"2.0","method":"textDocument/didChange","params":{
        "textDocument":{"uri":main_uri,"version":version},"contentChanges":edits}})
        .to_string()
    };
    let notices = server.handle(&change(2, json!([{"range":{"start":{"line":0,"character":number},"end":{"line":0,"character":number+1}},"text":"2"}])));
    assert_eq!(notices[0]["params"]["version"], 2);
    assert!(server
        .handle(&change(1, json!([{"text":"invalid("}])))
        .is_empty());
    let emoji = source[..source.find('😀').unwrap()].encode_utf16().count();
    assert!(server.handle(&change(3, json!([{"range":{"start":{"line":0,"character":emoji+1},"end":{"line":0,"character":emoji+1}},"text":"broken"}]))).is_empty());
    let formatted = request(
        &mut server,
        "textDocument/formatting",
        &main_uri,
        json!({"line":0,"character":0}),
        json!({}),
    );
    assert!(formatted[0]["newText"]
        .as_str()
        .unwrap()
        .contains("magac = 2"));
    assert!(formatted[0]["newText"].as_str().unwrap().contains('😀'));
    let edits = request(
        &mut server,
        "textDocument/rename",
        &main_uri,
        json!({"line":0,"character":usage}),
        json!({"newName":"other"}),
    );
    assert_eq!(edits["changes"][&main_uri].as_array().unwrap().len(), 2);
    let declared = source[..source.find("magac").unwrap()]
        .encode_utf16()
        .count();
    assert_eq!(
        edits["changes"][&main_uri][0]["range"]["start"]["character"],
        declared
    );
}

#[test]
fn closing_an_unsaved_import_clears_its_diagnostic_and_restores_disk_source() {
    let tree = Tree::new("close");
    let lib = tree.0.join("lib.sop");
    let main = tree.0.join("main.sop");
    fs::write(&lib, "hawl sal() { celi 1 }\n").unwrap();
    let (lib_uri, main_uri) = (uri(&lib), uri(&main));
    let mut server = Server::default();
    open(&mut server, &lib_uri, "hawl broken(\n");
    open(&mut server, &main_uri, "keen \"lib.sop\"\nqor(sal())\n");
    let notices = server.handle(&json!({"jsonrpc":"2.0","method":"textDocument/didClose","params":{"textDocument":{"uri":lib_uri}}}).to_string());
    assert!(notices
        .iter()
        .any(|notice| notice["params"]["uri"] == lib_uri
            && notice["params"]["diagnostics"]
                .as_array()
                .unwrap()
                .is_empty()));
    let definition = request(
        &mut server,
        "textDocument/definition",
        &main_uri,
        json!({"line":1,"character":5}),
        json!({}),
    );
    assert_eq!(definition["range"]["start"]["line"], 0);
}

#[test]
fn semantic_tokens_include_comment_and_literal_bytes_in_utf16() {
    let tree = Tree::new("tokens");
    let main_uri = uri(&tree.0.join("main.sop"));
    let source = "qoraal magac=e\"😀\"\n/* a\n b */\nqor(magac)\n";
    let mut server = Server::default();
    open(&mut server, &main_uri, source);
    let result = request(
        &mut server,
        "textDocument/semanticTokens/full",
        &main_uri,
        json!({"line":0,"character":0}),
        json!({}),
    );
    let data = result["data"].as_array().unwrap();
    let (mut line, mut column) = (0usize, 0usize);
    let mut string_found = false;
    let mut comments = 0;
    for token in data.chunks_exact(5) {
        let delta = token[0].as_u64().unwrap() as usize;
        line += delta;
        column = if delta == 0 {
            column + token[1].as_u64().unwrap() as usize
        } else {
            token[1].as_u64().unwrap() as usize
        };
        let length = token[2].as_u64().unwrap() as usize;
        let units = source
            .lines()
            .nth(line)
            .unwrap()
            .encode_utf16()
            .collect::<Vec<_>>();
        let text = String::from_utf16(&units[column..column + length]).unwrap();
        if token[3] == 3 {
            assert_eq!(text, "e\"😀\"");
            assert_eq!(length, 5);
            string_found = true;
        }
        if token[3] == 5 {
            comments += 1;
        }
    }
    assert!(string_found);
    assert_eq!(comments, 2);
}

#[test]
fn untitled_document_edits_keep_the_clients_uri() {
    let mut server = Server::default();
    let uri = "untitled:Untitled-1";
    open(&mut server, uri, "door name=1\nqor(name)\n");
    let edits = request(
        &mut server,
        "textDocument/rename",
        uri,
        json!({"line":1,"character":5}),
        json!({"newName":"other"}),
    );
    assert_eq!(
        edits["changes"]
            .as_object()
            .unwrap()
            .keys()
            .map(String::as_str)
            .collect::<Vec<_>>(),
        [uri]
    );
    assert_eq!(edits["changes"][uri].as_array().unwrap().len(), 2);
}

#[test]
fn stdio_framing_is_bounded_and_shutdown_exits_cleanly() {
    let mut wire = Vec::new();
    for message in [
        json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}),
        json!({"jsonrpc":"2.0","id":2,"method":"shutdown"}),
        json!({"jsonrpc":"2.0","method":"exit"}),
    ] {
        let body = message.to_string();
        wire.extend_from_slice(format!("Content-Length: {}\r\n\r\n{body}", body.len()).as_bytes());
    }
    let run = |wire: &[u8]| {
        let mut child = Command::new(env!("CARGO_BIN_EXE_soplang"))
            .arg("lsp")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        child.stdin.take().unwrap().write_all(wire).unwrap();
        child.wait_with_output().unwrap()
    };
    let output = run(&wire);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let mut remaining = output.stdout.as_slice();
    let mut replies = Vec::new();
    while !remaining.is_empty() {
        let end = remaining
            .windows(4)
            .position(|bytes| bytes == b"\r\n\r\n")
            .unwrap();
        let header = std::str::from_utf8(&remaining[..end]).unwrap();
        let length: usize = header
            .strip_prefix("Content-Length: ")
            .unwrap()
            .parse()
            .unwrap();
        let body = &remaining[end + 4..end + 4 + length];
        replies.push(serde_json::from_slice::<Value>(body).unwrap());
        remaining = &remaining[end + 4 + length..];
    }
    assert_eq!(replies.len(), 2);
    assert_eq!(
        replies[0]["result"]["capabilities"]["positionEncoding"],
        "utf-16"
    );
    let output = run(b"Content-Length: 999999999999\r\n\r\n");
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("aad ayay u weyn"));
}
