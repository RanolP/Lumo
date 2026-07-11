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
