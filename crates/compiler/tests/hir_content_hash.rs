use lumo_compiler::{
    hir::{self, Expr, Item},
    lexer::lex,
    lir,
    lst::lossless,
    parser::parse,
};

fn lower_typed(src: &str) -> hir::File {
    let lexed = lex(src);
    let parsed = parse(&lexed.tokens, &lexed.errors);
    hir::lower(&parsed.file)
}

fn lower_lossless(src: &str) -> hir::File {
    let parsed = lossless::parse(src);
    hir::lower_lossless(&parsed)
}

fn lower_lir(src: &str) -> lir::File {
    let hir = lower_typed(src);
    lir::lower(&hir)
}

#[test]
fn content_hash_is_deterministic() {
    let src = "data Option[A] { .some(A), .none } fn id[A](a: A): produce A / {} := produce a";
    let a = lower_typed(src);
    let b = lower_typed(src);
    assert_eq!(a.content_hash, b.content_hash);
    assert_eq!(a, b);
}

#[test]
fn top_level_name_does_not_affect_fn_id() {
    let a = lower_typed("fn alpha(): produce A / {} := produce x");
    let b = lower_typed("fn beta(): produce A / {} := produce x");

    let fn_a = match &a.items[0] {
        Item::Fn(f) => f,
        _ => panic!("expected fn"),
    };
    let fn_b = match &b.items[0] {
        Item::Fn(f) => f,
        _ => panic!("expected fn"),
    };

    assert_eq!(fn_a.id, fn_b.id);
    assert_eq!(fn_a.structural_hash, fn_b.structural_hash);
}

#[test]
fn body_change_affects_fn_id() {
    let a = lower_typed("fn f(): produce A / {} := produce x");
    let b = lower_typed("fn f(): produce A / {} := produce y");

    let fn_a = match &a.items[0] {
        Item::Fn(f) => f,
        _ => panic!("expected fn"),
    };
    let fn_b = match &b.items[0] {
        Item::Fn(f) => f,
        _ => panic!("expected fn"),
    };

    assert_ne!(fn_a.id, fn_b.id);
}

#[test]
fn expression_nodes_have_content_hash_ids() {
    let file = lower_typed("fn f(): produce A / {} := let x = y in produce x");
    let func = match &file.items[0] {
        Item::Fn(f) => f,
        _ => panic!("expected fn"),
    };

    match &func.body {
        Expr::LetIn {
            id,
            structural_hash,
            value,
            body,
            ..
        } => {
            assert_eq!(id, structural_hash);
            match value.as_ref() {
                Expr::Ident {
                    id,
                    structural_hash,
                    ..
                } => assert_eq!(id, structural_hash),
                _ => panic!("expected ident"),
            }
            match body.as_ref() {
                Expr::Produce {
                    id,
                    structural_hash,
                    expr,
                } => {
                    assert_eq!(id, structural_hash);
                    match expr.as_ref() {
                        Expr::Ident {
                            id,
                            structural_hash,
                            ..
                        } => assert_eq!(id, structural_hash),
                        _ => panic!("expected ident"),
                    }
                }
                _ => panic!("expected produce"),
            }
        }
        _ => panic!("expected let-in"),
    }
}

#[test]
fn lossless_lower_handles_let_and_produce() {
    let file = lower_lossless("fn f() := let x = y in produce x");
    let Item::Fn(f) = &file.items[0] else {
        panic!("expected fn")
    };

    match &f.body {
        Expr::LetIn {
            name, value, body, ..
        } => {
            assert_eq!(name, "x");
            match value.as_ref() {
                Expr::Ident { name, .. } => assert_eq!(name, "y"),
                other => panic!("expected ident value, got {other:?}"),
            }
            match body.as_ref() {
                Expr::Produce { expr, .. } => match expr.as_ref() {
                    Expr::Ident { name, .. } => assert_eq!(name, "x"),
                    other => panic!("expected produce ident, got {other:?}"),
                },
                other => panic!("expected produce body, got {other:?}"),
            }
        }
        other => panic!("expected let-in body, got {other:?}"),
    }
}

#[test]
fn query_path_matches_direct_lossless_lower() {
    let src = "data X { .a, .b } fn f() := produce x";

    let direct = lower_typed(src);

    let mut q = lumo_compiler::query::QueryEngine::new();
    q.set_file("main.lumo", src);
    let via_query = q.lower_hir("main.lumo").expect("lowered");

    assert_eq!(direct, via_query);
}

