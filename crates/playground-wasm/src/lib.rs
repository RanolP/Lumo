#[cfg(target_arch = "wasm32")]
mod wasm {
    use lumo_compiler::{
        backend::{self, BackendError, CodegenTarget},
        query::QueryEngine,
    };
    use lumo_lsp::server::Server;
    use serde::Serialize;
    use serde_wasm_bindgen::to_value;
    use wasm_bindgen::prelude::*;

    #[derive(Debug, Serialize)]
    struct LspBridgeResult {
        response: Option<String>,
        notifications: Vec<String>,
        should_exit: bool,
    }

    #[derive(Debug, Serialize)]
    struct DiagnosticView {
        start: usize,
        end: usize,
        start_line: u32,
        start_character: u32,
        end_line: u32,
        end_character: u32,
        message: String,
    }

    #[derive(Debug, Serialize)]
    struct CompilerStatsView {
        parse_computes: usize,
        lower_computes: usize,
        diagnostics_computes: usize,
    }

    #[derive(Debug, Serialize)]
    struct CompilerRunResult {
        parse_errors: Vec<DiagnosticView>,
        diagnostics: Vec<DiagnosticView>,
        stats: CompilerStatsView,
        typed_ast: String,
        lowered_typed_ast: String,
        lowered: String,
        emitted_ts: String,
        emitted_d_ts: String,
        emitted_js: String,
        backend_errors: Vec<String>,
    }

    #[wasm_bindgen]
    pub struct WasmLsp {
        server: Server,
    }

    #[wasm_bindgen]
    impl WasmLsp {
        #[wasm_bindgen(constructor)]
        pub fn new() -> Self {
            Self {
                server: Server::new(),
            }
        }

        #[wasm_bindgen]
        pub fn handle_message(&mut self, message: &str) -> Result<JsValue, JsValue> {
            let response = self.server.handle_json_message(message);
            let notifications = self.server.take_outgoing_notifications();
            to_value(&LspBridgeResult {
                response,
                notifications,
                should_exit: self.server.should_exit(),
            })
            .map_err(|error| JsValue::from_str(&format!("serialization error: {error}")))
        }
    }

    #[wasm_bindgen]
    pub struct WasmCompiler {
        query: QueryEngine,
        last_text: std::collections::HashMap<String, String>,
    }

    #[wasm_bindgen]
    impl WasmCompiler {
        #[wasm_bindgen(constructor)]
        pub fn new() -> Self {
            Self {
                query: QueryEngine::new(),
                last_text: std::collections::HashMap::new(),
            }
        }

        #[wasm_bindgen]
        pub fn set_file(&mut self, uri: &str, source: &str) {
            self.query.set_file(uri.to_owned(), source.to_owned());
            self.last_text.insert(uri.to_owned(), source.to_owned());
        }

