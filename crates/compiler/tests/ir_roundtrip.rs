//! Round-trip tests: Lumo source → HIR/LIR lower → print → parse → print → compare
//!
//! Verifies that the printer and parser for each IR are mutually consistent
//! when fed real compiler output (not hand-crafted IR strings).

use lumo_compiler::{hir, lexer::lex, lir, parser::parse};

fn lower_hir(src: &str) -> hir::File {
    let lexed = lex(src);
    let parsed = parse(&lexed.tokens, &lexed.errors);
    hir::lower(&parsed.file)
}

fn lower_lir(src: &str) -> lir::File {
    let hir = lower_hir(src);
    lir::lower(&hir)
}

/// HIR round-trip: source → lower → print → parse → print → compare
fn assert_hir_roundtrip(src: &str) {
    let hir_file = lower_hir(src);
    let printed1 = hir::print::print_file(&hir_file);
    let reparsed = hir::parse::parse(&printed1)
        .unwrap_or_else(|errs| panic!("HIR parse failed on:\n{printed1}\nerrors: {errs:?}"));
    let printed2 = hir::print::print_file(&reparsed);
    assert_eq!(
        printed1, printed2,
        "HIR round-trip mismatch for source:\n{src}\n\nFirst print:\n{printed1}\n\nSecond print:\n{printed2}"
    );
}

/// LIR round-trip: source → lower → print → parse → print → compare
fn assert_lir_roundtrip(src: &str) {
    let lir_file = lower_lir(src);
    let printed1 = lir::print::print_file(&lir_file);
    let reparsed = lir::parse::parse(&printed1)
        .unwrap_or_else(|errs| panic!("LIR parse failed on:\n{printed1}\nerrors: {errs:?}"));
    let printed2 = lir::print::print_file(&reparsed);
    assert_eq!(
        printed1, printed2,
        "LIR round-trip mismatch for source:\n{src}\n\nFirst print:\n{printed1}\n\nSecond print:\n{printed2}"
    );
}

// =========================================================================
// HIR round-trip tests
// =========================================================================

#[test]
fn hir_roundtrip_simple_fn() {
    assert_hir_roundtrip("fn id(x: Number): Number { x }");
}

#[test]
fn hir_roundtrip_data_and_match() {
    assert_hir_roundtrip(
        "data Bool { .true, .false }
         fn not(b: Bool): Bool { match b { .true => Bool.false .false => Bool.true } }",
    );
}

#[test]
fn hir_roundtrip_let_in() {
    assert_hir_roundtrip("fn f() { let x = 42; x }");
}

#[test]
fn hir_roundtrip_extern_items() {
    assert_hir_roundtrip(
        r#"extern type String
           extern type Number as "number"
           extern fn println(s: String) / {IO}"#,
    );
}

#[test]
fn hir_roundtrip_capability() {
    assert_hir_roundtrip(
        "cap IO { fn log(msg: String) }
         fn f() { handle IO with bundle { fn log(msg: String) { msg } } in 42 }",
    );
}

#[test]
fn hir_roundtrip_use_decl() {
    assert_hir_roundtrip("use std.io.{print, read};");
}

#[test]
fn hir_roundtrip_impl() {
    assert_hir_roundtrip(
        "impl Number: Add { fn add(self: Number, other: Number): Number { self } }",
    );
}

#[test]
fn hir_roundtrip_generic_data() {
    assert_hir_roundtrip("data List[A] { .nil, .cons(A, List[A]) }");
}

#[test]
fn hir_roundtrip_thunk_force() {
    assert_hir_roundtrip("fn f() { force thunk 1 }");
}

// =========================================================================
// LIR round-trip tests
// =========================================================================

#[test]
fn lir_roundtrip_simple_fn() {
    assert_lir_roundtrip("fn id(x: Number): Number { x }");
}

#[test]
fn lir_roundtrip_data_and_match() {
    assert_lir_roundtrip(
        "data Bool { .true, .false }
         fn not(b: Bool): Bool { match b { .true => Bool.false .false => Bool.true } }",
    );
}

#[test]
fn lir_roundtrip_let_in() {
    assert_lir_roundtrip("fn f() { let x = 42; x }");
}

#[test]
fn lir_roundtrip_extern_items() {
    assert_lir_roundtrip(
        r#"extern type String
           extern type Number as "number"
           extern fn println(s: String) / {IO}"#,
    );
}

#[test]
fn lir_roundtrip_capability() {
    assert_lir_roundtrip(
        "cap IO { fn log(msg: String) }
         fn f() { handle IO with bundle { fn log(msg: String) { msg } } in 42 }",
    );
}

#[test]
fn lir_roundtrip_multi_arg_fn() {
    assert_lir_roundtrip("fn add(a: Number, b: Number): Number { a }");
}

#[test]
fn lir_roundtrip_generic_data() {
    assert_lir_roundtrip("data List[A] { .nil, .cons(A, List[A]) }");
}

#[test]
fn lir_roundtrip_call_chain() {
    assert_lir_roundtrip(
        "extern fn add(a: Number, b: Number): Number;
         fn f(x: Number, y: Number) { add(x, y) }",
    );
}
