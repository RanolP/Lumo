//! The one aggregating corpus test (D-32): globs
//! `tests/fixtures/syn/**/*.test` at the repo root and drives every case
//! through the generated registry. Bless expected blocks with
//! `LANGC_UPDATE=1 cargo test -p lumo-syntax --test corpus`.

use std::path::Path;

#[test]
fn corpus() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/syn");
    let lookup =
        |lang: &str| lumo_syntax::registry::language(lang).map(|ops| ops.parse_report);
    match langc::corpus::run_dir(&root, lookup) {
        Ok(summary) => println!("corpus: {summary}"),
        Err(failures) => panic!("corpus failures\n{failures}"),
    }
}
