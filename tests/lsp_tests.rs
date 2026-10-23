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

fn at(source: &str, needle: &str, within: usize) -> Value {
    let offset = source.rfind(needle).unwrap() + within;
    let before = &source[..offset];
    json!({"line":before.bytes().filter(|byte| *byte == b'\n').count(),
        "character":before.rsplit('\n').next().unwrap().encode_utf16().count()})
}

#[test]
fn namespace_members_and_same_named_classes_keep_distinct_identities() {
    let tree = Tree::new("module-identities");
    let left = tree.0.join("left[,].sop");
    let right = tree.0.join("right.sop");
    let main = tree.0.join("main.sop");
    let left_source = "soosaar C, factory\nqaab C {\n hawl dhis(qoraal value) { nafta.name = value }\n hawl same(): qoraal { celi nafta.name }\n hawl child(): C { celi nafta }\n}\nhawl factory(): C { celi cusub C(\"test\") }\nhawl _private() {}\n";
    let right_source = "soosaar C\nqaab C {\n hawl same(): abn { celi 2 }\n}\n";
    let source = "keen \"left[,].sop\" sida a\nkeen \"right.sop\" sida b\nkeen \"left[,].sop\" ka C sida Local\nLocal p = cusub Local(\"x\")\ndoor q = cusub b.C()\nqor(p.same())\nqor(q.same())\nqor(a.factory().child().name)\ndoor rows = [p]\nqor(rows[0].same())\n";
    fs::write(&left, left_source).unwrap();
    fs::write(&right, right_source).unwrap();
    fs::write(&main, source).unwrap();
    let main_uri = uri(&main);
    let mut server = Server::default();
    let notices = open(&mut server, &main_uri, source);
    assert!(
        notices.iter().all(|notice| notice["params"]["diagnostics"]
            .as_array()
            .unwrap()
            .is_empty()),
        "{notices:?}"
    );
    for (needle, within, file, line) in [
        ("p.same", 3, &left, 3),
        ("q.same", 3, &right, 2),
        ("rows[0].same", 9, &left, 3),
        ("child().name", 9, &left, 2),
        ("Local p", 2, &left, 1),
    ] {
        let found = request(
            &mut server,
            "textDocument/definition",
            &main_uri,
            at(source, needle, within),
            json!({}),
        );
        assert_eq!(found["uri"], uri(file), "{needle}: {found}");
        assert_eq!(found["range"]["start"]["line"], line, "{needle}: {found}");
    }
    let labels = |items: Value| {
        items
            .as_array()
            .unwrap()
            .iter()
            .map(|item| item["label"].as_str().unwrap().to_string())
            .collect::<Vec<_>>()
    };
    let members = labels(request(
        &mut server,
        "textDocument/completion",
        &main_uri,
        at(source, "a.factory", 2),
        json!({}),
    ));
    assert_eq!(members, ["C", "factory"]);
    let members = labels(request(
        &mut server,
        "textDocument/completion",
        &main_uri,
        at(source, "p.same", 2),
        json!({}),
    ));
    assert!(
        members.contains(&"same".into())
            && members.contains(&"name".into())
            && members.contains(&"child".into()),
        "{members:?}"
    );
    assert!(!members.contains(&"_private".into()));
}

#[test]
fn explicit_alias_rename_changes_the_binding_and_target_rename_changes_import_names() {
    let tree = Tree::new("alias-rename");
    let lib = tree.0.join("lib.sop");
    let main = tree.0.join("main.sop");
    let source = "keen \"lib.sop\" ka original sida local\nqor(local())\n";
    fs::write(&lib, "soosaar original\nhawl original(): abn { celi 1 }\n").unwrap();
    fs::write(&main, source).unwrap();
    let (main_uri, lib_uri) = (uri(&main), uri(&lib));
    let mut server = Server::default();
    open(&mut server, &main_uri, source);
    let local = request(
        &mut server,
        "textDocument/rename",
        &main_uri,
        at(source, "local()", 1),
        json!({"newName":"mine"}),
    );
    assert_eq!(local["changes"].as_object().unwrap().len(), 1, "{local}");
    assert_eq!(
        local["changes"][&main_uri].as_array().unwrap().len(),
        2,
        "{local}"
    );
    let target = request(
        &mut server,
        "textDocument/rename",
        &main_uri,
        at(source, "original sida", 1),
        json!({"newName":"changed"}),
    );
    assert_eq!(target["changes"].as_object().unwrap().len(), 2, "{target}");
    assert_eq!(
        target["changes"][&main_uri].as_array().unwrap().len(),
        1,
        "{target}"
    );
    assert_eq!(
        target["changes"][&lib_uri].as_array().unwrap().len(),
        2,
        "{target}"
    );
}

