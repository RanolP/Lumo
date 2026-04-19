//! Link-Time Optimization for capability dispatch.
//!
//! See `docs/superpowers/specs/2026-04-19-lto-cap-monomorphization-design.md`.

use lumo_lir as lir;

use crate::diagnostics::Diagnostic;

pub mod call_graph;
pub use call_graph::*;

pub mod resolution;
pub use resolution::*;

pub mod dep_free;
pub use dep_free::*;

pub mod emit;
pub use emit::*;

pub mod dce;

/// Run LTO optimizations and return any validation diagnostics.
///
/// An empty `Vec` means success. A non-empty `Vec` means at least one
/// hard error was found (e.g. `#[inline(always)]` on an unresolvable fn)
/// and the caller should abort compilation.
pub fn optimize(file: &mut lir::File) -> Vec<Diagnostic> {
    let resolution = resolution::build_resolution_map(file);
    let cg = call_graph::build_call_graph(file);
    let analysis = dep_free::run(file, &resolution, &cg);

    // Validate: every fn marked #[inline(always)] must be proven dep-free.
    let mut errors: Vec<Diagnostic> = Vec::new();
    for item in &file.items {
        let lir::Item::Fn(f) = item else { continue };
        if !f.inline {
            continue;
        }
        let key = (f.name.clone(), Vec::<String>::new());
        if !matches!(analysis.status.get(&key), Some(dep_free::DepFreeStatus::DepFree)) {
            errors.push(Diagnostic {
                start: f.span.start,
                end: f.span.end,
                message: format!(
                    "fn `{}` is marked #[inline(always)] but has unresolved capability; \
                     remove the attribute or provide a default impl",
                    f.name
                ),
            });
        }
    }

    if errors.is_empty() {
        emit::transform(file, &analysis);
        dce::sweep(file);
    }

    errors
}
