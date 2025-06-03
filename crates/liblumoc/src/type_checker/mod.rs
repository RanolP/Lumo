mod environment;
mod error;
mod infer;
mod scan;
mod syntax;

pub use environment::Scope;
pub use error::InferError;
pub use infer::{infer_expr, infer_item};
pub use scan::scan;
