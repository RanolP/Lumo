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

#[test]
fn hello_world_compiles_to_js() {
    let mut q = QueryEngine::new();
    q.set_file(
        "main.lumo",
        r#"use libstd.io.{IO};

fn main() { IO.println("Hello, World!") }"#,
    );

    let lir = q
        .compile_with_deps(&["main.lumo"], stdlib_resolver)
        .expect("compilation should succeed");
    let js = backend::emit(&lir, CodegenTarget::JavaScript).expect("codegen should succeed");

    assert!(
        js.contains("function main("),
        "JS should contain main function, got:\n{js}"
    );
    assert!(
        js.contains("console.log"),
        "JS should reference console.log, got:\n{js}"
    );
    assert!(
        js.contains("Hello, World!"),
        "JS should contain the greeting, got:\n{js}"
    );
}

#[test]
#[ignore] // requires Node.js
fn hello_world_runs_on_node() {
    let mut q = QueryEngine::new();
    q.set_file(
        "main.lumo",
        r#"use libstd.io.{IO};

fn main() { IO.println("Hello, World!") }"#,
    );

    let lir = q
        .compile_with_deps(&["main.lumo"], stdlib_resolver)
        .expect("compilation should succeed");
    let js = backend::emit(&lir, CodegenTarget::JavaScript).expect("codegen should succeed");
    let js_with_entry = format!("{js}\nmain();\n");

    let output = std::process::Command::new("node")
        .arg("-e")
        .arg(&js_with_entry)
        .output()
        .expect("failed to execute node");

    assert!(
        output.status.success(),
        "node should exit successfully, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "Hello, World!"
    );
}

#[test]
fn stdlib_string_ops_compile_to_js() {
    let mut q = QueryEngine::new();
    q.set_file(
        "main.lumo",
        r#"use libcore.prelude.{String, Number};
use libstd.io.{IO};
use libcore.string.{StrOps};

fn main() {
  let greeting = "Hello" + " World";
  let len = StrOps.len(greeting);
  IO.println(greeting + " len=" + StrOps.num_to_string(len))
}"#,
    );

    let lir = q
        .compile_with_deps(&["main.lumo"], stdlib_resolver)
        .expect("compilation should succeed");
    let js = backend::emit(&lir, CodegenTarget::JavaScript).expect("codegen should succeed");

    assert!(
        js.contains("function main("),
        "JS should contain main, got:\n{js}"
    );
    assert!(
        js.contains("console.log"),
        "JS should reference console.log, got:\n{js}"
    );
}

#[test]
fn stdlib_list_compiles_to_js() {
    let mut q = QueryEngine::new();
    q.set_file(
        "main.lumo",
        r#"use libcore.prelude.{Number};
use libcore.string.{StrOps};
use libstd.io.{IO};
use libstd.list.{List};

fn list_length[A](xs: List[A]): Number {
  match xs { .nil => 0, .cons(_, t) => 1 + list_length(t) }
}

fn main() {
  let xs = List.cons("a", List.cons("b", List.nil));
  IO.println(StrOps.num_to_string(list_length(xs)))
}"#,
    );

    let lir = q
        .compile_with_deps(&["main.lumo"], stdlib_resolver)
        .expect("compilation should succeed");
    let js = backend::emit(&lir, CodegenTarget::JavaScript).expect("codegen should succeed");

    assert!(
        js.contains("function main("),
        "JS should contain main, got:\n{js}"
    );
}

#[test]
fn value_method_dispatch_inherent() {
    // "hello".len() should dispatch to impl String { fn len(self) } from libcore/string.lumo
    let mut q = QueryEngine::new();
    q.set_file(
        "main.lumo",
        r#"use libcore.prelude.{String, Number};
use libcore.string.{StrOps};

fn greet_len(): Number = "Hello".len()
"#,
    );

    let lir = q
        .compile_with_deps(&["main.lumo"], stdlib_resolver)
        .expect("compilation should succeed");
    let js = backend::emit(&lir, CodegenTarget::JavaScript).expect("codegen should succeed");

    // After rewrite, "Hello".len() → String.len("Hello") → StrOps.str_len("Hello")
    assert!(
        js.contains("function greet_len("),
        "JS should contain greet_len function, got:\n{js}"
    );
    // The impl const "String" should exist with a len method
    assert!(
        js.contains("String") && js.contains("len"),
        "JS should reference String.len, got:\n{js}"
    );
}

