#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenKind {
    Keyword(Keyword),
    Ident(String),
    Symbol(Symbol),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Keyword {
    Data,
    Fn,
    Let,
    In,
    Produce,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Symbol {
    LBracket,
    RBracket,
    LParen,
    RParen,
    LBrace,
    RBrace,
    Colon,
    Comma,
    Equals,
    ColonEquals,
    Slash,
    Star,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LexError {
    pub span: Span,
    pub message: String,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct LexOutput {
    pub tokens: Vec<Token>,
    pub errors: Vec<LexError>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LosslessToken {
    pub kind: LosslessTokenKind,
    pub span: Span,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LosslessTokenKind {
    Keyword(Keyword),
    Ident,
    Symbol(Symbol),
    Whitespace,
    Newline,
    Unknown,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct LosslessLexOutput {
    pub tokens: Vec<LosslessToken>,
}

pub fn lex(input: &str) -> LexOutput {
    let lossless = lex_lossless(input);
    let mut output = LexOutput::default();

    for token in lossless.tokens {
        match token.kind {
            LosslessTokenKind::Keyword(kw) => output.tokens.push(Token {
                kind: TokenKind::Keyword(kw),
                span: token.span,
            }),
            LosslessTokenKind::Ident => output.tokens.push(Token {
                kind: TokenKind::Ident(token.text),
                span: token.span,
            }),
            LosslessTokenKind::Symbol(sym) => output.tokens.push(Token {
                kind: TokenKind::Symbol(sym),
                span: token.span,
            }),
            LosslessTokenKind::Whitespace | LosslessTokenKind::Newline => {}
            LosslessTokenKind::Unknown => {
                let message = match token.text.chars().next() {
                    Some(ch) if token.text.chars().count() == 1 => {
                        format!("unexpected character: {ch:?}")
                    }
                    _ => format!("unexpected character: {:?}", token.text),
                };
                output.errors.push(LexError {
                    span: token.span,
                    message,
                });
            }
        }
    }

    output
}

pub fn lex_lossless(input: &str) -> LosslessLexOutput {
    let mut output = LosslessLexOutput::default();
    let mut index = 0;

    while index < input.len() {
        let ch = next_char(input, index);

        if ch == '\n' {
            let start = index;
            index += ch.len_utf8();
            output.tokens.push(LosslessToken {
                kind: LosslessTokenKind::Newline,
                span: Span::new(start, index),
                text: input[start..index].to_owned(),
            });
            continue;
        }

        if ch.is_whitespace() {
            let start = index;
            index += ch.len_utf8();
            while index < input.len() {
                let c = next_char(input, index);
                if !c.is_whitespace() || c == '\n' {
                    break;
                }
                index += c.len_utf8();
            }
            output.tokens.push(LosslessToken {
                kind: LosslessTokenKind::Whitespace,
                span: Span::new(start, index),
                text: input[start..index].to_owned(),
            });
            continue;
        }

        if is_ident_start(ch) {
            let start = index;
            index += ch.len_utf8();

            while index < input.len() {
                let c = next_char(input, index);
                if !is_ident_continue(c) {
                    break;
                }
                index += c.len_utf8();
            }

            let text = &input[start..index];
            let kind = match text {
                "data" => LosslessTokenKind::Keyword(Keyword::Data),
                "fn" => LosslessTokenKind::Keyword(Keyword::Fn),
                "let" => LosslessTokenKind::Keyword(Keyword::Let),
                "in" => LosslessTokenKind::Keyword(Keyword::In),
                "produce" => LosslessTokenKind::Keyword(Keyword::Produce),
                _ => LosslessTokenKind::Ident,
            };

            output.tokens.push(LosslessToken {
                kind,
                span: Span::new(start, index),
                text: text.to_owned(),
            });
            continue;
        }

        let start = index;
        if starts_with_at(input, index, ":=") {
            index += 2;
            output.tokens.push(LosslessToken {
                kind: LosslessTokenKind::Symbol(Symbol::ColonEquals),
                span: Span::new(start, index),
                text: input[start..index].to_owned(),
            });
            continue;
        }

        let symbol = match ch {
            '[' => Some(Symbol::LBracket),
            ']' => Some(Symbol::RBracket),
            '(' => Some(Symbol::LParen),
            ')' => Some(Symbol::RParen),
            '{' => Some(Symbol::LBrace),
            '}' => Some(Symbol::RBrace),
            ':' => Some(Symbol::Colon),
            ',' => Some(Symbol::Comma),
            '=' => Some(Symbol::Equals),
            '/' => Some(Symbol::Slash),
            '*' => Some(Symbol::Star),
            _ => None,
        };

        index += ch.len_utf8();
        if let Some(symbol) = symbol {
            output.tokens.push(LosslessToken {
                kind: LosslessTokenKind::Symbol(symbol),
                span: Span::new(start, index),
                text: input[start..index].to_owned(),
            });
        } else {
            output.tokens.push(LosslessToken {
                kind: LosslessTokenKind::Unknown,
                span: Span::new(start, index),
                text: input[start..index].to_owned(),
            });
        }
    }

    output
}

fn next_char(input: &str, index: usize) -> char {
    input[index..].chars().next().expect("index must be valid")
}

fn starts_with_at(input: &str, index: usize, pat: &str) -> bool {
    input[index..].starts_with(pat)
}

fn is_ident_start(ch: char) -> bool {
    ch == '_' || ch.is_alphabetic()
}

fn is_ident_continue(ch: char) -> bool {
    ch == '_' || ch.is_alphanumeric()
}
