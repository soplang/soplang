//! Stdio language server for `.sop` and `schema.sop`.

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::io::{BufRead, Write};
use std::path::{Path, PathBuf};

use serde_json::{json, Value};

use crate::analysis::index::{Index, Symbol};
use crate::analysis::{check_source, overlay_loader, source_identity, Limits, SourceFile};
use crate::data::parse_schema_file;
use crate::error::{runtime_error_ex, ErrorMeta, SoplangError};
use crate::frontend::{Lexer, TokenType};

mod position;
mod schema;
mod workspace;
use position::{offset as offset_at, range as range_of};

struct Indexed {
    index: Index,
    files: Vec<SourceFile>,
    path: PathBuf,
    offset: usize,
    complete: bool,
}

impl Indexed {
    fn source(&self, path: &Path) -> Option<&str> {
        self.files
            .iter()
            .find(|file| file.path == path)
            .map(|file| file.text.as_str())
            .or_else(|| {
                (path == Path::new(crate::native_api::SOURCE_PATH)).then(crate::native_api::source)
            })
    }
}

#[derive(Default)]
pub struct Server {
    documents: HashMap<String, String>,
    versions: HashMap<String, i64>,
    schemas: HashSet<String>,
    published: HashSet<String>,
    shutdown: bool,
    roots: BTreeSet<PathBuf>,
    pub exit: bool,
}

pub fn serve() -> Result<(), SoplangError> {
    let stdin = std::io::stdin();
    let mut reader = std::io::BufReader::new(stdin.lock());
    let mut stdout = std::io::stdout();
    let mut server = Server::default();
    loop {
        let Some(body) = read_message(&mut reader)? else {
            break;
        };
        for message in server.handle(&body) {
            write_message(&mut stdout, &message)?;
        }
        if server.exit {
            return if server.shutdown {
                Ok(())
            } else {
                Err(lsp_error(
                    "LSP waa la xidhay iyadoo aan shutdown la dalban.",
                ))
            };
        }
    }
    Ok(())
}

pub fn handle_text(server: &mut Server, body: &str) -> Vec<Value> {
    server.handle(body)
}

