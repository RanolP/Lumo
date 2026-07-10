//! Lossless syntax tree, generic over a language's `SyntaxKind` (D-18:
//! losslessness per language; M0 treats every language as lossless).
//! Tokens carry their text; a node's lossless print is the concatenation
//! of its token texts, byte-exact.

use crate::Span;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Token<K> {
    pub kind: K,
    pub text: String,
    pub span: Span,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum SyntaxElement<K> {
    Node(Box<SyntaxNode<K>>),
    Token(Token<K>),
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct SyntaxNode<K> {
    pub kind: K,
    pub span: Span,
    pub children: Vec<SyntaxElement<K>>,
}

impl<K> SyntaxNode<K> {
    pub fn from_children(kind: K, children: Vec<SyntaxElement<K>>) -> SyntaxNode<K> {
        let span = children_span(&children);
        SyntaxNode { kind, span, children }
    }

    /// Lossless print: concatenated token texts, byte-exact.
    pub fn text(&self) -> String {
        let mut out = String::new();
        self.write_text(&mut out);
        out
    }

    fn write_text(&self, out: &mut String) {
        for child in &self.children {
            match child {
                SyntaxElement::Node(n) => n.write_text(out),
                SyntaxElement::Token(t) => out.push_str(&t.text),
            }
        }
    }

    pub fn child_nodes(&self) -> impl Iterator<Item = &SyntaxNode<K>> {
        self.children.iter().filter_map(|c| match c {
            SyntaxElement::Node(n) => Some(n.as_ref()),
            SyntaxElement::Token(_) => None,
        })
    }

    pub fn child_tokens(&self) -> impl Iterator<Item = &Token<K>> {
        self.children.iter().filter_map(|c| match c {
            SyntaxElement::Token(t) => Some(t),
            SyntaxElement::Node(_) => None,
        })
    }

    /// All descendant tokens, in source order.
    pub fn descendant_tokens(&self) -> Vec<&Token<K>> {
        let mut out = Vec::new();
        self.collect_tokens(&mut out);
        out
    }

    fn collect_tokens<'t>(&'t self, out: &mut Vec<&'t Token<K>>) {
        for child in &self.children {
            match child {
                SyntaxElement::Node(n) => n.collect_tokens(out),
                SyntaxElement::Token(t) => out.push(t),
            }
        }
    }
}

fn children_span<K>(children: &[SyntaxElement<K>]) -> Span {
    let significant = |c: &SyntaxElement<K>| match c {
        SyntaxElement::Token(t) => Some(t.span),
        SyntaxElement::Node(n) => (!n.children.is_empty()).then_some(n.span),
    };
    let start = children.iter().find_map(significant);
    let end = children.iter().rev().find_map(significant);
    match (start, end) {
        (Some(s), Some(e)) => Span::new(s.start, e.end),
        _ => Span::default(),
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ParseError {
    pub span: Span,
    pub message: String,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ParseOutput<K> {
    pub root: SyntaxNode<K>,
    pub errors: Vec<ParseError>,
}
