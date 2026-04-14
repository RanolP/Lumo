use lumo_compiler::{
    hir::{self, Expr, Item},
    lexer::lex,
    lir,
    lst::lossless,
    parser::parse,
    types::TypeExpr,
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

fn unwrap_lir_fn_value<'a>(func: &'a lir::FnDecl) -> (&'a lir::Expr, Vec<&'a str>) {
    let lir::Expr::Thunk { expr, .. } = &func.value else {
        panic!("expected thunked function value")
    };

    let mut params = Vec::new();
    let mut cursor = expr.as_ref();
    while let lir::Expr::Lambda { param, body, .. } = cursor {
        params.push(param.as_str());
        cursor = body.as_ref();
    }
    (cursor, params)
}

#[test]
fn content_hash_is_deterministic() {
    let src = "data Option[A] { .some(A), .none } fn id[A](a: A): A / {} { a }";
    let a = lower_typed(src);
    let b = lower_typed(src);
    assert_eq!(a.content_hash, b.content_hash);
    assert_eq!(a, b);
}

#[test]
fn body_change_affects_file_content_hash() {
    let a = lower_typed("fn f(): A / {} { x }");
    let b = lower_typed("fn f(): A / {} { y }");
    assert_ne!(a.content_hash, b.content_hash);
}

#[test]
fn expression_nodes_have_spans() {
    let file = lower_typed("fn f(): A / {} { let x = y; x }");
    let func = match &file.items[0] {
        Item::Fn(f) => f,
        _ => panic!("expected fn"),
    };

    match &func.body {
        Expr::Let {
            value,
            body,
            span,
            ..
        } => {
            assert!(span.start < span.end);
            match value.as_ref() {
                Expr::Ident { span, .. } => assert!(span.start < span.end),
                _ => panic!("expected ident"),
            }
            match body.as_ref() {
                Expr::Produce { span, expr, .. } => {
                    assert!(span.start < span.end);
                    match expr.as_ref() {
                        Expr::Ident { span, .. } => assert!(span.start < span.end),
                        _ => panic!("expected ident"),
                    }
                }
                _ => panic!("expected produce"),
            }
        }
        _ => panic!("expected let"),
    }
}