#[test]
fn a_module_import_does_not_offer_private_or_unimported_globals() {
    let tree = Tree::new("private-completion");
    let lib = tree.0.join("lib.sop");
    let main = tree.0.join("main.sop");
    fs::write(&lib, "soosaar public\ndoor hidden = 7\ndoor public = 3\n").unwrap();
    let source = "keen \"lib.sop\" sida m\ndoor visible = 1\nqor(visible)\n";
    fs::write(&main, source).unwrap();
    let mut server = Server::default();
    let main_uri = uri(&main);
    open(&mut server, &main_uri, source);
    let result = request(
        &mut server,
        "textDocument/completion",
        &main_uri,
        at(source, "visible)", 2),
        json!({}),
    );
    let labels: Vec<_> = result
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item["label"].as_str().unwrap())
        .collect();
    assert!(
        labels.contains(&"m") && labels.contains(&"visible"),
        "{labels:?}"
    );
    assert!(
        !labels.contains(&"hidden") && !labels.contains(&"public"),
        "{labels:?}"
    );
}

#[test]
fn workspace_references_include_closed_importers_and_refuse_incomplete_renames() {
    let tree = Tree::new("workspace");
    let lib = tree.0.join("lib.sop");
    let main = tree.0.join("main.sop");
    let other = tree.0.join("other.sop");
    let lib_source = "hawl value(): abn { celi 7 }\n";
    fs::write(&lib, lib_source).unwrap();
    fs::write(&main, "keen \"lib.sop\" sida m\nqor(m.value())\n").unwrap();
    fs::write(
        &other,
        "keen \"lib.sop\" ka value sida local\nqor(local())\n",
    )
    .unwrap();
    let mut server = Server::default();
    server.handle(
        &json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"rootUri":uri(&tree.0)}})
            .to_string(),
    );
    let lib_uri = uri(&lib);
    open(&mut server, &lib_uri, lib_source);
    let refs = request(
        &mut server,
        "textDocument/references",
        &lib_uri,
        at(lib_source, "value()", 1),
        json!({"context":{"includeDeclaration":true}}),
    );
    assert_eq!(refs.as_array().unwrap().len(), 3, "{refs}");
    let edits = request(
        &mut server,
        "textDocument/rename",
        &lib_uri,
        at(lib_source, "value()", 1),
        json!({"newName":"result"}),
    );
    assert_eq!(edits["changes"].as_object().unwrap().len(), 3, "{edits}");
    assert_eq!(
        edits["changes"][uri(&other)].as_array().unwrap().len(),
        1,
        "explicit alias references retain their names: {edits}"
    );
    fs::write(tree.0.join("broken.sop"), "door x = @").unwrap();
    let refused = request(
        &mut server,
        "textDocument/rename",
        &lib_uri,
        at(lib_source, "value()", 1),
        json!({"newName":"result"}),
    );
    assert!(
        refused.is_null(),
        "must not return a partial workspace edit: {refused}"
    );
}

