//! M2 step 4 smoke test: the generated MIR judgment table (from the
//! `MIR.type.langue` seed) solves goals end to end on parsed MIR —
//! context write through `let`, read through `VarV`, U/F construction,
//! and the inout type parameter coming back resolved.

use langue_rt::{app, atom, Contexts};
use lumo_syntax::mir::ast::{self as m, AstNode as _};
use lumo_syntax::mir::judgments;
use lumo_syntax::mir::parser;

fn number() -> langue_rt::Term {
    app("NamedTypeV", vec![atom("Number"), atom("#none")])
}

#[test]
fn infers_thunked_let_over_num() {
    let parsed = parser::parse("def p = thunk { let x = ret 42 in ret x }");
    assert!(parsed.errors.is_empty(), "{:?}", parsed.errors);
    let file = m::File::cast(&parsed.root).unwrap();
    let value = file.defs().next().unwrap().value().unwrap();
    let derivation =
        judgments::solve("infer_V", value.syntax(), Contexts::new()).unwrap();
    let f_num = app("FTypeC", vec![number(), atom("#none")]);
    assert_eq!(derivation.args[1], app("UTypeV", vec![f_num]));
}

#[test]
fn seeded_context_types_free_variables() {
    let parsed = parser::parse("def p = thunk { ret y }");
    assert!(parsed.errors.is_empty());
    let file = m::File::cast(&parsed.root).unwrap();
    let value = file.defs().next().unwrap().value().unwrap();
    let mut ctxs = Contexts::new();
    ctxs.insert("Γ".into(), vec![(atom("y"), number())]);
    let derivation = judgments::solve("infer_V", value.syntax(), ctxs).unwrap();
    let f_num = app("FTypeC", vec![number(), atom("#none")]);
    assert_eq!(derivation.args[1], app("UTypeV", vec![f_num]));
}

#[test]
fn unbound_variable_bails_softly() {
    let parsed = parser::parse("def p = thunk { ret y }");
    let file = m::File::cast(&parsed.root).unwrap();
    let value = file.defs().next().unwrap().value().unwrap();
    let bail = judgments::solve("infer_V", value.syntax(), Contexts::new());
    assert!(!bail.unwrap_err().hard);
}
