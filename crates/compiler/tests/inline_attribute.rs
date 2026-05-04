use lumo_compiler::{
    hir,
    lir,
    query::QueryEngine,
};

fn lower(src: &str) -> lir::File {
    let lossless = lumo_compiler::lst::lossless::parse(src);
    let hir = hir::lower_lossless(&lossless);
    lir::lower(&hir)
}

#[test]
fn inline_always_attribute_sets_inline_flag() {
    let file = lower("#[inline(always)] fn id(x: Number): Number { x }");
    let lir::Item::Fn(f) = &file.items[0] else { panic!("expected fn, got {:?}", &file.items[0]) };
    assert!(f.inline, "expected inline=true on #[inline(always)] fn");
}

#[test]
fn no_inline_attribute_leaves_flag_false() {
    let file = lower("fn id(x: Number): Number { x }");
    let lir::Item::Fn(f) = &file.items[0] else { panic!("expected fn") };
    assert!(!f.inline, "expected inline=false on plain fn");
}

#[test]
fn inline_always_on_unresolvable_fn_is_an_error() {
    let mut q = QueryEngine::new();
    q.set_file("a.lumo", r#"
        cap MyCap { fn op(): Number }
        #[inline(always)]
        fn uses(): Number { MyCap.op }
        fn main(): Number { uses() }
    "#.to_owned());
    // lower_module should return None because `uses` is marked #[inline(always)]
    // but has an unresolvable cap (MyCap has no default impl).
    let result = q.lower_module(&["a.lumo"]);
    assert!(
        result.is_none(),
        "expected lower_module to return None when #[inline(always)] fn has unresolved cap"
    );
}
