use lumo_compiler::{
    backend::{self, BackendError, CodegenTarget},
    hir,
    lexer::lex,
    lir,
    parser::parse,
};

fn lower_typed(src: &str) -> lir::File {
    let lexed = lex(src);
    let parsed = parse(&lexed.tokens, &lexed.errors);
    let hir = hir::lower(&parsed.file);
    lir::lower(&hir)
}

#[test]
fn ts_backend_emits_ts_js_and_dts() {
    let file = lower_typed("data Bool { .true, .false } fn id(x: Bool): produce Bool := produce x");

    let ts = backend::emit(&file, CodegenTarget::TypeScript).expect("ts emit");
    let js = backend::emit(&file, CodegenTarget::JavaScript).expect("js emit");
    let dts = backend::emit(&file, CodegenTarget::TypeScriptDefinition).expect("d.ts emit");

    assert!(ts.contains("const __lumo_is ="), "{ts}");
    assert!(js.contains("const __lumo_is ="), "{js}");
    assert!(!ts.contains("__lumo_ctor"), "{ts}");
    assert!(!js.contains("__lumo_ctor"), "{js}");
    assert!(ts.contains("export function id(x: Bool): Bool"), "{ts}");
    assert!(
        ts.contains("export type Bool = { [LUMO_TAG]: 'true' } | { [LUMO_TAG]: 'false' };"),
        "{ts}"
    );
    assert!(
        ts.contains("export const Bool: { \"true\": Bool; \"false\": Bool }"),
        "{ts}"
    );
    assert!(js.contains("return x;"), "{js}");
    assert!(
        dts.contains("export declare function id(x: Bool): Bool;"),
        "{dts}"
    );
    assert!(
        dts.contains("export type Bool = { [LUMO_TAG]: 'true' } | { [LUMO_TAG]: 'false' };"),
        "{dts}"
    );
    assert!(
        dts.contains("export declare const Bool: { \"true\": Bool; \"false\": Bool };"),
        "{dts}"
    );
}

#[test]
fn unsupported_backend_target_is_explicit() {
    let file = lower_typed("fn f() := x");

    let err =
        backend::emit(&file, CodegenTarget::Python).expect_err("python backend is not implemented");
    assert_eq!(err, BackendError::UnsupportedTarget(CodegenTarget::Python));
}

#[test]
fn ts_backend_emits_even_without_semantic_checks() {
    let file = lower_typed("fn f() := x");
    let ts = backend::emit(&file, CodegenTarget::TypeScript).expect("ts emit");
    assert!(ts.contains("export function f(): void"), "{ts}");
    assert!(ts.contains("return x;"), "{ts}");
}

#[test]
fn ts_backend_treats_missing_return_type_as_unit() {
    let file = lower_typed("data Bool { .true, .false } fn id(x: Bool) := x");
    let ts = backend::emit(&file, CodegenTarget::TypeScript).expect("ts emit");
    assert!(ts.contains("export function id(x: Bool): void"), "{ts}");
}

#[test]
fn ts_backend_lowers_unit_value_expr_to_void_zero() {
    let file = lower_typed("fn unit(): produce Unit := produce Unit");
    let ts = backend::emit(&file, CodegenTarget::TypeScript).expect("ts emit");
    assert!(ts.contains("export function unit(): void"), "{ts}");
    assert!(ts.contains("return void 0;"), "{ts}");
}

#[test]
fn ts_backend_ctor_uses_variant_tag() {
    let file = lower_typed("data Bool { .true, .false } fn t(): Bool := Bool.true()");
    let js = backend::emit(&file, CodegenTarget::JavaScript).expect("js emit");
    assert!(js.contains("Bool[\"true\"]"), "{js}");
    assert!(!js.contains("Bool[\"true\"]()"), "{js}");
    assert!(js.contains("[LUMO_TAG]: \"true\""), "{js}");
}

#[test]
fn ts_backend_emits_recursive_data_payload_shape() {
    let file = lower_typed("data Nat { .zero, .succ(Nat) }");
    let ts = backend::emit(&file, CodegenTarget::TypeScript).expect("ts emit");
    assert!(
        ts.contains(
            "export type Nat = { [LUMO_TAG]: 'zero' } | { [LUMO_TAG]: 'succ', args: [Nat] };"
        ),
        "{ts}"
    );
}

