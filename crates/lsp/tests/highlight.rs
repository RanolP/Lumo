use lumo_lsp::highlight::{highlight, HighlightKind};

#[test]
fn highlights_keyword_identifier_symbol() {
    let tokens = highlight("fn id() { \"x\" }");
    assert!(tokens.iter().any(|t| t.kind == HighlightKind::Keyword));
    assert!(tokens.iter().any(|t| t.kind == HighlightKind::Identifier));
    assert!(tokens.iter().any(|t| t.kind == HighlightKind::String));
    assert!(tokens.iter().any(|t| t.kind == HighlightKind::Symbol));
}

#[test]
fn keeps_highlighting_even_with_syntax_error() {
    let tokens = highlight("fn id() { + x }");
    assert!(tokens.iter().any(|t| t.kind == HighlightKind::Keyword));
    assert!(tokens.iter().any(|t| t.kind == HighlightKind::Identifier));
}

#[test]
fn attribute_name_extern_is_not_highlighted_as_keyword() {
    let src = "#[extern(name = \"string\")] extern type String;";
    let tokens = highlight(src);
    let extern_tokens = tokens
        .iter()
        .filter(|t| &src[t.start..t.end] == "extern")
        .collect::<Vec<_>>();
    assert_eq!(extern_tokens.len(), 2, "{extern_tokens:?}");
    assert_eq!(extern_tokens[0].kind, HighlightKind::Identifier);
    assert_eq!(extern_tokens[1].kind, HighlightKind::Keyword);
}
