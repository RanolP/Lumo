use lumo_compiler::{
    backend::{self, CodegenTarget},
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

fn emit_rust(src: &str) -> String {
    let file = lower_typed(src);
    backend::emit(&file, CodegenTarget::Rust).expect("rust emit")
}

#[test]
fn rs_backend_emits_pure_function() {
    let rs = emit_rust("fn id(x: String): String { x }");
    assert!(rs.contains("fn id(x: String) -> String"), "{rs}");
    assert!(rs.contains("x"), "{rs}");
}

#[test]
fn rs_backend_emits_adt_as_enum() {
    let rs = emit_rust("data Bool { .true, .false }");
    assert!(rs.contains("enum Bool"), "{rs}");
    assert!(rs.contains("True"), "{rs}");
    assert!(rs.contains("False"), "{rs}");
    assert!(rs.contains("#[derive(Debug, Clone)]"), "{rs}");
}

#[test]
fn rs_backend_emits_adt_with_payloads() {
    let rs = emit_rust("data Option[T] { .some(T), .none }");
    assert!(rs.contains("enum Option<T:"), "{rs}");
    assert!(rs.contains("Some(T)"), "{rs}");
    assert!(rs.contains("None"), "{rs}");
}

#[test]
fn rs_backend_emits_match() {
    let rs = emit_rust(
        "data Bool { .true, .false } fn not(b: Bool): Bool { match b { Bool.true => Bool.false, Bool.false => Bool.true } }",
    );
    assert!(rs.contains("fn not(b: Bool) -> Bool"), "{rs}");
    assert!(rs.contains("match"), "{rs}");
    assert!(rs.contains("Bool::True"), "{rs}");
    assert!(rs.contains("Bool::False"), "{rs}");
}

#[test]
fn rs_backend_emits_string_literal() {
    let rs = emit_rust("fn greet(): String { \"hello\" }");
    assert!(rs.contains("\"hello\".to_string()"), "{rs}");
}

#[test]
fn rs_backend_emits_number_literal() {
    let rs = emit_rust("fn pi(): Number { 3.14 }");
    assert!(rs.contains("3.14"), "{rs}");
    assert!(rs.contains("f64"), "{rs}");
}

#[test]
fn rs_backend_emits_let_in() {
    let rs = emit_rust("fn f(): String { let x = \"hi\"; x }");
    assert!(rs.contains("let x ="), "{rs}");
}

#[test]
fn rs_backend_extern_fn_println() {
    let rs = emit_rust("#[extern(name = \"console.log\")] extern fn println(msg: String);");
    assert!(rs.contains("fn println(msg: String)"), "{rs}");
    assert!(rs.contains("println!"), "{rs}");
}

#[test]
fn rs_backend_main_fn_wrapper() {
    let rs = emit_rust(
        "#[extern(name = \"console.log\")] extern fn println(msg: String); fn main() { println(\"hi\") }",
    );
    assert!(rs.contains("fn main()"), "{rs}");
    assert!(rs.contains("println("), "{rs}");
}

#[test]
fn rs_backend_skips_builtin_extern_types() {
    let rs = emit_rust("#[extern = \"string\"] extern type String;");
    // Should NOT generate a type alias for built-in String
    assert!(!rs.contains("type String"), "{rs}");
}

#[test]
fn rs_backend_emits_unit_return() {
    let rs = emit_rust("fn noop() { Unit }");
    assert!(rs.contains("fn noop() -> ()"), "{rs}");
}

#[test]
fn rs_backend_emits_ctor_call() {
    let rs = emit_rust(
        "data Pair { .mk(String, String) } fn test(): Pair { Pair.mk(\"a\", \"b\") }",
    );
    assert!(rs.contains("Pair::Mk("), "{rs}");
}

#[test]
fn rs_backend_emits_match_with_bindings() {
    let rs = emit_rust(
        "data Box { .wrap(String) } fn unwrap(b: Box): String { match b { Box.wrap(x) => x } }",
    );
    assert!(rs.contains("Box::Wrap(x)"), "{rs}");
}

#[test]
fn rs_backend_emits_recursive_adt_with_box() {
    let rs = emit_rust("data List[A] { .nil, .cons(A, List[A]) }");
    assert!(
        rs.contains("Box<List"),
        "recursive field should be wrapped in Box, got:\n{rs}"
    );
    assert!(
        rs.contains("Nil"),
        "should have Nil variant, got:\n{rs}"
    );
    assert!(
        rs.contains("Cons(A, Box<"),
        "Cons should have A and Box<List>, got:\n{rs}"
    );
}

#[test]
fn rs_backend_emits_box_new_for_recursive_ctor() {
    let rs = emit_rust(
        "data List[A] { .nil, .cons(A, List[A]) } fn singleton(x: String): List[String] { List.cons(x, List.nil) }",
    );
    assert!(
        rs.contains("Box::new("),
        "recursive ctor arg should be wrapped in Box::new, got:\n{rs}"
    );
}

#[test]
fn rs_backend_emits_inherent_impl_as_standalone_fns() {
    let rs = emit_rust(
        "#[extern = \"string\"] extern type String; #[extern = \"number\"] extern type Number; #[extern = \"String.length\"] extern fn str_len(s: String): Number; impl String { fn len(self: String): Number { str_len(self) } }",
    );
    assert!(
        rs.contains("fn string__len("),
        "inherent impl method should be mangled as target__method: {rs}"
    );
    assert!(
        rs.contains("self_: String"),
        "self should be renamed to self_: {rs}"
    );
    assert!(
        rs.contains("str_len(self_)"),
        "body should reference self_: {rs}"
    );
}

#[test]
fn rs_backend_emits_unnamed_cap_impl() {
    let rs = emit_rust(
        "cap Clone { fn clone(self: A): A } impl String: Clone { fn clone(self: String): String { self } }",
    );
    assert!(
        rs.contains("__impl_string_clone_clone"),
        "unnamed cap impl should be __impl_target_cap_method: {rs}"
    );
}

#[test]
fn rs_backend_emits_named_cap_impl() {
    let rs = emit_rust(
        "cap Clone { fn clone(self: A): A } impl MyClone = String: Clone { fn clone(self: String): String { self } }",
    );
    assert!(
        rs.contains("myclone__clone"),
        "named impl method should be name__method: {rs}"
    );
}