impl Server {
    pub fn handle(&mut self, body: &str) -> Vec<Value> {
        let Ok(message) = serde_json::from_str::<Value>(body) else {
            return vec![
                json!({"jsonrpc": "2.0", "id": null, "error": {"code": -32700, "message": "JSON ma saxna."}}),
            ];
        };
        let method = message.get("method").and_then(Value::as_str).unwrap_or("");
        let id = message.get("id").cloned();
        if message.get("jsonrpc").and_then(Value::as_str) != Some("2.0")
            || method.is_empty()
            || id
                .as_ref()
                .is_some_and(|id| !(id.is_null() || id.is_string() || id.is_i64() || id.is_u64()))
        {
            return vec![
                json!({"jsonrpc":"2.0", "id":null, "error":{"code":-32600, "message":"Codsiga LSP ma saxna."}}),
            ];
        }
        if method == "exit" {
            self.exit = true;
            return Vec::new();
        }
        if self.shutdown {
            return id.map(|id| vec![json!({"jsonrpc":"2.0", "id":id, "error":{"code":-32600,"message":"LSP hore ayaa loo joojiyey."}})]).unwrap_or_default();
        }
        let result = match method {
            "initialize" => {
                self.roots = workspace::roots(&message["params"]);
                json!({
                    "capabilities": {
                        "workspace": {"workspaceFolders":{"supported":true,"changeNotifications":true}},
                        "textDocumentSync": 2,
                        "positionEncoding": "utf-16",
                        "completionProvider": {"triggerCharacters": ["."]},
                        "hoverProvider": true,
                        "signatureHelpProvider":{"triggerCharacters":["(",","],"retriggerCharacters":[","]},
                        "definitionProvider": true,
                        "referencesProvider": true,
                        "renameProvider": true,
                        "documentFormattingProvider": true,
                        "codeActionProvider": true,
                        "semanticTokensProvider": {
                            "legend": {
                                "tokenTypes": ["keyword", "function", "variable", "string", "number", "comment", "type"],
                                "tokenModifiers": []
                            },
                            "full": true
                        }
                    },
                    "serverInfo": {"name": "soplang", "version": env!("CARGO_PKG_VERSION")}
                })
            }
            "workspace/didChangeWorkspaceFolders" => {
                for folder in message
                    .pointer("/params/event/removed")
                    .and_then(Value::as_array)
                    .into_iter()
                    .flatten()
                {
                    if let Some(path) = folder["uri"].as_str().and_then(workspace::file_path) {
                        self.roots.remove(&path);
                    }
                }
                for folder in message
                    .pointer("/params/event/added")
                    .and_then(Value::as_array)
                    .into_iter()
                    .flatten()
                {
                    if let Some(path) = folder["uri"].as_str().and_then(workspace::file_path) {
                        self.roots.insert(path);
                    }
                }
                return self.refresh_diagnostics();
            }
            "workspace/didChangeWatchedFiles" => return self.refresh_diagnostics(),
            "shutdown" => {
                self.shutdown = true;
                Value::Null
            }
            "textDocument/didOpen" | "textDocument/didChange" => {
                let uri = message
                    .pointer("/params/textDocument/uri")
                    .and_then(Value::as_str)
                    .unwrap_or("");
                let version = message
                    .pointer("/params/textDocument/version")
                    .and_then(Value::as_i64);
                if method == "textDocument/didChange"
                    && version
                        .zip(self.versions.get(uri).copied())
                        .is_some_and(|(new, old)| new <= old)
                {
                    return Vec::new();
                }
                if let Some((uri, text)) = document_text(
                    &message,
                    method,
                    self.documents.get(uri).map(String::as_str),
                ) {
                    if let Some(version) = version {
                        self.versions.insert(uri.clone(), version);
                    }
                    if message
                        .pointer("/params/textDocument/languageId")
                        .and_then(Value::as_str)
                        == Some("soplang-schema")
                    {
                        self.schemas.insert(uri.clone());
                    }
                    self.documents.insert(uri, text);
                    return self.refresh_diagnostics();
                }
                return Vec::new();
            }
            "textDocument/didClose" => {
                if let Some(uri) = message
                    .pointer("/params/textDocument/uri")
                    .and_then(Value::as_str)
                {
                    self.documents.remove(uri);
                    self.versions.remove(uri);
                    self.schemas.remove(uri);
                }
                return self.refresh_diagnostics();
            }
            "textDocument/completion" => self.completion(&message),
            "textDocument/hover" => self.hover(&message),
            "textDocument/signatureHelp" => self.signature_help(&message),
            "textDocument/definition" => self.definition(&message),
            "textDocument/references" => self.references(&message),
            "soplang/source" => message
                .pointer("/params/uri")
                .and_then(Value::as_str)
                .and_then(bundled_path)
                .and_then(|path| reference_source(&path))
                .map(|text| Value::String(text.into()))
                .unwrap_or(Value::Null),
            "textDocument/rename" => self.rename(&message),
            "textDocument/formatting" => self.formatting(&message),
            "textDocument/codeAction" => self.code_action(&message),
            "textDocument/semanticTokens/full" => self.tokens(&message),
            _ => {
                if id.is_none() {
                    return Vec::new();
                }
                return vec![
                    json!({"jsonrpc": "2.0", "id": id, "error": {"code": -32601, "message": "Habka lama aqoonsan."}}),
                ];
            }
        };
        if let Some(id) = id {
            vec![json!({"jsonrpc": "2.0", "id": id, "result": result})]
        } else {
            Vec::new()
        }
    }

    fn text(&self, message: &Value) -> Option<(String, String)> {
        let uri = message
            .pointer("/params/textDocument/uri")
            .and_then(Value::as_str)?
            .to_string();
        let text = self.documents.get(&uri)?.clone();
        Some((uri, text))
    }

    fn is_schema(&self, uri: &str) -> bool {
        self.schemas.contains(uri)
            || path_of(uri)
                .file_name()
                .is_some_and(|name| name == "schema.sop")
    }

