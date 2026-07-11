//! Browser playground bindings: a single stateless `compile` entry point
//! over the lumo-syntax drivers — emitted JS, MIR, inferred types, and
//! the Lumo parse tree, plus positioned parse diagnostics for Monaco.

use serde::Serialize;

/// A parse error with Monaco-friendly zero-based line/UTF-16-column
/// coordinates (LSP convention, matching the legacy playground).
#[derive(Serialize, Debug, PartialEq, Eq)]
pub struct ParseDiag {
    pub start_line: u32,
    pub start_character: u32,
    pub end_line: u32,
    pub end_character: u32,
    pub message: String,
}

#[derive(Serialize, Debug)]
pub struct CompileResult {
    pub js: String,
    pub js_errors: Vec<String>,
    pub mir: String,
    pub mir_errors: Vec<String>,
    pub types: String,
    pub type_errors: Vec<String>,
    pub sexpr: String,
    pub parse_diags: Vec<ParseDiag>,
}

pub fn compile_result(source: &str) -> CompileResult {
    let js = lumo_syntax::compile_driver::compile_report(source);
    let mir_elab = lumo_syntax::registry::elab("Lumo", "MIR")
        .expect("Lumo -> MIR elab is registered");
    let mir = (mir_elab.elab_report)(source);
    let types = lumo_syntax::judge_driver::infer_report(source);
    let parsed = lumo_syntax::lumo::parser::parse(source);
    CompileResult {
        js: js.output,
        js_errors: js.errors,
        mir: mir.output,
        mir_errors: mir.errors,
        types: types.output,
        type_errors: types.errors,
        sexpr: lumo_syntax::lumo::printer::sexpr(&parsed.root),
        parse_diags: parsed
            .errors
            .iter()
            .map(|e| {
                let (start_line, start_character) = position(source, e.span.start);
                let (end_line, end_character) = position(source, e.span.end);
                ParseDiag {
                    start_line,
                    start_character,
                    end_line,
                    end_character,
                    message: e.message.clone(),
                }
            })
            .collect(),
    }
}

/// Byte offset → zero-based (line, UTF-16 column).
fn position(source: &str, byte_offset: u32) -> (u32, u32) {
    let clamped = (byte_offset as usize).min(source.len());
    let (mut line, mut col) = (0u32, 0u32);
    for (idx, ch) in source.char_indices() {
        if idx >= clamped {
            break;
        }
        if ch == '\n' {
            line += 1;
            col = 0;
        } else {
            col += ch.len_utf16() as u32;
        }
    }
    (line, col)
}

#[cfg(target_arch = "wasm32")]
mod wasm {
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen]
    pub fn compile(source: &str) -> Result<JsValue, JsValue> {
        serde_wasm_bindgen::to_value(&super::compile_result(source))
            .map_err(|error| JsValue::from_str(&format!("serialization error: {error}")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_program_compiles_on_all_views() {
        let r = compile_result("fn id(a: A): A / {} { a }");
        assert_eq!(r.js, "const id=()=>(a)=>a;");
        assert!(r.js_errors.is_empty(), "js errors: {:?}", r.js_errors);
        assert!(r.mir_errors.is_empty(), "mir errors: {:?}", r.mir_errors);
        assert!(r.type_errors.is_empty(), "type errors: {:?}", r.type_errors);
        assert!(r.parse_diags.is_empty(), "parse diags: {:?}", r.parse_diags);
        assert!(!r.mir.is_empty());
        assert_eq!(r.types, "id : U((A) -> F(A))");
        assert!(r.sexpr.contains("FILE"));
    }

    #[test]
    fn parse_error_positions_are_line_and_utf16_column() {
        let r = compile_result("fn id(x) = x\nfn broken( = 1");
        assert!(!r.parse_diags.is_empty());
        assert_eq!(r.parse_diags[0].start_line, 1);
    }
}
