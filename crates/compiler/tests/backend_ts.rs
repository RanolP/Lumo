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
    let file = lower_typed("data Bool { .true, .false } fn id(x: Bool): Bool { x }");

    let ts = backend::emit(&file, CodegenTarget::TypeScript).expect("ts emit");
    let js = backend::emit(&file, CodegenTarget::JavaScript).expect("js emit");
    let dts = backend::emit(&file, CodegenTarget::TypeScriptDefinition).expect("d.ts emit");

    assert!(ts.contains("const LUMO_TAG ="), "{ts}");
    assert!(js.contains("const LUMO_TAG ="), "{js}");
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
    let file = lower_typed("fn f() { x }");

    let err =
        backend::emit(&file, CodegenTarget::Python).expect_err("python backend is not implemented");
    assert_eq!(err, BackendError::UnsupportedTarget(CodegenTarget::Python));
}

#[test]
fn ts_backend_emits_even_without_semantic_checks() {
    let file = lower_typed("fn f() { x }");
    let ts = backend::emit(&file, CodegenTarget::TypeScript).expect("ts emit");
    assert!(ts.contains("export function f(): void"), "{ts}");
    assert!(ts.contains("return x;"), "{ts}");
}

#[test]
fn ts_backend_treats_missing_return_type_as_unit() {
    let file = lower_typed("data Bool { .true, .false } fn id(x: Bool) { x }");
    let ts = backend::emit(&file, CodegenTarget::TypeScript).expect("ts emit");
    assert!(ts.contains("export function id(x: Bool): void"), "{ts}");
}

#[test]
fn ts_backend_lowers_unit_value_expr_to_void_zero() {
    let file = lower_typed("fn unit(): Unit { Unit }");
    let ts = backend::emit(&file, CodegenTarget::TypeScript).expect("ts emit");
    assert!(ts.contains("export function unit(): void"), "{ts}");
    assert!(ts.contains("return void 0;"), "{ts}");
}

#[test]
fn ts_backend_ctor_uses_variant_tag() {
    let file = lower_typed("data Bool { .true, .false } fn t(): Bool { Bool.true() }");
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
        "data Bool { .true, .false } fn not(x: Bool): Bool { match x { .true => Bool.false(), .false => Bool.true() } }",
    );
    let js = backend::emit(&file, CodegenTarget::JavaScript).expect("js emit");
    // exhaustive match: last variant emitted as unconditional else, no condition check for "false"
    assert!(js.contains("x[LUMO_TAG] === \"true\""), "{js}");
    assert!(!js.contains("x[LUMO_TAG] === \"false\""), "{js}");
}

#[test]
fn ts_backend_lowers_nested_match_patterns_as_tree() {
    let file = lower_typed(
        "data Nat { .zero, .succ(Nat) } fn down2(n: Nat): Nat { match n { .succ(.succ(let m)) => m, .succ(.zero) => Nat.zero(), .zero => Nat.zero() } }",
    );
    let js = backend::emit(&file, CodegenTarget::JavaScript).expect("js emit");
    assert!(js.contains("n[LUMO_TAG] === \"succ\""), "{js}");
    assert!(js.contains("n.args[0][LUMO_TAG] === \"succ\""), "{js}");
    // single-use `m` is inlined: const m = ...; return m; → return ...;
    assert!(js.contains("return n.args[0].args[0];"), "{js}");
    assert!(!js.contains("const m ="), "{js}");
    // exhaustive match: .zero is the last case, emitted as unconditional else
    assert!(!js.contains("n.args[0][LUMO_TAG] === \"zero\""), "{js}");
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
    let file = lower_typed("fn f(x: A): thunk A { thunk x }");
    let js = backend::emit(&file, CodegenTarget::JavaScript).expect("js emit");
    assert!(js.contains("return () => {"), "{js}");
    assert!(js.contains("return x;"), "{js}");
}

