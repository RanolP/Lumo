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
    let errs = check("fn f(job: thunk produce A): produce A / {} := force job");
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
        .any(|e| e.contains("type mismatch: expected B, got C")));
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
        .any(|e| e.contains("non-exhaustive match: missing patterns .false")));
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
        .any(|e| e.contains("unreachable match arm: pattern already covered")));
}

#[test]
fn reports_unreachable_duplicate_variant_arm() {
    let errs = check(
        "data Bool { .true, .false } fn f(b: Bool): produce Bool / {} := match b { .true => produce b, .true => produce b, .false => produce b }",
    );
    assert!(errs
        .iter()
        .any(|e| e.contains("unreachable match arm: pattern already covered")));
}

#[test]
fn nested_patterns_are_checked_for_usefulness_and_exhaustiveness() {
    let errs = check(
        "data Bool { .true, .false } data Nat { .zero, .succ(Nat) } fn is_even(n: Nat, t: Bool, f: Bool): produce Bool / {} := match n { .zero => produce t, .succ(.zero) => produce f, .succ(.succ(let n)) => is_even(n, t, f) }",
    );
    assert!(errs.is_empty(), "errors: {errs:?}");
}

#[test]
fn reports_all_uncovered_nested_patterns() {
    let errs = check(
        "data Bool { .true, .false } data Nat { .zero, .succ(Nat) } fn f(n: Nat, b: Bool): produce Bool / {} := match n { .succ(.zero) => produce b }",
    );
    assert!(errs
        .iter()
        .any(|e| e.contains("non-exhaustive match: missing patterns .succ(.succ(_)), .zero")));
}

#[test]
fn typechecks_bundle_field_application_as_computation() {
    let errs = check(
        "data OptionA { .some(A), .none } fn mk(a: A): produce OptionA / {} := OptionA.some(a)",
    );
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

#[test]
fn generic_constructor_none_unifies_with_expected_return_type() {
    let errs = check(
        "data Nat { .zero, .succ(Nat) } data Option[A] { .some(A), .none } fn sub1(x: Nat): produce Option[Nat] / {} := match x { .zero => Option.none, .succ(let x) => Option.some(x) }",
    );
    assert!(errs.is_empty(), "errors: {errs:?}");
}

#[test]
fn generic_match_payload_binds_with_instantiated_type() {
    let errs = check(
        "data Nat { .zero, .succ(Nat) } data Option[A] { .some(A), .none } fn get_or_zero(o: Option[Nat], z: Nat): produce Nat / {} := match o { .some(x) => produce x, .none => produce z }",
    );
    assert!(errs.is_empty(), "errors: {errs:?}");
}

#[test]
fn generic_constructor_without_constraints_reports_inference_failure() {
    let errs = check(
        "data Option[A] { .some(A), .none } fn bad(u: Unit): produce Unit / {} := let x = Option.none in produce u",
    );
    assert!(errs
        .iter()
        .any(|e| e.contains("cannot infer type arguments for bundle field `Option.none`")));
}

#[test]
fn match_arms_allow_value_sugar_when_expected_is_produce() {
    let errs = check(
        "data Bool { .true, .false } fn not(x: Bool): produce Bool / {} := match x { .true => Bool.false, .false => Bool.true }",
    );
    assert!(errs.is_empty(), "errors: {errs:?}");
}

#[test]
fn match_arm_rejects_calling_nullary_bundle_field() {
    let errs = check(
        "data Bool { .true, .false } fn not(x: Bool): produce Bool / {} := match x { .true => Bool.false(), .false => Bool.true() }",
    );
    assert!(errs.iter().any(|e| e.contains("which is not a function")));
}

#[test]
fn produce_cannot_wrap_bundle_computation() {
    let errs =
        check("data OptionA { .some(A), .none } fn mk(a: A): produce OptionA / {} := produce OptionA.some(a)");
    assert!(errs.iter().any(|e| e.contains("expected value expression")));
}

#[test]
fn recursive_function_call_typechecks() {
    let errs = check(
        "data Nat { .zero, .succ(Nat) } fn add(n: Nat, m: Nat): produce Nat / {} := match n { .zero => produce m, .succ(let n) => let next = Nat.succ(m) in add(n, next) }",
    );
    assert!(errs.is_empty(), "errors: {errs:?}");
}

#[test]
fn mutual_recursive_function_calls_typecheck() {
    let errs = check(
        "data Bool { .true, .false } data Nat { .zero, .succ(Nat) } fn even(n: Nat): produce Bool / {} := match n { .zero => Bool.true, .succ(let n) => odd(n) } fn odd(n: Nat): produce Bool / {} := match n { .zero => Bool.false, .succ(let n) => even(n) }",
    );
    assert!(errs.is_empty(), "errors: {errs:?}");
}

#[test]
fn extern_fn_call_typechecks() {
    let errs = check(
        "#[extern = \"string\"] extern type String; #[extern = \"console.log\"] extern fn console_log(msg: String); fn main(msg: String): produce Unit / {} := console_log(msg)",
    );
    assert!(errs.is_empty(), "errors: {errs:?}");
}

#[test]
fn extern_attribute_value_must_be_string() {
    let errs = check("#[extern = thunk produce \"x\"] extern type String;");
    assert!(errs.iter().any(|e| e.contains("expected String")));
}
