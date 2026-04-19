use lumo_compiler::{
    hir,
    lexer::lex,
    lir,
    parser::parse,
};

fn lower(src: &str) -> lir::File {
    let lexed = lex(src);
    let parsed = parse(&lexed.tokens, &lexed.errors);
    let hir = hir::lower(&parsed.file);
    lir::lower(&hir)
}

#[test]
fn inline_always_attribute_sets_inline_flag() {
    let file = lower("#[inline(always)] fn id(x: Number): Number { x }");
    let lir::Item::Fn(f) = &file.items[0] else { panic!("expected fn, got {:?}", &file.items[0]) };
    assert!(f.inline, "expected inline=true on #[inline(always)] fn");
}

#[test]
fn no_inline_attribute_leaves_flag_false() {
    let file = lower("fn id(x: Number): Number { x }");
    let lir::Item::Fn(f) = &file.items[0] else { panic!("expected fn") };
    assert!(!f.inline, "expected inline=false on plain fn");
}