#[test]
fn typed_and_lossless_lower_match_on_mvp_samples() {
    let cases = [
        "fn f() := produce x",
        "fn f() := let x = y in produce x",
        "data Option[A] { .some, .none } fn id() := produce a",
        "data Pair { .pair } fn mk() := let p = q in p",
    ];

    for src in cases {
        let typed = lower_typed(src);
        let lossless = lower_lossless(src);
        assert_eq!(typed, lossless, "mismatch on source: {src}");
    }
}

#[test]
fn hir_keeps_match_scrutinee_as_user_syntax() {
    let file = lower_typed("fn f() := match a { x => produce x }");
    let Item::Fn(f) = &file.items[0] else {
        panic!("expected fn")
    };

    match &f.body {
        Expr::Match { scrutinee, .. } => match scrutinee.as_ref() {
            Expr::Ident { name, .. } => assert_eq!(name, "a"),
            other => panic!("expected raw ident scrutinee, got {other:?}"),
        },
        other => panic!("expected match body, got {other:?}"),
    }
}

#[test]
fn hir_keeps_ctor_call_as_user_syntax() {
    let file = lower_typed("data OptionA { .some(A), .none } fn mk() := OptionA.some(a)");
    let Item::Fn(f) = &file.items[1] else {
        panic!("expected fn")
    };

    match &f.body {
        Expr::Call { callee, args, .. } => {
            let Expr::Member { object, member, .. } = callee.as_ref() else {
                panic!("expected member callee")
            };
            let Expr::Ident { name, .. } = object.as_ref() else {
                panic!("expected ident owner")
            };
            assert_eq!(name, "OptionA");
            assert_eq!(member, "some");
            assert_eq!(args.len(), 1);
        }
        other => panic!("expected user-level call, got {other:?}"),
    }
}

#[test]
fn lir_inserts_implicit_unroll_for_match_scrutinee() {
    let file = lower_lir("fn f() := match a { x => produce x }");
    let lir::Item::Fn(f) = &file.items[0] else {
        panic!("expected fn")
    };

    match &f.body {
        lir::Expr::Match { scrutinee, .. } => match scrutinee.as_ref() {
            lir::Expr::Unroll { expr, .. } => match expr.as_ref() {
                lir::Expr::Ident { name, .. } => assert_eq!(name, "a"),
                other => panic!("expected ident in unroll, got {other:?}"),
            },
            other => panic!("expected implicit unroll scrutinee, got {other:?}"),
        },
        other => panic!("expected match body, got {other:?}"),
    }
}

#[test]
fn lir_inserts_implicit_roll_for_data_ctor() {
    let file = lower_lir("data OptionA { .some(A), .none } fn mk() := OptionA.some(a)");
    let lir::Item::Fn(f) = &file.items[1] else {
        panic!("expected fn")
    };

    match &f.body {
        lir::Expr::Roll { expr, .. } => match expr.as_ref() {
            lir::Expr::Ctor { name, args, .. } => {
                assert_eq!(name, "OptionA.some");
                assert_eq!(args.len(), 1);
            }
            other => panic!("expected ctor inside roll, got {other:?}"),
        },
        other => panic!("expected implicit roll, got {other:?}"),
    }
}

#[test]
fn data_variant_payload_types_are_preserved_in_hir() {
    let file = lower_typed("data Maybe { .some(Bool), .none }");
    let Item::Data(d) = &file.items[0] else {
        panic!("expected data")
    };
    assert_eq!(d.variants[0].name, "some");
    assert_eq!(d.variants[0].payload_types, vec!["Bool"]);
    assert_eq!(d.variants[1].name, "none");
    assert!(d.variants[1].payload_types.is_empty());
}

#[test]
fn data_generics_are_preserved_in_hir() {
    let typed = lower_typed("data Option[A, B] { .some(A), .none }");
    let lossless = lower_lossless("data Option[A, B] { .some, .none }");
    let Item::Data(d) = &typed.items[0] else {
        panic!("expected data")
    };
    assert_eq!(d.generics, vec!["A", "B"]);
    let Item::Data(ld) = &lossless.items[0] else {
        panic!("expected data")
    };
    assert_eq!(ld.generics, vec!["A", "B"]);
}