    fn overlays(&self) -> HashMap<PathBuf, String> {
        let mut documents: Vec<_> = self.documents.iter().collect();
        documents.sort_by(|a, b| b.0.cmp(a.0));
        documents
            .into_iter()
            .map(|(uri, text)| (path_of(uri), text.clone()))
            .collect()
    }

    fn ambiguous_paths(&self) -> HashSet<PathBuf> {
        let mut seen = HashSet::new();
        self.documents
            .keys()
            .map(|uri| path_of(uri))
            .filter(|path| !seen.insert(path.clone()))
            .collect()
    }

    fn uri_for_path(&self, path: &Path) -> String {
        self.documents
            .keys()
            .filter(|uri| path_of(uri) == path)
            .min()
            .cloned()
            .unwrap_or_else(|| uri_of(path))
    }

    fn uri_for_request(&self, path: &Path, message: &Value) -> String {
        message
            .pointer("/params/textDocument/uri")
            .and_then(Value::as_str)
            .filter(|uri| path_of(uri) == path)
            .map(str::to_owned)
            .unwrap_or_else(|| self.uri_for_path(path))
    }

    fn indexed(&self, uri: &str, text: &str, position: (u64, u64)) -> Option<Indexed> {
        if self.is_schema(uri) {
            return None;
        }
        let path = path_of(uri);
        let mut overlays = self.overlays();
        overlays.insert(path.clone(), text.into());
        let limits = Limits::default();
        let (mut candidates, mut complete) = workspace::discover(&self.roots, limits.max_files);
        complete &= self.ambiguous_paths().is_empty();
        candidates.extend(
            overlays
                .keys()
                .filter(|path| path.file_name().is_none_or(|name| name != "schema.sop"))
                .cloned(),
        );
        for uri in self.documents.keys().filter(|uri| self.is_schema(uri)) {
            candidates.remove(&path_of(uri));
        }
        candidates.remove(&path);
        let mut files = BTreeMap::<PathBuf, SourceFile>::new();
        let mut bytes = 0;
        let sources = std::iter::once((path.clone(), Ok(text.to_string()))).chain(
            candidates.into_iter().map(|path| {
                let source = overlay_loader(&overlays, &path);
                (path, source)
            }),
        );
        for (source_path, source) in sources {
            if files.contains_key(&source_path) {
                continue;
            }
            let Ok(source) = source else {
                complete = false;
                continue;
            };
            let analysis = check_source(
                &source,
                &source_path,
                false,
                limits,
                &|imported| overlay_loader(&overlays, imported),
                &|| false,
            );
            complete &= analysis.error.is_none();
            for file in analysis.files {
                if files.contains_key(&file.path) {
                    continue;
                }
                if files.len() >= limits.max_files
                    || bytes + file.text.len() > limits.max_total_bytes
                {
                    complete = false;
                    break;
                }
                bytes += file.text.len();
                files.insert(file.path.clone(), file);
            }
        }
        let files: Vec<_> = files.into_values().collect();
        Some(Indexed {
            index: Index::build(&files),
            files,
            offset: offset_at(text, position.0, position.1)?,
            path,
            complete,
        })
    }

    fn refresh_diagnostics(&mut self) -> Vec<Value> {
        let overlays = self.overlays();
        let mut grouped = BTreeMap::<String, Vec<Value>>::new();
        for uri in self.published.iter().chain(self.documents.keys()) {
            grouped.entry(uri.clone()).or_default();
        }
        for (uri, text) in &self.documents {
            for notice in diagnostics(uri.clone(), text, &overlays, self.is_schema(uri)) {
                let uri = notice["params"]["uri"].as_str().unwrap().to_string();
                let items = grouped.entry(uri).or_default();
                for item in notice["params"]["diagnostics"].as_array().unwrap() {
                    if !items.contains(item) {
                        items.push(item.clone());
                    }
                }
            }
        }
        self.published = self.documents.keys().cloned().collect();
        grouped
            .into_iter()
            .map(|(uri, diagnostics)| {
                if !diagnostics.is_empty() {
                    self.published.insert(uri.clone());
                }
                let mut notice = json!({"jsonrpc":"2.0", "method":"textDocument/publishDiagnostics",
                "params": {"uri": uri, "diagnostics": diagnostics}});
                if let Some(version) = self.versions.get(&uri) {
                    notice["params"]["version"] = json!(version);
                }
                notice
            })
            .collect()
    }

