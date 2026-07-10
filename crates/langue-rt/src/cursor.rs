//! Token cursor shared by all generated parsers: lookahead-1 over
//! non-trivia tokens, with trivia attached to the children list right
//! before each consumed token (the legacy lossless parser's shape).

use crate::tree::{ParseError, SyntaxElement, SyntaxNode, Token};
use crate::Span;

pub struct Cursor<K: Copy + PartialEq> {
    tokens: Vec<Token<K>>,
    pos: usize,
    pub errors: Vec<ParseError>,
    is_trivia: fn(K) -> bool,
}

impl<K: Copy + PartialEq + std::fmt::Debug> Cursor<K> {
    pub fn new(tokens: Vec<Token<K>>, is_trivia: fn(K) -> bool) -> Cursor<K> {
        Cursor { tokens, pos: 0, errors: Vec::new(), is_trivia }
    }

    fn nth_non_trivia(&self, n: usize) -> Option<&Token<K>> {
        let mut seen = 0;
        for t in &self.tokens[self.pos..] {
            if !(self.is_trivia)(t.kind) {
                if seen == n {
                    return Some(t);
                }
                seen += 1;
            }
        }
        None
    }

    /// No non-trivia input left.
    pub fn eof(&self) -> bool {
        self.nth_non_trivia(0).is_none()
    }

    pub fn at(&self, kind: K) -> bool {
        self.nth_non_trivia(0).is_some_and(|t| t.kind == kind)
    }

    pub fn at_any(&self, kinds: &[K]) -> bool {
        self.nth_non_trivia(0).is_some_and(|t| kinds.contains(&t.kind))
    }

    pub fn skip_trivia_into(&mut self, children: &mut Vec<SyntaxElement<K>>) {
        while let Some(t) = self.tokens.get(self.pos) {
            if (self.is_trivia)(t.kind) {
                children.push(SyntaxElement::Token(t.clone()));
                self.pos += 1;
            } else {
                break;
            }
        }
    }

    /// Skip trivia into `children`, then push the next token whatever it
    /// is. Panics at EOF — guard with `at`/`eof` first.
    pub fn bump_into(&mut self, children: &mut Vec<SyntaxElement<K>>) {
        self.skip_trivia_into(children);
        let t = self.tokens[self.pos].clone();
        self.pos += 1;
        children.push(SyntaxElement::Token(t));
    }

    /// Consume `kind` (with its leading trivia) or record an error and
    /// consume nothing.
    pub fn expect_into(&mut self, kind: K, children: &mut Vec<SyntaxElement<K>>) {
        self.skip_trivia_into(children);
        if self.at(kind) {
            self.bump_into(children);
        } else {
            self.error_here(format!("expected {kind:?}"));
        }
    }

    pub fn error_here(&mut self, message: String) {
        let span = self
            .nth_non_trivia(0)
            .map(|t| t.span)
            .unwrap_or_else(|| {
                let end = self.tokens.last().map(|t| t.span.end).unwrap_or(0);
                Span::new(end, end)
            });
        self.errors.push(ParseError { span, message });
    }

    /// Trailing sweep for losslessness: whatever remains after the start
    /// rule goes into `children` — trivia as plain tokens, anything else
    /// wrapped in one `error_kind` node (one error reported for the run).
    pub fn drain_rest_into(&mut self, error_kind: K, children: &mut Vec<SyntaxElement<K>>) {
        self.skip_trivia_into(children);
        if self.pos >= self.tokens.len() {
            return;
        }
        self.error_here("unexpected input after the last item".to_owned());
        let mut bad = Vec::new();
        while self.pos < self.tokens.len() {
            bad.push(SyntaxElement::Token(self.tokens[self.pos].clone()));
            self.pos += 1;
        }
        children.push(SyntaxElement::Node(Box::new(SyntaxNode::from_children(
            error_kind, bad,
        ))));
    }
}
