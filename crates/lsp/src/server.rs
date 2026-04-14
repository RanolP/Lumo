use std::collections::HashMap;
use std::io;

use crate::highlight::{self, HighlightKind};
use lsp_server::{Connection, Message};
use lsp_types::{
    SemanticTokenType, SemanticTokensFullOptions, SemanticTokensLegend, SemanticTokensOptions,
    SemanticTokensServerCapabilities, ServerCapabilities, TextDocumentSyncCapability,
    TextDocumentSyncKind, TextDocumentSyncOptions,
};
use lumo_compiler::lexer::LosslessTokenKind;
use lumo_compiler::query::QueryEngine;
use serde_json::{json, Value};

#[derive(Debug, Default)]
pub struct Server {
    files: HashMap<String, String>,
    query: QueryEngine,
    semantic_cache: HashMap<String, SemanticCacheEntry>,
    semantic_version: u64,
    outgoing_notifications: Vec<String>,
    shutdown_requested: bool,
    should_exit: bool,
}

#[derive(Debug, Clone)]
struct SemanticCacheEntry {
    result_id: String,
    data: Vec<u32>,
}

impl Server {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn should_exit(&self) -> bool {
        self.should_exit
    }

    pub fn take_outgoing_notifications(&mut self) -> Vec<String> {
        std::mem::take(&mut self.outgoing_notifications)
    }

