//! Registry emitter: `name → parse-report fn` for every language (so the
//! corpus harness dispatches `:parse(L)`), plus `(from, to) → elab-report
//! fn` for every elab pair (`:elab(A -> B)`).

use crate::project::model::Definition;

use super::elab::pair_module;
use super::naming::module_name;
use super::Buf;

pub fn generate(def: &Definition) -> String {
    let mut buf = Buf::new();
    buf.blank();
    if def.elabs.is_empty() {
        buf.line("use langue_rt::ParseReport;");
    } else {
        buf.line("use langue_rt::{ElabReport, ParseReport};");
    }
    buf.blank();
    buf.open("pub struct LanguageOps {");
    buf.line("pub name: &'static str,");
    buf.line("pub parse_report: fn(&str) -> ParseReport,");
    buf.close("}");
    buf.blank();
    buf.open("pub static LANGUAGES: &[LanguageOps] = &[");
    for lang_name in def.languages.keys() {
        buf.line(&format!(
            "LanguageOps {{ name: {lang_name:?}, parse_report: {}_report }},",
            module_name(lang_name)
        ));
    }
    buf.close("];");
    buf.blank();
    buf.open("pub fn language(name: &str) -> Option<&'static LanguageOps> {");
    buf.line("LANGUAGES.iter().find(|l| l.name == name)");
    buf.close("}");
    if !def.elabs.is_empty() {
        buf.blank();
        buf.open("pub struct ElabOps {");
        buf.line("pub from: &'static str,");
        buf.line("pub to: &'static str,");
        buf.line("pub elab_report: fn(&str) -> ElabReport,");
        buf.close("}");
        buf.blank();
        buf.open("pub static ELABS: &[ElabOps] = &[");
        for (from, to) in def.elabs.keys() {
            buf.line(&format!(
                "ElabOps {{ from: {from:?}, to: {to:?}, elab_report: {}_elab_report }},",
                pair_module(from, to)
            ));
        }
        buf.close("];");
        buf.blank();
        buf.open("pub fn elab(from: &str, to: &str) -> Option<&'static ElabOps> {");
        buf.line("ELABS.iter().find(|e| e.from == from && e.to == to)");
        buf.close("}");
        for (from, to) in def.elabs.keys() {
            let module = pair_module(from, to);
            buf.blank();
            buf.line(&format!(
                "/// Externs come from the handwritten `crate::elab_externs::{module}()`."
            ));
            buf.open(&format!("fn {module}_elab_report(text: &str) -> ElabReport {{"));
            buf.line(&format!("let mut externs = crate::elab_externs::{module}();"));
            buf.line(&format!("crate::elab::{module}::elab(text, externs.as_mut())"));
            buf.close("}");
        }
    }
    for lang_name in def.languages.keys() {
        let module = module_name(lang_name);
        buf.blank();
        buf.open(&format!("fn {module}_report(text: &str) -> ParseReport {{"));
        buf.line(&format!("let out = crate::{module}::parser::parse(text);"));
        buf.line(&format!("let canonical = crate::{module}::printer::canonical(&out.root);"));
        buf.line(&format!("let reparse = crate::{module}::parser::parse(&canonical);"));
        buf.open("ParseReport {");
        buf.line(&format!("sexpr: crate::{module}::printer::sexpr(&out.root),"));
        buf.line("errors: out.errors.iter().map(|e| format!(\"{}: {}\", e.span, e.message)).collect(),");
        buf.line("lossless: out.root.text(),");
        buf.line(&format!(
            "round_trip_sexpr: crate::{module}::printer::sexpr(&reparse.root),"
        ));
        buf.line("canonical,");
        buf.close("}");
        buf.close("}");
    }
    buf.finish()
}
