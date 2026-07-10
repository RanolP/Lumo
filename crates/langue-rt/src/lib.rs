//! Runtime shared by all code that `langc gen` emits: source spans and the
//! DFA-walk lexer engine. Generated crates depend on this crate and on
//! nothing else.

mod lexer;
mod span;

pub use lexer::{regex_escape, LexDfa, RawToken};
pub use span::Span;