    fn completion(&self, message: &Value) -> Value {
        let Some((uri, text)) = self.text(message) else {
            return json!([]);
        };
        if self.is_schema(&uri) {
            return offset_at(&text, position(message).0, position(message).1)
                .map(|offset| schema::completion(&text, offset))
                .unwrap_or_else(|| json!([]));
        }
        let Some(analysis) = self.indexed(&uri, &text, position(message)) else {
            return json!([]);
        };
        let symbols = analysis
            .index
            .members_at(&analysis.path, analysis.offset)
            .unwrap_or_else(|| analysis.index.visible(&analysis.path, analysis.offset));
        let items: Vec<Value> = symbols
            .into_iter()
            .map(|symbol| {
                let mut item = json!({
                    "label": symbol.name, "kind": completion_kind(symbol),
                    "detail": analysis.index.native_api(&symbol.id).map(|api|api.label(&api.signatures[0],analysis.index.completion_receiver(&analysis.path,analysis.offset).as_deref())).map(|label|analysis.index.display_type(&label)).unwrap_or_else(||analysis.index.symbol_label(symbol))
                });
                if let Some(api) = analysis.index.native_api(&symbol.id) {
                    item["documentation"] = json!(api.documentation);
                }
                item
            })
            .collect();
        json!(items)
    }

    fn hover(&self, message: &Value) -> Value {
        if let Some((uri, text)) = self.text(message) {
            if self.is_schema(&uri) {
                return offset_at(&text, position(message).0, position(message).1)
                    .map(|offset| schema::hover(&text, offset))
                    .unwrap_or(Value::Null);
            }
        }
        let Some((symbol, analysis)) = self.symbol_at(message) else {
            return Value::Null;
        };
        let text = if let Some(api) = analysis.index.native_api(&symbol.id) {
            let receiver = analysis
                .index
                .member_receiver(&analysis.path, analysis.offset);
            let text = api.hover(receiver.as_deref());
            analysis.index.display_type(&text)
        } else {
            analysis.index.symbol_label(&symbol)
        };
        json!({"contents": {"kind": "plaintext", "value":text}})
    }

    fn signature_help(&self, message: &Value) -> Value {
        let Some((uri, text)) = self.text(message) else {
            return Value::Null;
        };
        let Some(analysis) = self.indexed(&uri, &text, position(message)) else {
            return Value::Null;
        };
        let Some((symbol, receiver, active, count, name)) =
            analysis.index.call_site(&analysis.path, analysis.offset)
        else {
            return Value::Null;
        };
        let (labels, documentation): (Vec<(String, Vec<String>)>, Option<&str>) =
            if let Some(api) = analysis.index.native_api(&symbol.id) {
                (
                    api.signatures
                        .iter()
                        .map(|signature| {
                            (
                                analysis
                                    .index
                                    .display_type(&api.label(signature, receiver.as_deref())),
                                signature.parameters.clone(),
                            )
                        })
                        .collect(),
                    Some(api.documentation.as_str()),
                )
            } else if symbol.kind == 5 {
                let parameters = analysis
                    .index
                    .method(&symbol.ty, "dhis")
                    .and_then(|method| method.parameters.clone())
                    .unwrap_or_default();
                let label = analysis.index.display_type(&format!(
                    "cusub {}({})",
                    symbol.name,
                    parameters.join(", ")
                ));
                (vec![(label, parameters)], None)
            } else {
                (
                    vec![(
                        analysis.index.symbol_label(&symbol),
                        symbol.parameters.clone().unwrap_or_default(),
                    )],
                    None,
                )
            };
        let selected = if let Some(api) = analysis.index.native_api(&symbol.id) {
            api.signatures
                .iter()
                .position(|signature| {
                    let (min, max) = signature.bounds();
                    count >= min && count <= max
                })
                .or_else(|| {
                    api.signatures
                        .iter()
                        .position(|signature| signature.bounds().1 >= count)
                })
                .unwrap_or(api.signatures.len() - 1)
        } else {
            0
        };
        let parameters = &labels[selected].1;
        let active = name
            .as_ref()
            .and_then(|name| {
                parameters.iter().position(|parameter| {
                    parameter
                        .split(" = ")
                        .next()
                        .unwrap_or(parameter)
                        .split_whitespace()
                        .last()
                        .unwrap_or("")
                        .trim_start_matches('*')
                        == name
                })
            })
            .unwrap_or(active);
        let signatures: Vec<_> = labels.iter().map(|(label, parameters)| {
            let mut entry = json!({"label":label,"parameters":parameters.iter().map(|label|json!({"label":label})).collect::<Vec<_>>()});
            if let Some(documentation) = documentation {
                entry["documentation"] = json!(documentation);
            }
            entry
        }).collect();
        let mut result = json!({"signatures":signatures,"activeSignature":selected});
        if !parameters.is_empty() {
            result["activeParameter"] = json!(active.min(parameters.len() - 1));
        }
        result
    }

