//! Generated syntax crate for the Lumo definition (D-21: all generated
//! code is committed). Everything under `lumo/` is emitted by
//! `langc gen lumo -o crates/lumo-syntax/src` — edit the `.langue` files
//! at the repo-root `lumo/` directory instead, then regenerate.

pub mod compile_driver;
pub mod elab;
pub mod elab_externs;
pub mod js;
pub mod judge_driver;
pub mod lumo;
pub mod mir;
#[cfg(feature = "optimize")]
pub mod optimize_driver;
pub mod registry;