    pub fn handle_json_message(&mut self, msg: &str) -> Option<String> {
        let value: Value = serde_json::from_str(msg).ok()?;
        let method = value.get("method")?.as_str()?;
        let id = value.get("id").cloned();

        match method {
            "initialize" => {
                let id = id?;
                Some(
                    json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "result": {
                            "capabilities": capabilities_json()
                        }
                    })
                    .to_string(),
                )
            }
            "initialized" => None,
            "shutdown" => {
                self.shutdown_requested = true;
                let id = id?;
                Some(json!({ "jsonrpc": "2.0", "id": id, "result": Value::Null }).to_string())
            }
            "exit" => {
                self.should_exit = true;
                None
            }
            "textDocument/didOpen" => {
                if let Some(params) = value.get("params") {
                    self.did_open(params);
                }
                None
            }
            "textDocument/didChange" => {
                if let Some(params) = value.get("params") {
                    self.did_change(params);
                }
                None
            }
            "textDocument/didClose" => {
                if let Some(params) = value.get("params") {
                    self.did_close(params);
                }
                None
            }
            "textDocument/semanticTokens/full" => {
                let id = id?;
                let uri = value
                    .get("params")
                    .and_then(|p| p.get("textDocument"))
                    .and_then(|td| td.get("uri"))
                    .and_then(Value::as_str)
                    .or_else(|| {
                        value
                            .get("params")
                            .and_then(|p| p.get("uri"))
                            .and_then(Value::as_str)
                    })
                    .unwrap_or_default();

                let data = self
                    .query
                    .parse(uri)
                    .map(|parsed| {
                        let src = self.files.get(uri).map(String::as_str).unwrap_or("");
                        semantic_tokens_data_from_lossless(src, &parsed.lossless.root)
                    })
                    .unwrap_or_default();

                let result_id = self.next_semantic_result_id();
                self.semantic_cache.insert(
                    uri.to_owned(),
                    SemanticCacheEntry {
                        result_id: result_id.clone(),
                        data: data.clone(),
                    },
                );

                Some(
                    json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "result": { "resultId": result_id, "data": data }
                    })
                    .to_string(),
                )
            }
            "textDocument/semanticTokens/full/delta" => {
                let id = id?;
                let uri = value
                    .get("params")
                    .and_then(|p| p.get("textDocument"))
                    .and_then(|td| td.get("uri"))
                    .and_then(Value::as_str)
                    .unwrap_or_default();
                let previous_result_id = value
                    .get("params")
                    .and_then(|p| p.get("previousResultId"))
                    .and_then(Value::as_str)
                    .unwrap_or_default();

                let new_data = self
                    .query
                    .parse(uri)
                    .map(|parsed| {
                        let src = self.files.get(uri).map(String::as_str).unwrap_or("");
                        semantic_tokens_data_from_lossless(src, &parsed.lossless.root)
                    })
                    .unwrap_or_default();

                let new_result_id = self.next_semantic_result_id();
                let result = if let Some(old) = self.semantic_cache.get(uri) {
                    if old.result_id == previous_result_id {
                        let edits = compute_semantic_token_edits(&old.data, &new_data);
                        json!({ "resultId": new_result_id.clone(), "edits": edits })
                    } else {
                        json!({ "resultId": new_result_id.clone(), "data": new_data.clone() })
                    }
                } else {
                    json!({ "resultId": new_result_id.clone(), "data": new_data.clone() })
                };

                self.semantic_cache.insert(
                    uri.to_owned(),
                    SemanticCacheEntry {
                        result_id: new_result_id,
                        data: new_data,
                    },
                );

                Some(
                    json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "result": result
                    })
                    .to_string(),
                )
            }
            _ => id.map(|id| {
                json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "error": {
                        "code": -32601,
                        "message": "Method not found"
                    }
                })
                .to_string()
            }),
        }
    }

    fn did_open(&mut self, params: &Value) {
        let text_document = params.get("textDocument").unwrap_or(params);
        let Some(uri) = text_document.get("uri").and_then(Value::as_str) else {
            return;
        };
        let text = text_document
            .get("text")
            .and_then(Value::as_str)
            .unwrap_or_default();

        self.files.insert(uri.to_owned(), text.to_owned());
        self.query.set_file(uri.to_owned(), text.to_owned());
        self.enqueue_diagnostics(uri);
    }

    fn did_change(&mut self, params: &Value) {
        let uri = params
            .get("textDocument")
            .and_then(|td| td.get("uri"))
            .and_then(Value::as_str)
            .or_else(|| params.get("uri").and_then(Value::as_str))
            .unwrap_or_default();

        if uri.is_empty() {
            return;
        }

        let mut text = self.files.get(uri).cloned().unwrap_or_default();
        let mut changed = false;

        if let Some(changes) = params.get("contentChanges").and_then(Value::as_array) {
            for change in changes {
                let Some(new_text) = change.get("text").and_then(Value::as_str) else {
                    continue;
                };

                if let Some(range) = change.get("range") {
                    if !apply_incremental_change(&mut text, range, new_text) {
                        text = new_text.to_owned();
                    }
                    changed = true;
                } else {
                    text = new_text.to_owned();
                    changed = true;
                }
            }
        }

        if !changed {
            return;
        }

        self.files.insert(uri.to_owned(), text.to_owned());
        self.query.set_file(uri.to_owned(), text.to_owned());
        self.enqueue_diagnostics(uri);
    }

    fn did_close(&mut self, params: &Value) {
        let uri = params
            .get("textDocument")
            .and_then(|td| td.get("uri"))
            .and_then(Value::as_str)
            .or_else(|| params.get("uri").and_then(Value::as_str))
            .unwrap_or_default();
        if uri.is_empty() {
            return;
        }

        self.files.remove(uri);
        self.semantic_cache.remove(uri);
        let _ = self.query.remove_file(uri);

        self.outgoing_notifications.push(
            json!({
                "jsonrpc": "2.0",
                "method": "textDocument/publishDiagnostics",
                "params": {
                    "uri": uri,
                    "diagnostics": []
                }
            })
            .to_string(),
        );
    }

    fn enqueue_diagnostics(&mut self, uri: &str) {
        let Some(diags) = self.query.diagnostics(uri) else {
            return;
        };
        let source = self.files.get(uri).map(String::as_str).unwrap_or("");

        let diagnostics = diags
            .into_iter()
            .map(|d| {
                let (start_line, start_char) = byte_to_lsp_position(source, d.start);
                let (end_line, end_char) = byte_to_lsp_position(source, d.end);
                json!({
                    "range": {
                        "start": { "line": start_line, "character": start_char },
                        "end": { "line": end_line, "character": end_char }
                    },
                    "severity": 1,
                    "message": d.message,
                    "source": "lumo"
                })
            })
            .collect::<Vec<_>>();

        self.outgoing_notifications.push(
            json!({
                "jsonrpc": "2.0",
                "method": "textDocument/publishDiagnostics",
                "params": {
                    "uri": uri,
                    "diagnostics": diagnostics
                }
            })
            .to_string(),
        );
    }

    fn next_semantic_result_id(&mut self) -> String {
        self.semantic_version = self.semantic_version.wrapping_add(1);
        format!("sem-{}", self.semantic_version)
    }
}