    fn definition(&self, message: &Value) -> Value {
        let Some((symbol, analysis)) = self.symbol_at(message) else {
            return Value::Null;
        };
        let symbol = analysis.index.definition_symbol(&symbol);
        json!({
            "uri": self.uri_for_request(&symbol.location.path,message),
            "range": range_of(analysis.source(&symbol.location.path).unwrap_or(""), symbol.location.start, symbol.location.end)
        })
    }

    fn references(&self, message: &Value) -> Value {
        let Some((symbol, analysis)) = self.symbol_at(message) else {
            return json!([]);
        };
        let include_declaration = message
            .pointer("/params/context/includeDeclaration")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        let items: Vec<Value> = analysis.index
            .occurrences
            .iter()
            .filter(|occurrence| occurrence.symbol == symbol.id && (include_declaration || !occurrence.declaration))
            .map(|occurrence| {
                json!({
                    "uri": self.uri_for_request(&occurrence.location.path,message),
                    "range": range_of(analysis.source(&occurrence.location.path).unwrap_or(""), occurrence.location.start, occurrence.location.end)
                })
            })
            .collect();
        json!(items)
    }

    fn rename(&self, message: &Value) -> Value {
        let Some(new_name) = message.pointer("/params/newName").and_then(Value::as_str) else {
            return Value::Null;
        };
        let Some((symbol, analysis)) = self.symbol_at(message) else {
            return Value::Null;
        };
        // Prefix recovery is suitable for navigation, but cannot prove that
        // references after an actual syntax error have all been indexed.
        if analysis
            .files
            .iter()
            .find(|file| file.path == symbol.location.path)
            .is_some_and(|file| {
                crate::frontend::parse_source(&file.text, Some(&file.path)).is_err()
            })
        {
            return Value::Null;
        }
        if symbol.location.path.starts_with("<soplang>")
            || (!analysis.complete && analysis.index.workspace_symbol(&symbol.id))
        {
            return Value::Null;
        }
        let Ok(tokens) = Lexer::new(new_name).tokenize() else {
            return Value::Null;
        };
        if tokens.len() != 2
            || tokens[0].kind != TokenType::Identifier
            || tokens[0].lexeme != new_name
            || !analysis
                .index
                .rename_preserves_bindings(&symbol.id, new_name)
        {
            return Value::Null;
        }
        let ambiguous = self.ambiguous_paths();
        if analysis.index.occurrences.iter().any(|occurrence| {
            occurrence.symbol == symbol.id && ambiguous.contains(&occurrence.location.path)
        }) {
            return Value::Null;
        }
        let mut changes = BTreeMap::<String, Vec<Value>>::new();
        for occurrence in analysis
            .index
            .occurrences
            .iter()
            .filter(|occurrence| occurrence.symbol == symbol.id)
        {
            let Some(text) = analysis.source(&occurrence.location.path) else {
                return Value::Null;
            };
            changes.entry(self.uri_for_path(&occurrence.location.path)).or_default().push(json!({
                "range": range_of(text, occurrence.location.start, occurrence.location.end), "newText": new_name
            }));
        }
        json!({"changes": changes})
    }