#[test]
fn bundled_definitions_have_read_only_source_uris_and_cannot_be_renamed() {
    let mut server = Server::default();
    let source = "keen \"web\" sida w\ndoor app = cusub w.Web()\napp.hel(\"GET\", \"/\", hawl(c) { celi w.json({}) })\n";
    let uri = "untitled:bundled-navigation";
    open(&mut server, uri, source);
    let definition = request(
        &mut server,
        "textDocument/definition",
        uri,
        at(source, "app.hel", 5),
        json!({}),
    );
    assert_eq!(definition["uri"], "soplang-stdlib:/web.sop", "{definition}");
    let reply = server.handle(&json!({"jsonrpc":"2.0","id":2,"method":"soplang/source","params":{"uri":definition["uri"]}}).to_string());
    assert!(reply[0]["result"].as_str().unwrap().contains("hawl hel("));
    let refused = request(
        &mut server,
        "textDocument/rename",
        uri,
        at(source, "app.hel", 5),
        json!({"newName":"other"}),
    );
    assert!(refused.is_null());
    let invalid = server.handle(&json!({"jsonrpc":"2.0","id":2,"method":"soplang/source","params":{"uri":"file:///etc/passwd"}}).to_string());
    assert!(invalid[0]["result"].is_null());
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
            .args(["lsp", "--stdio"])
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

#[test]
fn unfinished_function_completion_keeps_imported_models_and_local_bindings() {
    let tree = Tree::new("unfinished");
    let main = uri(&tree.0.join("main.sop"));
    fs::write(
        tree.0.join("models.sop"),
        "qaab Qof { hawl magac(): qoraal { celi \"Amina\" } }\n",
    )
    .unwrap();
    let source =
        "keen \"models.sop\" sida m\nhawl edit(abn count) {\n m.Qof local = cusub m.Qof()\n local.";
    let mut server = Server::default();
    let notice = open(&mut server, &main, source);
    assert!(!notice[0]["params"]["diagnostics"]
        .as_array()
        .unwrap()
        .is_empty());
    let result = request(
        &mut server,
        "textDocument/completion",
        &main,
        json!({"line":3,"character":7}),
        json!({}),
    );
    assert!(
        result
            .as_array()
            .unwrap()
            .iter()
            .any(|item| item["label"] == "magac"),
        "{result}"
    );
    open(
        &mut server,
        &main,
        "hawl edit(abn count) {\n qoraal local = \"Amina\"\n qoraal pending = \"unfinished",
    );
    let result = request(
        &mut server,
        "textDocument/completion",
        &main,
        json!({"line":2,"character":30}),
        json!({}),
    );
    assert!(
        result
            .as_array()
            .unwrap()
            .iter()
            .any(|item| item["label"] == "count"),
        "{result}"
    );
}

#[test]
fn aliased_document_uris_keep_their_own_text_and_refuse_ambiguous_renames() {
    let tree = Tree::new("aliases");
    let path = tree.0.join("a.sop");
    fs::write(&path, "abn value=0\n").unwrap();
    let first = uri(&path);
    let second = first.replace("/a.sop", "/%61.sop");
    let mut server = Server::default();
    open(&mut server, &first, "abn value=1\nqor(value)\n");
    open(&mut server, &second, "// 😀\nabn value=2\nqor(value)\n");
    for (uri, line, definition) in [(&first, 1, 0), (&second, 2, 1)] {
        let result = request(
            &mut server,
            "textDocument/definition",
            uri,
            json!({"line":line,"character":5}),
            json!({}),
        );
        assert_eq!(result["uri"], *uri);
        assert_eq!(result["range"]["start"]["line"], definition);
        let result = request(
            &mut server,
            "textDocument/rename",
            uri,
            json!({"line":line,"character":5}),
            json!({"newName":"renamed"}),
        );
        assert!(result.is_null(), "{result}");
    }
    server.handle(&json!({"jsonrpc":"2.0","method":"textDocument/didClose","params":{"textDocument":{"uri":second}}}).to_string());
    let result = request(
        &mut server,
        "textDocument/rename",
        &first,
        json!({"line":1,"character":5}),
        json!({"newName":"renamed"}),
    );
    assert_eq!(result["changes"][&first].as_array().unwrap().len(), 2);
}

#[test]
fn local_rename_refuses_to_skip_references_after_a_syntax_error() {
    let mut server = Server::default();
    let uri = "untitled:broken-function.sop";
    open(
        &mut server,
        uri,
        "hawl f(abn value) {\n qor(value)\n @\n qor(value)\n}\n",
    );
    let result = request(
        &mut server,
        "textDocument/rename",
        uri,
        json!({"line":0,"character":13}),
        json!({"newName":"renamed"}),
    );
    assert!(result.is_null(), "{result}");
}

#[test]
fn native_completion_navigation_and_references_keep_lexical_identities() {
    let source = "qoraal text = qoraal(2)\ndoor emit = qor\nqor(text, \"qor qoraal\")\nhawl local() {\n hawl json_akhris(qoraal value) { celi value }\n json_akhris(\"local\")\n}\n";
    let uri = "untitled:native-identities";
    let mut server = Server::default();
    open(&mut server, uri, source);
    let items = request(
        &mut server,
        "textDocument/completion",
        uri,
        at(source, "door emit", 0),
        json!({}),
    );
    for name in ["qor", "gelin", "json_akhris", "xog_fur", "http_codso"] {
        assert!(
            items
                .as_array()
                .unwrap()
                .iter()
                .any(|item| item["label"] == name),
            "{name}: {items}"
        );
    }
    assert!(items
        .as_array()
        .unwrap()
        .iter()
        .all(|item| !item["label"].as_str().unwrap().starts_with('#')));
    let definition = request(
        &mut server,
        "textDocument/definition",
        uri,
        at(source, "qor(text", 1),
        json!({}),
    );
    assert_eq!(
        definition["uri"], "soplang-stdlib:/native.sop",
        "{definition}"
    );
    let response = server.handle(&json!({"jsonrpc":"2.0","id":3,"method":"soplang/source","params":{"uri":definition["uri"]}}).to_string());
    let native = response[0]["result"].as_str().unwrap();
    let line = definition["range"]["start"]["line"].as_u64().unwrap() as usize;
    assert!(native.lines().nth(line).unwrap().contains("hawl qor("));
    let hover = request(
        &mut server,
        "textDocument/hover",
        uri,
        at(source, "qor(text", 1),
        json!({}),
    );
    assert!(
        hover["contents"]["value"]
            .as_str()
            .unwrap()
            .contains("*qiimayaal"),
        "{hover}"
    );
    let references = request(
        &mut server,
        "textDocument/references",
        uri,
        at(source, "qor(text", 1),
        json!({"context":{"includeDeclaration":false}}),
    );
    let lines: Vec<_> = references
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["range"]["start"]["line"].as_u64().unwrap())
        .collect();
    assert_eq!(lines, [1, 2], "{references}");
    let casts = request(
        &mut server,
        "textDocument/references",
        uri,
        at(source, "qoraal(2)", 1),
        json!({"context":{"includeDeclaration":false}}),
    );
    assert_eq!(casts.as_array().unwrap().len(), 1, "{casts}");
    assert_eq!(casts[0]["range"]["start"]["character"], 14);
    let local = request(
        &mut server,
        "textDocument/definition",
        uri,
        at(source, "json_akhris(\"local", 1),
        json!({}),
    );
    assert_eq!(local["uri"], uri);
    assert_eq!(local["range"]["start"]["line"], 4);
    assert!(request(
        &mut server,
        "textDocument/rename",
        uri,
        at(source, "qor(text", 1),
        json!({"newName":"print"})
    )
    .is_null());
}

