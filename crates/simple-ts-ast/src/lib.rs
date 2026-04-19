pub mod ast;
pub mod emit;
pub mod pass;
#[cfg(test)]
mod tests;

pub use ast::*;
pub use emit::{EmitTarget, Emitter};
pub use pass::{
    expr_to_block, flatten_iifes, inline_always_calls, inline_trivial_consts,
    lower_expression_bodies, return_lifting,
};
