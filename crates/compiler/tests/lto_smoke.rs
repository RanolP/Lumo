use lumo_compiler::{
    hir,
    lexer::lex,
    lir,
    lto,
    parser::parse,
};

fn lower(src: &str) -> lir::File {
    let lexed = lex(src);
    let parsed = parse(&lexed.tokens, &lexed.errors);
    let hir = hir::lower(&parsed.file);
    lir::lower(&hir)
}

#[test]
fn lto_optimize_is_callable_and_idempotent_on_no_op_input() {
    let mut file = lower("fn id(x: Number): Number { x }");
    let before = file.clone();
    let errors = lto::optimize(&mut file);
    assert!(errors.is_empty(), "expected no errors, got: {errors:?}");
    // No caps, no Performs — nothing to do; file stays bit-equal.
    assert_eq!(file, before);
}
