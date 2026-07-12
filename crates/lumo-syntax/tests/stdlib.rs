//! D-45 gate: the ported stdlib (`packages/stdlib.manifest`, one
//! concatenated compilation unit) must parse, typecheck, and compile
//! to JS. The node smoke run is `scripts/stdlib_smoke.sh`.

use std::path::Path;

fn unit() -> String {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../packages");
    let manifest = std::fs::read_to_string(root.join("stdlib.manifest")).unwrap();
    let mut parts = Vec::new();
    for line in manifest.lines() {
        let line = line.trim();
        // Comment lines start with `#`; source paths only contain it
        // mid-string (`src#js`).
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        parts.push(
            std::fs::read_to_string(root.join(line)).unwrap_or_else(|e| panic!("{line}: {e}")),
        );
    }
    parts.join("\n")
}

#[test]
fn stdlib_parses() {
    let out = lumo_syntax::lumo::parser::parse(&unit());
    let messages: Vec<String> =
        out.errors.iter().map(|e| format!("{}: {}", e.span, e.message)).collect();
    assert!(messages.is_empty(), "stdlib unit has parse errors:\n{}", messages.join("\n"));
}

#[test]
fn stdlib_typechecks() {
    let report = lumo_syntax::judge_driver::infer_report(&unit());
    assert!(report.errors.is_empty(), "infer failed:\n{}", report.errors.join("\n"));
    assert!(!report.output.starts_with("ERROR"), "stdlib unit has a type error:\n{}", report.output);
    for line in [
        "__impl_NumOps : NumOps",
        "__impl_StrOps : StrOps",
        "__impl_IO : IO",
        "__impl_FS : FS",
        "__impl_Process : Process",
    ] {
        assert!(report.output.contains(line), "missing `{line}` in:\n{}", report.output);
    }
}

#[test]
fn stdlib_compiles_to_js() {
    let report = lumo_syntax::compile_driver::compile_report(&unit());
    assert!(report.errors.is_empty(), "compile failed:\n{}", report.errors.join("\n"));
    assert!(report.output.contains("main"), "no main in emitted JS:\n{}", report.output);
}
