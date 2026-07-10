//! Printer emitter. Lossless print is the tree's own token text
//! (byte-exact); canonical print delegates to the shared rt logic with
//! this language's lexer, so every `:parse` fixture validates it via the
//! automatic round-trip (D-32).

use super::Buf;

pub fn generate() -> String {
    let mut buf = Buf::new();
    buf.blank();
    buf.line("use super::lossless::SyntaxNode;");
    buf.line("use super::syntax_kind::SyntaxKind;");
    buf.blank();
    buf.line("/// Byte-exact source reproduction.");
    buf.open("pub fn lossless(node: &SyntaxNode) -> String {");
    buf.line("node.text()");
    buf.close("}");
    buf.blank();
    buf.line("/// Non-trivia tokens; a space appears iff re-lexing the bare");
    buf.line("/// concatenation would merge the neighbors.");
    buf.open("pub fn canonical(node: &SyntaxNode) -> String {");
    buf.line("langue_rt::print_canonical(node, super::lexer::lex, SyntaxKind::is_trivia)");
    buf.close("}");
    buf.blank();
    buf.line("/// Named-node S-expression (the `:parse` fixture expectation).");
    buf.open("pub fn sexpr(node: &SyntaxNode) -> String {");
    buf.line("langue_rt::sexpr(node)");
    buf.close("}");
    buf.finish()
}
