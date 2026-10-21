//! Stdio language server for `.sop` and `schema.sop`.

use std::collections::HashMap;
use std::io::{BufRead, Write};
use std::path::{Path, PathBuf};

use serde_json::{json, Value};

use crate::analysis::index::{Index, Symbol};
use crate::analysis::{check_source, Limits};
use crate::data::parse_schema;
use crate::error::{runtime_error_ex, ErrorMeta, SoplangError};
use crate::frontend::{Lexer, TokenType};

#[derive(Default)]
pub struct Server {
    documents: HashMap<String, String>,
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
            break;
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
        if method == "exit" {
            self.exit = true;
            return Vec::new();
        }
        let result = match method {
            "initialize" => json!({
                "capabilities": {
                    "textDocumentSync": 1,
                    "completionProvider": {"triggerCharacters": ["."]},
                    "hoverProvider": true,
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
            }),
            "shutdown" => Value::Null,
            "textDocument/didOpen" | "textDocument/didChange" => {
                if let Some((uri, text)) = document_text(&message, method) {
                    let notice = diagnostics(uri.clone(), &text);
                    self.documents.insert(uri, text);
                    return vec![notice];
                }
                return Vec::new();
            }
            "textDocument/completion" => self.completion(&message),
            "textDocument/hover" => self.hover(&message),
            "textDocument/definition" => self.definition(&message),
            "textDocument/references" => self.references(&message),
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

    fn completion(&self, message: &Value) -> Value {
        let Some((uri, text)) = self.text(message) else {
            return json!([]);
        };
        let Some((index, path, offset)) = indexed(&uri, &text, position(message)) else {
            return json!([]);
        };
        let items: Vec<Value> = index
            .visible(&path, offset)
            .into_iter()
            .map(|symbol| json!({"label": symbol.name, "kind": completion_kind(symbol)}))
            .collect();
        json!(items)
    }

    fn hover(&self, message: &Value) -> Value {
        let Some((symbol, _, text)) = self.symbol_at(message) else {
            return Value::Null;
        };
        json!({"contents": {"kind": "plaintext", "value": format!("{}: {}", symbol.name, symbol.ty)}})
            .pipe_text(&text)
    }

    fn definition(&self, message: &Value) -> Value {
        let Some((symbol, _, text)) = self.symbol_at(message) else {
            return Value::Null;
        };
        json!({
            "uri": uri_of(&symbol.location.path),
            "range": range_of(&text, symbol.location.start, symbol.location.end)
        })
    }

    fn references(&self, message: &Value) -> Value {
        let Some((symbol, index, text)) = self.symbol_at(message) else {
            return json!([]);
        };
        let items: Vec<Value> = index
            .occurrences
            .iter()
            .filter(|occurrence| occurrence.symbol == symbol.id)
            .map(|occurrence| {
                json!({
                    "uri": uri_of(&occurrence.location.path),
                    "range": range_of(&text, occurrence.location.start, occurrence.location.end)
                })
            })
            .collect();
        json!(items)
    }

    fn rename(&self, message: &Value) -> Value {
        let Some(new_name) = message.pointer("/params/newName").and_then(Value::as_str) else {
            return Value::Null;
        };
        let Some((symbol, index, text)) = self.symbol_at(message) else {
            return Value::Null;
        };
        if !index.rename_preserves_bindings(&symbol.id, new_name) {
            return Value::Null;
        }
        let edits: Vec<Value> = index
            .occurrences
            .iter()
            .filter(|occurrence| occurrence.symbol == symbol.id)
            .map(|occurrence| {
                json!({"range": range_of(&text, occurrence.location.start, occurrence.location.end), "newText": new_name})
            })
            .collect();
        let uri = message
            .pointer("/params/textDocument/uri")
            .and_then(Value::as_str)
            .unwrap_or("");
        json!({"changes": {uri: edits}})
    }

    fn formatting(&self, message: &Value) -> Value {
        let Some((uri, text)) = self.text(message) else {
            return json!([]);
        };
        match crate::cli::tools::format_source(&text, &path_of(&uri)) {
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
        let Some((_, text)) = self.text(message) else {
            return json!({"data": []});
        };
        let Ok(tokens) = Lexer::new(&text).tokenize() else {
            return json!({"data": []});
        };
        let mut data = Vec::new();
        let mut previous_line = 0u32;
        let mut previous_col = 0u32;
        for token in tokens {
            let Some(kind) = token_kind(&token.kind) else {
                continue;
            };
            let line = token.line.saturating_sub(1) as u32;
            let col = token.col.saturating_sub(1) as u32;
            let delta_line = line.saturating_sub(previous_line);
            let delta_col = if delta_line == 0 {
                col.saturating_sub(previous_col)
            } else {
                col
            };
            data.extend([
                delta_line,
                delta_col,
                token.lexeme.chars().count() as u32,
                kind,
                0,
            ]);
            previous_line = line;
            previous_col = col;
        }
        json!({"data": data})
    }

    fn symbol_at(&self, message: &Value) -> Option<(Symbol, Index, String)> {
        let (uri, text) = self.text(message)?;
        let (index, path, offset) = indexed(&uri, &text, position(message))?;
        let occurrence = index.at(&path, offset)?;
        let symbol = index.symbol(&occurrence.symbol)?.clone();
        Some((symbol, index, text))
    }
}

trait HoverText {
    fn pipe_text(self, _text: &str) -> Value;
}

impl HoverText for Value {
    fn pipe_text(self, _text: &str) -> Value {
        self
    }
}

fn indexed(uri: &str, text: &str, position: (u64, u64)) -> Option<(Index, PathBuf, usize)> {
    let path = path_of(uri);
    let analysis = check_source(
        text,
        &path,
        false,
        Limits::default(),
        &|imported| std::fs::read_to_string(imported).map_err(|error| error.to_string()),
        &|| false,
    );
    let index = Index::build(&analysis.files);
    let offset = offset_at(text, position.0, position.1);
    Some((index, path, offset))
}

fn diagnostics(uri: String, text: &str) -> Value {
    let path = path_of(&uri);
    let mut items = Vec::new();
    if path.file_name().and_then(|name| name.to_str()) == Some("schema.sop") {
        if let Err(error) = parse_schema(text) {
            items.push(diagnostic_message(1, 1, &error));
        }
    } else {
        let analysis = check_source(
            text,
            &path,
            false,
            Limits::default(),
            &|imported| std::fs::read_to_string(imported).map_err(|error| error.to_string()),
            &|| false,
        );
        if let Some(error) = analysis.error {
            let (line, col) = error.location();
            items.push(diagnostic_message(line, col, error.message()));
        }
    }
    json!({
        "jsonrpc": "2.0",
        "method": "textDocument/publishDiagnostics",
        "params": {"uri": uri, "diagnostics": items}
    })
}

fn diagnostic_message(line: usize, col: usize, message: &str) -> Value {
    json!({
        "range": {
            "start": {"line": line.saturating_sub(1), "character": col.saturating_sub(1)},
            "end": {"line": line.saturating_sub(1), "character": col}
        },
        "severity": 1,
        "source": "soplang",
        "message": message
    })
}

fn document_text(message: &Value, method: &str) -> Option<(String, String)> {
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
        message
            .pointer("/params/contentChanges/0/text")
            .and_then(Value::as_str)?
            .to_string()
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

fn path_of(uri: &str) -> PathBuf {
    let path = uri.strip_prefix("file://").unwrap_or(uri);
    PathBuf::from(path)
}

fn uri_of(path: &Path) -> String {
    if path.as_os_str().is_empty() {
        "file://".into()
    } else {
        format!("file://{}", path.display())
    }
}

fn offset_at(text: &str, line: u64, character: u64) -> usize {
    let mut current_line = 0u64;
    let mut current_char = 0u64;
    for (index, ch) in text.char_indices() {
        if current_line == line && current_char == character {
            return index;
        }
        if ch == '\n' {
            current_line += 1;
            current_char = 0;
        } else {
            current_char += 1;
        }
    }
    text.len()
}

fn range_of(text: &str, start: usize, end: usize) -> Value {
    let (start_line, start_char) = position_at(text, start);
    let (end_line, end_char) = position_at(text, end);
    json!({
        "start": {"line": start_line, "character": start_char},
        "end": {"line": end_line, "character": end_char}
    })
}

fn position_at(text: &str, offset: usize) -> (u64, u64) {
    let mut line = 0u64;
    let mut character = 0u64;
    for (index, ch) in text.char_indices() {
        if index >= offset {
            break;
        }
        if ch == '\n' {
            line += 1;
            character = 0;
        } else {
            character += 1;
        }
    }
    (line, character)
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
    let mut length = None;
    loop {
        let mut line = String::new();
        let read = reader
            .read_line(&mut line)
            .map_err(|error| lsp_error(error.to_string()))?;
        if read == 0 {
            return Ok(None);
        }
        if line == "\n" || line == "\r\n" {
            break;
        }
        if let Some(value) = line.to_ascii_lowercase().strip_prefix("content-length:") {
            length = value.trim().parse::<usize>().ok();
        }
    }
    let Some(length) = length else {
        return Err(lsp_error("Fariinta LSP ma leh Content-Length."));
    };
    let mut buffer = vec![0; length];
    reader
        .read_exact(&mut buffer)
        .map_err(|error| lsp_error(error.to_string()))?;
    Ok(Some(String::from_utf8_lossy(&buffer).into_owned()))
}

fn write_message<W: Write>(writer: &mut W, message: &Value) -> Result<(), SoplangError> {
    let body = serde_json::to_string(message).map_err(|error| lsp_error(error.to_string()))?;
    write!(writer, "Content-Length: {}\r\n\r\n{body}", body.len())
        .map_err(|error| lsp_error(error.to_string()))?;
    writer.flush().map_err(|error| lsp_error(error.to_string()))
}

fn lsp_error(message: impl Into<String>) -> SoplangError {
    runtime_error_ex(message, 0, 0, ErrorMeta::default())
}