pub fn semantic_tokens_data(source: &str) -> Vec<u32> {
    let mut tokens = highlight::highlight(source);
    tokens.sort_by_key(|t| (t.start, t.end));

    encode_highlight_tokens(source, &tokens)
}

fn semantic_tokens_data_from_lossless(
    source: &str,
    root: &lumo_compiler::lst::lossless::SyntaxNode,
) -> Vec<u32> {
    let mut tokens = Vec::new();
    let mut state = AttrSemanticState::default();
    collect_lossless_tokens(root, &mut tokens, &mut state);
    tokens.sort_by_key(|t| (t.start, t.end));

    encode_highlight_tokens(source, &tokens)
}

fn encode_highlight_tokens(source: &str, tokens: &[highlight::HighlightToken]) -> Vec<u32> {
    let mut out = Vec::with_capacity(tokens.len() * 5);
    let mut prev_line = 0_u32;
    let mut prev_start = 0_u32;

    for token in tokens {
        let (line, start_char) = byte_to_lsp_position(source, token.start);
        let (_, end_char) = byte_to_lsp_position(source, token.end);
        let length = end_char.saturating_sub(start_char);
        if length == 0 {
            continue;
        }

        let delta_line = line.saturating_sub(prev_line);
        let delta_start = if delta_line == 0 {
            start_char.saturating_sub(prev_start)
        } else {
            start_char
        };

        out.push(delta_line);
        out.push(delta_start);
        out.push(length);
        out.push(token_type_index(token.kind.clone()));
        out.push(0);

        prev_line = line;
        prev_start = start_char;
    }

    out
}

fn collect_lossless_tokens(
    node: &lumo_compiler::lst::lossless::SyntaxNode,
    out: &mut Vec<highlight::HighlightToken>,
    state: &mut AttrSemanticState,
) {
    for child in &node.children {
        match child {
            lumo_compiler::lst::lossless::SyntaxElement::Node(n) => {
                collect_lossless_tokens(n, out, state)
            }
            lumo_compiler::lst::lossless::SyntaxElement::Token(t) => {
                let kind = match t.kind {
                    LosslessTokenKind::Keyword(_) if state.expect_attr_name && state.depth > 0 => {
                        Some(HighlightKind::Identifier)
                    }
                    LosslessTokenKind::Keyword(_) => Some(HighlightKind::Keyword),
                    LosslessTokenKind::Ident if t.text == "type" => Some(HighlightKind::Keyword),
                    LosslessTokenKind::Ident => Some(HighlightKind::Identifier),
                    LosslessTokenKind::StringLit => Some(HighlightKind::String),
                    LosslessTokenKind::NumberLit => Some(HighlightKind::Number),
                    LosslessTokenKind::Symbol(_) => Some(HighlightKind::Symbol),
                    LosslessTokenKind::Whitespace
                    | LosslessTokenKind::Newline
                    | LosslessTokenKind::Unknown => None,
                };

                if let Some(kind) = kind {
                    out.push(highlight::HighlightToken {
                        start: t.span.start,
                        end: t.span.end,
                        kind,
                    });
                }

                state.observe(&t.kind, &t.text);
            }
        }
    }
}

#[derive(Default)]
struct AttrSemanticState {
    pending_hash: bool,
    depth: usize,
    expect_attr_name: bool,
}

