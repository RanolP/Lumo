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

fn body_sexpr(src: &str) -> String {
    let report = lumo_syntax::registry::language("Lumo").unwrap().parse_report;
    let r = report(src);
    assert!(r.errors.is_empty(), "{:?}", r.errors);
    assert_eq!(r.round_trip_sexpr, r.sexpr, "canonical print must re-parse to the same tree");
    // Strip the constant (FILE (FN_DECL …)) wrapper around the body.
    let inner = r
        .sexpr
        .strip_prefix("(FILE (FN_DECL ")
        .and_then(|s| s.strip_suffix("))"))
        .unwrap_or(&r.sexpr)
        .to_owned();
    inner
}

#[test]
fn precedence_multiplication_binds_tighter() {
    assert_eq!(
        body_sexpr("fn t() = 1 + 2 * 3"),
        "(EXPR_INFIX (NUMBER_EXPR) (EXPR_INFIX (NUMBER_EXPR) (NUMBER_EXPR)))"
    );
    assert_eq!(
        body_sexpr("fn t() = 1 * 2 + 3"),
        "(EXPR_INFIX (EXPR_INFIX (NUMBER_EXPR) (NUMBER_EXPR)) (NUMBER_EXPR))"
    );
}

#[test]
fn associativity_matches_binding_powers() {
    // `@70 '+' @69` (lbp > rbp) is left-associative.
    assert_eq!(
        body_sexpr("fn t() = 1 - 2 - 3"),
        "(EXPR_INFIX (EXPR_INFIX (NUMBER_EXPR) (NUMBER_EXPR)) (NUMBER_EXPR))"
    );
    // `@89 '**' @90` (lbp < rbp) is right-associative.
    assert_eq!(
        body_sexpr("fn t() = 2 ** 3 ** 4"),
        "(EXPR_INFIX (NUMBER_EXPR) (EXPR_INFIX (NUMBER_EXPR) (NUMBER_EXPR)))"
    );
}

#[test]
fn calls_prefix_and_parens() {
    assert_eq!(
        body_sexpr("fn t() = f(1)(2)"),
        "(EXPR_POSTFIX (EXPR_POSTFIX (IDENT_EXPR) (CALL_ARGS (NUMBER_EXPR))) (CALL_ARGS (NUMBER_EXPR)))"
    );
    assert_eq!(
        body_sexpr("fn t() = -f() * (1 + 2)"),
        "(EXPR_INFIX (EXPR_PREFIX (EXPR_POSTFIX (IDENT_EXPR) (CALL_ARGS))) (PAREN_EXPR (EXPR_INFIX (NUMBER_EXPR) (NUMBER_EXPR))))"
    );
}

#[test]
fn ast_accessors() {
    use lumo_syntax::lumo::ast::{AstNode, Expr, File};
    use lumo_syntax::lumo::parser::parse;

    let out = parse("fn add(a, b) = a + b");
    let file = File::cast(&out.root).unwrap();
    let fn_decl = file.items().next().unwrap();
    assert_eq!(fn_decl.name().unwrap().text, "add");
    let params: Vec<String> =
        fn_decl.params().map(|p| p.name().unwrap().text.clone()).collect();
    assert_eq!(params, ["a", "b"]);
    let Some(Expr::Infix(infix)) = fn_decl.body() else {
        panic!("body should be an infix expression")
    };
    assert_eq!(infix.op().unwrap().text, "+");
    assert!(matches!(infix.lhs(), Some(Expr::IdentExpr(_))));
    assert!(matches!(infix.rhs(), Some(Expr::IdentExpr(_))));
}

#[test]
fn canonical_print_separates_merging_tokens() {
    use lumo_syntax::lumo::parser::parse;
    use lumo_syntax::lumo::printer::canonical;

    let out = parse("fn  t ( x )  =  x + 1  // comment\n");
    assert!(out.errors.is_empty(), "{:?}", out.errors);
    // `fn` and `t` must keep a space (they would merge into one ident);
    // punctuation needs none; trivia is dropped.
    assert_eq!(canonical(&out.root), "fn t(x)=x+1");
}

#[test]
fn broken_input_still_round_trips() {
    let src = "fn broken( = @@ fn ok(x) = x";
    let out = parse(src);
    assert!(!out.errors.is_empty());
    assert_eq!(out.root.text(), src, "losslessness survives errors");
}

#[test]
fn recovery_resyncs_to_next_decl() {
    use lumo_syntax::lumo::printer::sexpr;

    // Garbage between decls: both decls survive, garbage becomes ERROR.
    let src = "fn a() = 1\n]]]]\nfn b() = 2";
    let out = parse(src);
    assert!(!out.errors.is_empty());
    assert_eq!(out.root.text(), src);
    let tree = sexpr(&out.root);
    assert_eq!(tree.matches("(FN_DECL").count(), 2, "{tree}");
    assert!(tree.contains("(ERROR"), "{tree}");

    // Broken argument list: the next decl still parses.
    let src = "fn a() = f(, 3)\nfn b() = 1";
    let out = parse(src);
    assert!(!out.errors.is_empty());
    assert_eq!(out.root.text(), src);
    let tree = sexpr(&out.root);
    assert_eq!(tree.matches("(FN_DECL").count(), 2, "{tree}");
}
