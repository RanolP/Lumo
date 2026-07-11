//! Runtime shared by all code that `langc gen` emits: source spans and the
//! DFA-walk lexer engine. Generated crates depend on this crate and on
//! nothing else.

mod cursor;
mod elab;
mod judge;
mod lexer;
mod print;
mod span;
mod tree;

pub use cursor::Cursor;
pub use elab::{
    first_token, nodes_in, nth_node_in, nth_token_of, tokens_of, ElabCtx, ElabReport, Frag,
    PassPhase,
};
pub use judge::{app, atom, set, var, Bail, Contexts, Derivation, Engine, Goal, Rule, Term};
pub use lexer::{regex_escape, LexDfa, RawToken};
pub use print::{print_canonical, sexpr, ParseReport};
pub use span::Span;
pub use tree::{ParseError, ParseOutput, SyntaxElement, SyntaxNode, Token};