#[test]
fn ts_backend_emits_generic_function_type_params_only_in_ts_targets() {
    let file = lower_typed("fn id[A](x: A): A { x }");
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
        "data Nat { .zero, .succ(Nat) } data Option[A] { .some(A), .none } fn sub1(x: Nat): Option[Nat] { match x { .zero => Option.none(), .succ(let x) => Option.some(x) } }",
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
        "#[extern = \"string\"] extern type String; #[extern = \"console.log()\"] extern fn console_log(msg: String); fn main(msg: String): Unit { console_log(msg) }",
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
        "#[extern = \"string\"] extern type String; #[extern = \"number\"] extern type Number; #[extern = \"String._+_\"] extern fn String_concat(a: String, b: String): String; #[extern = \"Number._*_\"] extern fn Number_mul(a: Number, b: Number): Number; #[extern = \"Number._^_\"] extern fn Number_pow(a: Number, b: Number): Number; fn main(): String { String_concat(\"Hello, \", \"world!\") }",
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
        "#[extern = \"boolean\"] extern type Boolean; #[extern = \"Boolean.!_\"] extern fn Boolean_not(value: Boolean): Boolean; fn main(value: Boolean): Boolean { Boolean_not(value) }",
    );
    let js = backend::emit(&file, CodegenTarget::JavaScript).expect("js emit");
    assert!(js.contains("return (!value);"), "{js}");
}

#[test]
fn ts_backend_flattens_let_iife_to_const() {
    let file = lower_typed(
        "#[extern = \"string\"] extern type String; #[extern = \"String._+_\"] extern fn String_concat(a: String, b: String): String; fn main(): String { let s = String_concat(\"Hello, \", \"world!\"); String_concat(s, \"!\") }",
    );
    let js = backend::emit(&file, CodegenTarget::JavaScript).expect("js emit");
    // IIFE is flattened, single-use `s` is then inlined away
    assert!(!js.contains("((s) =>"), "IIFE should be flattened: {js}");
    assert!(
        js.contains("return String_concat(String_concat(\"Hello, \", \"world!\"), \"!\");"),
        "{js}"
    );
}

#[test]
fn ts_backend_handle_always_uses_cps() {
    let file = lower_typed(
        "cap E { fn op(): A } fn f(a: A): A / {} { handle E with bundle { fn op() { a } } in E.op }",
    );
    let js = backend::emit(&file, CodegenTarget::JavaScript).expect("js emit");
    // Deep CPS: ALL handles use CPS, even without resume
    // Handler entries get __k parameter, body is CPS-transformed
    // Cap is accessed through ambient __caps bundle: `__caps.E.op(...)`
    assert!(js.contains("__caps.E"), "{js}");
    assert!(js.contains("__k"), "all handles should have CPS __k param: {js}");
    assert!(js.contains("__v"), "CPS identity continuation: {js}");
}

#[test]
fn ts_backend_handle_with_resume_uses_cps() {
    let file = lower_typed(
        "cap E { fn op(): A } fn f(a: A): A / {} { handle E with bundle { fn op() { resume(a) } } in E.op }",
    );
    let js = backend::emit(&file, CodegenTarget::JavaScript).expect("js emit");
    // CPS: handler entries get a __k_perform param; body is CPS-transformed.
    // `resume(a)` in tail position emits inline as `__k_handle(__k_perform(...))`
    // — there is no local `resume` binding any more, just direct
    // continuation invocation.
    assert!(
        js.contains("__k_perform"),
        "handler should reference __k_perform: {js}"
    );
    assert!(
        js.contains("__k_handle"),
        "handler should reference __k_handle: {js}"
    );
}

#[test]
fn ts_backend_cps_handle_with_let_perform() {
    let file = lower_typed(
        "cap E { fn op(): A } fn f(a: A): A / {} { handle E with bundle { fn op() { resume(a) } } in { let x = E.op; x } }",
    );
    let js = backend::emit(&file, CodegenTarget::JavaScript).expect("js emit");
    // CPS transforms: let x = E.op; x
    // → __caps.E.op((x) => ((v) => v)(x))
    assert!(js.contains("__caps.E.op"), "CPS perform call: {js}");
    assert!(js.contains("__k"), "handler has __k: {js}");
}

