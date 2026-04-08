pub use lumo_span::Span;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenKind {
    Keyword(Keyword),
    Ident(String),
    StringLit(String),
    NumberLit(String),
    Symbol(Symbol),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Keyword {
    Data,
    Fn,
    Extern,
    Let,
    In,
    Produce,
    Thunk,
    Force,
    Match,
    Cap,
    Perform,
    Handle,
    Bundle,
    Use,
    Impl,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Symbol {
    Hash,
    LBracket,
    RBracket,
    LParen,
    RParen,
    LBrace,
    RBrace,
    Semi,
    Colon,
    Comma,
    Equals,
    ColonEquals,
    Slash,
    Star,
    FatArrow,
    Dot,
    Plus,
    Minus,
    Percent,
    Bang,
    Lt,
    Gt,
    LtEq,
    GtEq,
    EqEq,
    BangEq,
    AmpAmp,
    PipePipe,
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
    StringLit,
    NumberLit,
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
            LosslessTokenKind::StringLit => output.tokens.push(Token {
                kind: TokenKind::StringLit(token.text),
                span: token.span,
            }),
            LosslessTokenKind::NumberLit => output.tokens.push(Token {
                kind: TokenKind::NumberLit(token.text),
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
                "extern" => LosslessTokenKind::Keyword(Keyword::Extern),
                "let" => LosslessTokenKind::Keyword(Keyword::Let),
                "in" => LosslessTokenKind::Keyword(Keyword::In),
                "produce" => LosslessTokenKind::Keyword(Keyword::Produce),
                "thunk" => LosslessTokenKind::Keyword(Keyword::Thunk),
                "force" => LosslessTokenKind::Keyword(Keyword::Force),
                "match" => LosslessTokenKind::Keyword(Keyword::Match),
                "cap" => LosslessTokenKind::Keyword(Keyword::Cap),
                "perform" => LosslessTokenKind::Keyword(Keyword::Perform),
                "handle" => LosslessTokenKind::Keyword(Keyword::Handle),
                "bundle" => LosslessTokenKind::Keyword(Keyword::Bundle),
                "use" => LosslessTokenKind::Keyword(Keyword::Use),
                "impl" => LosslessTokenKind::Keyword(Keyword::Impl),
                _ => LosslessTokenKind::Ident,
            };

            output.tokens.push(LosslessToken {
                kind,
                span: Span::new(start, index),
                text: text.to_owned(),
            });
            continue;
        }

        if ch == '"' {
            let start = index;
            index += ch.len_utf8();
            let mut escaped = false;
            while index < input.len() {
                let c = next_char(input, index);
                index += c.len_utf8();
                if escaped {
                    escaped = false;
                    continue;
                }
                if c == '\\' {
                    escaped = true;
                    continue;
                }
                if c == '"' {
                    break;
                }
            }
            output.tokens.push(LosslessToken {
                kind: LosslessTokenKind::StringLit,
                span: Span::new(start, index),
                text: input[start..index].to_owned(),
            });
            continue;
        }

        if ch.is_ascii_digit() {
            let start = index;
            index += ch.len_utf8();
            while index < input.len() {
                let c = next_char(input, index);
                if !c.is_ascii_digit() {
                    break;
                }
                index += c.len_utf8();
            }
            if index < input.len() && next_char(input, index) == '.' {
                let dot_pos = index;
                index += 1;
                if index < input.len() && next_char(input, index).is_ascii_digit() {
                    while index < input.len() {
                        let c = next_char(input, index);
                        if !c.is_ascii_digit() {
                            break;
                        }
                        index += c.len_utf8();
                    }
                } else {
                    index = dot_pos;
                }
            }
            output.tokens.push(LosslessToken {
                kind: LosslessTokenKind::NumberLit,
                span: Span::new(start, index),
                text: input[start..index].to_owned(),
            });
            continue;
        }

        // Line comments: // ...
        if starts_with_at(input, index, "//") {
            let start = index;
            index += 2;
            while index < input.len() {
                let c = next_char(input, index);
                if c == '\n' {
                    break;
                }
                index += c.len_utf8();
            }
            // Emit comment as whitespace (trivia) so it's ignored
            output.tokens.push(LosslessToken {
                kind: LosslessTokenKind::Whitespace,
                span: Span::new(start, index),
                text: input[start..index].to_owned(),
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
        if starts_with_at(input, index, "=>") {
            index += 2;
            output.tokens.push(LosslessToken {
                kind: LosslessTokenKind::Symbol(Symbol::FatArrow),
                span: Span::new(start, index),
                text: input[start..index].to_owned(),
            });
            continue;
        }
        if starts_with_at(input, index, "==") {
            index += 2;
            output.tokens.push(LosslessToken {
                kind: LosslessTokenKind::Symbol(Symbol::EqEq),
                span: Span::new(start, index),
                text: input[start..index].to_owned(),
            });
            continue;
        }
        if starts_with_at(input, index, "!=") {
            index += 2;
            output.tokens.push(LosslessToken {
                kind: LosslessTokenKind::Symbol(Symbol::BangEq),
                span: Span::new(start, index),
                text: input[start..index].to_owned(),
            });
            continue;
        }
        if starts_with_at(input, index, "<=") {
            index += 2;
            output.tokens.push(LosslessToken {
                kind: LosslessTokenKind::Symbol(Symbol::LtEq),
                span: Span::new(start, index),
                text: input[start..index].to_owned(),
            });
            continue;
        }
        if starts_with_at(input, index, ">=") {
            index += 2;
            output.tokens.push(LosslessToken {
                kind: LosslessTokenKind::Symbol(Symbol::GtEq),
                span: Span::new(start, index),
                text: input[start..index].to_owned(),
            });
            continue;
        }
        if starts_with_at(input, index, "&&") {
            index += 2;
            output.tokens.push(LosslessToken {
                kind: LosslessTokenKind::Symbol(Symbol::AmpAmp),
                span: Span::new(start, index),
                text: input[start..index].to_owned(),
            });
            continue;
        }
        if starts_with_at(input, index, "||") {
            index += 2;
            output.tokens.push(LosslessToken {
                kind: LosslessTokenKind::Symbol(Symbol::PipePipe),
                span: Span::new(start, index),
                text: input[start..index].to_owned(),
            });
            continue;
        }

        let symbol = match ch {
            '#' => Some(Symbol::Hash),
            '[' => Some(Symbol::LBracket),
            ']' => Some(Symbol::RBracket),
            '(' => Some(Symbol::LParen),
            ')' => Some(Symbol::RParen),
            '{' => Some(Symbol::LBrace),
            '}' => Some(Symbol::RBrace),
            ';' => Some(Symbol::Semi),
            ':' => Some(Symbol::Colon),
            ',' => Some(Symbol::Comma),
            '=' => Some(Symbol::Equals),
            '/' => Some(Symbol::Slash),
            '*' => Some(Symbol::Star),
            '.' => Some(Symbol::Dot),
            '+' => Some(Symbol::Plus),
            '-' => Some(Symbol::Minus),
            '%' => Some(Symbol::Percent),
            '!' => Some(Symbol::Bang),
            '<' => Some(Symbol::Lt),
            '>' => Some(Symbol::Gt),
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
