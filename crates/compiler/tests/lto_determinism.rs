use lumo_compiler::{
    backend::{self, CodegenTarget},
    query::QueryEngine,
};

fn compile(src: &str) -> String {
    let mut q = QueryEngine::new();
    q.set_file("main.lumo", src.to_owned());
    let lir = q.lower_module(&["main.lumo"]).expect("lower_module failed");
    backend::emit(&lir, CodegenTarget::JavaScript).expect("js emit")
}

#[test]
fn lto_output_is_deterministic() {
    let src = include_str!("fixtures/lto/01_trivial_leaf.txt")
        .split("===EXPECT===")
        .next()
        .unwrap()
        .trim();
    let first = compile(src);
    for i in 0..10 {
        let next = compile(src);
        assert_eq!(
            first,
            next,
            "LTO output differs on run {}: \n--- first ---\n{}\n--- run {} ---\n{}",
            i + 1,
            first,
            i + 1,
            next
        );
    }
}

#[test]
fn lto_output_is_deterministic_multi_fn() {
    // Two dep-free fns — ordering of clones in output must be stable.
    let src = r#"
extern type Number;
cap Add { fn add(a: Self, b: Self): Self }
cap Sub { fn sub(a: Self, b: Self): Self }
extern fn js_add(a: Number, b: Number): Number;
extern fn js_sub(a: Number, b: Number): Number;
impl Number: Add { fn add(a: Self, b: Self): Self = resume(js_add(a, b)) }
impl Number: Sub { fn sub(a: Self, b: Self): Self = resume(js_sub(a, b)) }

fn helper_a(x: Number): Number = Add.add(x, 1)
fn helper_b(x: Number): Number = Sub.sub(x, 1)
fn main(): Number = helper_a(helper_b(5))
"#;
    let first = compile(src);
    for i in 0..10 {
        let next = compile(src);
        assert_eq!(
            first,
            next,
            "LTO output differs on run {}:\n--- first ---\n{}\n--- run {} ---\n{}",
            i + 1,
            first,
            i + 1,
            next
        );
    }
}