#[test]
fn ts_backend_mixed_resume_entries() {
    let file = lower_typed(
        "cap E { fn op1(): A; fn op2(x: A): B } fn f(a: A, b: B): A / {} { handle E with bundle { fn op1() { resume(a) }; fn op2(x) { b } } in E.op1 }",
    );
    let js = backend::emit(&file, CodegenTarget::JavaScript).expect("js emit");
    // op1 has tail `resume(a)` → emits `__k_handle(__k_perform(a))`.
    // op2 aborts (no resume) → its body value flows directly to __k_handle.
    assert!(js.contains("__k_perform"), "tail resume should reach __k_perform: {js}");
    assert!(js.contains("__k_handle"), "abort path should reach __k_handle: {js}");
}

#[test]
fn ts_backend_nested_perform_in_handler_body_cps_lowered() {
    // Stage B: the handler body is CPS-lowered. A nested Perform on an
    // outer-handled cap inside the handler body must be dispatched through
    // the ambient `__caps` bundle (threaded CPS), not emitted as a raw call.
    let file = lower_typed(
        "cap E { fn op(): A } cap F { fn go(): A } \
         fn f(a: A): A / { F } { handle E with bundle { fn op() { F.go } } in E.op }",
    );
    let js = backend::emit(&file, CodegenTarget::JavaScript).expect("js emit");
    assert!(
        js.contains("__caps.F"),
        "nested F.go inside handler body should dispatch through __caps bundle: {js}"
    );
}

#[test]
fn ts_backend_handler_aborts_by_default() {
    // Without `resume`, the handler body's value aborts — it flows through
    // `__k_handle` (the `handle` expression's continuation) rather than
    // implicitly resuming at the perform site.
    let file = lower_typed(
        "cap E { fn op(): A } fn f(a: A, b: A): A / {} { handle E with bundle { fn op() { b } } in E.op }",
    );
    let js = backend::emit(&file, CodegenTarget::JavaScript).expect("js emit");
    assert!(
        js.contains("__k_handle"),
        "handler's abort path should reference __k_handle: {js}"
    );
    assert!(
        !js.contains("resume"),
        "abort-only handler shouldn't emit a resume binding: {js}"
    );
}

#[test]
fn ts_backend_handler_factory_closes_over_k_handle() {
    // The handle site invokes a factory `(__k_handle) => { ... }` with the
    // outer continuation. The resulting handler object is what's installed
    // into the `__caps` bundle.
    let file = lower_typed(
        "cap E { fn op(): A } fn f(a: A): A / {} { handle E with bundle { fn op() { a } } in E.op }",
    );
    let js = backend::emit(&file, CodegenTarget::JavaScript).expect("js emit");
    assert!(
        js.contains("(__k_handle) =>"),
        "handler should be produced by a factory arrow over __k_handle: {js}"
    );
}

/// Extract the body region of a function declaration from emitted JS.
/// Assumes the function is terminated by a line containing just `}`.
fn extract_fn_body<'a>(js: &'a str, fn_signature_prefix: &str) -> Option<&'a str> {
    let start = js.find(fn_signature_prefix)?;
    let rest = &js[start..];
    // Find the matching closing `}` at column 0 (i.e. function-level, not nested).
    let close = rest.find("\n}")?;
    Some(&rest[..close + 2])
}

#[test]
fn ts_backend_identity_continuation_is_shared_const() {
    // Pass 1: instead of allocating a fresh `(__v) => __v` closure at every
    // CPS entry point, emit a shared runtime constant `__identity` and
    // reference it. The main entry wrapper invokes `__main_cps({...}, __identity)`.
    // Uses a cap with a default impl so main's wrapper compiles cleanly.
    let file = lower_typed(
        "cap E { fn op(a: A): A } \
         impl E { fn op(a) { a } } \
         fn main(): A / { E } { E.op(a) } \
         extern fn a(): A",
    );
    let js = backend::emit(&file, CodegenTarget::JavaScript).expect("js emit");
    assert!(
        js.contains("const __identity = (__v) => __v;"),
        "runtime prelude should emit the shared __identity const: {js}"
    );
    assert!(
        js.contains(", __identity)"),
        "entry wrapper should reference the shared __identity const as continuation: {js}"
    );
    // No inline `(__v) => __v` arrow at the main-wrapper call site.
    assert!(
        !js.contains("(__v) => __v\n") && !js.contains("(__v) => __v)"),
        "no inline identity arrow should remain — all sites use __identity: {js}"
    );
}

