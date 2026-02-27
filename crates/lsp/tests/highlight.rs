use lumo_lsp::highlight::{highlight, HighlightKind};

#[test]
fn highlights_keyword_identifier_symbol() {
    let tokens = highlight("fn id := produce x");
    assert!(tokens.iter().any(|t| t.kind == HighlightKind::Keyword));
    assert!(tokens.iter().any(|t| t.kind == HighlightKind::Identifier));
    assert!(tokens.iter().any(|t| t.kind == HighlightKind::Symbol));
}

#[test]
fn keeps_highlighting_even_with_syntax_error() {
    let tokens = highlight("fn id := produce + x");
    assert!(tokens.iter().any(|t| t.kind == HighlightKind::Keyword));
    assert!(tokens.iter().any(|t| t.kind == HighlightKind::Identifier));
}