        #[wasm_bindgen]
        pub fn compile(&mut self, uri: &str) -> Result<JsValue, JsValue> {
            let source = self.last_text.get(uri).cloned().unwrap_or_default();
            let parsed = self.query.parse(uri);
            let parse_errors_raw = parsed.as_ref().map(|parsed| parsed.errors.clone());
            let typed_ast = parsed
                .as_ref()
                .map(|parsed| format!("{:#?}", parsed.file))
                .unwrap_or_default();
            let diagnostics_raw = self.query.diagnostics(uri).unwrap_or_default();
            let lowered_hir = self.query.lower_hir(uri);
            let lowered_typed_ast = lowered_hir
                .as_ref()
                .map(|hir| format!("{hir:#?}"))
                .unwrap_or_default();
            let lowered_lir = self.query.lower(uri);
            let lowered = lowered_lir
                .as_ref()
                .map(|lir| format!("{lir:#?}"))
                .unwrap_or_default();
            let stats = self.query.stats();
            let mut backend_errors = Vec::new();
            let mut backend_diagnostics = Vec::new();
            let (emitted_ts, emitted_d_ts, emitted_js) = if !diagnostics_raw.is_empty() {
                (
                    emit_error_stub("typescript", "TypeScript emit skipped: diagnostics present"),
                    emit_error_stub(
                        "typescript-definition",
                        "TypeScriptDefinition emit skipped: diagnostics present",
                    ),
                    emit_error_stub("javascript", "JavaScript emit skipped: diagnostics present"),
                )
            } else if let Some(lir) = lowered_lir.as_ref() {
                let ts = match backend::emit(lir, CodegenTarget::TypeScript) {
                    Ok(out) => out,
                    Err(err) => {
                        let message =
                            format!("TypeScript emit failed: {}", format_backend_error(&err));
                        backend_errors.push(message.clone());
                        backend_diagnostics.push(to_diagnostic_view(
                            &source,
                            0,
                            source.len(),
                            message.clone(),
                        ));
                        emit_error_stub("typescript", &message)
                    }
                };
                let d_ts = match backend::emit(lir, CodegenTarget::TypeScriptDefinition) {
                    Ok(out) => out,
                    Err(err) => {
                        let message = format!(
                            "TypeScriptDefinition emit failed: {}",
                            format_backend_error(&err)
                        );
                        backend_errors.push(message.clone());
                        backend_diagnostics.push(to_diagnostic_view(
                            &source,
                            0,
                            source.len(),
                            message.clone(),
                        ));
                        emit_error_stub("typescript-definition", &message)
                    }
                };
                let js = match backend::emit(lir, CodegenTarget::JavaScript) {
                    Ok(out) => out,
                    Err(err) => {
                        let message =
                            format!("JavaScript emit failed: {}", format_backend_error(&err));
                        backend_errors.push(message.clone());
                        backend_diagnostics.push(to_diagnostic_view(
                            &source,
                            0,
                            source.len(),
                            message.clone(),
                        ));
                        emit_error_stub("javascript", &message)
                    }
                };
                (ts, d_ts, js)
            } else {
                (
                    emit_error_stub("typescript", "TypeScript emit skipped: no lowered HIR"),
                    emit_error_stub(
                        "typescript-definition",
                        "TypeScriptDefinition emit skipped: no lowered HIR",
                    ),
                    emit_error_stub("javascript", "JavaScript emit skipped: no lowered HIR"),
                )
            };

            let parse_errors = parse_errors_raw
                .unwrap_or_default()
                .into_iter()
                .map(|diag| {
                    to_diagnostic_view(&source, diag.span.start, diag.span.end, diag.message)
                })
                .collect();

            let mut diagnostics = diagnostics_raw
                .into_iter()
                .map(|diag| to_diagnostic_view(&source, diag.start, diag.end, diag.message))
                .collect::<Vec<_>>();
            diagnostics.extend(backend_diagnostics);

            to_value(&CompilerRunResult {
                parse_errors,
                diagnostics,
                stats: CompilerStatsView {
                    parse_computes: stats.parse_computes,
                    lower_computes: stats.lower_computes,
                    diagnostics_computes: stats.diagnostics_computes,
                },
                typed_ast,
                lowered_typed_ast,
                lowered,
                emitted_ts,
                emitted_d_ts,
                emitted_js,
                backend_errors,
            })
            .map_err(|error| JsValue::from_str(&format!("serialization error: {error}")))
        }
    }

    fn to_diagnostic_view(
        source: &str,
        start: usize,
        end: usize,
        message: String,
    ) -> DiagnosticView {
        let (start_line, start_character) = byte_to_lsp_position(source, start);
        let (end_line, end_character) = byte_to_lsp_position(source, end);
        DiagnosticView {
            start,
            end,
            start_line,
            start_character,
            end_line,
            end_character,
            message,
        }
    }

    fn format_backend_error(err: &BackendError) -> String {
        match err {
            BackendError::UnsupportedTarget(target) => format!("unsupported target: {target:?}"),
            BackendError::EmitFailed(message) => message.clone(),
        }
    }

    fn emit_error_stub(channel: &str, message: &str) -> String {
        format!("/* lumo {channel} emission error: {message} */")
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
}

#[cfg(target_arch = "wasm32")]
pub use wasm::*;