#[test]
fn primitive_completion_tracks_generic_receivers_and_readonly_methods() {
    let source = "teed[qoraal] words = [\"a\"]\nwalax fields = {}\ndoor bytes = baytyo()\ndoor pair = xirmo(1, 2)\ndoor keys = urur(1)\ndoor iter = taxane(3)\ndoor unknown = gelin\nwords.nuqul().hore().weynee()\nwords.hel(0).yaree()\nfields.furayaal().hore().masax()\nbytes.teed()\npair.dherer()\nkeys.leeyahay(1)\niter.xiga()\ndoor count = [1, 2].dherer()\ndoor upper = (\"x\").yaree()\nunknown.other()\n";
    let uri = "untitled:native-methods";
    let mut server = Server::default();
    open(&mut server, uri, source);
    for (needle, within, required, absent) in [
        ("words.nuqul", 6, "geli", "furayaal"),
        ("[1, 2].dherer", 7, "geli", "furayaal"),
        ("(\"x\").yaree", 6, "weynee", "geli"),
        ("hore().weynee", 7, "yaree", "geli"),
        ("hel(0).yaree", 7, "weynee", "geli"),
        ("hore().masax", 7, "qeybi", "geli"),
        ("bytes.teed", 6, "teed", "furayaal"),
        ("pair.dherer", 5, "hore", "geli"),
        ("keys.leeyahay", 5, "geli", "habee"),
        ("iter.xiga", 5, "xiga", "geli"),
    ] {
        let items = request(
            &mut server,
            "textDocument/completion",
            uri,
            at(source, needle, within),
            json!({}),
        );
        assert!(
            items
                .as_array()
                .unwrap()
                .iter()
                .any(|item| item["label"] == required),
            "{needle}: {items}"
        );
        assert!(
            items
                .as_array()
                .unwrap()
                .iter()
                .all(|item| item["label"] != absent),
            "{needle}: {items}"
        );
    }
    let unknown = request(
        &mut server,
        "textDocument/completion",
        uri,
        at(source, "unknown.other", 8),
        json!({}),
    );
    assert_eq!(unknown, json!([]));
    let hover = request(
        &mut server,
        "textDocument/hover",
        uri,
        at(source, "words.nuqul", 8),
        json!({}),
    );
    assert!(
        hover["contents"]["value"]
            .as_str()
            .unwrap()
            .contains("teed[qoraal]"),
        "{hover}"
    );
    let definition = request(
        &mut server,
        "textDocument/definition",
        uri,
        at(source, "bytes.teed", 8),
        json!({}),
    );
    assert_eq!(
        definition["uri"], "soplang-stdlib:/native.sop",
        "{definition}"
    );
}