#[test]
fn value_method_dispatch_typeclass() {
    // 1.add(2) should dispatch to impl Number: Add { fn add(self, other) }
    let mut q = QueryEngine::new();
    q.set_file(
        "main.lumo",
        r#"use libcore.prelude.{Number};
use libcore.number.{NumOps};
use libcore.ops.{Add};

fn sum(): Number = 1.add(2)
"#,
    );

    let lir = q
        .compile_with_deps(&["main.lumo"], stdlib_resolver)
        .expect("compilation should succeed");
    let js = backend::emit(&lir, CodegenTarget::JavaScript).expect("codegen should succeed");

    assert!(
        js.contains("function sum("),
        "JS should contain sum function, got:\n{js}"
    );
    // Should dispatch to __impl_Number_Add.add(1, 2)
    assert!(
        js.contains("__impl_Number_Add"),
        "JS should reference __impl_Number_Add, got:\n{js}"
    );
}

#[test]
fn main_with_default_impl_caps_emits_cps_main_and_wrapper() {
    let mut q = QueryEngine::new();
    q.set_file(
        "main.lumo",
        r#"use libcore.prelude.{Number};
use libcore.ops.{Add};
use libcore.number.{NumOps};

fn main(): Number = 1 + 2
"#,
    );
    let lir = q
        .compile_with_deps(&["main.lumo"], stdlib_resolver)
        .expect("compilation should succeed");
    let js = backend::emit(&lir, CodegenTarget::JavaScript).expect("codegen should succeed");
    assert!(
        js.contains("function __main_cps("),
        "JS should rename effectful user main → __main_cps, got:\n{js}"
    );
    assert!(
        js.contains("function main()"),
        "JS should expose a no-arg main() entry wrapper, got:\n{js}"
    );
    assert!(
        js.contains("__impl_Number_Add"),
        "JS wrapper should pass __impl_Number_Add as the default for Add[Number], got:\n{js}"
    );
    // The bundled __caps object is built once and threaded through CPS calls.
    assert!(
        js.contains("__caps.Add_Number"),
        "body should access caps via __caps bundle, got:\n{js}"
    );
    assert!(
        js.contains("__main_cps({"),
        "wrapper should call __main_cps with bundle literal, got:\n{js}"
    );
}

#[test]
fn main_with_default_impl_caps_compiles_and_wraps_entry() {
    // `+` desugars to `Perform Add`. main is effectful with `Add[Number]`.
    // Since `impl Number: Add` provides a default impl, the backend should
    // emit __main_cps + a public main() wrapper that injects __impl_Number_Add.
    let mut q = QueryEngine::new();
    q.set_file(
        "main.lumo",
        r#"use libcore.prelude.{Number};
use libcore.ops.{Add};
use libcore.number.{NumOps};
use libstd.io.{IO};

fn main() {
  IO.println("ok")
}
"#,
    );
    let lir = q
        .compile_with_deps(&["main.lumo"], stdlib_resolver)
        .expect("compilation should succeed");
    let js = backend::emit(&lir, CodegenTarget::JavaScript).expect("codegen should succeed");
    // Pure main (no caps) → no wrapper, just regular main()
    assert!(
        js.contains("function main()"),
        "JS should contain a public main(), got:\n{js}"
    );
}

#[test]
fn main_requiring_undefaulted_cap_is_compile_error() {
    // Cap `MyCap` has no default impl (no `impl MyCap { ... }`),
    // so main() requiring it must fail to codegen.
    let mut q = QueryEngine::new();
    q.set_file(
        "main.lumo",
        r#"cap MyCap { fn frobnicate() }

fn main(): Unit / { MyCap } {
  perform MyCap.frobnicate()
}
"#,
    );
    let lir = q
        .lower_module(&["main.lumo"])
        .expect("lower_module should succeed (validation happens at backend)");
    let result = backend::emit(&lir, CodegenTarget::JavaScript);
    assert!(
        result.is_err(),
        "codegen should fail for main requiring cap with no default impl"
    );
    let err = format!("{:?}", result.unwrap_err());
    assert!(
        err.contains("MyCap") && err.contains("default impl"),
        "error should mention MyCap and default impl, got: {err}"
    );
}

