use lumo_compiler::lexer::{self, Symbol, TokenKind};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HighlightKind {
    Keyword,
    Identifier,
    Symbol,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HighlightToken {
    pub start: usize,
    pub end: usize,
    pub kind: HighlightKind,
}

pub fn highlight(source: &str) -> Vec<HighlightToken> {
    let out = lexer::lex(source);
    let mut tokens = Vec::with_capacity(out.tokens.len());

    for token in out.tokens {
        let kind = match token.kind {
            TokenKind::Keyword(_) => HighlightKind::Keyword,
            TokenKind::Ident(_) => HighlightKind::Identifier,
            TokenKind::Symbol(Symbol::LBracket)
            | TokenKind::Symbol(Symbol::RBracket)
            | TokenKind::Symbol(Symbol::LParen)
            | TokenKind::Symbol(Symbol::RParen)
            | TokenKind::Symbol(Symbol::LBrace)
            | TokenKind::Symbol(Symbol::RBrace)
            | TokenKind::Symbol(Symbol::Colon)
            | TokenKind::Symbol(Symbol::Comma)
            | TokenKind::Symbol(Symbol::Equals)
            | TokenKind::Symbol(Symbol::ColonEquals)
            | TokenKind::Symbol(Symbol::Slash)
            | TokenKind::Symbol(Symbol::Star) => HighlightKind::Symbol,
        };

        tokens.push(HighlightToken {
            start: token.span.start,
            end: token.span.end,
            kind,
        });
    }

    tokens
}
