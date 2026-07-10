//! M1 step 4 gate: composed builder output reparses to exactly the tree
//! it was built to represent (correct by construction).

use lumo_syntax::mir::builder as b;
use lumo_syntax::mir::builder::Operand;
use lumo_syntax::mir::syntax_kind::SyntaxKind;
use lumo_syntax::mir::{parser, printer};

fn assert_reparses_to(text: &str, expected_sexpr: &str) {
    let out = parser::parse(text);
    assert!(out.errors.is_empty(), "`{text}` has parse errors: {:?}", out.errors);
    assert_eq!(printer::sexpr(&out.root), expected_sexpr, "for `{text}`");
}

#[test]
fn identity_def_round_trips() {
    let text = b::file(&[&b::def(
        "id",
        &b::thunk_v(&b::lam_c("x", &b::ret_c(&b::var_v("x")))),
    )]);
    assert_reparses_to(&text, "(FILE (DEF (THUNK_V (LAM_C (RET_C (VAR_V))))))");
}

#[test]
fn application_of_closed_operand_needs_no_parens() {
    // force f(x)(y) — postfix chains attach outside, no parens inserted.
    let app1 = b::comp_postfix(
        Operand { text: &b::force_c(&b::var_v("f")), kind: Some(SyntaxKind::FORCE_C) },
        &b::value_args(&[&b::var_v("x")]),
    );
    let app2 = b::comp_postfix(
        Operand { text: &app1, kind: Some(SyntaxKind::COMP_POSTFIX) },
        &b::value_args(&[&b::var_v("y")]),
    );
    assert_eq!(app2, "force f ( x ) ( y )");
    let text = b::file(&[&b::def("app", &b::thunk_v(&app2))]);
    assert_reparses_to(
        &text,
        "(FILE (DEF (THUNK_V (COMP_POSTFIX (COMP_POSTFIX (FORCE_C (VAR_V)) (VALUE_ARGS (VAR_V))) (VALUE_ARGS (VAR_V))))))",
    );
}

#[test]
fn application_of_open_operand_is_parenthesized() {
    // (let a = ret x in force a)(y) — without parens, `(y)` would be
    // absorbed into the let body on reparse.
    let letc = b::let_c("a", &b::ret_c(&b::var_v("x")), &b::force_c(&b::var_v("a")));
    let app = b::comp_postfix(
        Operand { text: &letc, kind: Some(SyntaxKind::LET_C) },
        &b::value_args(&[&b::var_v("y")]),
    );
    let text = b::file(&[&b::def("k", &b::thunk_v(&app))]);
    assert_reparses_to(
        &text,
        "(FILE (DEF (THUNK_V (COMP_POSTFIX (PAREN_C (LET_C (RET_C (VAR_V)) (FORCE_C (VAR_V)))) (VALUE_ARGS (VAR_V))))))",
    );
}

#[test]
fn unknown_operand_kind_is_parenthesized_defensively() {
    let app = b::comp_postfix(
        Operand { text: "ret x", kind: None },
        &b::value_args(&[]),
    );
    assert_eq!(app, "( ret x ) ( )");
}

#[test]
fn case_ctor_and_optionals_round_trip() {
    let arm_zero = b::case_arm("zero", None, &b::ret_c(&b::roll_v(&b::ctor_v("zero", None))));
    let arm_succ = b::case_arm(
        "succ",
        Some(&b::case_binders(&["m"])),
        &b::ret_c(&b::var_v("m")),
    );
    let case = b::case_c(&b::unroll_v(&b::var_v("n")), &[&arm_zero, &arm_succ]);
    let text = b::file(&[&b::def("pred", &b::thunk_v(&b::lam_c("n", &case)))]);
    assert_reparses_to(
        &text,
        "(FILE (DEF (THUNK_V (LAM_C (CASE_C (UNROLL_V (VAR_V)) (CASE_ARM (RET_C (ROLL_V (CTOR_V)))) (CASE_ARM (CASE_BINDERS) (RET_C (VAR_V))))))))",
    );
}

#[test]
fn annotated_value_with_types_round_trips() {
    let ty = b::u_type_v(&b::fn_type_c(
        &[&b::named_type_v("Number", None)],
        &b::f_type_c(&b::named_type_v("List", Some(&b::type_args(&[&b::named_type_v("Number", None)])))),
    ));
    let text = b::file(&[&b::def("v", &b::paren_v(&b::var_v("f"), Some(&ty)))]);
    assert_reparses_to(
        &text,
        "(FILE (DEF (PAREN_V (VAR_V) (U_TYPE_V (FN_TYPE_C (NAMED_TYPE_V) (F_TYPE_C (NAMED_TYPE_V (TYPE_ARGS (NAMED_TYPE_V)))))))))",
    );
}

#[test]
fn bundle_handle_sel_round_trips() {
    let clause = b::bundle_clause("log", &["msg"], &b::ret_c(&b::var_v("msg")));
    let handler = b::bundle_v(&[&clause]);
    let body = b::let_c(
        "c",
        &b::perform_c("Console"),
        &b::comp_postfix(
            Operand {
                text: &b::sel_c(&b::var_v("c"), "log"),
                kind: Some(SyntaxKind::SEL_C),
            },
            &b::value_args(&[&b::str_v("\"hi\"")]),
        ),
    );
    let text = b::file(&[&b::def(
        "prog",
        &b::thunk_v(&b::handle_c("Console", &handler, &body)),
    )]);
    assert_reparses_to(
        &text,
        "(FILE (DEF (THUNK_V (HANDLE_C (BUNDLE_V (BUNDLE_CLAUSE (RET_C (VAR_V)))) (LET_C (PERFORM_C) (COMP_POSTFIX (SEL_C (VAR_V)) (VALUE_ARGS (STR_V))))))))",
    );
}
