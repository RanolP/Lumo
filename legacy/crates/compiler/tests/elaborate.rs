use lumo_compiler::{elaborate, hir, lir, lir_memaware};
use lir_memaware::Expr;

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

fn has_dup(expr: &Expr) -> bool {
    match expr {
        Expr::Pure(_) => false,
        Expr::Dup { .. } => true,
        Expr::Drop { body, .. } => has_dup(body),
        Expr::IsUnique { unique_branch, shared_branch, .. } => {
            has_dup(unique_branch) || has_dup(shared_branch)
        }
    }
}

fn has_drop(expr: &Expr) -> bool {
    match expr {
        Expr::Pure(_) => false,
        Expr::Drop { .. } => true,
        Expr::Dup { expr, .. } => has_drop(expr),
        Expr::IsUnique { unique_branch, shared_branch, .. } => {
            has_drop(unique_branch) || has_drop(shared_branch)
        }
    }
}

#[test]
fn elaborate_no_dup_for_single_use() {
    let lir_file = parse_and_lower("fn id(x: Number): Number / {} { x }");
    let mem = elaborate::elaborate(&lir_file);
    let fn_decl = mem.items.iter().find_map(|i| {
        if let lir_memaware::Item::Fn(f) = i { Some(f) } else { None }
    }).unwrap();
    assert!(!has_dup(&fn_decl.value), "single-use binding must not be Dup'd");
}

#[test]
fn elaborate_dup_for_repeated_binding() {
    use lumo_compiler::{lir, span, types};

    let body = lir::Expr::Apply {
        id: types::ExprId(3),
        callee: Box::new(lir::Expr::Apply {
            id: types::ExprId(4),
            callee: Box::new(lir::Expr::Ident { id: types::ExprId(5), name: "x".to_owned() }),
            arg:    Box::new(lir::Expr::Ident { id: types::ExprId(6), name: "x".to_owned() }),
        }),
        arg: Box::new(lir::Expr::Number { id: types::ExprId(7), value: "0".to_owned() }),
    };
    let let_expr = lir::Expr::Let {
        id: types::ExprId(0),
        name: "x".to_owned(),
        value: Box::new(lir::Expr::Number { id: types::ExprId(1), value: "1".to_owned() }),
        body: Box::new(body),
    };
    let file = lir::File {
        items: vec![lir::Item::Fn(lir::FnDecl {
            name: "f".to_owned(), generics: vec![], params: vec![],
            return_type: None, cap: None, inline: false,
            span: span::Span::new(0, 0), value: let_expr,
        })],
        content_hash: types::ContentHash(0),
        spans: (0..8).map(|_| span::Span::new(0, 0)).collect(),
    };
    let mem = elaborate::elaborate(&file);
    let fn_decl = mem.items.iter().find_map(|i| {
        if let lir_memaware::Item::Fn(f) = i { Some(f) } else { None }
    }).unwrap();
    assert!(has_dup(&fn_decl.value), "binding used twice must produce a Dup node");
}

#[test]
fn elaborate_drop_for_unused_binding() {
    use lumo_compiler::{lir, span, types};

    let unused_let = lir::Expr::Let {
        id: types::ExprId(0),
        name: "_unused".to_owned(),
        value: Box::new(lir::Expr::Number { id: types::ExprId(1), value: "1".to_owned() }),
        body:  Box::new(lir::Expr::Number { id: types::ExprId(2), value: "2".to_owned() }),
    };
    let file = lir::File {
        items: vec![lir::Item::Fn(lir::FnDecl {
            name: "f".to_owned(), generics: vec![], params: vec![],
            return_type: None, cap: None, inline: false,
            span: span::Span::new(0, 0), value: unused_let,
        })],
        content_hash: types::ContentHash(0),
        spans: vec![span::Span::new(0, 0); 3],
    };
    let mem = elaborate::elaborate(&file);
    let mfn = mem.items.iter().find_map(|i| {
        if let lir_memaware::Item::Fn(f) = i { Some(f) } else { None }
    }).unwrap();
    assert!(has_drop(&mfn.value), "unused binding should produce a Drop node");
}