#[test]
fn ts_backend_match_checks_variant_tag_without_dot() {
    let file = lower_typed(
        "data Bool { .true, .false } fn not(x: Bool): Bool := match x { .true => Bool.false(), .false => Bool.true() }",
    );
    let js = backend::emit(&file, CodegenTarget::JavaScript).expect("js emit");
    assert!(js.contains("__lumo_is(__match_0, \"true\")"), "{js}");
    assert!(js.contains("__lumo_is(__match_0, \"false\")"), "{js}");
}

#[test]
fn ts_backend_lowers_nested_match_patterns_as_tree() {
    let file = lower_typed(
        "data Nat { .zero, .succ(Nat) } fn down2(n: Nat): produce Nat := match n { .succ(.succ(let m)) => produce m, .succ(.zero) => produce Nat.zero(), .zero => produce Nat.zero() }",
    );
    let js = backend::emit(&file, CodegenTarget::JavaScript).expect("js emit");
    assert!(js.contains("__lumo_is(__match_0, \"succ\")"), "{js}");
    assert!(js.contains("__lumo_is(__match_0.args[0], \"succ\")"), "{js}");
    assert!(js.contains("return m;"), "{js}");
    assert!(js.contains("const m = __match_0.args[0].args[0];"), "{js}");
    assert!(js.contains("__lumo_is(__match_0.args[0], \"zero\")"), "{js}");
}

#[test]
fn ts_backend_emits_generic_data_alias() {
    let file = lower_typed("data Option[A] { .some(A), .none }");
    let ts = backend::emit(&file, CodegenTarget::TypeScript).expect("ts emit");
    assert!(
        ts.contains(
            "export type Option<A> = { [LUMO_TAG]: 'some', args: [A] } | { [LUMO_TAG]: 'none' };"
        ),
        "{ts}"
    );
    assert!(
        ts.contains(
            "export const Option: { \"some\": <A>(arg0: A) => Option<A>; \"none\": Option<never> }",
        ),
        "{ts}"
    );
}

#[test]
fn ts_backend_thunk_is_lowered_lazily() {
    let file = lower_typed("fn f(x: A): thunk A := thunk x");
    let js = backend::emit(&file, CodegenTarget::JavaScript).expect("js emit");
    assert!(js.contains("return () => {"), "{js}");
    assert!(js.contains("return x;"), "{js}");
}

#[test]
fn ts_backend_emits_generic_function_type_params_only_in_ts_targets() {
    let file = lower_typed("fn id[A](x: A): produce A := produce x");
    let ts = backend::emit(&file, CodegenTarget::TypeScript).expect("ts emit");
    let dts = backend::emit(&file, CodegenTarget::TypeScriptDefinition).expect("dts emit");
    let js = backend::emit(&file, CodegenTarget::JavaScript).expect("js emit");
    assert!(ts.contains("export function id<A>(x: A): A"), "{ts}");
    assert!(
        dts.contains("export declare function id<A>(x: A): A;"),
        "{dts}"
    );
    assert!(js.contains("function id(x)"), "{js}");
    assert!(!js.contains("id<A>"), "{js}");
}

#[test]
fn ts_backend_accepts_generic_none_branch_when_return_type_is_constrained() {
    let file = lower_typed(
        "data Nat { .zero, .succ(Nat) } data Option[A] { .some(A), .none } fn sub1(x: Nat): produce Option[Nat] := match x { .zero => produce Option.none(), .succ(let x) => produce Option.some(x) }",
    );
    let ts = backend::emit(&file, CodegenTarget::TypeScript).expect("ts emit");
    assert!(
        ts.contains("export function sub1(x: Nat): Option<Nat>"),
        "{ts}"
    );
}

#[test]
fn ts_backend_emits_extern_type_and_extern_fn() {
    let file = lower_typed(
        "#[extern = \"string\"] extern type String; #[extern = \"console.log\"] extern fn console_log(msg: String); fn main(msg: String): produce Unit := console_log(msg)",
    );
    let ts = backend::emit(&file, CodegenTarget::TypeScript).expect("ts emit");
    let js = backend::emit(&file, CodegenTarget::JavaScript).expect("js emit");
    let dts = backend::emit(&file, CodegenTarget::TypeScriptDefinition).expect("d.ts emit");

    assert!(ts.contains("export type String = string;"), "{ts}");
    assert!(dts.contains("export type String = string;"), "{dts}");
    assert!(ts.contains("console.log(msg)"), "{ts}");
    assert!(js.contains("console.log(msg)"), "{js}");
}

