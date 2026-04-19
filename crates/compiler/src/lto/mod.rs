//! Link-Time Optimization for capability dispatch.
//!
//! See `docs/superpowers/specs/2026-04-19-lto-cap-monomorphization-design.md`.

use lumo_lir as lir;

pub mod call_graph;
pub use call_graph::*;

pub mod resolution;
pub use resolution::*;

pub mod dep_free;
pub use dep_free::*;

pub mod emit;
pub use emit::*;

pub fn optimize(file: &mut lir::File) {
    let resolution = resolution::build_resolution_map(file);
    let cg = call_graph::build_call_graph(file);
    let analysis = dep_free::run(file, &resolution, &cg);
    emit::transform(file, &analysis);
}
