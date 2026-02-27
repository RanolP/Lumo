use lumo_compiler::{
    lexer::lex,
    parser::{parse, Expr, Item},
};

#[test]
fn parses_data_and_fn() {
    let src = "data Option[A] { Some(A), None } fn id[A](a: A): produce A / {} := produce a";
    let lexed = lex(src);
    let parsed = parse(&lexed.tokens, &lexed.errors);

    assert_eq!(parsed.file.items.len(), 2);
    assert!(parsed.errors.is_empty(), "errors: {:?}", parsed.errors);

    match &parsed.file.items[0] {
        Item::Data(data) => {
            assert_eq!(data.name, "Option");
            assert_eq!(data.variants.len(), 2);
            assert_eq!(data.variants[0].name, "Some");
            assert_eq!(data.variants[1].name, "None");
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
    let src = "fn broken := produce data Good { A }";
    let lexed = lex(src);
    let parsed = parse(&lexed.tokens, &lexed.errors);

    assert!(!parsed.errors.is_empty());
    assert_eq!(parsed.file.items.len(), 2, "items: {:?}", parsed.file.items);

    match &parsed.file.items[1] {
        Item::Data(data) => assert_eq!(data.name, "Good"),
        other => panic!("expected recovered data item, got {other:?}"),
    }
}
