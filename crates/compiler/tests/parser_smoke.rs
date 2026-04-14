use lumo_compiler::{
    lexer::lex,
    parser::{parse, BinaryOp, Expr, Item, UnaryOp},
};

#[test]
fn parses_data_and_fn() {
    let src = "data Option[A] { .some(A), .none } fn id[A](a: A): A / {} { a }";
    let lexed = lex(src);
    let parsed = parse(&lexed.tokens, &lexed.errors);

    assert_eq!(parsed.file.items.len(), 2);
    assert!(parsed.errors.is_empty(), "errors: {:?}", parsed.errors);

    match &parsed.file.items[0] {
        Item::Data(data) => {
            assert_eq!(data.name, "Option");
            assert_eq!(data.generics.len(), 1);
            assert_eq!(data.generics[0].name, "A");
            assert_eq!(data.variants.len(), 2);
            assert_eq!(data.variants[0].name, "some");
            assert_eq!(data.variants[0].payload.len(), 1);
            assert_eq!(data.variants[0].payload[0].repr, "A");
            assert_eq!(data.variants[1].name, "none");
            assert!(data.variants[1].payload.is_empty());
        }
        other => panic!("unexpected first item: {other:?}"),
    }

    match &parsed.file.items[1] {
        Item::Fn(f) => {
            let Expr::Block { stmts, result, .. } = &f.body else {
                panic!("expected block body, got {:?}", f.body)
            };
            assert!(stmts.is_empty());
            match result.as_ref() {
                Expr::Ident { name, .. } => assert_eq!(name, "a"),
                other => panic!("unexpected block result: {other:?}"),
            }
        }
        other => panic!("unexpected second item: {other:?}"),
    }

    let Item::Fn(f) = &parsed.file.items[1] else {
        panic!("expected fn item")
    };
    assert_eq!(f.generics.len(), 1);
    assert_eq!(f.generics[0].name, "A");
    assert!(f.generics[0].constraint.is_none());
    assert_eq!(f.params.len(), 1);
    assert_eq!(f.params[0].name, "a");
    assert_eq!(f.params[0].ty.repr, "A");
    assert_eq!(
        f.return_type.as_ref().map(|t| t.repr.as_str()),
        Some("A")
    );
    assert_eq!(f.cap.as_ref().map(|e| e.repr.as_str()), Some("{ }"));
}

#[test]
fn parses_let_in_expression() {
    let src = "fn main(): A / {} { let x = y; x }";
    let lexed = lex(src);
    let parsed = parse(&lexed.tokens, &lexed.errors);

    assert_eq!(parsed.file.items.len(), 1);
    assert!(parsed.errors.is_empty(), "errors: {:?}", parsed.errors);

    let Item::Fn(f) = &parsed.file.items[0] else {
        panic!("expected fn item")
    };

    use lumo_compiler::parser::BlockStmt;
    let Expr::Block { stmts, result, .. } = &f.body else {
        panic!("expected block body, got {:?}", f.body)
    };
    assert_eq!(stmts.len(), 1);
    let BlockStmt::Let { name, value, .. } = &stmts[0] else {
        panic!("expected let stmt")
    };
    assert_eq!(name, "x");
    match value {
        Expr::Ident { name, .. } => assert_eq!(name, "y"),
        other => panic!("unexpected let value: {other:?}"),
    }
    match result.as_ref() {
        Expr::Ident { name, .. } => assert_eq!(name, "x"),
        other => panic!("unexpected block result: {other:?}"),
    }
}

#[test]
fn recovers_and_keeps_parsing_next_item() {
    let src = "fn broken { data Good { .a } }";
    let lexed = lex(src);
    let parsed = parse(&lexed.tokens, &lexed.errors);

    assert!(!parsed.errors.is_empty());
    assert_eq!(parsed.file.items.len(), 2, "items: {:?}", parsed.file.items);

    match &parsed.file.items[1] {
        Item::Data(data) => assert_eq!(data.name, "Good"),
        other => panic!("expected recovered data item, got {other:?}"),
    }
}