#[test]
fn lossless_lower_handles_let_and_produce() {
    let file = lower_lossless("fn f() { let x = y; x }");
    let Item::Fn(f) = &file.items[0] else {
        panic!("expected fn")
    };

    match &f.body {
        Expr::Let {
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
        other => panic!("expected let body, got {other:?}"),
    }
}

#[test]
fn query_path_matches_direct_lossless_lower() {
    let src = "data X { .a, .b } fn f() { x }";

    let direct = lower_typed(src);

    let mut q = lumo_compiler::query::QueryEngine::new();
    q.set_file("main.lumo", src);
    let via_query = q.lower_hir("main.lumo").expect("lowered");

    assert_eq!(direct, via_query);
}

#[test]
fn typed_and_lossless_lower_match_on_mvp_samples() {
    let cases = [
        "fn f() { x }",
        "fn f() { let x = y; x }",
        "data Option[A] { .some, .none } fn id() { a }",
        "data Pair { .pair } fn mk() { let p = q; p }",
    ];

    for src in cases {
        let typed = lower_typed(src);
        let lossless = lower_lossless(src);
        assert_eq!(typed, lossless, "mismatch on source: {src}");
    }
}

#[test]
fn hir_keeps_match_scrutinee_as_user_syntax() {
    let file = lower_typed("fn f() { match a { x => x } }");
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
    let file = lower_typed("data OptionA { .some(A), .none } fn mk() { OptionA.some(a) }");
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
    let file = lower_lir("fn f() { match a { x => x } }");
    let lir::Item::Fn(f) = &file.items[0] else {
        panic!("expected fn")
    };
    let (body, params) = unwrap_lir_fn_value(f);
    assert!(params.is_empty(), "unexpected params: {params:?}");

    match body {
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
fn lir_inlines_data_ctor_bundle_as_rolled_ctor() {
    let file = lower_lir("data OptionA { .some(A), .none } fn mk() { OptionA.some(a) }");
    let lir::Item::Fn(f) = &file.items[1] else {
        panic!("expected fn")
    };
    let (body, params) = unwrap_lir_fn_value(f);
    assert!(params.is_empty(), "unexpected params: {params:?}");

    match body {
        lir::Expr::Roll { expr, .. } => match expr.as_ref() {
            lir::Expr::Ctor { name, args, .. } => {
                assert_eq!(name, "OptionA.some");
                assert_eq!(args.len(), 1);
            }
            other => panic!("expected ctor inside roll, got {other:?}"),
        },
        other => panic!("expected rolled ctor bundle body, got {other:?}"),
    }
}

#[test]
fn lir_lowers_fn_item_to_thunk_lambda_spine() {
    let file = lower_lir("fn id(x: A, y: B): A { x }");
    let lir::Item::Fn(f) = &file.items[0] else {
        panic!("expected fn")
    };
    let (body, params) = unwrap_lir_fn_value(f);

    assert_eq!(params, vec!["x", "y"]);
    match body {
        lir::Expr::Produce { expr, .. } => match expr.as_ref() {
            lir::Expr::Ident { name, .. } => assert_eq!(name, "x"),
            other => panic!("expected produce ident, got {other:?}"),
        },
        other => panic!("expected produced body, got {other:?}"),
    }
}

#[test]
fn lir_lowers_function_call_to_force_apply_chain() {
    let file = lower_lir("fn main(x: A, y: B): C { f(x, y) }");
    let lir::Item::Fn(f) = &file.items[0] else {
        panic!("expected fn")
    };
    let (body, params) = unwrap_lir_fn_value(f);

    assert_eq!(params, vec!["x", "y"]);
    let lir::Expr::Apply { callee, arg, .. } = body else {
        panic!("expected outer apply")
    };
    let lir::Expr::Ident { name, .. } = arg.as_ref() else {
        panic!("expected second arg ident")
    };
    assert_eq!(name, "y");

    let lir::Expr::Apply {
        callee: inner_callee,
        arg: inner_arg,
        ..
    } = callee.as_ref()
    else {
        panic!("expected inner apply")
    };
    let lir::Expr::Ident { name, .. } = inner_arg.as_ref() else {
        panic!("expected first arg ident")
    };
    assert_eq!(name, "x");

    let lir::Expr::Force { expr, .. } = inner_callee.as_ref() else {
        panic!("expected forced callee")
    };
    let lir::Expr::Ident { name, .. } = expr.as_ref() else {
        panic!("expected function ident")
    };
    assert_eq!(name, "f");
}

#[test]
fn data_variant_payload_types_are_preserved_in_hir() {
    let file = lower_typed("data Maybe { .some(Bool), .none }");
    let Item::Data(d) = &file.items[0] else {
        panic!("expected data")
    };
    assert_eq!(d.variants[0].name, "some");
    assert_eq!(d.variants[0].payload.len(), 1);
    assert_eq!(d.variants[0].payload[0].value, TypeExpr::Named("Bool".into()));
    assert_eq!(d.variants[1].name, "none");
    assert!(d.variants[1].payload.is_empty());
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

#[test]
fn cap_decl_is_preserved_in_hir() {
    let file = lower_typed("cap Console { fn log(msg: String): Unit }");
    let Item::Cap(e) = &file.items[0] else {
        panic!("expected cap, got {:?}", file.items[0])
    };
    assert_eq!(e.name, "Console");
    assert_eq!(e.operations.len(), 1);
    assert_eq!(e.operations[0].name, "log");
    assert_eq!(e.operations[0].params.len(), 1);
    assert_eq!(e.operations[0].params[0].name, "msg");
    assert_eq!(
        e.operations[0].params[0].ty.value,
        TypeExpr::Named("String".into())
    );
    // Return type is "Unit"
    assert!(e.operations[0].return_type.is_some());

    // Hash stability: re-lowering yields the same hash
    let file2 = lower_typed("cap Console { fn log(msg: String): Unit }");
    assert_eq!(file.content_hash, file2.content_hash);
}
