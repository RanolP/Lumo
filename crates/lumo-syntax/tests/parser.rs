//! Generated-parser sanity: byte-exact lossless round-trips.

use lumo_syntax::lumo::parser::parse;
use lumo_syntax::lumo::syntax_kind::SyntaxKind;

#[test]
fn byte_exact_round_trip() {
    let src = "fn add(a, b) = a";
    let out = parse(src);
    assert!(out.errors.is_empty(), "{:?}", out.errors);
    assert_eq!(out.root.text(), src);
    assert_eq!(out.root.kind, SyntaxKind::FILE);
}

#[test]
fn round_trip_preserves_trivia_and_calls() {
    let src = "// leading comment\nfn f(x) = g(x + 1, 2)  // trailing\nfn g(a, b) = let y = a in y\n";
    let out = parse(src);
    assert!(out.errors.is_empty(), "{:?}", out.errors);
    assert_eq!(out.root.text(), src);
}

#[test]
fn broken_input_still_round_trips() {
    let src = "fn broken( = @@ fn ok(x) = x";
    let out = parse(src);
    assert!(!out.errors.is_empty());
    assert_eq!(out.root.text(), src, "losslessness survives errors");
}