#[test]
fn parses_thunk_force_match() {
    let src = "fn f() { match x { left => thunk a, right => force job } }";
    let lexed = lex(src);
    let parsed = parse(&lexed.tokens, &lexed.errors);
    assert!(parsed.errors.is_empty(), "errors: {:?}", parsed.errors);

    let Item::Fn(f) = &parsed.file.items[0] else {
        panic!("expected fn item")
    };
    let Expr::Block { stmts, result, .. } = &f.body else {
        panic!("expected block body")
    };
    assert!(stmts.is_empty());
    let Expr::Match {
        scrutinee, arms, ..
    } = result.as_ref()
    else {
        panic!("expected match body")
    };
    match scrutinee.as_ref() {
        Expr::Ident { name, .. } => assert_eq!(name, "x"),
        other => panic!("unexpected scrutinee: {other:?}"),
    }
    assert_eq!(arms.len(), 2);
    assert_eq!(arms[0].pattern, "left");
    assert_eq!(arms[1].pattern, "right");
}

#[test]
fn parses_apply_expression() {
    let src = "fn mk() { Option.some(a) }";
    let lexed = lex(src);
    let parsed = parse(&lexed.tokens, &lexed.errors);
    assert!(parsed.errors.is_empty(), "errors: {:?}", parsed.errors);

    let Item::Fn(f) = &parsed.file.items[0] else {
        panic!("expected fn item")
    };
    let Expr::Block { result, .. } = &f.body else {
        panic!("expected block body")
    };
    let Expr::Call { callee, args, .. } = result.as_ref() else {
        panic!("expected call body")
    };
    let Expr::Member { object, member, .. } = callee.as_ref() else {
        panic!("expected member callee")
    };
    let Expr::Ident { name, .. } = object.as_ref() else {
        panic!("expected ident object")
    };
    assert_eq!(name, "Option");
    assert_eq!(member, "some");
    assert_eq!(args.len(), 1);
}

#[test]
fn parses_projection_without_call_for_nullary_ctor() {
    let src = "fn t() { Bool.true }";
    let lexed = lex(src);
    let parsed = parse(&lexed.tokens, &lexed.errors);
    assert!(parsed.errors.is_empty(), "errors: {:?}", parsed.errors);

    let Item::Fn(f) = &parsed.file.items[0] else {
        panic!("expected fn item")
    };
    let Expr::Block { result, .. } = &f.body else {
        panic!("expected block body")
    };
    let Expr::Member { object, member, .. } = result.as_ref() else {
        panic!("expected member body")
    };
    let Expr::Ident { name, .. } = object.as_ref() else {
        panic!("expected ident object")
    };
    assert_eq!(name, "Bool");
    assert_eq!(member, "true");
}

#[test]
fn parses_extern_items_with_attributes() {
    let src =
        "#[extern = \"string\"] extern type String;\n#[extern(name = \"console.log\")] extern fn console_log(msg: String);";
    let lexed = lex(src);
    let parsed = parse(&lexed.tokens, &lexed.errors);
    assert!(parsed.errors.is_empty(), "errors: {:?}", parsed.errors);
    assert_eq!(parsed.file.items.len(), 2);

    let Item::ExternType(ext_ty) = &parsed.file.items[0] else {
        panic!("expected extern type item")
    };
    assert_eq!(ext_ty.name, "String");
    assert_eq!(ext_ty.attrs.len(), 1);
    assert_eq!(ext_ty.attrs[0].name, "extern");
    assert!(ext_ty.attrs[0].args.is_empty());
    let Some(Expr::String { value, .. }) = ext_ty.attrs[0].value.as_ref() else {
        panic!("expected direct extern attribute string value")
    };
    assert_eq!(value, "string");

    let Item::ExternFn(ext_fn) = &parsed.file.items[1] else {
        panic!("expected extern fn item")
    };
    assert_eq!(ext_fn.name, "console_log");
    assert_eq!(ext_fn.attrs.len(), 1);
    let Expr::String { value, .. } = &ext_fn.attrs[0].args[0].value else {
        panic!("expected extern name arg string value")
    };
    assert_eq!(value, "console.log");
    assert_eq!(ext_fn.params.len(), 1);
    assert_eq!(ext_fn.params[0].name, "msg");
    assert_eq!(ext_fn.params[0].ty.repr, "String");
}

