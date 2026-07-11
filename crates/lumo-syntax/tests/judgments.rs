//! M2 step 6: the real MIR judgments, end to end on parsed MIR.
//! Contexts are seeded the way the driver will (Δ = data variants,
//! Σ = cap operation signatures, Γ = term bindings), per the
//! seed-shape contracts documented in `lumo/MIR.type.langue`.

use langue_rt::{app, atom, set, Contexts, Term};
use lumo_syntax::mir::ast::{self as m, AstNode as _};
use lumo_syntax::mir::judgments;
use lumo_syntax::mir::parser;

fn cons(items: Vec<Term>) -> Term {
    items.into_iter().rev().fold(atom("#nil"), |t, h| app("#cons", vec![h, t]))
}

fn named(name: &str) -> Term {
    app("NamedTypeV", vec![atom(name), atom("#none")])
}

fn named_args(name: &str, args: Vec<Term>) -> Term {
    app("NamedTypeV", vec![atom(name), app("TypeArgs", vec![cons(args)])])
}

fn u(inner: Term) -> Term {
    app("UTypeV", vec![inner])
}

fn f_row(inner: Term, row: Term) -> Term {
    app("FTypeC", vec![inner, row])
}

fn pure_f(inner: Term) -> Term {
    f_row(inner, set(vec![], None))
}

fn fnt(param: Term, ret: Term) -> Term {
    app("FnTypeC", vec![cons(vec![param]), ret])
}

fn variant(owner: Term, binders: Vec<Term>, params: Vec<Term>) -> Term {
    app("Variant", vec![owner, cons(binders), cons(params)])
}

/// Solve `infer_V` on the first def's value.
fn infer_def(source: &str, ctxs: Contexts) -> Result<Term, langue_rt::Bail> {
    let parsed = parser::parse(source);
    assert!(parsed.errors.is_empty(), "{:?}", parsed.errors);
    let file = m::File::cast(&parsed.root).unwrap();
    let value = file.defs().next().unwrap().value().unwrap();
    judgments::solve("infer_V", value.syntax(), ctxs).map(|d| d.args[1].clone())
}

#[test]
fn annotated_identity_checks_and_returns_its_type() {
    let ty = infer_def(
        "def inc = (thunk { fn(x) => ret x } : U((Number) -> F(Number)))",
        Contexts::new(),
    )
    .unwrap();
    assert_eq!(ty, u(fnt(named("Number"), pure_f(named("Number")))));
}

#[test]
fn body_type_mismatch_bails_softly() {
    let bail = infer_def(
        "def bad = (thunk { fn(x) => ret 42 } : U((Number) -> F(String)))",
        Contexts::new(),
    );
    assert!(!bail.unwrap_err().hard);
}

fn console_sigs() -> Contexts {
    let mut ctxs = Contexts::new();
    ctxs.insert(
        "Σ".into(),
        vec![
            (
                app("Op", vec![atom("Console"), atom("log")]),
                fnt(named("String"), pure_f(named("String"))),
            ),
            (app("Ops", vec![atom("Console")]), set(vec![atom("log")], None)),
        ],
    );
    ctxs
}

#[test]
fn declared_row_permits_the_perform() {
    let ty = infer_def(
        "def hi = (thunk { fn(x) => let c = perform Console in sel c.log(x) } \
         : U((String) -> F(String) / {Console}))",
        console_sigs(),
    )
    .unwrap();
    let row = set(vec![atom("Console")], None);
    assert_eq!(ty, u(fnt(named("String"), f_row(named("String"), row))));
}

#[test]
fn undeclared_perform_bails_softly() {
    let bail = infer_def(
        "def hi = (thunk { fn(x) => let c = perform Console in sel c.log(x) } \
         : U((String) -> F(String)))",
        console_sigs(),
    );
    assert!(!bail.unwrap_err().hard);
}

#[test]
fn handle_discharges_the_row_and_bundles_check_against_sigs() {
    let mut ctxs = Contexts::new();
    ctxs.insert(
        "Σ".into(),
        vec![
            (app("Op", vec![atom("E"), atom("op")]), pure_f(named("A"))),
            (app("Ops", vec![atom("E")]), set(vec![atom("op")], None)),
        ],
    );
    // The def's own row is empty: the handle discharges E.
    let ty = infer_def(
        "def f = (thunk { fn(a) => \
           handle E with bundle { fn op() => ret a; } in \
           let c = perform E in sel c.op } : U((A) -> F(A)))",
        ctxs.clone(),
    )
    .unwrap();
    assert_eq!(ty, u(fnt(named("A"), pure_f(named("A")))));
    // A bundle whose clause set differs from the cap's ops bails.
    let bail = infer_def(
        "def f = (thunk { fn(a) => \
           handle E with bundle { fn wrong() => ret a; } in \
           let c = perform E in sel c.op } : U((A) -> F(A)))",
        ctxs,
    );
    assert!(!bail.unwrap_err().hard);
}

