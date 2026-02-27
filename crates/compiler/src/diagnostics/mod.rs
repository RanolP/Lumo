use crate::{lexer, parser};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub start: usize,
    pub end: usize,
    pub message: String,
}

pub fn from_lex_and_parse(
    lex_errors: &[lexer::LexError],
    parse_errors: &[parser::ParseError],
) -> Vec<Diagnostic> {
    let mut out = Vec::new();

    for e in lex_errors {
        out.push(Diagnostic {
            start: e.span.start,
            end: e.span.end,
            message: e.message.clone(),
        });
    }

    for e in parse_errors {
        out.push(Diagnostic {
            start: e.span.start,
            end: e.span.end,
            message: e.message.clone(),
        });
    }

    out
}