#[test]
fn parses_extern_fn_without_semicolon_before_next_item() {
    let src = "#[extern(name = \"console.log\")] extern fn console_log(msg: String) fn main(msg: String): Unit { console_log(msg) }";
    let lexed = lex(src);
    let parsed = parse(&lexed.tokens, &lexed.errors);
    assert!(parsed.errors.is_empty(), "errors: {:?}", parsed.errors);
    assert_eq!(parsed.file.items.len(), 2);
    assert!(matches!(parsed.file.items[0], Item::ExternFn(_)));
    assert!(matches!(parsed.file.items[1], Item::Fn(_)));
}

#[test]
fn parses_use_simple_path() {
    let src = "use myapp.utils.helper;";
    let lexed = lex(src);
    let parsed = parse(&lexed.tokens, &lexed.errors);
    assert!(parsed.errors.is_empty(), "errors: {:?}", parsed.errors);
    assert_eq!(parsed.file.items.len(), 1);

    let Item::Use(u) = &parsed.file.items[0] else {
        panic!("expected use item")
    };
    assert_eq!(u.path, vec!["myapp", "utils", "helper"]);
    assert!(u.names.is_none());
}

#[test]
fn parses_use_destructuring() {
    let src = "use myapp.utils.{a, b, self};";
    let lexed = lex(src);
    let parsed = parse(&lexed.tokens, &lexed.errors);
    assert!(parsed.errors.is_empty(), "errors: {:?}", parsed.errors);
    assert_eq!(parsed.file.items.len(), 1);

    let Item::Use(u) = &parsed.file.items[0] else {
        panic!("expected use item")
    };
    assert_eq!(u.path, vec!["myapp", "utils"]);
    assert_eq!(u.names, Some(vec!["a".to_owned(), "b".to_owned(), "self".to_owned()]));
}

#[test]
fn parses_use_alongside_fn() {
    let src = "use myapp;\nfn id() { x }";
    let lexed = lex(src);
    let parsed = parse(&lexed.tokens, &lexed.errors);
    assert!(parsed.errors.is_empty(), "errors: {:?}", parsed.errors);
    assert_eq!(parsed.file.items.len(), 2);
    assert!(matches!(parsed.file.items[0], Item::Use(_)));
    assert!(matches!(parsed.file.items[1], Item::Fn(_)));
}

fn parse_fn_body(src: &str) -> Expr {
    let lexed = lex(src);
    let parsed = parse(&lexed.tokens, &lexed.errors);
    assert!(parsed.errors.is_empty(), "errors: {:?}", parsed.errors);
    let Item::Fn(f) = &parsed.file.items[0] else {
        panic!("expected fn item")
    };
    // Unwrap single-expression blocks for test convenience
    match &f.body {
        Expr::Block { stmts, result, .. } if stmts.is_empty() => result.as_ref().clone(),
        other => other.clone(),
    }
}

#[test]
fn operator_precedence_mul_before_add() {
    // 1 + 2 * 3 → Binary(1, Add, Binary(2, Mul, 3))
    let body = parse_fn_body("fn f() { 1 + 2 * 3 }");
    let Expr::Binary { op, left, right, .. } = &body else {
        panic!("expected binary, got {body:?}")
    };
    assert_eq!(*op, BinaryOp::Add);
    assert!(matches!(left.as_ref(), Expr::Number { value, .. } if value == "1"));
    let Expr::Binary { op: inner_op, .. } = right.as_ref() else {
        panic!("expected inner binary")
    };
    assert_eq!(*inner_op, BinaryOp::Mul);
}

#[test]
fn operator_left_associativity() {
    // 1 - 2 - 3 → Binary(Binary(1, Sub, 2), Sub, 3)
    let body = parse_fn_body("fn f() { 1 - 2 - 3 }");
    let Expr::Binary { op, left, right, .. } = &body else {
        panic!("expected binary, got {body:?}")
    };
    assert_eq!(*op, BinaryOp::Sub);
    assert!(matches!(right.as_ref(), Expr::Number { value, .. } if value == "3"));
    let Expr::Binary { op: inner_op, .. } = left.as_ref() else {
        panic!("expected inner binary")
    };
    assert_eq!(*inner_op, BinaryOp::Sub);
}