fn nat_variants() -> Contexts {
    let mut ctxs = Contexts::new();
    ctxs.insert(
        "Δ".into(),
        vec![
            (atom("zero"), variant(named("Nat"), vec![], vec![])),
            (atom("succ"), variant(named("Nat"), vec![], vec![named("Nat")])),
        ],
    );
    ctxs
}

#[test]
fn match_types_variants_and_binders() {
    let ty = infer_def(
        "def pred = (thunk { fn(n) => case unroll n { \
           .zero => ret (roll .zero), \
           .succ(m) => ret m, \
         } } : U((Nat) -> F(Nat)))",
        nat_variants(),
    )
    .unwrap();
    assert_eq!(ty, u(fnt(named("Nat"), pure_f(named("Nat")))));
}

#[test]
fn wrong_binder_arity_bails() {
    let bail = infer_def(
        "def bad = (thunk { fn(n) => case unroll n { \
           .zero => ret n, \
           .succ(m, extra) => ret m, \
         } } : U((Nat) -> F(Nat)))",
        nat_variants(),
    );
    assert!(!bail.unwrap_err().hard);
}

#[test]
fn generic_data_instantiates_from_the_scrutinee() {
    let mut ctxs = Contexts::new();
    let option = |arg: Term| named_args("Option", vec![arg]);
    ctxs.insert(
        "Δ".into(),
        vec![
            (
                atom("some"),
                variant(option(named("A")), vec![atom("A")], vec![named("A")]),
            ),
            (atom("none"), variant(option(named("A")), vec![atom("A")], vec![])),
        ],
    );
    let ty = infer_def(
        "def get = (thunk { fn(o) => fn(d) => case unroll o { \
           .some(x) => ret x, \
           .none => ret d, \
         } } : U((Option[Number]) -> (Number) -> F(Number)))",
        ctxs,
    )
    .unwrap();
    assert_eq!(
        ty,
        u(fnt(
            option(named("Number")),
            fnt(named("Number"), pure_f(named("Number")))
        ))
    );
}

#[test]
fn forall_instantiates_at_application() {
    let mut ctxs = Contexts::new();
    // id : U(forall a. (a) -> F(a)) — as the step-2 elab annotates it.
    let a = named("a");
    ctxs.insert(
        "Γ".into(),
        vec![(
            atom("id"),
            u(app(
                "ForallTypeC",
                vec![cons(vec![atom("a")]), fnt(a.clone(), pure_f(a))],
            )),
        )],
    );
    let ty = infer_def(
        "def use_id = (thunk { force id(42) } : U(F(Number)))",
        ctxs.clone(),
    )
    .unwrap();
    assert_eq!(ty, u(pure_f(named("Number"))));
    // The same instantiation rejects a wrong result annotation.
    let bail = infer_def(
        "def use_id = (thunk { force id(42) } : U(F(String)))",
        ctxs,
    );
    assert!(!bail.unwrap_err().hard);
}

#[test]
fn row_polymorphic_annotation_stays_rigid() {
    // pure_fn[cap c](a: A): A / {..c} — the body may not perform
    // anything beyond the listed entries (rigid tail).
    let ty = infer_def(
        "def p = (thunk { fn(a) => ret a } : U(forall c. (A) -> F(A) / {..c}))",
        Contexts::new(),
    )
    .unwrap();
    // An empty open set collapses to its (rigid) tail on resolution.
    let row = app("RowVar", vec![atom("c")]);
    assert_eq!(ty, u(app(
        "ForallTypeC",
        vec![cons(vec![atom("c")]), fnt(named("A"), f_row(named("A"), row))],
    )));
    let bail = infer_def(
        "def p = (thunk { fn(a) => let x = perform IO in ret a } \
         : U(forall c. (A) -> F(A) / {..c}))",
        Contexts::new(),
    );
    assert!(!bail.unwrap_err().hard);
}
