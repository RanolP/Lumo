//! M1 step 8 golden test (D-37): the compiled `between MIR` egglog
//! program matches the committed fixture. Bless by copying the
//! generated `mir::between::PROGRAM` into the fixture.
//! M3 step 2 (D-42): the same program also has to *execute* — the
//! smoke tests below run it through real egglog.

use std::path::Path;

use langue_rt::{EggTerm, Optimizer};

#[test]
fn between_mir_egglog_golden() {
    let path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/egglog/MIR.egg");
    let expected = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
    assert_eq!(
        lumo_syntax::mir::between::PROGRAM.trim_start_matches('\n'),
        expected,
        "compiled egglog program drifted from the golden fixture — \
         review and re-bless tests/fixtures/egglog/MIR.egg"
    );
}

#[test]
fn between_mir_program_executes_u_beta() {
    let mut opt = Optimizer::new(lumo_syntax::mir::between::PROGRAM).unwrap();
    opt.define_root(r#"(ForceC (ThunkV (RetC (NumV "1"))))"#).unwrap();
    opt.run(10).unwrap();
    let term = opt.extract_root().unwrap();
    assert_eq!(
        term,
        EggTerm::app("RetC", vec![EggTerm::app("NumV", vec![EggTerm::str("1")])])
    );
}

#[test]
fn between_mir_paren_and_handle_perform_rules_execute() {
    // M3 step 4 rules: paren transparency + perform resolution under a
    // visible handle (nonlinear pattern — same cap twice).
    let mut opt = Optimizer::new(lumo_syntax::mir::between::PROGRAM).unwrap();
    opt.define_root(r#"(ParenC (HandleC "C" (VarV "h") (PerformC "C")))"#).unwrap();
    opt.run(10).unwrap();
    let term = opt.extract_root().unwrap();
    assert_eq!(
        term,
        EggTerm::app("RetC", vec![EggTerm::app("VarV", vec![EggTerm::str("h")])])
    );
}

#[test]
fn between_mir_handle_of_other_cap_stays() {
    // The nonlinear pattern must NOT fire across different caps.
    let mut opt = Optimizer::new(lumo_syntax::mir::between::PROGRAM).unwrap();
    opt.define_root(r#"(HandleC "C" (VarV "h") (PerformC "D"))"#).unwrap();
    opt.run(10).unwrap();
    let term = opt.extract_root().unwrap();
    assert_eq!(
        term,
        EggTerm::app(
            "HandleC",
            vec![
                EggTerm::str("C"),
                EggTerm::app("VarV", vec![EggTerm::str("h")]),
                EggTerm::app("PerformC", vec![EggTerm::str("D")]),
            ]
        )
    );
}

#[test]
fn between_mir_f_beta_keeps_subst_out_of_extraction() {
    // F-beta unions in `(subst body b a)`; its :cost 1000 keeps the
    // original `let` form the extraction winner until a host reduction
    // offers a cheaper subst-free term (D-42).
    let mut opt = Optimizer::new(lumo_syntax::mir::between::PROGRAM).unwrap();
    opt.define_root(r#"(LetC "x" (RetC (NumV "1")) (RetC (VarV "x")))"#).unwrap();
    opt.run(10).unwrap();
    let term = opt.extract_root().unwrap();
    assert!(!term.contains_app("subst"), "subst leaked into extraction: {term:?}");
    // Host-reduced form unioned back: now the cheaper `ret 1` wins.
    opt.union_root(&EggTerm::app(
        "RetC",
        vec![EggTerm::app("NumV", vec![EggTerm::str("1")])],
    ))
    .unwrap();
    let term = opt.extract_root().unwrap();
    assert_eq!(
        term,
        EggTerm::app("RetC", vec![EggTerm::app("NumV", vec![EggTerm::str("1")])])
    );
}
