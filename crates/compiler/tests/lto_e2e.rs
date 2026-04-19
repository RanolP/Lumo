use lumo_compiler::{
    backend::{self, CodegenTarget},
    query::QueryEngine,
};

// ---------------------------------------------------------------------------
// libcore sources (common + JS platform)
// ---------------------------------------------------------------------------
const PRELUDE_SRC: &str = include_str!("../../../packages/libcore/src/prelude.lumo");
const CMP_SRC: &str = include_str!("../../../packages/libcore/src/cmp.lumo");
const OPS_SRC: &str = include_str!("../../../packages/libcore/src/ops.lumo");
const OPS_JS_SRC: &str = include_str!("../../../packages/libcore/src#js/ops.lumo");
const STRING_SRC: &str = include_str!("../../../packages/libcore/src/string.lumo");
const STRING_JS_SRC: &str = include_str!("../../../packages/libcore/src#js/string.lumo");
const NUMBER_SRC: &str = include_str!("../../../packages/libcore/src/number.lumo");
const NUMBER_JS_SRC: &str = include_str!("../../../packages/libcore/src#js/number.lumo");

// ---------------------------------------------------------------------------
// libstd sources (common + JS platform)
// ---------------------------------------------------------------------------
const IO_SRC: &str = include_str!("../../../packages/libstd/src/io.lumo");
const IO_JS_SRC: &str = include_str!("../../../packages/libstd/src#js/io.lumo");
const LIST_SRC: &str = include_str!("../../../packages/libstd/src/list.lumo");

fn stdlib_resolver(path: &[String]) -> Option<(String, String)> {
    match path {
        [pkg, module] if pkg == "libcore" => {
            let (file, src) = match module.as_str() {
                "prelude" => ("prelude.lumo", PRELUDE_SRC.to_owned()),
                "cmp" => ("cmp.lumo", CMP_SRC.to_owned()),
                "ops" => ("ops.lumo", format!("{OPS_SRC}\n{OPS_JS_SRC}")),
                "string" => ("string.lumo", format!("{STRING_SRC}\n{STRING_JS_SRC}")),
                "number" => ("number.lumo", format!("{NUMBER_SRC}\n{NUMBER_JS_SRC}")),
                _ => return None,
            };
            Some((format!("libcore/{file}"), src))
        }
        [pkg, module] if pkg == "libstd" => {
            let (file, src) = match module.as_str() {
                "io" => ("io.lumo", format!("{IO_SRC}\n{IO_JS_SRC}")),
                "list" => ("list.lumo", LIST_SRC.to_owned()),
                _ => return None,
            };
            Some((format!("libstd/{file}"), src))
        }
        _ => None,
    }
}

/// Smoke test: `1 + 2 + 3` directly in main compiles through real stdlib.
///
/// LTO gap — direct arithmetic in main's own body (tracked in plans/unify-impl-and-handler.md):
/// When main itself contains the cap calls (no helper fn), LTO v1 creates a `main__lto`
/// clone but DCE immediately removes it — nothing calls `main__lto`. The original `main`
/// keeps its Perform(Add) nodes, so the backend emits `__main_cps` with `__caps.Add_Number`.
/// This test documents the current (pre-fix) behavior: `__caps.Add_Number` IS present in
/// `__main_cps`. When the LTO entry-point rewrite is implemented, this test should be
/// updated to assert the opposite.
#[test]
fn arithmetic_direct_in_main_compiles_with_stdlib() {
    let mut q = QueryEngine::new();
    q.set_file(
        "main.lumo",
        r#"use libcore.prelude.{Number};
use libcore.ops.{Add};
use libcore.number.{NumOps};

fn main(): Number = 1 + 2 + 3
"#
        .to_owned(),
    );
    let lir = q
        .compile_with_deps(&["main.lumo"], stdlib_resolver)
        .expect("compile_with_deps failed");
    let js = backend::emit(&lir, CodegenTarget::JavaScript).expect("js emit");

    // Compilation must succeed and produce a `__main_cps` wrapper.
    assert!(
        js.contains("function __main_cps("),
        "JS should contain __main_cps, got:\n{js}"
    );

    // The impl const `__impl_Number_Add` must be present — LTO built the chain.
    assert!(
        js.contains("__impl_Number_Add"),
        "JS should reference __impl_Number_Add (the Add[Number] default impl), got:\n{js}"
    );

    // `__num_add` (JS infix+ wrapper) must be present — stdlib chain is intact.
    assert!(
        js.contains("__num_add"),
        "JS should reference __num_add (the JS extern for Number addition), got:\n{js}"
    );

    // LTO v1 gap: `__caps.Add_Number` is still present in `__main_cps` when main
    // itself (not a helper) is the dep-free function. The entry-point shape blocks
    // the in-place rewrite. When this gap is fixed, flip the assertion.
    assert!(
        js.contains("__caps.Add_Number"),
        "LTO gap: expected __caps.Add_Number to still be present in __main_cps body \
         (direct-main-arithmetic is not yet rewritten in-place by LTO v1), got:\n{js}"
    );
}

