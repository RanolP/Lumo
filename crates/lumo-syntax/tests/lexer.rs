//! End-to-end sanity for the generated lexer table (the engine itself is
//! unit-tested in langue-rt).

use lumo_syntax::lumo::lexer::lex;
use lumo_syntax::lumo::syntax_kind::SyntaxKind;

#[test]
fn generated_table_maps_kinds() {
    let kinds: Vec<SyntaxKind> = lex("fn fnord = 2 ** 3").iter().map(|t| t.kind).collect();
    assert_eq!(
        kinds,
        vec![
            SyntaxKind::KEYWORD_FN,
            SyntaxKind::WHITESPACE,
            SyntaxKind::IDENT, // longest match beats the `fn` literal prefix
            SyntaxKind::WHITESPACE,
            SyntaxKind::OP_EQ,
            SyntaxKind::WHITESPACE,
            SyntaxKind::LIT_NUMBER,
            SyntaxKind::WHITESPACE,
            SyntaxKind::OP_POW, // `**` beats `*` twice
            SyntaxKind::WHITESPACE,
            SyntaxKind::LIT_NUMBER,
        ]
    );
}

#[test]
fn lossless_and_scopes() {
    let text = "let x = \"s\" // c";
    let tokens = lex(text);
    let rebuilt: String = tokens.iter().map(|t| t.text.as_str()).collect();
    assert_eq!(rebuilt, text, "concatenated token texts reproduce the input");
    assert_eq!(tokens[0].kind.highlight_scope(), Some("keyword.let"));
    assert!(tokens.last().unwrap().kind.is_trivia(), "line comment is trivia");
}
