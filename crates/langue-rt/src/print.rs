//! Printing helpers shared by generated printers and the fixture
//! harness (D-32).

use crate::tree::{SyntaxNode, Token};

/// Canonical print: non-trivia token texts, with a space inserted iff
/// re-lexing the bare concatenation would merge the two tokens.
pub fn print_canonical<K: Copy>(
    node: &SyntaxNode<K>,
    lex: fn(&str) -> Vec<Token<K>>,
    is_trivia: fn(K) -> bool,
) -> String {
    let mut out = String::new();
    let mut prev: Option<&Token<K>> = None;
    for token in node.descendant_tokens() {
        if is_trivia(token.kind) {
            continue;
        }
        if let Some(p) = prev {
            if merges(&p.text, &token.text, lex) {
                out.push(' ');
            }
        }
        out.push_str(&token.text);
        prev = Some(token);
    }
    out
}

fn merges<K>(a: &str, b: &str, lex: fn(&str) -> Vec<Token<K>>) -> bool {
    let joined = format!("{a}{b}");
    lex(&joined).first().map(|t| t.text.len() != a.len()).unwrap_or(false)
}

/// Named-node S-expression: node kinds only, tokens elided —
/// `(FILE (FN_DECL (PARAM) (EXPR_INFIX ...)))`.
pub fn sexpr<K: Copy + std::fmt::Debug>(node: &SyntaxNode<K>) -> String {
    let mut out = String::new();
    write_sexpr(node, &mut out);
    out
}

fn write_sexpr<K: Copy + std::fmt::Debug>(node: &SyntaxNode<K>, out: &mut String) {
    out.push('(');
    out.push_str(&format!("{:?}", node.kind));
    for child in node.child_nodes() {
        out.push(' ');
        write_sexpr(child, out);
    }
    out.push(')');
}

/// Everything the corpus harness needs from one parse, produced by the
/// generated registry so the harness stays language-agnostic.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ParseReport {
    pub sexpr: String,
    pub errors: Vec<String>,
    /// Byte-exact lossless print.
    pub lossless: String,
    pub canonical: String,
    /// S-expression after parse → canonical print → re-parse; must equal
    /// `sexpr` (the D-32 automatic round-trip).
    pub round_trip_sexpr: String,
}