#[test]
fn multi_shot_resume_compiles() {
    // A handler that calls `resume` twice and combines the results with `+`.
    // Under algebraic-effect semantics with split k_perform/k_handle, the
    // handler body value flows to the outer continuation (abort); each
    // explicit `resume(v)` drives the perform's continuation synchronously.
    let mut q = QueryEngine::new();
    q.set_file(
        "main.lumo",
        r#"use libcore.prelude.{Number};
use libcore.ops.{Add};
use libcore.number.{NumOps};

cap Choice { fn pick(): Number }

fn main(): Number = handle Choice with bundle {
  fn pick() { resume(1) + resume(2) }
} in perform Choice.pick()
"#,
    );
    let lir = q
        .compile_with_deps(&["main.lumo"], stdlib_resolver)
        .expect("compilation should succeed");
    let js = backend::emit(&lir, CodegenTarget::JavaScript).expect("codegen should succeed");
    assert!(
        js.contains("__trampoline(__k_perform("),
        "resume should drive __k_perform via __trampoline: {js}"
    );
    assert!(
        js.contains("(__k_handle) =>"),
        "handler should be built by a factory closure over __k_handle: {js}"
    );
}

#[test]
#[ignore] // requires Node.js
fn multi_shot_resume_on_node() {
    // Handler resumes twice, each with a different string; the perform site
    // prints whatever came back. Two println calls prove the continuation ran
    // twice, confirming multi-shot semantics.
    let mut q = QueryEngine::new();
    q.set_file(
        "main.lumo",
        r#"use libcore.prelude.{String, Number};
use libstd.io.{IO};

cap Choice { fn pick(n: Number): String }

fn greet(): Unit / { Choice, IO } = IO.println(Choice.pick(0))

fn main() = handle Choice with bundle {
  fn pick(n) { let _ = resume("hello"); resume("world") }
} in greet()
"#,
    );
    let lir = q
        .compile_with_deps(&["main.lumo"], stdlib_resolver)
        .expect("compilation should succeed");
    let js = backend::emit(&lir, CodegenTarget::JavaScript).expect("codegen should succeed");
    let js_with_entry = format!("{js}\nmain();\n");

    let output = std::process::Command::new("node")
        .arg("-e")
        .arg(&js_with_entry)
        .output()
        .expect("failed to execute node");
    assert!(
        output.status.success(),
        "node should exit successfully, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    // Each resume call re-runs the perform's continuation with a fresh value.
    // Expected stdout: "hello\nworld".
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    assert_eq!(
        stdout, "hello\nworld",
        "multi-shot handler should invoke the perform continuation twice; got:\n{stdout}"
    );
}

#[test]
fn arithmetic_operator_desugars_to_cap_call() {
    // 1 + 2 should desugar to `perform Add.add(1, 2)`. With an explicit handler
    // providing Add, the handle expression must evaluate the handler body with
    // the perform's continuation. Under algebraic-effect semantics (abort by
    // default), the handler must *explicitly* `resume(a)` to thread `a` back
    // into the perform site — otherwise `sum()` never completes.
    let mut q = QueryEngine::new();
    q.set_file(
        "main.lumo",
        r#"
cap Add { fn add(a, b) }

fn sum() { 1 + 2 }

fn main() {
  handle Add with bundle { fn add(a, b) { resume(a) } } in
    sum()
}
"#,
    );

    let lir = q
        .lower_module(&["main.lumo"])
        .expect("compilation should succeed");
    let js = backend::emit(&lir, CodegenTarget::JavaScript).expect("codegen should succeed");

    // The generated JS should contain the number literals
    assert!(
        js.contains('1') && js.contains('2'),
        "JS should contain number literals, got:\n{js}"
    );
    // The generated JS should have the sum function
    assert!(
        js.contains("sum"),
        "JS should contain sum function, got:\n{js}"
    );
    // The handler invokes `resume` to drive the perform's continuation.
    assert!(
        js.contains("__trampoline(__k_perform("),
        "handler should invoke __k_perform via resume(a): {js}"
    );
}

