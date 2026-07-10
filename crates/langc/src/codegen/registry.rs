//! Registry emitter: `name → parse-report fn` for every language, so the
//! corpus harness dispatches `:parse(L)` without knowing any language.

use crate::project::model::Definition;

use super::naming::module_name;
use super::Buf;

pub fn generate(def: &Definition) -> String {
    let mut buf = Buf::new();
    buf.blank();
    buf.line("use langue_rt::ParseReport;");
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
