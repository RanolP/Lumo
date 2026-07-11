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

/// Navigate to the annotated type of the first def: `(v : U(<fn>))`.
fn fn_type_of(parsed: &lumo_syntax::mir::lossless::ParseOutput) -> m::FnTypeC<'_> {
    let file = m::File::cast(&parsed.root).unwrap();
    let m::Value::ParenV(paren) = file.defs().next().unwrap().value().unwrap() else {
        panic!("expected an annotated def")
    };
    let m::TypeV::UTypeV(u) = paren.ty().unwrap() else { panic!() };
    let m::TypeC::FnTypeC(f) = u.inner().unwrap() else { panic!() };
    f
}

#[test]
fn hash_turns_cap_rows_into_sets() {
    let parsed =
        parser::parse("def g = (g : U(() -> F(String) / {State[Number], Console}))");
    assert!(parsed.errors.is_empty(), "{:?}", parsed.errors);
    let f = fn_type_of(&parsed);
    let m::TypeC::FTypeC(ft) = f.ret().unwrap() else { panic!() };
    let row = ft.row().unwrap();
    let derivation =
        judgments::solve("row_of", row.syntax(), Contexts::new()).unwrap();
    let entry = |sig: langue_rt::Term| app("CapEntry", vec![sig]);
    let console = entry(app("CapSig", vec![atom("Console"), atom("#none")]));
    let state = entry(app("CapSig", vec![
        atom("State"),
        app("TypeArgs", vec![app("#list", vec![number()])]),
    ]));
    // Canonical set order (by structural key), not source order.
    assert_eq!(derivation.args[1], langue_rt::set(vec![console, state], None));
}

#[test]
fn subst_monomorphizes_through_the_dsl() {
    let parsed = parser::parse("def i = (i : U((a) -> F(a)))");
    assert!(parsed.errors.is_empty(), "{:?}", parsed.errors);
    let f = fn_type_of(&parsed);
    let derivation = judgments::solve("mono_a", f.syntax(), Contexts::new()).unwrap();
    let a = app("NamedTypeV", vec![atom("a"), atom("#none")]);
    let expected = app("FnTypeC", vec![
        app("#list", vec![a]),
        app("FTypeC", vec![number(), atom("#none")]),
    ]);
    assert_eq!(derivation.args[1], expected);
}
