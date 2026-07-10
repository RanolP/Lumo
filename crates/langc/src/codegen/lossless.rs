//! Lossless-tree module emitter. The tree itself is generic in
//! `langue-rt`; the generated module pins it to this language's
//! `SyntaxKind` so downstream code reads naturally.

use super::Buf;

pub fn generate() -> String {
    let mut buf = Buf::new();
    buf.blank();
    buf.line("use super::syntax_kind::SyntaxKind;");
    buf.blank();
    buf.line("pub type Token = langue_rt::Token<SyntaxKind>;");
    buf.line("pub type SyntaxElement = langue_rt::SyntaxElement<SyntaxKind>;");
    buf.line("pub type SyntaxNode = langue_rt::SyntaxNode<SyntaxKind>;");
    buf.line("pub type ParseOutput = langue_rt::ParseOutput<SyntaxKind>;");
    buf.line("pub use langue_rt::ParseError;");
    buf.finish()
}
