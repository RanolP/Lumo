use lumo_compiler::lexer::{self, Symbol, TokenKind};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HighlightKind {
    Keyword,
    Identifier,
    String,
    Number,
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
    let mut state = AttrState::default();

    for token in out.tokens {
        let kind = match &token.kind {
            TokenKind::Keyword(_) if state.expect_attr_name && state.attr_bracket_depth > 0 => {
                HighlightKind::Identifier
            }
            TokenKind::Keyword(_) => HighlightKind::Keyword,
            TokenKind::Ident(text) if text == "type" => HighlightKind::Keyword,
            TokenKind::Ident(_) => HighlightKind::Identifier,
            TokenKind::StringLit(_) => HighlightKind::String,
            TokenKind::Symbol(Symbol::Hash)
            | TokenKind::Symbol(Symbol::LBracket)
            | TokenKind::Symbol(Symbol::RBracket)
            | TokenKind::Symbol(Symbol::LParen)
            | TokenKind::Symbol(Symbol::RParen)
            | TokenKind::Symbol(Symbol::LBrace)
            | TokenKind::Symbol(Symbol::RBrace)
            | TokenKind::Symbol(Symbol::Semi)
            | TokenKind::Symbol(Symbol::Colon)
            | TokenKind::Symbol(Symbol::Comma)
            | TokenKind::Symbol(Symbol::Equals)
            | TokenKind::Symbol(Symbol::ColonEquals)
            | TokenKind::Symbol(Symbol::Slash)
            | TokenKind::Symbol(Symbol::Star)
            | TokenKind::Symbol(Symbol::FatArrow)
            | TokenKind::Symbol(Symbol::Dot)
            | TokenKind::Symbol(Symbol::Plus)
            | TokenKind::Symbol(Symbol::Minus)
            | TokenKind::Symbol(Symbol::Percent)
            | TokenKind::Symbol(Symbol::Bang)
            | TokenKind::Symbol(Symbol::Lt)
            | TokenKind::Symbol(Symbol::Gt)
            | TokenKind::Symbol(Symbol::LtEq)
            | TokenKind::Symbol(Symbol::GtEq)
            | TokenKind::Symbol(Symbol::EqEq)
            | TokenKind::Symbol(Symbol::BangEq)
            | TokenKind::Symbol(Symbol::AmpAmp)
            | TokenKind::Symbol(Symbol::PipePipe) => HighlightKind::Symbol,
            TokenKind::NumberLit(_) => HighlightKind::Number,
        };

        state.observe(&token.kind);

        tokens.push(HighlightToken {
            start: token.span.start,
            end: token.span.end,
            kind,
        });
    }

    tokens
}

#[derive(Default)]
struct AttrState {
    pending_hash: bool,
    attr_bracket_depth: usize,
    expect_attr_name: bool,
}

impl AttrState {
    fn observe(&mut self, token: &TokenKind) {
        match token {
            TokenKind::Symbol(Symbol::Hash) => {
                self.pending_hash = true;
            }
            TokenKind::Symbol(Symbol::LBracket) => {
                if self.pending_hash {
                    self.pending_hash = false;
                    self.attr_bracket_depth = 1;
                    self.expect_attr_name = true;
                } else if self.attr_bracket_depth > 0 {
                    self.attr_bracket_depth += 1;
                }
            }
            TokenKind::Symbol(Symbol::RBracket) => {
                self.pending_hash = false;
                if self.attr_bracket_depth > 0 {
                    self.attr_bracket_depth -= 1;
                    if self.attr_bracket_depth == 0 {
                        self.expect_attr_name = false;
                    }
                }
            }
            TokenKind::Ident(_) | TokenKind::Keyword(_) => {
                self.pending_hash = false;
                if self.expect_attr_name && self.attr_bracket_depth > 0 {
                    self.expect_attr_name = false;
                }
            }
            _ => {
                self.pending_hash = false;
            }
        }
    }
}
