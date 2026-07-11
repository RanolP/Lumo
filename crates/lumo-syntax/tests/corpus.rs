//! The one aggregating corpus test (D-32): globs
//! `tests/fixtures/**/*.test` at the repo root (`syn` and `elab`) and
//! drives every case through the generated registry. Bless expected
//! blocks with `LANGC_UPDATE=1 cargo test -p lumo-syntax --test corpus`.

use std::path::Path;

#[test]
fn corpus() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures");
    let lookup =
        |lang: &str| lumo_syntax::registry::language(lang).map(|ops| ops.parse_report);
    let elab_lookup = |from: &str, to: &str| {
        lumo_syntax::registry::elab(from, to).map(|ops| ops.elab_report)
    };
    // The `:infer(Lumo)` driver is handwritten (M2 step 7) — it seeds
    // judgment contexts from the Lumo tree, so it lives next to the
    // extern impls rather than in the generated registry.
    let infer_lookup = |lang: &str| {
        (lang == "Lumo")
            .then_some(lumo_syntax::judge_driver::infer_report as langc::corpus::InferFn)
    };
    match langc::corpus::run_dir(&root, lookup, elab_lookup, infer_lookup) {
        Ok(summary) => println!("corpus: {summary}"),
        Err(failures) => panic!("corpus failures\n{failures}"),
    }
}
