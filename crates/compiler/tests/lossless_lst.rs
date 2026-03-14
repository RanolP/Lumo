use lumo_compiler::{
    lexer::{lex_lossless, LosslessTokenKind},
    lst::lossless::{node_text, parse, SyntaxElement, SyntaxKind},
};

#[test]
fn lossless_lexer_roundtrips_original_text() {
    let src = "data Option[A] { .some(A), .none }\nfn id() := produce x+";
    let out = lex_lossless(src);
    let rebuilt = out
        .tokens
        .iter()
        .map(|t| t.text.as_str())
        .collect::<String>();

    assert_eq!(rebuilt, src);
    assert!(out.tokens.iter().any(|t| matches!(
        t.kind,
        LosslessTokenKind::Whitespace | LosslessTokenKind::Newline
    )));
    assert!(out
        .tokens
        .iter()
        .any(|t| matches!(t.kind, LosslessTokenKind::Unknown)));
}

#[test]
fn lossless_lst_preserves_text_on_valid_source() {
    let src = "data X { .a, .b }\nfn id() := produce x";
    let parsed = parse(src);

    assert!(parsed.errors.is_empty(), "errors: {:?}", parsed.errors);
    assert_eq!(parsed.root.kind, SyntaxKind::File);
    assert_eq!(node_text(&parsed.root), src);
}

#[test]
fn lossless_lst_allows_member_projection_without_call() {
    let src = "data Bool { .true, .false }\nfn not(x: Bool): produce Bool := match x { .true => Bool.false, .false => Bool.true }";
    let parsed = parse(src);

    assert!(parsed.errors.is_empty(), "errors: {:?}", parsed.errors);
    assert_eq!(node_text(&parsed.root), src);
}

#[test]
fn lossless_lst_preserves_text_on_broken_source() {
    let src = "fn broken() := produce +\ndata Good { .a }";
    let parsed = parse(src);

    assert!(!parsed.errors.is_empty());
    assert_eq!(node_text(&parsed.root), src);

    assert!(contains_error_node(&parsed.root));
}

fn contains_error_node(node: &lumo_compiler::lst::lossless::SyntaxNode) -> bool {
    if node.kind == SyntaxKind::Error {
        return true;
    }

    node.children.iter().any(|c| match c {
        SyntaxElement::Node(child) => contains_error_node(child),
        SyntaxElement::Token(_) => false,
    })
}