    fn formatting(&self, message: &Value) -> Value {
        let Some((uri, text)) = self.text(message) else {
            return json!([]);
        };
        let formatted = if self.is_schema(&uri) {
            crate::data::format_schema(&text, &path_of(&uri))
        } else {
            crate::cli::tools::format_source(&text, &path_of(&uri))
        };
        match formatted {
            Ok(formatted) => json!([{
                "range": range_of(&text, 0, text.len()),
                "newText": formatted
            }]),
            Err(_) => json!([]),
        }
    }

    fn code_action(&self, message: &Value) -> Value {
        let edits = self.formatting(message);
        if edits.as_array().is_some_and(|edits| edits.is_empty()) {
            return json!([]);
        }
        json!([{
            "title": "Hagaaji qaabka",
            "kind": "source.format",
            "edit": {"changes": {message.pointer("/params/textDocument/uri").and_then(Value::as_str).unwrap_or(""): edits}}
        }])
    }

    fn tokens(&self, message: &Value) -> Value {
        let Some((uri, text)) = self.text(message) else {
            return json!({"data": []});
        };
        if self.is_schema(&uri) {
            return json!({"data": []});
        }
        use crate::frontend::trivia::{lossless_tokens, PieceKind};
        let Ok(pieces) = lossless_tokens(&text) else {
            return json!({"data": []});
        };
        let mut data = Vec::new();
        let mut previous_line = 0u32;
        let mut previous_col = 0u32;
        for piece in pieces {
            let kind = match &piece.kind {
                PieceKind::Token(kind) => token_kind(kind),
                PieceKind::LineComment | PieceKind::BlockComment => Some(5),
                PieceKind::Whitespace => None,
            };
            let Some(kind) = kind else { continue };
            let mut offset = piece.range.start;
            for segment in text[piece.range].split_inclusive('\n') {
                let content = segment.trim_end_matches(['\r', '\n']);
                if !content.is_empty() {
                    let (line, col) = position::at(&text, offset);
                    let (line, col) = (line as u32, col as u32);
                    let delta_line = line - previous_line;
                    let delta_col = if delta_line == 0 {
                        col - previous_col
                    } else {
                        col
                    };
                    data.extend([
                        delta_line,
                        delta_col,
                        content.encode_utf16().count() as u32,
                        kind,
                        0,
                    ]);
                    previous_line = line;
                    previous_col = col;
                }
                offset += segment.len();
            }
        }
        json!({"data": data})
    }

    fn symbol_at(&self, message: &Value) -> Option<(Symbol, Indexed)> {
        let (uri, text) = self.text(message)?;
        let analysis = self.indexed(&uri, &text, position(message))?;
        let occurrence = analysis.index.at(&analysis.path, analysis.offset)?;
        let symbol = analysis.index.symbol(&occurrence.symbol)?.clone();
        Some((symbol, analysis))
    }
}

fn diagnostics(
    uri: String,
    text: &str,
    overlays: &HashMap<PathBuf, String>,
    schema: bool,
) -> Vec<Value> {
    let path = path_of(&uri);
    let mut targets = BTreeMap::<String, Vec<Value>>::from([(uri.clone(), Vec::new())]);
    if schema {
        if let Err(error) = parse_schema_file(text, &path) {
            targets
                .get_mut(&uri)
                .unwrap()
                .push(diagnostic_error(text, &error));
        }
    } else {
        let analysis = check_source(
            text,
            &path,
            false,
            Limits::default(),
            &|imported| overlay_loader(overlays, imported),
            &|| false,
        );
        if let Some(error) = analysis.error {
            let source_path = error
                .meta()
                .file
                .as_deref()
                .map(source_identity)
                .unwrap_or(path.clone());
            let source = analysis
                .files
                .iter()
                .find(|file| file.path == source_path)
                .map(|file| file.text.as_str())
                .unwrap_or(text);
            let target = if source_path == path {
                uri.clone()
            } else {
                uri_of(&source_path)
            };
            targets
                .entry(target)
                .or_default()
                .push(diagnostic_error(source, &error));
        }
    }
    targets
        .into_iter()
        .map(|(uri, diagnostics)| {
            json!({"jsonrpc":"2.0", "method":"textDocument/publishDiagnostics",
        "params": {"uri":uri, "diagnostics":diagnostics}})
        })
        .collect()
}