impl AttrSemanticState {
    fn observe(&mut self, kind: &LosslessTokenKind, text: &str) {
        match kind {
            LosslessTokenKind::Ident | LosslessTokenKind::Keyword(_) => {
                self.pending_hash = false;
                if self.expect_attr_name && self.depth > 0 {
                    self.expect_attr_name = false;
                }
            }
            LosslessTokenKind::Whitespace | LosslessTokenKind::Newline => {}
            _ => {
                self.pending_hash = false;
            }
        }

        match text {
            "#" => {
                self.pending_hash = true;
            }
            "[" => {
                if self.pending_hash {
                    self.pending_hash = false;
                    self.depth = 1;
                    self.expect_attr_name = true;
                } else if self.depth > 0 {
                    self.depth += 1;
                }
            }
            "]" => {
                self.pending_hash = false;
                if self.depth > 0 {
                    self.depth -= 1;
                    if self.depth == 0 {
                        self.expect_attr_name = false;
                    }
                }
            }
            _ => {}
        }
    }
}

fn token_type_index(kind: HighlightKind) -> u32 {
    match kind {
        HighlightKind::Keyword => 0,
        HighlightKind::Identifier => 1,
        HighlightKind::String => 2,
        HighlightKind::Number => 3,
        HighlightKind::Symbol => 4,
    }
}

fn byte_to_lsp_position(source: &str, byte_offset: usize) -> (u32, u32) {
    let clamped = byte_offset.min(source.len());
    let mut line = 0_u32;
    let mut col_utf16 = 0_u32;
    let mut idx = 0_usize;

    for ch in source.chars() {
        if idx >= clamped {
            break;
        }

        let len = ch.len_utf8();
        if idx + len > clamped {
            break;
        }

        if ch == '\n' {
            line += 1;
            col_utf16 = 0;
        } else {
            col_utf16 += ch.len_utf16() as u32;
        }

        idx += len;
    }

    (line, col_utf16)
}

fn capabilities_json() -> Value {
    let caps = ServerCapabilities {
        text_document_sync: Some(TextDocumentSyncCapability::Options(
            TextDocumentSyncOptions {
                open_close: Some(true),
                change: Some(TextDocumentSyncKind::INCREMENTAL),
                will_save: None,
                will_save_wait_until: None,
                save: None,
            },
        )),
        semantic_tokens_provider: Some(SemanticTokensServerCapabilities::SemanticTokensOptions(
            SemanticTokensOptions {
                work_done_progress_options: Default::default(),
                legend: SemanticTokensLegend {
                    token_types: vec![
                        SemanticTokenType::KEYWORD,
                        SemanticTokenType::VARIABLE,
                        SemanticTokenType::STRING,
                        SemanticTokenType::NUMBER,
                        SemanticTokenType::OPERATOR,
                    ],
                    token_modifiers: vec![],
                },
                range: None,
                full: Some(SemanticTokensFullOptions::Delta { delta: Some(true) }),
            },
        )),
        ..Default::default()
    };
    serde_json::to_value(caps).expect("server capabilities must serialize")
}

fn compute_semantic_token_edits(old: &[u32], new: &[u32]) -> Vec<Value> {
    let mut prefix = 0usize;
    let limit = old.len().min(new.len());
    while prefix < limit && old[prefix] == new[prefix] {
        prefix += 1;
    }

    let mut old_suffix = old.len();
    let mut new_suffix = new.len();
    while old_suffix > prefix && new_suffix > prefix && old[old_suffix - 1] == new[new_suffix - 1] {
        old_suffix -= 1;
        new_suffix -= 1;
    }

    let delete_count = old_suffix.saturating_sub(prefix);
    let data = new[prefix..new_suffix].to_vec();

    vec![json!({
        "start": prefix,
        "deleteCount": delete_count,
        "data": data
    })]
}