#[test]
fn ts_backend_cps_runtime_params_are_typed() {
    // TS strict-mode rejects implicit-any. The CPS plumbing is typed via
    // the generic union types (`__CpsValue` / `__Ret` / `__Kont<T>`) plus
    // a per-cap bundle alias (`__Bundle_<Cap>`) and an inline precise
    // `__caps` shape per effectful function.
    let file = lower_typed(
        "cap E { fn op(): A } \
         fn inner(): A / { E } { E.op } \
         fn outer(): A / { E } { handle E with bundle { fn op() { resume(a) } } in E.op } \
         extern fn a(): A",
    );
    let ts = backend::emit(&file, CodegenTarget::TypeScript).expect("ts emit");

    // Runtime prelude must define the CPS type plumbing.
    assert!(
        ts.contains("type __Ret = "),
        "TS prelude should declare __Ret alias: {ts}"
    );
    assert!(
        ts.contains("type __Kont<"),
        "TS prelude should declare generic __Kont<T>: {ts}"
    );
    assert!(
        ts.contains("type __CpsValue = "),
        "TS prelude should declare __CpsValue union: {ts}"
    );
    // Per-cap bundle alias is emitted.
    assert!(
        ts.contains("type __Bundle_E = "),
        "TS output should emit __Bundle_E per-cap alias: {ts}"
    );

    // Effectful user fn signatures use the precise inline `__caps` bundle
    // type and the continuation type-parametrized on return.
    assert!(
        ts.contains("__caps: { readonly E: __Bundle_E }"),
        "__caps should be typed with inline per-cap bundle: {ts}"
    );
    assert!(
        ts.contains("__k: __Kont<A>"),
        "__k should be typed __Kont<A> (the fn's return type): {ts}"
    );
    // Handler plumbing inside the bundle factory falls back to generic shapes.
    assert!(
        ts.contains("__k_perform: __Kont<"),
        "__k_perform should be typed as a __Kont: {ts}"
    );
    assert!(
        ts.contains("__k_handle: __Kont<"),
        "__k_handle should be typed as a __Kont: {ts}"
    );
    // And absolutely no `any` anywhere in the emitted output.
    assert!(
        !ts.contains(": any") && !ts.contains("as any"),
        "TS output must not contain `any`: {ts}"
    );
}

#[test]
fn ts_backend_tail_thunk_elided_for_fn_call() {
    // Pass 2: a passthrough fn whose body is a tail call to another effectful
    // fn should NOT wrap in `__thunk(() => ...)` — the callee already returns
    // a thunk, so the outer wrap would double-bounce the trampoline.
    let file = lower_typed(
        "cap E { fn op(): A } fn inner(): A / { E } { E.op } fn outer(): A / { E } { inner() }",
    );
    let js = backend::emit(&file, CodegenTarget::JavaScript).expect("js emit");
    let outer_body = extract_fn_body(&js, "export function outer(")
        .expect("outer function should be in emitted JS");
    assert!(
        !outer_body.contains("__thunk("),
        "outer's tail call to inner should skip the __thunk wrap; got:\n{outer_body}"
    );
    assert!(
        outer_body.contains("inner(__caps, __k)"),
        "outer should directly return inner(__caps, __k); got:\n{outer_body}"
    );
}

#[test]
fn ts_backend_tail_thunk_kept_for_k_call() {
    // Pass 2's predicate is conservative: a fn whose body is `__k(x)` (tail
    // resume, not a known-thunked call) must KEEP the outer __thunk so the
    // trampoline can unwind deep chains safely.
    let file = lower_typed(
        "cap E { fn op(): A } \
         fn f(a: A): A / { E } { handle E with bundle { fn op() { resume(a) } } in E.op }",
    );
    let js = backend::emit(&file, CodegenTarget::JavaScript).expect("js emit");
    // Handler bundle methods still wrap in __thunk — check we haven't broken
    // that uniformly. Body of handler `op` references `__k_perform` and
    // wraps in __thunk.
    assert!(
        js.contains("__thunk("),
        "handler method's __thunk wrap must survive Pass 2: {js}"
    );
}