fn diagnostic_error(source: &str, error: &SoplangError) -> Value {
    let (line, col) = error.location();
    let start = position::scalar_offset(source, line, col);
    let end = position::scalar_offset(
        source,
        error.meta().end_line.unwrap_or(line),
        error.meta().end_col.unwrap_or(col + 1),
    )
    .max(start);
    let mut diagnostic = json!({"range":range_of(source,start,end),"severity":1,"source":"soplang","message":error.message()});
    if let Some(code) = error.code() {
        diagnostic["code"] = json!(code);
    }
    diagnostic
}

fn document_text(
    message: &Value,
    method: &str,
    previous: Option<&str>,
) -> Option<(String, String)> {
    let uri = message
        .pointer("/params/textDocument/uri")
        .and_then(Value::as_str)?
        .to_string();
    let text = if method == "textDocument/didOpen" {
        message
            .pointer("/params/textDocument/text")
            .and_then(Value::as_str)?
            .to_string()
    } else {
        let mut text = previous?.to_string();
        for change in message.pointer("/params/contentChanges")?.as_array()? {
            let inserted = change.get("text")?.as_str()?;
            if let Some(range) = change.get("range") {
                let start = offset_at(
                    &text,
                    range.pointer("/start/line")?.as_u64()?,
                    range.pointer("/start/character")?.as_u64()?,
                )?;
                let end = offset_at(
                    &text,
                    range.pointer("/end/line")?.as_u64()?,
                    range.pointer("/end/character")?.as_u64()?,
                )?;
                if start > end {
                    return None;
                }
                text.replace_range(start..end, inserted);
            } else {
                text = inserted.to_string();
            }
        }
        text
    };
    Some((uri, text))
}

fn position(message: &Value) -> (u64, u64) {
    let line = message
        .pointer("/params/position/line")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let character = message
        .pointer("/params/position/character")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    (line, character)
}

fn bundled_path(uri: &str) -> Option<PathBuf> {
    let uri = url::Url::parse(uri).ok()?;
    if uri.scheme() != "soplang-stdlib"
        || uri.host_str().is_some()
        || uri.query().is_some()
        || uri.fragment().is_some()
    {
        return None;
    }
    let name = uri.path().strip_prefix('/')?;
    if name == "native.sop" {
        return Some(crate::native_api::SOURCE_PATH.into());
    }
    let (name, _) = crate::frontend::imports::bundled_source(name)?;
    Some(PathBuf::from("<soplang>").join(name))
}

fn path_of(uri: &str) -> PathBuf {
    if let Some(path) = bundled_path(uri) {
        return path;
    }
    if let Ok(url) = url::Url::parse(uri) {
        if let Ok(path) = url.to_file_path() {
            return source_identity(&path);
        }
    }
    source_identity(Path::new(uri))
}

fn reference_source(path: &Path) -> Option<&'static str> {
    if path == Path::new(crate::native_api::SOURCE_PATH) {
        return Some(crate::native_api::source());
    }
    path.file_name()
        .and_then(|name| name.to_str())
        .and_then(crate::frontend::imports::bundled_source)
        .map(|(_, text)| text)
}
fn uri_of(path: &Path) -> String {
    if path == Path::new(crate::native_api::SOURCE_PATH) {
        return "soplang-stdlib:/native.sop".into();
    }
    if path.starts_with("<soplang>") {
        if let Some(name) = path
            .file_name()
            .and_then(|name| name.to_str())
            .and_then(crate::frontend::imports::bundled_source)
            .map(|(name, _)| name)
        {
            return format!("soplang-stdlib:/{name}");
        }
    }
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir().unwrap_or_default().join(path)
    };
    url::Url::from_file_path(absolute)
        .map(|url| url.to_string())
        .unwrap_or_default()
}

fn completion_kind(symbol: &Symbol) -> i64 {
    match symbol.kind {
        12 | 6 => 3,
        14 => 21,
        _ => 6,
    }
}

