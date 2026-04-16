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
  let len = StrOps.str_len(greeting);
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
fn arithmetic_operator_desugars_to_cap_call() {
    // 1 + 2 should desugar to perform Add.add(1, 2)
    // With a handler, it should compile to JS that invokes the handler's add method
    let mut q = QueryEngine::new();
    q.set_file(
        "main.lumo",
        r#"
cap Add { fn add(a, b) }

fn sum() { 1 + 2 }

fn main() {
  handle Add with bundle { fn add(a, b) { a } } in
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
}
