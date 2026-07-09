use lumo_compiler::{
    hir,
    lir,
    lto,
};

fn lower(src: &str) -> lir::File {
    let lossless = lumo_compiler::lst::lossless::parse(src);
    let hir = hir::lower_lossless(&lossless);
    lir::lower(&hir)
}

#[test]
fn lto_optimize_is_callable_and_idempotent_on_no_op_input() {
    let mut file = lower("fn id(x: Number): Number { x }");
    let before = file.clone();
    let errors = lto::optimize(&mut file);
    assert!(errors.is_empty(), "expected no errors, got: {errors:?}");
    // No caps, no Performs — nothing to do; file stays bit-equal.
    assert_eq!(file, before);
}

#[test]
#[ignore]
fn cap_annotation_debug() {
    let src = "cap E { fn op(): A }\nfn inner(): A / { E } { E.op }\nfn outer(): A / {} { inner() }";
    let lossless = lumo_compiler::lst::lossless::parse(src);
    let hir = lumo_compiler::hir::lower_lossless(&lossless);
    for item in &hir.items {
        if let lumo_compiler::hir::Item::Fn(f) = item {
            eprintln!("fn {} cap={:?}", f.name, f.cap);
        }
    }
    eprintln!("hir errors: {:?}", hir.errors);
    panic!("debug output above");
}

#[test]
#[ignore]
fn typecheck_cap_debug() {
    let src = "cap E { fn op(): A }\nfn inner(): A / { E } { E.op }\nfn outer(): A / {} { inner() }";
    let lossless = lumo_compiler::lst::lossless::parse(src);
    let hir = lumo_compiler::hir::lower_lossless(&lossless);
    let lir_file = lumo_compiler::lir::lower(&hir);
    let (_, errors) = lumo_compiler::typecheck::typecheck_and_bindings(&lir_file);
    eprintln!("typecheck errors: {:?}", errors);
    panic!("debug output above");
}

#[test]
#[ignore]
fn lir_cap_debug() {
    let src = "cap E { fn op(): A }\nfn inner(): A / { E } { E.op }\nfn outer(): A / {} { inner() }";
    let lossless = lumo_compiler::lst::lossless::parse(src);
    let hir = lumo_compiler::hir::lower_lossless(&lossless);
    let lir_file = lumo_compiler::lir::lower(&hir);
    for item in &lir_file.items {
        if let lumo_compiler::lir::Item::Fn(f) = item {
            eprintln!("fn {} caps={:?}", f.name, f.cap);
        }
    }
    panic!("debug output above");
}