fn token_kind(kind: &TokenType) -> Option<u32> {
    match kind {
        TokenType::Identifier => Some(2),
        TokenType::Number => Some(4),
        TokenType::String | TokenType::FormattedString => Some(3),
        TokenType::Abn
        | TokenType::Jajab
        | TokenType::Qoraal
        | TokenType::Bool
        | TokenType::Teed
        | TokenType::Walax => Some(6),
        TokenType::Door
        | TokenType::Madoor
        | TokenType::Hawl
        | TokenType::Celi
        | TokenType::Qor
        | TokenType::Gelin
        | TokenType::Haddii
        | TokenType::HaddiiKale
        | TokenType::Ugudambeyn
        | TokenType::Dooro
        | TokenType::Xaalad
        | TokenType::Kuceli
        | TokenType::Intay
        | TokenType::Jooji
        | TokenType::Soco
        | TokenType::Fasax
        | TokenType::Qabo
        | TokenType::Keen
        | TokenType::Qaab
        | TokenType::Dhaxal
        | TokenType::Cusub
        | TokenType::Nafta
        | TokenType::Waalid
        | TokenType::True
        | TokenType::False
        | TokenType::Null => Some(0),
        _ => None,
    }
}

fn read_message<R: BufRead>(reader: &mut R) -> Result<Option<String>, SoplangError> {
    const MAX_BODY: usize = 8 * 1024 * 1024;
    const MAX_HEADER: usize = 8 * 1024;
    let mut length = None;
    let mut header_bytes = 0;
    loop {
        let mut line = String::new();
        let mut limited = std::io::Read::take(&mut *reader, (MAX_HEADER - header_bytes + 1) as u64);
        let read = limited
            .read_line(&mut line)
            .map_err(|error| lsp_details("Madaxa fariinta LSP lama akhriyi karin.", error))?;
        if read == 0 {
            return if header_bytes == 0 {
                Ok(None)
            } else {
                Err(lsp_error("Madaxa fariinta LSP ma dhammaystirna."))
            };
        }
        header_bytes += read;
        if header_bytes > MAX_HEADER {
            return Err(lsp_error("Madaxa fariinta LSP aad ayuu u weyn yahay."));
        }
        if line == "\n" || line == "\r\n" {
            break;
        }
        let (name, value) = line
            .split_once(':')
            .ok_or_else(|| lsp_error("Madaxa fariinta LSP ma saxna."))?;
        if name.eq_ignore_ascii_case("content-length") {
            let parsed = value
                .trim()
                .parse::<usize>()
                .map_err(|_| lsp_error("Content-Length ma aha tiro sax ah."))?;
            if parsed > MAX_BODY {
                return Err(lsp_error("Fariinta LSP aad ayay u weyn tahay."));
            }
            if length.replace(parsed).is_some() {
                return Err(lsp_error("Content-Length waa la soo celiyey."));
            }
        }
    }
    let length = length.ok_or_else(|| lsp_error("Fariinta LSP ma leh Content-Length."))?;
    let mut buffer = vec![0; length];
    reader
        .read_exact(&mut buffer)
        .map_err(|error| lsp_details("Fariinta LSP ma dhammaystirna.", error))?;
    // Invalid UTF-8 must produce a parse error, never silently replace source
    // characters and apply a different document update.
    Ok(Some(String::from_utf8(buffer).unwrap_or_default()))
}

fn write_message<W: Write>(writer: &mut W, message: &Value) -> Result<(), SoplangError> {
    let body = serde_json::to_string(message)
        .map_err(|error| lsp_details("Fariinta LSP lama diyaarin karin.", error))?;
    write!(writer, "Content-Length: {}\r\n\r\n{body}", body.len())
        .map_err(|error| lsp_details("Fariinta LSP lama qori karin.", error))?;
    writer
        .flush()
        .map_err(|error| lsp_details("Fariinta LSP lama qori karin.", error))
}

fn lsp_error(message: impl Into<String>) -> SoplangError {
    runtime_error_ex(message, 0, 0, ErrorMeta::default())
}

fn lsp_details(message: &str, error: impl std::fmt::Display) -> SoplangError {
    runtime_error_ex(
        message,
        0,
        0,
        ErrorMeta::default().with_details(error.to_string()),
    )
}
