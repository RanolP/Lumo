// Re-export sub-crates for backward compatibility
pub use lumo_span as span;
pub use lumo_types as types;
pub use lumo_lexer as lexer;
pub use lumo_lst as lst;
pub use lumo_hir as hir;
pub use lumo_lir as lir;

// Modules that remain in the compiler crate
pub mod backend;
pub mod diagnostics;
pub mod query;
pub mod typecheck;

// Re-export parser from lst for backward compatibility
pub mod parser {
    pub use lumo_lst::parser::*;
}