#[test]
fn ts_backend_specializes_binary_operator_externs() {
    let file = lower_typed(
        "#[extern = \"string\"] extern type String; #[extern = \"number\"] extern type Number; #[extern = \"String._+_\"] extern fn String_concat(a: String, b: String): produce String; #[extern = \"Number._*_\"] extern fn Number_mul(a: Number, b: Number): produce Number; #[extern = \"Number._^_\"] extern fn Number_pow(a: Number, b: Number): produce Number; fn main(): produce String := String_concat(\"Hello, \", \"world!\")",
    );
    let js = backend::emit(&file, CodegenTarget::JavaScript).expect("js emit");
    assert!(js.contains("return (a + b);"), "{js}");
    assert!(js.contains("return (a * b);"), "{js}");
    assert!(js.contains("return (a ** b);"), "{js}");
    assert!(
        js.contains("return String_concat(\"Hello, \", \"world!\");"),
        "{js}"
    );
    assert!(!js.contains("globalThis[\"String_+\"]"), "{js}");
}

#[test]
fn ts_backend_specializes_unary_operator_externs() {
    let file = lower_typed(
        "#[extern = \"boolean\"] extern type Boolean; #[extern = \"Boolean.!_\"] extern fn Boolean_not(value: Boolean): produce Boolean; fn main(value: Boolean): produce Boolean := Boolean_not(value)",
    );
    let js = backend::emit(&file, CodegenTarget::JavaScript).expect("js emit");
    assert!(js.contains("return (!value);"), "{js}");
}

#[test]
fn ts_backend_flattens_let_iife_to_const() {
    let file = lower_typed(
        "#[extern = \"string\"] extern type String; #[extern = \"String._+_\"] extern fn String_concat(a: String, b: String): produce String; fn main(): produce String := let s = String_concat(\"Hello, \", \"world!\") in String_concat(s, \"!\")",
    );
    let js = backend::emit(&file, CodegenTarget::JavaScript).expect("js emit");
    // IIFE is flattened: ((s) => ...)(expr) → const s = expr; ...
    assert!(
        js.contains("const s = String_concat(\"Hello, \", \"world!\");"),
        "{js}"
    );
    assert!(js.contains("return String_concat(s, \"!\");"), "{js}");
    assert!(!js.contains("((s) =>"), "IIFE should be flattened: {js}");
}

#[test]
fn ts_backend_handle_always_uses_cps() {
    let file = lower_typed(
        "cap E { fn op(): produce A } fn f(a: A): produce A / {} := handle E with bundle { fn op() := produce a } in perform E.op",
    );
    let js = backend::emit(&file, CodegenTarget::JavaScript).expect("js emit");
    // Deep CPS: ALL handles use CPS, even without resume
    // Handler entries get __k parameter, body is CPS-transformed
    assert!(js.contains("__cap_E"), "{js}");
    assert!(js.contains("__k"), "all handles should have CPS __k param: {js}");
    assert!(js.contains("__v"), "CPS identity continuation: {js}");
}

#[test]
fn ts_backend_handle_with_resume_uses_cps() {
    let file = lower_typed(
        "cap E { fn op(): produce A } fn f(a: A): produce A / {} := handle E with bundle { fn op() := resume(a) } in perform E.op",
    );
    let js = backend::emit(&file, CodegenTarget::JavaScript).expect("js emit");
    // CPS: handler entries get __k param, body is CPS-transformed
    assert!(js.contains("__k"), "handler should have CPS __k param: {js}");
    assert!(
        js.contains("resume"),
        "handler should bind resume: {js}"
    );
}

#[test]
fn ts_backend_cps_handle_with_let_perform() {
    let file = lower_typed(
        "cap E { fn op(): produce A } fn f(a: A): produce A / {} := handle E with bundle { fn op() := resume(a) } in let x = perform E.op in produce x",
    );
    let js = backend::emit(&file, CodegenTarget::JavaScript).expect("js emit");
    // CPS transforms: let x = perform E.op in produce x
    // → __cap_E.op((x) => ((v) => v)(x))
    assert!(js.contains("__cap_E.op"), "CPS perform call: {js}");
    assert!(js.contains("__k"), "handler has __k: {js}");
}

