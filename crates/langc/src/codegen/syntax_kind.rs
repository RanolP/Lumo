//! SyntaxKind emitter: one fieldless enum per language — fixed UNKNOWN /
//! ERROR, then token kinds in lexer-table order, node kinds, and
//! synthesized praat row kinds.

use crate::project::model::Language;
use crate::project::praat::{classify_row, RowKind};
use crate::syntax::ast::RuleBody;

use super::naming::kind_name;
use super::{token_order, Buf};

/// Kinds synthesized for a praat rule's operator rows, in emission
/// order. All rows of one placement share one kind (like the legacy
/// BINARY_EXPR across precedence levels).
pub fn praat_kinds(rule_name: &str, praat: &crate::syntax::ast::Praat) -> Vec<String> {
    let mut has = [false; 4];
    for row in &praat.rows {
        match classify_row(row) {
            Ok(RowKind::Prefix { .. }) => has[0] = true,
            Ok(RowKind::Infix { .. }) => has[1] = true,
            Ok(RowKind::Postfix { .. }) => has[2] = true,
            Ok(RowKind::Mixfix { .. }) => has[3] = true,
            Err(_) => {}
        }
    }
    ["Prefix", "Infix", "Postfix", "Mixfix"]
        .iter()
        .zip(has)
        .filter(|(_, present)| *present)
        .map(|(suffix, _)| format!("{rule_name}{suffix}"))
        .collect()
}

/// Every kind of the language in declaration order: `(variant, doc)`.
pub fn all_kinds(lang: &Language) -> Vec<(String, String)> {
    let mut kinds = vec![
        ("UNKNOWN".to_owned(), "an unlexable byte".to_owned()),
        ("ERROR".to_owned(), "a recovery node wrapping skipped input".to_owned()),
    ];
    for token in token_order(lang) {
        kinds.push((kind_name(&token.name), format!("token `{}`", token.name)));
    }
    for (name, rule) in &lang.rules {
        kinds.push((kind_name(name), format!("rule `{name}`")));
        if let RuleBody::Praat(praat) = &rule.body {
            for row_kind in praat_kinds(name, praat) {
                kinds.push((kind_name(&row_kind), format!("praat row of `{name}`")));
            }
        }
    }
    kinds
}

pub fn generate(lang_name: &str, lang: &Language) -> String {
    let mut buf = Buf::new();
    buf.blank();
    buf.line("#[allow(non_camel_case_types)]");
    buf.line("#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]");
    buf.open(&format!("pub enum SyntaxKind {{ // language `{lang_name}`"));
    for (variant, doc) in all_kinds(lang) {
        buf.line(&format!("/// {doc}"));
        buf.line(&format!("{variant},"));
    }
    buf.close("}");

    buf.blank();
    buf.open("impl SyntaxKind {");
    buf.open("pub fn is_trivia(self) -> bool {");
    let trivia: Vec<String> = token_order(lang)
        .iter()
        .filter(|t| t.is_trivia)
        .map(|t| format!("SyntaxKind::{}", kind_name(&t.name)))
        .collect();
    if trivia.is_empty() {
        buf.line("false");
    } else {
        buf.line(&format!("matches!(self, {})", trivia.join(" | ")));
    }
    buf.close("}");

    buf.blank();
    buf.line("/// Dotted token names double as highlight scopes (D-09).");
    buf.open("pub fn highlight_scope(self) -> Option<&'static str> {");
    buf.open("match self {");
    for token in token_order(lang) {
        if token.name.contains('.') {
            buf.line(&format!(
                "SyntaxKind::{} => Some({:?}),",
                kind_name(&token.name),
                token.name
            ));
        }
    }
    buf.line("_ => None,");
    buf.close("}");
    buf.close("}");
    buf.close("}");
    buf.finish()
}
