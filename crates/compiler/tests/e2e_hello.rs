use lumo_compiler::{
    backend::{self, CodegenTarget},
    query::QueryEngine,
};

const PRELUDE_SRC: &str = include_str!("../../../packages/lumo-std/prelude.lumo");
const IO_SRC: &str = include_str!("../../../packages/lumo-std/io.lumo");

fn stdlib_resolver(path: &[String]) -> Option<(String, String)> {
    match path {
        [pkg, module] if pkg == "lumo_std" && module == "prelude" => {
            Some(("lumo_std/prelude.lumo".into(), PRELUDE_SRC.into()))
        }
        [pkg, module] if pkg == "lumo_std" && module == "io" => {
            Some(("lumo_std/io.lumo".into(), IO_SRC.into()))
        }
        _ => None,
    }
}

#[test]
fn hello_world_compiles_to_js() {
    let mut q = QueryEngine::new();
    q.set_file(
        "main.lumo",
        r#"use lumo_std.io.{println};

fn main() := println("Hello, World!")"#,
    );

    let lir = q
        .compile_with_deps(&["main.lumo"], stdlib_resolver)
        .expect("compilation should succeed");
    let js = backend::emit(&lir, CodegenTarget::JavaScript).expect("codegen should succeed");

    assert!(
        js.contains("function main()"),
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
        r#"use lumo_std.io.{println};

fn main() := println("Hello, World!")"#,
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
fn arithmetic_operator_desugars_to_cap_call() {
    // 1 + 2 should desugar to perform Add.add(1, 2)
    // With a handler, it should compile to JS that invokes the handler's add method
    let mut q = QueryEngine::new();
    q.set_file(
        "main.lumo",
        r#"
cap Add { fn add(a, b) }

fn sum() := 1 + 2

fn main() :=
  handle Add with bundle { fn add(a, b) := a } in
    sum()
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