fn apply_incremental_change(source: &mut String, range: &Value, new_text: &str) -> bool {
    let start_line = range
        .get("start")
        .and_then(|s| s.get("line"))
        .and_then(Value::as_u64)
        .and_then(|n| usize::try_from(n).ok());
    let start_char = range
        .get("start")
        .and_then(|s| s.get("character"))
        .and_then(Value::as_u64)
        .and_then(|n| usize::try_from(n).ok());
    let end_line = range
        .get("end")
        .and_then(|s| s.get("line"))
        .and_then(Value::as_u64)
        .and_then(|n| usize::try_from(n).ok());
    let end_char = range
        .get("end")
        .and_then(|s| s.get("character"))
        .and_then(Value::as_u64)
        .and_then(|n| usize::try_from(n).ok());

    let (Some(start_line), Some(start_char), Some(end_line), Some(end_char)) =
        (start_line, start_char, end_line, end_char)
    else {
        return false;
    };

    let Some(start_byte) = lsp_position_to_byte_offset(source, start_line, start_char) else {
        return false;
    };
    let Some(end_byte) = lsp_position_to_byte_offset(source, end_line, end_char) else {
        return false;
    };
    if start_byte > end_byte || end_byte > source.len() {
        return false;
    }

    source.replace_range(start_byte..end_byte, new_text);
    true
}

fn lsp_position_to_byte_offset(source: &str, line: usize, character_utf16: usize) -> Option<usize> {
    let line_start = line_start_byte_offset(source, line)?;
    let mut col = 0usize;
    let mut byte = line_start;

    for ch in source[line_start..].chars() {
        if ch == '\n' {
            break;
        }

        if col == character_utf16 {
            return Some(byte);
        }

        let width = ch.len_utf16();
        if col + width > character_utf16 {
            return None;
        }
        col += width;
        byte += ch.len_utf8();
    }

    if col == character_utf16 {
        Some(byte)
    } else {
        None
    }
}

fn line_start_byte_offset(source: &str, target_line: usize) -> Option<usize> {
    if target_line == 0 {
        return Some(0);
    }

    let mut line = 0usize;
    for (idx, ch) in source.char_indices() {
        if ch == '\n' {
            line += 1;
            if line == target_line {
                return Some(idx + 1);
            }
        }
    }
    None
}