#[test]
fn ts_backend_mixed_resume_entries() {
    let file = lower_typed(
        "cap E { fn op1(): produce A; fn op2(x: A): produce B } fn f(a: A, b: B): produce A / {} := handle E with bundle { fn op1() := resume(a); fn op2(x) := produce b } in perform E.op1",
    );
    let js = backend::emit(&file, CodegenTarget::JavaScript).expect("js emit");
    // op1 uses resume → gets ((resume) => ...) wrapper
    // op2 doesn't use resume → gets __k(body) wrapper
    assert!(js.contains("__k"), "CPS mode active: {js}");
    assert!(js.contains("resume"), "op1 binds resume: {js}");
}

#[test]
fn ts_backend_effectful_fn_decl_has_extra_params() {
    let file = lower_typed(
        "cap E { fn op(): produce A } fn inner(): produce A / E := perform E.op",
    );
    let js = backend::emit(&file, CodegenTarget::JavaScript).expect("js emit");
    // Effectful function should have __cap_E and __k params
    assert!(
        js.contains("__cap_E"),
        "effectful fn should have __cap_E param: {js}"
    );
    assert!(
        js.contains("__k"),
        "effectful fn should have __k param: {js}"
    );
    // Body should be CPS-transformed: perform E.op → __cap_E.op(__k)
    assert!(
        js.contains("__cap_E.op"),
        "body should call handler op: {js}"
    );
}

#[test]
fn ts_backend_deep_cps_effectful_fn_call() {
    let file = lower_typed(
        "cap E { fn op(): produce A } fn inner(): produce A / E := perform E.op fn f(a: A): produce A / {} := handle E with bundle { fn op() := resume(a) } in let x = force (thunk inner) in produce x",
    );
    let js = backend::emit(&file, CodegenTarget::JavaScript).expect("js emit");
    // inner should be called with __cap_E and continuation
    // inner(__cap_E, (x) => ...)
    assert!(
        js.contains("inner("),
        "should call inner with args: {js}"
    );
    assert!(
        js.contains("__cap_E"),
        "should pass handler to inner: {js}"
    );
}

#[test]
fn ts_backend_pure_fn_unchanged() {
    let file = lower_typed("fn f(a: A): produce A / {} := produce a");
    let js = backend::emit(&file, CodegenTarget::JavaScript).expect("js emit");
    // Pure function should NOT have __cap or __k params
    assert!(
        !js.contains("__cap"),
        "pure fn should not have __cap param: {js}"
    );
    assert!(
        !js.contains("__k"),
        "pure fn should not have __k param: {js}"
    );
}

#[test]
fn ts_backend_emits_inherent_impl_as_const_object() {
    let file = lower_typed(
        "#[extern = \"string\"] extern type String; #[extern = \"number\"] extern type Number; #[extern = \"String.length\"] extern fn str_len(s: String): produce Number; impl String { fn len(self: String): Number := str_len(self) }",
    );
    let ts = backend::emit(&file, CodegenTarget::TypeScript).expect("ts emit");
    let js = backend::emit(&file, CodegenTarget::JavaScript).expect("js emit");
    // Inherent impl named after target type
    assert!(ts.contains("export const String"), "should export const named after target: {ts}");
    assert!(js.contains("len"), "should contain len method: {js}");
    assert!(js.contains("str_len(self)"), "method body calls str_len: {js}");
}

#[test]
fn ts_backend_emits_unnamed_cap_impl() {
    let file = lower_typed(
        "cap Clone { fn clone(self: A): produce A } impl String: Clone { fn clone(self: String): String := self }",
    );
    let js = backend::emit(&file, CodegenTarget::JavaScript).expect("js emit");
    assert!(
        js.contains("__impl_String_Clone"),
        "unnamed cap impl should be named __impl_Target_Cap: {js}"
    );
    assert!(js.contains("clone"), "should contain clone method: {js}");
}

#[test]
fn ts_backend_emits_named_cap_impl() {
    let file = lower_typed(
        "cap Clone { fn clone(self: A): produce A } impl MyClone = String: Clone { fn clone(self: String): String := self }",
    );
    let js = backend::emit(&file, CodegenTarget::JavaScript).expect("js emit");
    assert!(
        js.contains("MyClone"),
        "named impl should use given name: {js}"
    );
    assert!(
        !js.contains("__impl_"),
        "named impl should not have mangled name: {js}"
    );
}