#[test]
fn ts_backend_tail_resume_emits_inline_kperform() {
    // Tail `resume(v)` emits inline as `__k_handle(__k_perform(v))` —
    // no local `resume` binding, no nested `__trampoline`. The outer
    // trampoline unwinds the returned thunk iteratively, so nested
    // handler chains stay stack-safe.
    let file = lower_typed(
        "cap E { fn op(): A } fn f(a: A): A / {} { handle E with bundle { fn op() { resume(a) } } in E.op }",
    );
    let js = backend::emit(&file, CodegenTarget::JavaScript).expect("js emit");
    assert!(
        js.contains("__k_handle(__k_perform("),
        "tail resume should emit `__k_handle(__k_perform(...))`: {js}"
    );
    assert!(
        !js.contains("const resume ="),
        "no local `resume` binding should be emitted: {js}"
    );
    // Tail position never wraps __k_perform in a synchronous trampoline.
    assert!(
        !js.contains("__trampoline(__k_perform("),
        "tail resume must not invoke a nested __trampoline: {js}"
    );
}

#[test]
fn ts_backend_non_tail_resume_drives_synchronously() {
    // Non-tail `resume(v)` (used in let-value or compound expression
    // position) must drive `__k_perform(v)` through `__trampoline` at the
    // call site so the resumed continuation's side effects actually run.
    // This is what makes multi-shot non-determinism (Coin/Choice) work.
    let file = lower_typed(
        "cap Coin { fn flip(): A } \
         fn f(a: A): A / {} { \
           handle Coin with bundle { fn flip() { let _ = resume(a); resume(a) } } in Coin.flip \
         }",
    );
    let js = backend::emit(&file, CodegenTarget::JavaScript).expect("js emit");
    // First resume (let-value, non-tail) → synchronous __trampoline.
    assert!(
        js.contains("__trampoline(__k_perform("),
        "non-tail resume should synchronously drive __k_perform: {js}"
    );
}

#[test]
fn ts_backend_effectful_fn_decl_has_extra_params() {
    let file = lower_typed(
        "cap E { fn op(): A } fn inner(): A / { E } { E.op }",
    );
    let js = backend::emit(&file, CodegenTarget::JavaScript).expect("js emit");
    // Effectful function takes a single __caps bundle plus __k continuation.
    assert!(
        js.contains("__caps"),
        "effectful fn should have __caps bundle param: {js}"
    );
    assert!(
        js.contains("__k"),
        "effectful fn should have __k param: {js}"
    );
    // Body should be CPS-transformed: E.op → __caps.E.op(__k)
    assert!(
        js.contains("__caps.E.op"),
        "body should call handler op via bundle: {js}"
    );
}

#[test]
fn ts_backend_deep_cps_effectful_fn_call() {
    let file = lower_typed(
        "cap E { fn op(): A } fn inner(): A / { E } { E.op } fn f(a: A): A / {} { handle E with bundle { fn op() { resume(a) } } in { let x = force (thunk inner); x } }",
    );
    let js = backend::emit(&file, CodegenTarget::JavaScript).expect("js emit");
    // inner should be called with the ambient __caps bundle and continuation:
    // inner(__caps, (x) => ...)
    assert!(
        js.contains("inner("),
        "should call inner with args: {js}"
    );
    assert!(
        js.contains("__caps"),
        "should pass __caps bundle to inner: {js}"
    );
}

#[test]
fn ts_backend_pure_fn_unchanged() {
    let file = lower_typed("fn f(a: A): A / {} { a }");
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
        "#[extern = \"string\"] extern type String; #[extern = \"number\"] extern type Number; #[extern = \"String.length\"] extern fn str_len(s: String): Number; impl String { fn len(self: String): Number { str_len(self) } }",
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
        "cap Clone { fn clone(self: A): A } impl String: Clone { fn clone(self: String): String { self } }",
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
        "cap Clone { fn clone(self: A): A } impl MyClone = String: Clone { fn clone(self: String): String { self } }",
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