pub fn run_stdio_server() -> io::Result<()> {
    let (connection, io_threads) = Connection::stdio();
    connection
        .initialize(capabilities_json())
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;

    let mut server = Server::new();

    for msg in &connection.receiver {
        match msg {
            Message::Request(req) => {
                if connection
                    .handle_shutdown(&req)
                    .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?
                {
                    break;
                }
                let raw = serde_json::to_string(&req)
                    .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
                if let Some(resp) = server.handle_json_message(&raw) {
                    let message: Message = serde_json::from_str(&resp)
                        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
                    connection
                        .sender
                        .send(message)
                        .map_err(|e| io::Error::new(io::ErrorKind::BrokenPipe, e.to_string()))?;
                }
            }
            Message::Notification(not) => {
                let raw = serde_json::to_string(&not)
                    .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
                let _ = server.handle_json_message(&raw);
            }
            Message::Response(_) => {}
        }

        for note in server.take_outgoing_notifications() {
            let message: Message = serde_json::from_str(&note)
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
            connection
                .sender
                .send(message)
                .map_err(|e| io::Error::new(io::ErrorKind::BrokenPipe, e.to_string()))?;
        }

        if server.should_exit() {
            break;
        }
    }

    io_threads
        .join()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{semantic_tokens_data, Server};

    #[test]
    fn initialize_returns_semantic_token_capabilities() {
        let mut server = Server::new();
        let msg = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#;
        let resp = server.handle_json_message(msg).expect("response expected");

        assert!(resp.contains("semanticTokensProvider"));
        assert!(resp.contains("tokenTypes"));
    }

    #[test]
    fn semantic_tokens_request_uses_opened_document() {
        let mut server = Server::new();
        let open = r#"{"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"textDocument":{"uri":"file:///main.lumo","text":"fn id() { x }"}}}"#;
        server.handle_json_message(open);

        let req = r#"{"jsonrpc":"2.0","id":2,"method":"textDocument/semanticTokens/full","params":{"textDocument":{"uri":"file:///main.lumo"}}}"#;
        let resp = server.handle_json_message(req).expect("response expected");

        assert!(resp.contains("\"data\":["));
        assert!(!resp.contains("\"data\":[]"));
    }

    #[test]
    fn semantic_tokens_delta_returns_edits() {
        let mut server = Server::new();
        let open = r#"{"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"textDocument":{"uri":"file:///main.lumo","text":"fn id() { x }"}}}"#;
        server.handle_json_message(open);

        let full = r#"{"jsonrpc":"2.0","id":2,"method":"textDocument/semanticTokens/full","params":{"textDocument":{"uri":"file:///main.lumo"}}}"#;
        let full_resp = server.handle_json_message(full).expect("full response");
        let full_json: serde_json::Value = serde_json::from_str(&full_resp).expect("valid json");
        let result_id = full_json
            .get("result")
            .and_then(|r| r.get("resultId"))
            .and_then(serde_json::Value::as_str)
            .expect("resultId")
            .to_owned();

        let change = r#"{"jsonrpc":"2.0","method":"textDocument/didChange","params":{"textDocument":{"uri":"file:///main.lumo"},"contentChanges":[{"text":"fn id() { y }"}]}}"#;
        server.handle_json_message(change);

        let delta = format!(
            "{{\"jsonrpc\":\"2.0\",\"id\":3,\"method\":\"textDocument/semanticTokens/full/delta\",\"params\":{{\"textDocument\":{{\"uri\":\"file:///main.lumo\"}},\"previousResultId\":\"{}\"}}}}",
            result_id
        );
        let delta_resp = server.handle_json_message(&delta).expect("delta response");
        assert!(delta_resp.contains("\"edits\""));
    }

    #[test]
    fn semantic_tokens_delta_falls_back_to_full_on_stale_result_id() {
        let mut server = Server::new();
        let open = r#"{"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"textDocument":{"uri":"file:///main.lumo","text":"fn id() { x }"}}}"#;
        server.handle_json_message(open);

        let _ = server
            .handle_json_message(
                r#"{"jsonrpc":"2.0","id":2,"method":"textDocument/semanticTokens/full","params":{"textDocument":{"uri":"file:///main.lumo"}}}"#,
            )
            .expect("full response");

        let change = r#"{"jsonrpc":"2.0","method":"textDocument/didChange","params":{"textDocument":{"uri":"file:///main.lumo"},"contentChanges":[{"text":"fn id() { y }"}]}}"#;
        server.handle_json_message(change);

        let stale_delta = r#"{"jsonrpc":"2.0","id":3,"method":"textDocument/semanticTokens/full/delta","params":{"textDocument":{"uri":"file:///main.lumo"},"previousResultId":"stale-id"}}"#;
        let resp = server
            .handle_json_message(stale_delta)
            .expect("delta response");

        assert!(resp.contains("\"data\""));
        assert!(!resp.contains("\"edits\""));
    }
    #[test]
    fn diagnostics_are_published_on_open_and_change() {
        let mut server = Server::new();
        let open = r#"{"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"textDocument":{"uri":"file:///main.lumo","text":"fn id() { + }"}}}"#;
        server.handle_json_message(open);

        let notes = server.take_outgoing_notifications();
        assert_eq!(notes.len(), 1);
        assert!(notes[0].contains("textDocument/publishDiagnostics"));
        assert!(notes[0].contains("\"diagnostics\":["));
        assert!(!notes[0].contains("\"diagnostics\":[]"));

        let change = r#"{"jsonrpc":"2.0","method":"textDocument/didChange","params":{"textDocument":{"uri":"file:///main.lumo"},"contentChanges":[{"text":"fn id[A](a: A): A / {} { a }"}]}}"#;
        server.handle_json_message(change);

        let notes = server.take_outgoing_notifications();
        assert_eq!(notes.len(), 1);
        assert!(notes[0].contains("\"diagnostics\":[]"));
    }

    #[test]
    fn did_close_clears_diagnostics() {
        let mut server = Server::new();
        let open = r#"{"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"textDocument":{"uri":"file:///main.lumo","text":"fn id() { + }"}}}"#;
        server.handle_json_message(open);
        let _ = server.take_outgoing_notifications();

        let close = r#"{"jsonrpc":"2.0","method":"textDocument/didClose","params":{"textDocument":{"uri":"file:///main.lumo"}}}"#;
        server.handle_json_message(close);
        let notes = server.take_outgoing_notifications();

        assert_eq!(notes.len(), 1);
        assert!(notes[0].contains("textDocument/publishDiagnostics"));
        assert!(notes[0].contains("\"diagnostics\":[]"));
    }

    #[test]
    fn did_change_incremental_range_is_applied() {
        let mut server = Server::new();
        let open = r#"{"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"textDocument":{"uri":"file:///main.lumo","text":"fn id() { x }"}}}"#;
        server.handle_json_message(open);
        let _ = server.take_outgoing_notifications();

        let inc = r#"{"jsonrpc":"2.0","method":"textDocument/didChange","params":{"textDocument":{"uri":"file:///main.lumo"},"contentChanges":[{"range":{"start":{"line":0,"character":10},"end":{"line":0,"character":10}},"text":"+"}]}}"#;
        server.handle_json_message(inc);

        let notes = server.take_outgoing_notifications();
        assert_eq!(notes.len(), 1);
        assert!(!notes[0].contains("\"diagnostics\":[]"));

        let req = r#"{"jsonrpc":"2.0","id":9,"method":"textDocument/semanticTokens/full","params":{"textDocument":{"uri":"file:///main.lumo"}}}"#;
        let resp = server.handle_json_message(req).expect("response");
        assert!(!resp.contains("\"data\":[]"));
    }

    #[test]
    fn did_change_incremental_handles_utf16_positions() {
        let mut server = Server::new();
        let open = r#"{"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"textDocument":{"uri":"file:///main.lumo","text":"fn id() { 😀x }"}}}"#;
        server.handle_json_message(open);
        let _ = server.take_outgoing_notifications();

        let inc = r#"{"jsonrpc":"2.0","method":"textDocument/didChange","params":{"textDocument":{"uri":"file:///main.lumo"},"contentChanges":[{"range":{"start":{"line":0,"character":12},"end":{"line":0,"character":13}},"text":"y"}]}}"#;
        server.handle_json_message(inc);

        let req = r#"{"jsonrpc":"2.0","id":10,"method":"textDocument/semanticTokens/full","params":{"textDocument":{"uri":"file:///main.lumo"}}}"#;
        let resp = server.handle_json_message(req).expect("response");
        assert!(!resp.contains("\"data\":[]"));
    }

    #[test]
    fn did_change_applies_multiple_content_changes_in_order() {
        let mut server = Server::new();
        let open = r#"{"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"textDocument":{"uri":"file:///main.lumo","text":"fn id() { x }"}}}"#;
        server.handle_json_message(open);
        let _ = server.take_outgoing_notifications();

        let inc = r#"{"jsonrpc":"2.0","method":"textDocument/didChange","params":{"textDocument":{"uri":"file:///main.lumo"},"contentChanges":[{"range":{"start":{"line":0,"character":10},"end":{"line":0,"character":11}},"text":"z"},{"range":{"start":{"line":0,"character":0},"end":{"line":0,"character":0}},"text":"+"}]}}"#;
        server.handle_json_message(inc);
        let _ = server.take_outgoing_notifications();

        let req = r#"{"jsonrpc":"2.0","id":11,"method":"textDocument/semanticTokens/full","params":{"textDocument":{"uri":"file:///main.lumo"}}}"#;
        let resp = server.handle_json_message(req).expect("response");
        let json: serde_json::Value = serde_json::from_str(&resp).expect("valid json");
        let data = json
            .get("result")
            .and_then(|r| r.get("data"))
            .and_then(serde_json::Value::as_array)
            .expect("semantic data");
        assert!(!data.is_empty());
    }

    #[test]
    fn highlighting_survives_syntax_error() {
        let data = semantic_tokens_data("fn id() { + } x");
        assert!(!data.is_empty());
    }
}
