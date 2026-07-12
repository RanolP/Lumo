//! D-45/D-53 gate: the stdlib-backed `hello` package, assembled by lbs,
//! must parse, typecheck, and compile to JS. The node smoke run is
//! `scripts/stdlib_smoke.sh`.

use std::path::Path;

fn unit() -> lbs::Unit {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../packages/hello");
    lbs::assemble(&root, "js.node").unwrap()
}

#[test]
fn stdlib_assembles_as_a_bin_with_the_runtime_prelude() {
    let unit = unit();
    assert_eq!(unit.name, "hello");
    assert!(unit.is_bin);
    assert_eq!(unit.preludes.len(), 1, "runtime prelude missing: {:?}", unit.preludes);
    // Per-module merge: a platform half lands right after its common half.
    let files: Vec<String> =
        unit.parts.iter().map(|p| p.file.to_string_lossy().into_owned()).collect();
    let number = files.iter().position(|f| f.ends_with("src/number.lumo")).unwrap();
    assert!(files[number + 1].ends_with("src#js/number.lumo"), "{files:#?}");
}

#[test]
fn stdlib_parses() {
    let unit = unit();
    let out = lumo_syntax::lumo::parser::parse(&unit.text);
    let messages: Vec<String> = out
        .errors
        .iter()
        .map(|e| format!("{}: {}", lbs::locate(&unit, e.span.start as usize).unwrap(), e.message))
        .collect();
    assert!(messages.is_empty(), "stdlib unit has parse errors:\n{}", messages.join("\n"));
}

#[test]
fn stdlib_typechecks() {
    let report = lumo_syntax::judge_driver::infer_report(&unit().text);
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
    let report = lumo_syntax::compile_driver::compile_report(&unit().text);
    assert!(report.errors.is_empty(), "compile failed:\n{}", report.errors.join("\n"));
    assert!(report.output.contains("main"), "no main in emitted JS:\n{}", report.output);
}