/// End-to-end LTO smoke: arithmetic through a helper fn eliminates the cap bundle.
///
/// `fn sum() = 1 + 2 + 3` is a small dep-free helper with a single caller (main).
/// LTO applies heuristic D → inline (C form): `sum`'s resolved body is substituted
/// directly into `main`'s call site. After inlining, `__main_cps`'s own body calls
/// `__num_add` directly with no `__caps.*` dispatch.
///
/// This validates the two-level real-stdlib LTO chain:
///   Add[Number] → __impl_Number_Add.add → NumOps.add (Perform[NumOps]) → __num_add
#[test]
fn arithmetic_via_helper_lto_eliminates_cap_bundle() {
    let mut q = QueryEngine::new();
    q.set_file(
        "main.lumo",
        r#"use libcore.prelude.{Number};
use libcore.ops.{Add};
use libcore.number.{NumOps};

fn sum(): Number = 1 + 2 + 3

fn main(): Number = sum()
"#
        .to_owned(),
    );
    let lir = q
        .compile_with_deps(&["main.lumo"], stdlib_resolver)
        .expect("compile_with_deps failed");
    let js = backend::emit(&lir, CodegenTarget::JavaScript).expect("js emit");

    // Compilation must succeed.
    assert!(
        js.contains("function __main_cps("),
        "JS should contain __main_cps, got:\n{js}"
    );

    // Heuristic D: `sum` is small + single-caller → inlined, not cloned.
    // If this ever flips (e.g. body grows), update the assertion to check for
    // `sum__lto` instead and verify the clone body has no __caps.* dispatch.
    assert!(
        !js.contains("sum__lto"),
        "expected `sum` to be inlined (not cloned as sum__lto) by heuristic D, got:\n{js}"
    );

    // Extract `__main_cps`'s body to check for cap bundle elimination.
    let main_cps_start = js
        .find("function __main_cps")
        .expect("__main_cps not found in JS");
    // Find the closing `}` of the function body — look for two newlines after `}`.
    let main_cps_end = js[main_cps_start..]
        .find("}\n\n")
        .map(|i| main_cps_start + i)
        .unwrap_or(js.len());
    let main_cps_body = &js[main_cps_start..main_cps_end];

    // Core invariant: after LTO inlining sum's body into main,
    // `__main_cps` must NOT access `__caps.Add_Number` or `__caps.NumOps_NumOps`.
    assert!(
        !main_cps_body.contains("__caps.Add_Number"),
        "expected __main_cps body to not dispatch through __caps.Add_Number (LTO should have \
         inlined sum's resolved body directly), got:\n{}\n\n(full js)\n{}",
        main_cps_body,
        js
    );
    assert!(
        !main_cps_body.contains("__caps.NumOps_NumOps"),
        "expected __main_cps body to not dispatch through __caps.NumOps_NumOps (LTO should have \
         fully resolved the stdlib chain), got:\n{}\n\n(full js)\n{}",
        main_cps_body,
        js
    );

    // After full LTO inlining + recursive Perform resolution, the body should
    // call `__num_add` directly (the two-level chain resolves all the way down).
    assert!(
        main_cps_body.contains("__num_add"),
        "expected __main_cps body to call __num_add directly after LTO inlining, \
         got:\n{}\n\n(full js)\n{}",
        main_cps_body,
        js
    );
}
