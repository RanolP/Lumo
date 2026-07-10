//! `langc` — the Langue 2 definition compiler.
//!
//! Loads a multi-file `.langue` project (cat + stdlib + DCE, D-05), checks
//! it (`langc check`), and generates Rust for every declared language
//! (`langc gen`, D-21). The `.langue` parser in `syntax/` is the only
//! hand-written parser in the system (D-01).

pub mod check;
pub mod codegen;
pub mod db;
pub mod diag;
pub mod project;
pub mod syntax;
