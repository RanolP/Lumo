use lumo_compiler::{elaborate, hir, lir, lir_memaware};

fn parse_and_lower(src: &str) -> lir::File {
    let lossless = lumo_compiler::lst::lossless::parse(src);
    let hir = hir::lower_lossless(&lossless);
    lir::lower(&hir)
}

#[test]
fn elaborate_trivial_fn() {
    let lir_file = parse_and_lower("fn id(x: Number): Number / {} { x }");
    let mem_file = elaborate::elaborate(&lir_file);

    // There should be exactly one Fn item.
    let fn_decl = mem_file.items.iter().find_map(|i| {
        if let lir_memaware::Item::Fn(f) = i { Some(f) } else { None }
    }).expect("no Fn item");

    assert_eq!(fn_decl.name, "id");

    // The body should be Pure wrapping the original lir expr.
    assert!(
        matches!(&fn_decl.value, lir_memaware::Expr::Pure(_)),
        "expected Pure(…), got {:?}", fn_decl.value
    );
}

#[test]
fn elaborate_impl_method() {
    let src = r#"
        data List[A] { .nil, .cons(A, List[A]) }
        impl[T] List[T] { fn len(self: List[T]): Number / {} { 0 } }
    "#;
    let lir_file = parse_and_lower(src);
    let mem_file = elaborate::elaborate(&lir_file);

    let impl_decl = mem_file.items.iter().find_map(|i| {
        if let lir_memaware::Item::Impl(d) = i { Some(d) } else { None }
    }).expect("no Impl item");

    let method = impl_decl.methods.first().expect("no methods");
    assert!(
        matches!(&method.value, lir_memaware::Expr::Pure(_)),
        "expected Pure(…), got {:?}", method.value
    );
}
