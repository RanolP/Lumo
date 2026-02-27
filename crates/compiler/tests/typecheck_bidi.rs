use lumo_compiler::{lexer::lex, parser::parse, typecheck::typecheck_file};

fn check(src: &str) -> Vec<String> {
    let lexed = lex(src);
    let parsed = parse(&lexed.tokens, &lexed.errors);
    typecheck_file(&parsed.file)
        .into_iter()
        .map(|e| e.message)
        .collect()
}

#[test]
fn typechecks_simple_produce() {
    let errs = check("fn id(a: A): produce A / {} := produce a");
    assert!(errs.is_empty(), "errors: {errs:?}");
}

#[test]
fn typechecks_let_in() {
    let errs = check("fn f(a: A): produce A / {} := let x = a in produce x");
    assert!(errs.is_empty(), "errors: {errs:?}");
}

#[test]
fn reports_unknown_variable() {
    let errs = check("fn f(): produce A / {} := produce x");
    assert!(errs.iter().any(|e| e.contains("unknown variable `x`")));
}

#[test]
fn reports_type_mismatch() {
    let errs = check("fn f(a: A): produce B / {} := produce a");
    assert!(errs.iter().any(|e| e.contains("type mismatch")));
}

#[test]
fn typechecks_thunk_and_force() {
    let errs = check("fn f(job: thunk A): A / {} := force job");
    assert!(errs.is_empty(), "errors: {errs:?}");
}

#[test]
fn typechecks_match_with_pattern_binding() {
    let errs = check("fn f(a: A): produce A / {} := match a { x => produce x }");
    assert!(errs.is_empty(), "errors: {errs:?}");
}

#[test]
fn typechecks_match_with_constructor_pattern_binding() {
    let errs = check(
        "data OptionA { .some(A), .none } fn f(a: OptionA, b: A): produce A / {} := match a { .some(let x) => produce x, .none => produce b }",
    );
    assert!(errs.is_empty(), "errors: {errs:?}");
}

#[test]
fn reports_match_pattern_arity_mismatch() {
    let errs = check("data Option { .some(A), .none } fn f(a: Option): produce Option / {} := match a { .some(let x, let y) => produce a, .none => produce a }");
    assert!(errs.iter().any(|e| e.contains("expects 1 args, got 2")));
}

#[test]
fn reports_unknown_variant_in_match_pattern() {
    let errs = check("data Option { .some(A), .none } fn f(a: Option): produce Option / {} := match a { .nope => produce a }");
    assert!(errs.iter().any(|e| e.contains("unknown variant `nope`")));
}

#[test]
fn pattern_binding_uses_variant_payload_type() {
    let errs = check(
        "data OptionInt { .some(B), .none } fn f(a: OptionInt, b: B): produce B / {} := match a { .some(x) => produce x, .none => produce b }",
    );
    assert!(errs.is_empty(), "errors: {errs:?}");
}

#[test]
fn pattern_binding_payload_type_mismatch_is_reported() {
    let errs = check(
        "data OptionInt { .some(B), .none } fn f(a: OptionInt, c: C): produce B / {} := match a { .some(x) => produce c, .none => produce c }",
    );
    assert!(errs
        .iter()
        .any(|e| e.contains("type mismatch: expected produce B, got produce C")));
}

#[test]
fn nested_constructor_pattern_binding_type_propagates() {
    let errs = check(
        "data Pair { .pair(B, C) } data Wrap { .wrap(Pair), .empty } fn f(w: Wrap, b: B): produce B / {} := match w { .wrap(.pair(x, _)) => produce x, .empty => produce b }",
    );
    assert!(errs.is_empty(), "errors: {errs:?}");
}

#[test]
fn nested_constructor_pattern_unknown_variant_is_reported() {
    let errs = check(
        "data Pair { .pair(B, C) } data Wrap { .wrap(Pair), .empty } fn f(w: Wrap, b: B): produce B / {} := match w { .wrap(.nope(x)) => produce b, .empty => produce b }",
    );
    assert!(errs
        .iter()
        .any(|e| e.contains("unknown variant `nope` in match pattern")));
}

#[test]
fn reports_non_exhaustive_match() {
    let errs = check(
        "data Bool { .true, .false } fn f(b: Bool): produce Bool / {} := match b { .true => produce b }",
    );
    assert!(errs
        .iter()
        .any(|e| e.contains("non-exhaustive match: missing variants false")));
}

#[test]
fn wildcard_makes_match_exhaustive() {
    let errs = check(
        "data Bool { .true, .false } fn f(b: Bool): produce Bool / {} := match b { .true => produce b, _ => produce b }",
    );
    assert!(errs.is_empty(), "errors: {errs:?}");
}

#[test]
fn reports_unreachable_arm_after_catchall() {
    let errs = check(
        "data Bool { .true, .false } fn f(b: Bool): produce Bool / {} := match b { _ => produce b, .true => produce b }",
    );
    assert!(errs
        .iter()
        .any(|e| e.contains("unreachable match arm: prior catch-all pattern")));
}

#[test]
fn reports_unreachable_duplicate_variant_arm() {
    let errs = check(
        "data Bool { .true, .false } fn f(b: Bool): produce Bool / {} := match b { .true => produce b, .true => produce b, .false => produce b }",
    );
    assert!(errs
        .iter()
        .any(|e| e.contains("unreachable match arm: variant `true` already covered")));
}

#[test]
fn typechecks_constructor_application_as_data_value() {
    let errs = check("data OptionA { .some(A), .none } fn mk(a: A): OptionA / {} := OptionA.some(a)");
    assert!(errs.is_empty(), "errors: {errs:?}");
}

#[test]
fn reports_missing_dot_in_constructor_pattern_call_shape() {
    let errs = check(
        "data OptionA { .some(A), .none } fn f(a: OptionA, b: A): produce A / {} := match a { some(x) => produce b, .none => produce b }",
    );
    assert!(errs
        .iter()
        .any(|e| e.contains("constructor pattern must start with `.`")));
}

#[test]
fn reports_missing_dot_in_variant_pattern_name() {
    let errs = check(
        "data OptionA { .some(A), .none } fn f(a: OptionA, b: A): produce A / {} := match a { some => produce b, .none => produce b }",
    );
    assert!(errs
        .iter()
        .any(|e| e.contains("variant pattern `some` must be written `.some`")));
}
