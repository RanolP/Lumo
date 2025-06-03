mod environment;
mod error;
mod infer;

pub use environment::Scope;
pub use error::InferError;
pub use infer::{infer_expr, infer_item};
