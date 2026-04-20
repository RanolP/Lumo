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
/// After the LTO entry-point in-place rewrite fix: when `main` itself (no helper fn)
/// contains the cap calls, LTO detects that `main` has zero callers and rewrites it in
/// place instead of creating a `main__lto` clone that DCE would discard. The body's
/// Perform(Add) nodes are eliminated, `cap` is cleared to `None`, and the backend emits
/// `main` as a plain `export function main()` with direct `__num_add` calls — no CPS
/// wrapper, no `__caps` bundle.
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

    // After in-place rewrite: cap=None means no CPS wrapper — backend emits a direct fn.
    assert!(
        !js.contains("function __main_cps("),
        "JS should NOT contain __main_cps after LTO in-place rewrite (cap cleared), got:\n{js}"
    );

    // The impl const `__impl_Number_Add` must be present — LTO built the chain.
    assert!(
        js.contains("__impl_Number_Add"),
        "JS should reference __impl_Number_Add (the Add[Number] default impl), got:\n{js}"
    );

    // The stdlib chain must be intact (either __num_add or inlined `+` operator).
    // With the let-dedup fix enabling deeper IIFE flattening, the arithmetic may be
    // fully inlined as `(a + b)` rather than a `__num_add(a, b)` call.
    assert!(
        js.contains("__num_add") || js.contains("(a + b)") || js.contains("+ b"),
        "JS should reference direct arithmetic (via __num_add or inlined +), got:\n{js}"
    );

    // Extract `main`'s body (now a plain fn, not CPS) to check for cap elimination.
    let main_fn_start = js
        .find("export function main()")
        .expect("export function main() not found in JS");
    let main_fn_end = js[main_fn_start..]
        .find("}\n\n")
        .map(|i| main_fn_start + i)
        .unwrap_or(js.len());
    let main_body = &js[main_fn_start..main_fn_end];

    // After the in-place rewrite: `main` body must NOT access `__caps.Add_Number`.
    // main has zero callers, so LTO rewrites it in place rather than cloning (which
    // DCE would drop). The Perform(Add) nodes in main's own body are now eliminated.
    assert!(
        !main_body.contains("__caps.Add_Number"),
        "expected main body to NOT dispatch through __caps.Add_Number after LTO \
         in-place rewrite (main is a zero-caller entry point), got:\n{}\n\n(full js)\n{}",
        main_body,
        js
    );

    // The resolved arithmetic must appear directly in main body. After the
    // let-dedup + alias-inline wave, variable names change but the `+` operator
    // always survives — match on it directly.
    assert!(
        main_body.contains("__num_add") || main_body.contains(" + "),
        "expected main body to use direct arithmetic after LTO in-place rewrite, \
         got:\n{}\n\n(full js)\n{}",
        main_body,
        js
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
    // use direct arithmetic — either `__num_add` or inlined `+` operator.
    // With the let-dedup fix enabling deeper IIFE flattening, the arithmetic may be
    // fully inlined as `(a + b)` rather than a `__num_add(a, b)` call.
    assert!(
        main_cps_body.contains("__num_add") || main_cps_body.contains("(a + b)") || main_cps_body.contains("+ b"),
        "expected __main_cps body to use direct arithmetic after LTO inlining, \
         got:\n{}\n\n(full js)\n{}",
        main_cps_body,
        js
    );
}