#[test]
fn signature_help_selects_overloads_and_skips_nested_argument_commas() {
    let source = "hawl greet(qoraal name, abn times = 1) {}\nqaab Person { hawl dhis(qoraal name, abn age) {} }\nteed values = []\ndoor emit = qor\nvalues.geli(1)\nvalues.geli(0, 1)\nvalues.saar()\nqor({x: [1, 2], y: 3}, f\"{walax(\"a\", 2)}\", 7)\ngreet(times: 4, name: \"test\")\ndoor p = cusub Person(\"x\", 4)\nemit(1, 2, 3)\n";
    let uri = "untitled:native-signatures";
    let mut server = Server::default();
    open(&mut server, uri, source);
    for (needle, within, selected, parameter, label) in [
        ("geli(1)", 6, 0, Some(0), "geli(qiime)"),
        ("geli(0, 1)", 9, 1, Some(1), "geli(abn tusmo, qiime)"),
        ("saar()", 5, 0, None, "saar()"),
        ("}, f\"", 3, 0, Some(0), "qor(*qiimayaal)"),
        ("times: 4", 7, 0, Some(1), "greet("),
        ("name: \"test\"", 7, 0, Some(0), "greet("),
        ("Person(\"x\", 4)", 13, 0, Some(1), "Person"),
        ("emit(1, 2, 3)", 12, 0, Some(0), "qor(*qiimayaal)"),
    ] {
        let help = request(
            &mut server,
            "textDocument/signatureHelp",
            uri,
            at(source, needle, within),
            json!({}),
        );
        assert_eq!(help["activeSignature"], selected, "{needle}: {help}");
        assert_eq!(
            help["activeParameter"].as_u64(),
            parameter,
            "{needle}: {help}"
        );
        assert!(
            help["signatures"][selected as usize]["label"]
                .as_str()
                .unwrap()
                .contains(label),
            "{needle}: {help}"
        );
    }
    let unfinished = "teed values = []\nvalues.geli(0, ";
    let uri2 = "untitled:native-unfinished";
    open(&mut server, uri2, unfinished);
    let help = request(
        &mut server,
        "textDocument/signatureHelp",
        uri2,
        at(unfinished, "geli(0, ", 8),
        json!({}),
    );
    assert_eq!(help["activeSignature"], 1, "{help}");
    assert_eq!(help["activeParameter"], 1, "{help}");
}

#[test]
fn signature_help_does_not_reuse_reassigned_callable_contracts() {
    let source = "hawl emit = qor\nemit = json_akhris\nemit(\"{}\")\n";
    let uri = "untitled:reassigned-native-signature";
    let mut server = Server::default();
    open(&mut server, uri, source);
    let help = request(
        &mut server,
        "textDocument/signatureHelp",
        uri,
        at(source, "emit(\"{}\")", 9),
        json!({}),
    );
    assert!(help.is_null(), "{help}");

    let source =
        "hawl consume(a, b, c) {}\nconsume({x: [1, 2], y: 3}, f\"{walax(\"a\", 2)}\", 7)\n";
    let uri = "untitled:nested-signature-commas";
    open(&mut server, uri, source);
    let help = request(
        &mut server,
        "textDocument/signatureHelp",
        uri,
        at(source, ", 7)", 3),
        json!({}),
    );
    assert_eq!(help["activeParameter"], 2, "{help}");
}
