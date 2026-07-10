//! Lexer emitter: the pattern table (literals escaped, literals first)
//! and a `lex` function on the shared `langue_rt::LexDfa` engine.

use langue_rt::regex_escape;

use crate::project::model::Language;
use crate::syntax::ast::TokenPattern;

use super::naming::kind_name;
use super::{token_order, Buf};

pub fn generate(lang: &Language) -> String {
    let tokens = token_order(lang);

    let mut buf = Buf::new();
    buf.blank();
    buf.line("use std::sync::LazyLock;");
    buf.blank();
    buf.line("use langue_rt::LexDfa;");
    buf.blank();
    buf.line("use super::lossless::Token;");
    buf.line("use super::syntax_kind::SyntaxKind;");

    buf.blank();
    buf.open("static PATTERNS: &[&str] = &[");
    for token in &tokens {
        let pattern = match &token.pattern {
            TokenPattern::Literal(text) => regex_escape(text),
            TokenPattern::Regex(pattern) => pattern.clone(),
        };
        buf.line(&format!("{pattern:?}, // {}", token.name));
    }
    buf.close("];");

    buf.blank();
    buf.open("const KINDS: &[SyntaxKind] = &[");
    for token in &tokens {
        buf.line(&format!("SyntaxKind::{},", kind_name(&token.name)));
    }
    buf.close("];");

    buf.blank();
    buf.line("static DFA: LazyLock<LexDfa> = LazyLock::new(|| LexDfa::build(PATTERNS));");
    buf.blank();
    buf.line("/// Lossless tokenization: every input byte lands in exactly one");
    buf.line("/// token; unlexable bytes become 1-byte UNKNOWN tokens.");
    buf.open("pub fn lex(text: &str) -> Vec<Token> {");
    buf.line("DFA.lex(text)");
    buf.line("    .into_iter()");
    buf.line("    .map(|raw| Token {");
    buf.line("        kind: raw.pattern.map_or(SyntaxKind::UNKNOWN, |p| KINDS[p as usize]),");
    buf.line("        text: raw.span.slice(text).to_owned(),");
    buf.line("        span: raw.span,");
    buf.line("    })");
    buf.line("    .collect()");
    buf.close("}");
    buf.finish()
}
