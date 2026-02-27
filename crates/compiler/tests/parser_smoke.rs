use lumo_compiler::{
    lexer::lex,
    parser::{parse, Expr, Item},
};

#[test]
fn parses_data_and_fn() {
    let src = "data Option[A] { .some(A), .none } fn id[A](a: A): produce A / {} := produce a";
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
        Item::Fn(f) => match &f.body {
            Expr::Produce { expr, .. } => match expr.as_ref() {
                Expr::Ident { name, .. } => assert_eq!(name, "a"),
                other => panic!("unexpected produce payload: {other:?}"),
            },
            other => panic!("unexpected fn body: {other:?}"),
        },
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
        Some("produce A")
    );
    assert_eq!(f.effect.as_ref().map(|e| e.repr.as_str()), Some("{ }"));
}

#[test]
fn parses_let_in_expression() {
    let src = "fn main(): produce A / {} := let x = y in produce x";
    let lexed = lex(src);
    let parsed = parse(&lexed.tokens, &lexed.errors);

    assert_eq!(parsed.file.items.len(), 1);
    assert!(parsed.errors.is_empty(), "errors: {:?}", parsed.errors);

    let Item::Fn(f) = &parsed.file.items[0] else {
        panic!("expected fn item")
    };

    match &f.body {
        Expr::LetIn {
            name, value, body, ..
        } => {
            assert_eq!(name, "x");
            match value.as_ref() {
                Expr::Ident { name, .. } => assert_eq!(name, "y"),
                other => panic!("unexpected let value: {other:?}"),
            }
            match body.as_ref() {
                Expr::Produce { expr, .. } => match expr.as_ref() {
                    Expr::Ident { name, .. } => assert_eq!(name, "x"),
                    other => panic!("unexpected produce payload: {other:?}"),
                },
                other => panic!("unexpected let body: {other:?}"),
            }
        }
        other => panic!("unexpected fn body: {other:?}"),
    }
}

#[test]
fn recovers_and_keeps_parsing_next_item() {
    let src = "fn broken := produce data Good { .a }";
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
    let src = "fn f() := match x { left => thunk produce a, right => force job }";
    let lexed = lex(src);
    let parsed = parse(&lexed.tokens, &lexed.errors);
    assert!(parsed.errors.is_empty(), "errors: {:?}", parsed.errors);

    let Item::Fn(f) = &parsed.file.items[0] else {
        panic!("expected fn item")
    };
    let Expr::Match {
        scrutinee, arms, ..
    } = &f.body
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
    let src = "fn mk() := Option.some(a)";
    let lexed = lex(src);
    let parsed = parse(&lexed.tokens, &lexed.errors);
    assert!(parsed.errors.is_empty(), "errors: {:?}", parsed.errors);

    let Item::Fn(f) = &parsed.file.items[0] else {
        panic!("expected fn item")
    };
    let Expr::Apply {
        owner,
        member,
        args,
        ..
    } = &f.body
    else {
        panic!("expected apply body")
    };
    assert_eq!(owner, "Option");
    assert_eq!(member, "some");
    assert_eq!(args.len(), 1);
}
