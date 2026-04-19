//! Link-Time Optimization for capability dispatch.
//!
//! See `docs/superpowers/specs/2026-04-19-lto-cap-monomorphization-design.md`.

use lumo_lir as lir;

pub fn optimize(_file: &mut lir::File) {
    // Phases will be added in subsequent tasks:
    // 1. resolution::build
    // 2. call_graph::build
    // 3. dep_free::run
    // 4. emit::transform
    // 5. dce::sweep
}