#[test]
fn unary_prefix_neg() {
    // -x → Unary(Neg, x)
    let body = parse_fn_body("fn f() { -x }");
    let Expr::Unary { op, expr, .. } = &body else {
        panic!("expected unary, got {body:?}")
    };
    assert_eq!(*op, UnaryOp::Neg);
    assert!(matches!(expr.as_ref(), Expr::Ident { name, .. } if name == "x"));
}

#[test]
fn unary_binds_tighter_than_binary() {
    // -a + b → Binary(Unary(Neg, a), Add, b)
    let body = parse_fn_body("fn f() { -a + b }");
    let Expr::Binary { op, left, .. } = &body else {
        panic!("expected binary, got {body:?}")
    };
    assert_eq!(*op, BinaryOp::Add);
    assert!(matches!(left.as_ref(), Expr::Unary { op: UnaryOp::Neg, .. }));
}

#[test]
fn comparison_operators_parse() {
    let body = parse_fn_body("fn f() { a == b }");
    assert!(matches!(&body, Expr::Binary { op: BinaryOp::EqEq, .. }));

    let body = parse_fn_body("fn f() { a != b }");
    assert!(matches!(&body, Expr::Binary { op: BinaryOp::NotEq, .. }));

    let body = parse_fn_body("fn f() { a < b }");
    assert!(matches!(&body, Expr::Binary { op: BinaryOp::Lt, .. }));

    let body = parse_fn_body("fn f() { a >= b }");
    assert!(matches!(&body, Expr::Binary { op: BinaryOp::GtEq, .. }));
}

#[test]
fn logical_operators_parse() {
    // a && b || c → Binary(Binary(a, AndAnd, b), OrOr, c)
    let body = parse_fn_body("fn f() { a && b || c }");
    let Expr::Binary { op, left, .. } = &body else {
        panic!("expected binary, got {body:?}")
    };
    assert_eq!(*op, BinaryOp::OrOr);
    assert!(matches!(left.as_ref(), Expr::Binary { op: BinaryOp::AndAnd, .. }));
}

#[test]
fn number_literal_parse() {
    let body = parse_fn_body("fn f() { 42 }");
    assert!(matches!(&body, Expr::Number { value, .. } if value == "42"));

    let body = parse_fn_body("fn f() { 3.14 }");
    assert!(matches!(&body, Expr::Number { value, .. } if value == "3.14"));
}

#[test]
fn postfix_binds_tighter_than_operators() {
    // a.b + c → Binary(Member(a, b), Add, c)
    let body = parse_fn_body("fn f() { a.b + c }");
    let Expr::Binary { op, left, .. } = &body else {
        panic!("expected binary, got {body:?}")
    };
    assert_eq!(*op, BinaryOp::Add);
    assert!(matches!(left.as_ref(), Expr::Member { .. }));
}

#[test]
fn assignment_desugars_in_parser() {
    // x = 1; y → Assign(x, 1, y)
    let body = parse_fn_body("fn f() { x = 1; y }");
    let Expr::Assign { name, value, body: body_expr, .. } = &body else {
        panic!("expected assign, got {body:?}")
    };
    assert_eq!(name, "x");
    assert!(matches!(value.as_ref(), Expr::Number { value, .. } if value == "1"));
    assert!(matches!(body_expr.as_ref(), Expr::Ident { name, .. } if name == "y"));
}

#[test]
fn parses_inherent_impl() {
    let src = "impl String { fn len(self): Number { str_len(self) } }";
    let lexed = lex(src);
    let parsed = parse(&lexed.tokens, &lexed.errors);
    assert!(parsed.errors.is_empty(), "errors: {:?}", parsed.errors);
    assert_eq!(parsed.file.items.len(), 1);
    let Item::Impl(i) = &parsed.file.items[0] else {
        panic!("expected impl, got {:?}", parsed.file.items[0])
    };
    assert!(i.name.is_none());
    assert!(i.capability.is_none());
    assert_eq!(i.target_type.repr.trim(), "String");
    assert_eq!(i.methods.len(), 1);
    assert_eq!(i.methods[0].name, "len");
    assert_eq!(i.methods[0].params.len(), 1);
    assert_eq!(i.methods[0].params[0].name, "self");
    // self has synthetic "Self" type
    assert_eq!(i.methods[0].params[0].ty.repr, "Self");
    assert_eq!(
        i.methods[0].return_type.as_ref().map(|t| t.repr.trim()),
        Some("Number")
    );
}

