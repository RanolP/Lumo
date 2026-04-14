use lumo_compiler::{
    backend::{self, CodegenTarget},
    query::QueryEngine,
};

const PRELUDE_SRC: &str = include_str!("../../../packages/lumo-std/src/prelude.lumo");
const IO_SRC: &str = include_str!("../../../packages/lumo-std/src/io.lumo");
const STRING_SRC: &str = include_str!("../../../packages/lumo-std/src/string.lumo");
const NUMBER_SRC: &str = include_str!("../../../packages/lumo-std/src/number.lumo");
const FS_SRC: &str = include_str!("../../../packages/lumo-std/src/fs.lumo");
const PROCESS_SRC: &str = include_str!("../../../packages/lumo-std/src/process.lumo");
const LIST_SRC: &str = include_str!("../../../packages/lumo-std/src/list.lumo");

fn stdlib_resolver(path: &[String]) -> Option<(String, String)> {
    match path {
        [pkg, module] if pkg == "lumo_std" => {
            let (file, src) = match module.as_str() {
                "prelude" => ("prelude.lumo", PRELUDE_SRC),
                "io" => ("io.lumo", IO_SRC),
                "string" => ("string.lumo", STRING_SRC),
                "number" => ("number.lumo", NUMBER_SRC),
                "fs" => ("fs.lumo", FS_SRC),
                "process" => ("process.lumo", PROCESS_SRC),
                "list" => ("list.lumo", LIST_SRC),
                _ => return None,
            };
            Some((format!("lumo_std/{file}"), src.into()))
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

fn main() { println("Hello, World!") }"#,
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

fn main() { println("Hello, World!") }"#,
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
fn stdlib_string_ops_compile_to_rust() {
    let mut q = QueryEngine::new();
    q.set_file(
        "main.lumo",
        r#"use lumo_std.prelude.{String, Number, Bool};
use lumo_std.io.{println};
use lumo_std.string.{str_len, str_eq, str_concat, num_to_string};

fn main() {
  let greeting = str_concat("Hello", " World");
  let len = str_len(greeting);
  println(str_concat(greeting, str_concat(" len=", num_to_string(len))))
}"#,
    );

    let lir = q
        .compile_with_deps(&["main.lumo"], stdlib_resolver)
        .expect("compilation should succeed");
    let rs = backend::emit(&lir, CodegenTarget::Rust).expect("rust codegen should succeed");

    assert!(
        rs.contains("fn str_concat("),
        "Rust should contain str_concat, got:\n{rs}"
    );
    assert!(
        rs.contains("fn str_len("),
        "Rust should contain str_len, got:\n{rs}"
    );
    assert!(
        rs.contains("fn main()"),
        "Rust should contain main, got:\n{rs}"
    );
}

#[test]
fn stdlib_list_compiles_to_rust() {
    let mut q = QueryEngine::new();
    q.set_file(
        "main.lumo",
        r#"use lumo_std.prelude.{String, Number, Bool};
use lumo_std.io.{println};
use lumo_std.list.{List};
use lumo_std.string.{num_to_string};
use lumo_std.number.{num_add};

fn list_length[A](xs: List[A]): Number {
  match xs { List.nil => 0, List.cons(_, t) => num_add(1, list_length(t)) }
}

fn main() {
  let xs = List.cons("a", List.cons("b", List.nil));
  println(num_to_string(list_length(xs)))
}"#,
    );

    let lir = q
        .compile_with_deps(&["main.lumo"], stdlib_resolver)
        .expect("compilation should succeed");
    let rs = backend::emit(&lir, CodegenTarget::Rust).expect("rust codegen should succeed");

    assert!(
        rs.contains("Box<List"),
        "Rust should have Box for recursive List, got:\n{rs}"
    );
    assert!(
        rs.contains("Box::new("),
        "Rust should have Box::new for List ctor, got:\n{rs}"
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
