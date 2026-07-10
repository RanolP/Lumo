//! M1 step 7 gate: `elab Lumo -> MIR` end to end through the registry,
//! compared canonically (parse the expected MIR text, canonical-print
//! both sides — the D-32 comparison).

fn elab(source: &str) -> String {
    let ops = lumo_syntax::registry::elab("Lumo", "MIR").expect("Lumo -> MIR is registered");
    let report = (ops.elab_report)(source);
    assert!(report.errors.is_empty(), "elab errors for `{source}`: {:?}", report.errors);
    report.output
}

fn canonical_mir(text: &str) -> String {
    let out = lumo_syntax::mir::parser::parse(text);
    assert!(out.errors.is_empty(), "expected MIR does not parse: `{text}`: {:?}", out.errors);
    lumo_syntax::mir::printer::canonical(&out.root)
}

#[track_caller]
fn assert_elab(source: &str, expected_mir: &str) {
    assert_eq!(elab(source), canonical_mir(expected_mir), "for `{source}`");
}

#[test]
fn gate_identity_fn() {
    assert_elab("fn id(x) = x", "def id = thunk { fn(x) => ret x }");
}

#[test]
fn multi_param_fns_curry() {
    assert_elab(
        "fn konst(a, b) = a",
        "def konst = thunk { fn(a) => fn(b) => ret a }",
    );
}

#[test]
fn calls_force_then_apply() {
    assert_elab(
        "fn ap(f, x) = f(x)",
        "def ap = thunk { fn(f) => fn(x) => force f(x) }",
    );
}

#[test]
fn computation_args_bind_via_auto_let() {
    // g(h(x)): the inner call is a computation in value position — the
    // engine inserts `let __t1 = … in …` (D-38).
    assert_elab(
        "fn go(g, h, x) = g(h(x))",
        "def go = thunk { fn(g) => fn(h) => fn(x) =>
           let __t1 = force h(x) in force g(__t1) }",
    );
}

#[test]
fn self_recursion_gets_fix() {
    assert_elab(
        "fn spin(x) = spin(x)",
        "def spin = thunk { fix spin => fn(x) => force spin(x) }",
    );
}