#[test]
fn parses_capability_impl() {
    let src = "impl String: Clone { fn clone(self): String { self } }";
    let lexed = lex(src);
    let parsed = parse(&lexed.tokens, &lexed.errors);
    assert!(parsed.errors.is_empty(), "errors: {:?}", parsed.errors);
    let Item::Impl(i) = &parsed.file.items[0] else {
        panic!("expected impl")
    };
    assert!(i.name.is_none());
    assert_eq!(i.target_type.repr.trim(), "String");
    assert_eq!(i.capability.as_ref().unwrap().repr.trim(), "Clone");
}

#[test]
fn parses_named_impl() {
    let src = "impl MyClone = String: Clone { fn clone(self): String { self } }";
    let lexed = lex(src);
    let parsed = parse(&lexed.tokens, &lexed.errors);
    assert!(parsed.errors.is_empty(), "errors: {:?}", parsed.errors);
    let Item::Impl(i) = &parsed.file.items[0] else {
        panic!("expected impl")
    };
    assert_eq!(i.name.as_deref(), Some("MyClone"));
    assert_eq!(i.target_type.repr.trim(), "String");
    assert_eq!(i.capability.as_ref().unwrap().repr.trim(), "Clone");
}

#[test]
fn parses_generic_impl() {
    let src = "impl[T: Clone] List[T]: Clone { fn clone(self): List[T] { self } }";
    let lexed = lex(src);
    let parsed = parse(&lexed.tokens, &lexed.errors);
    assert!(parsed.errors.is_empty(), "errors: {:?}", parsed.errors);
    let Item::Impl(i) = &parsed.file.items[0] else {
        panic!("expected impl")
    };
    assert_eq!(i.generics.len(), 1);
    assert_eq!(i.generics[0].name, "T");
    assert_eq!(
        i.generics[0].constraint.as_ref().map(|c| c.repr.trim()),
        Some("Clone")
    );
    assert!(i.target_type.repr.contains("List"));
    assert_eq!(i.capability.as_ref().unwrap().repr.trim(), "Clone");
}

#[test]
fn parses_impl_with_typed_self() {
    // self can also have an explicit type
    let src = "impl String { fn len(self: String): Number { str_len(self) } }";
    let lexed = lex(src);
    let parsed = parse(&lexed.tokens, &lexed.errors);
    assert!(parsed.errors.is_empty(), "errors: {:?}", parsed.errors);
    let Item::Impl(i) = &parsed.file.items[0] else {
        panic!("expected impl")
    };
    assert_eq!(i.methods[0].params[0].name, "self");
    assert_eq!(i.methods[0].params[0].ty.repr.trim(), "String");
}

#[test]
fn parses_if_else_expression() {
    let src = "fn f(x: Bool) { if x { a } else { b } }";
    let lexed = lex(src);
    let parsed = parse(&lexed.tokens, &lexed.errors);
    assert!(parsed.errors.is_empty(), "errors: {:?}", parsed.errors);

    let Item::Fn(f) = &parsed.file.items[0] else {
        panic!("expected fn")
    };
    let Expr::Block { result, .. } = &f.body else {
        panic!("expected block, got {:?}", f.body)
    };
    assert!(matches!(result.as_ref(), Expr::IfElse { .. }), "expected if-else, got {:?}", result);
}

#[test]
fn parses_if_else_if_chain() {
    let src = "fn f(x: Bool, y: Bool) { if x { a } else if y { b } else { c } }";
    let lexed = lex(src);
    let parsed = parse(&lexed.tokens, &lexed.errors);
    assert!(parsed.errors.is_empty(), "errors: {:?}", parsed.errors);

    let Item::Fn(f) = &parsed.file.items[0] else {
        panic!("expected fn")
    };
    let Expr::Block { result, .. } = &f.body else {
        panic!("expected block")
    };
    let Expr::IfElse { else_body, .. } = result.as_ref() else {
        panic!("expected if-else, got {:?}", result)
    };
    assert!(matches!(else_body.as_deref(), Some(Expr::IfElse { .. })));
}
